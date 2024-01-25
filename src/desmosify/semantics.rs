use super::*;

impl Signatures {
    pub fn get_type_from_name(&self, scope: &Scope, name: &str) -> Option<DataType> {
        scope.parameters.get(name).map(|data_type| data_type.clone()).or_else(
            || self.user_defined.get(name).and_then(|signature| match signature {
                Signature::Const { parameters, value_type, .. } => if parameters.is_some() {
                    Some(DataType::Function { name: String::from(name) })
                } else {
                    Some(value_type.clone())
                },
                Signature::Let { parameters, value_type, .. } => if parameters.is_some() {
                    Some(DataType::Function { name: String::from(name) })
                } else {
                    Some(value_type.clone())
                },
                Signature::Var { value_type, .. } => Some(value_type.clone()),
                _ => None
            }),
        )
    }
}

impl DataType {
    pub fn can_coerce_to(&self, target: &DataType) -> bool {
        use DataType::*;
        match self {
            Unknown => match target {
                Void | Function { .. } | Action { .. } | Str => false,
                _ => true
            },
            Real => match target {
                Unknown | Real => true,
                _ => false
            },
            Int => match target {
                Unknown | Real | Int | User { .. } => true,
                _ => false
            },
            Bool => match target {
                Unknown | Real | Int | Bool => true,
                _ => false
            },
            Point => match target {
                Unknown | Point => true,
                _ => false
            },
            IPoint => match target {
                Unknown | Point | IPoint => true,
                _ => false
            },
            Color => match target {
                Unknown | Color => true,
                _ => false
            },
            Polygon => match target {
                Unknown | Polygon => true,
                _ => false
            },
            Segment => match target {
                Unknown | Segment => true,
                _ => false
            },
            List { item_type } => match target {
                List { item_type: target_item_type } => item_type.can_coerce_to(target_item_type),
                _ => item_type.can_coerce_to(target)
            },
            User { name } => match target {
                Unknown | Real | Int => true,
                User { name: target_name } => target_name == name,
                _ => false
            },
            _ => false
        }
    }

    pub fn merge(&self, other: &Self) -> Option<Self> {
        if self.can_coerce_to(other) {
            Some(other.clone())
        }
        else if other.can_coerce_to(self) {
            Some(self.clone())
        }
        else {
            None
        }
    }

    pub fn merge_numeric(&self, other: &Self) -> Option<Self> {
        if self == &Self::Unknown || other == &Self::Unknown {
            Some(Self::Unknown)
        }
        else if self.can_coerce_to(&Self::Int) && other.can_coerce_to(&Self::Int) {
            Some(Self::Int)
        }
        else if self.can_coerce_to(&Self::Real) && other.can_coerce_to(&Self::Real) {
            Some(Self::Real)
        }
        else {
            None
        }
    }

    pub fn point_type(&self, components: usize) -> Option<Self> {
        use DataType::*;
        match (self, components) {
            (Unknown, 2) => Some(Point),
            (Int, 2) => Some(IPoint),
            (Real, 2) => Some(Point),
            _ => None
        }
    }

    pub fn list_type(&self) -> Option<Self> {
        use DataType::*;
        match self {
            List { .. } | Void | Function { .. } | Action { .. } | Str => None,
            _ => Some(List { item_type: Box::new(self.clone()) })
        }
    }
}

pub fn message_cannot_coerce(from_type: &DataType, to_type: &DataType) -> String {
    format!("cannot coerce value of type '{from_type}' to '{to_type}'")
}

#[derive(Clone, Debug)]
pub struct Scope {
    pub can_use_dt: bool,
    pub can_use_index: bool,
    pub parameters: BTreeMap<String, DataType>,
}

impl Default for Scope {
    fn default() -> Self {
        Self {
            can_use_dt: false,
            can_use_index: false,
            parameters: BTreeMap::new(),
        }
    }
}

pub fn analyze(signatures: &Signatures, definitions: &mut Definitions) -> Result<(), DesmosifyError> {
    let scope = Scope::default();

    for (name, value) in &mut definitions.identifiers {
        analyze_identifier(signatures, &scope, signatures.user_defined.get(name).unwrap(), value.as_mut())?;
    }

    for (name, content) in &mut definitions.actions {
        analyze_named_action(signatures, &scope, signatures.user_defined.get(name).unwrap(), content.as_mut())?;
    }

    if let Some(elements) = &mut definitions.public {
        for element in elements {
            analyze_public_element(signatures, &scope, element)?;
        }
    }

    if let Some(ticker) = &mut definitions.ticker {
        analyze_ticker(signatures, &scope, ticker)?;
    }

    if let Some(elements) = &mut definitions.display {
        for element in elements {
            analyze_display_element(signatures, &scope, element)?;
        }
    }

    Ok(())
}

pub fn analyze_expression(signatures: &Signatures, scope: &Scope, expression: &mut Expression) -> Result<(), DesmosifyError> {
    match &mut expression.value {
        ExpressionValue::Name(name) => if let Some(data_type) = signatures.get_type_from_name(scope, name) {
            expression.data_type = data_type;
            Ok(())
        } else {
            Err(DesmosifyError::new(
                format!("could not find a definition for '{name}'"),
                expression.start,
                expression.end,
            ))
        },
        ExpressionValue::Operator(operation, operands) => match *operation {
            Operation::PointLiteral => {
                for component in operands.iter_mut() {
                    analyze_expression(signatures, scope, component)?;
                }
                expression.data_type = operands[0].data_type.merge_numeric(&operands[1].data_type)
                        .ok_or_else(|| DesmosifyError::new(
                            format!("cannot create a point of types ({}, {})", operands[0].data_type, operands[1].data_type),
                            expression.start,
                            expression.end,
                        ))?
                        .point_type(operands.len()).unwrap();
                if let (Some(x_value), Some(y_value)) = (operands[0].constant_value(), operands[1].constant_value()) {
                    let value = match (x_value, y_value) {
                        (&ConstantValue::Real(x_value), &ConstantValue::Real(y_value)) => {
                            ConstantValue::Point(x_value, y_value)
                        },
                        (&ConstantValue::Int(x_value), &ConstantValue::Real(y_value)) => {
                            ConstantValue::Point(x_value as f64, y_value)
                        },
                        (&ConstantValue::Real(x_value), &ConstantValue::Int(y_value)) => {
                            ConstantValue::Point(x_value, y_value as f64)
                        },
                        (&ConstantValue::Int(x_value), &ConstantValue::Int(y_value)) => {
                            ConstantValue::IPoint(x_value, y_value)
                        },
                        _ => return Err(DesmosifyError::new(
                            format!("cannot create a point of types ({}, {})", operands[0].data_type, operands[1].data_type),
                            expression.start,
                            expression.end,
                        ))
                    };

                    expression.value = ExpressionValue::Literal(value);
                }
                Ok(())
            },

            Operation::ListLiteral => {
                let mut item_type = DataType::Unknown;
                for item in operands.iter_mut() {
                    analyze_expression(signatures, scope, item)?;
                    item_type = item_type.merge(&item.data_type)
                            .ok_or_else(|| DesmosifyError::new(
                                format!("unexpected item type '{}'", item.data_type),
                                item.start,
                                item.end,
                            ))?;
                }
                expression.data_type = item_type.list_type()
                        .ok_or_else(|| DesmosifyError::new(
                            format!("cannot create a list of type '{item_type}'"),
                            expression.start,
                            expression.end,
                        ))?;
                Ok(())
            },

            Operation::ListFill => {
                for operand in operands.iter_mut() {
                    analyze_expression(signatures, scope, operand)?;
                }
                if !operands[1].data_type.can_coerce_to(&DataType::Int) {
                    Err(DesmosifyError::new(
                        message_cannot_coerce(&operands[1].data_type, &DataType::Int),
                        operands[1].start,
                        operands[1].end,
                    ))
                } else {
                    expression.data_type = operands[0].data_type.list_type()
                            .ok_or_else(|| DesmosifyError::new(
                                format!("cannot create a list of type '{}'", operands[0].data_type),
                                operands[0].start,
                                operands[0].end,
                            ))?;
                    Ok(())
                }
            },

            Operation::ListMap => Ok(()),
            Operation::ListFilter => Ok(()),
            Operation::MemberAccess => Ok(()),
            Operation::BuiltIn => Ok(()),
            Operation::Call => Ok(()),
            Operation::ActionCall => Ok(()),
            Operation::Index => Ok(()),
            Operation::Posate => Ok(()),
            Operation::Negate => Ok(()),
            Operation::Not => Ok(()),
            Operation::Exponent => Ok(()),
            Operation::Multiply => Ok(()),
            Operation::Divide => Ok(()),
            Operation::Modulus => Ok(()),
            Operation::Add => Ok(()),
            Operation::Subtract => Ok(()),
            Operation::LessThan => Ok(()),
            Operation::GreaterThan => Ok(()),
            Operation::LessEqual => Ok(()),
            Operation::GreaterEqual => Ok(()),
            Operation::Equal => Ok(()),
            Operation::NotEqual => Ok(()),
            Operation::And => Ok(()),
            Operation::Or => Ok(()),
            Operation::ExclusiveRange => Ok(()),
            Operation::InclusiveRange => Ok(()),
            Operation::Conditional => Ok(()),
            Operation::Assignment => Ok(()),
            Operation::Update => Ok(()),
            Operation::With => Ok(()),
        },
        _ => Ok(())
    }
}

pub fn analyze_action(signatures: &Signatures, scope: &Scope, action: &mut Action) -> Result<(), DesmosifyError> {
    // TODO: check concurrent modification
    match action {
        Action::Block(sub_actions) => {
            for sub_action in sub_actions {
                analyze_action(signatures, scope, sub_action)?;
            }
            Ok(())
        },
        Action::Update(target, value) => {
            if let ExpressionValue::Name(name) = &target.value {
                if let Some(Signature::Var { value_type, .. }) = signatures.user_defined.get(name) {
                    analyze_expression(signatures, scope, value.as_mut())?;
                    if value.data_type.can_coerce_to(value_type) {
                        Ok(())
                    } else {
                        Err(DesmosifyError::new(
                            message_cannot_coerce(&value.data_type, value_type),
                            target.start,
                            target.end,
                        ))
                    }
                } else {
                    Err(DesmosifyError::new(
                        format!("cannot update the value of '{name}' as it is not declared with 'var'"),
                        target.start,
                        target.end,
                    ))
                }
            } else {
                Err(DesmosifyError::new(
                    String::from("expected a variable name"),
                    target.start,
                    target.end,
                ))
            }
        },
        Action::Call(callee, arguments) => {
            if let ExpressionValue::Name(name) = &callee.value {
                for argument in arguments.iter_mut() {
                    analyze_expression(signatures, scope, argument)?;
                }
                if let Some(Signature::Action { parameters, .. }) = signatures.user_defined.get(name) {
                    if arguments.len() == parameters.len() {
                        arguments.iter().zip(parameters.iter())
                                .find(|&(argument, parameter)| !argument.data_type.can_coerce_to(&parameter.data_type))
                                .map_or(Ok(()), |(argument, parameter)| Err(DesmosifyError::new(
                                    message_cannot_coerce(&argument.data_type, &parameter.data_type),
                                    argument.start,
                                    argument.end,
                                )))
                    } else {
                        Err(DesmosifyError::new(
                            format!("action {name} expects {} argument(s), but was provided {}", parameters.len(), arguments.len()),
                            callee.start,
                            callee.end,
                        ))
                    }
                } else {
                    Err(DesmosifyError::new(
                        format!("could not find an action named '{name}'"),
                        callee.start,
                        callee.end,
                    ))
                }
            } else {
                Err(DesmosifyError::new(
                    String::from("expected an action name"),
                    callee.start,
                    callee.end,
                ))
            }
        },
        Action::Conditional(branches, default_branch) => {
            for (condition, branch) in branches {
                analyze_expression(signatures, scope, condition)?;
                analyze_action(signatures, scope, branch)?;
            }
            if let Some(default_branch) = default_branch {
                analyze_action(signatures, scope, default_branch.as_mut())
            } else {
                Ok(())
            }
        },
    }
}

pub fn analyze_identifier(signatures: &Signatures, scope: &Scope, signature: &Signature, value: &mut Expression) -> Result<(), DesmosifyError> {
    match signature {
        Signature::Const { name, parameters, value_type, .. } => {
            if let Some(parameters) = parameters {
                let mut call_scope = scope.clone();
                for Parameter { name, data_type } in parameters {
                    call_scope.parameters.insert(name.clone(), data_type.clone());
                }
                analyze_expression(signatures, &call_scope, value)?;
            } else {
                analyze_expression(signatures, scope, value)?;
            }
            if value.constant_value().is_none() {
                Err(DesmosifyError::new(
                    format!("the definition of const {name} could not be evaluated at compile-time"),
                    value.start,
                    value.end,
                ))
            } else if !value.data_type.can_coerce_to(value_type) {
                Err(DesmosifyError::new(
                    message_cannot_coerce(&value.data_type, &value_type),
                    value.start,
                    value.end,
                ))
            } else {
                Ok(())
            }
        },
        Signature::Let { parameters, value_type, .. } => {
            if let Some(parameters) = parameters {
                let mut call_scope = scope.clone();
                for Parameter { name, data_type } in parameters {
                    call_scope.parameters.insert(name.clone(), data_type.clone());
                }
                analyze_expression(signatures, &call_scope, value)?;
            } else {
                analyze_expression(signatures, scope, value)?;
            }
            if !value.data_type.can_coerce_to(value_type) {
                Err(DesmosifyError::new(
                    message_cannot_coerce(&value.data_type, &value_type),
                    value.start,
                    value.end,
                ))
            } else {
                Ok(())
            }
        },
        Signature::Var { name, value_type, .. } => {
            analyze_expression(signatures, scope, value)?;
            if value.constant_value().is_none() {
                Err(DesmosifyError::new(
                    format!("var {name} must be initialized with a constant value"),
                    value.start,
                    value.end,
                ))
            } else if !value.data_type.can_coerce_to(value_type) {
                Err(DesmosifyError::new(
                    message_cannot_coerce(&value.data_type, &value_type),
                    value.start,
                    value.end,
                ))
            } else {
                Ok(())
            }
        },
        Signature::Enum { name, variants } => {
            Ok(())
        },
        _ => Err(DesmosifyError::new(
            format!("got unexpected signature '{}' for {}", signature.variant_name(), signature.name()),
            None,
            None,
        ))
    }
}

pub fn analyze_named_action(signatures: &Signatures, scope: &Scope, signature: &Signature, content: &mut Action) -> Result<(), DesmosifyError> {
    if let Signature::Action { parameters, .. } = signature {
        let mut call_scope = scope.clone();
        for Parameter { name, data_type } in parameters {
            call_scope.parameters.insert(name.clone(), data_type.clone());
        }
        analyze_action(signatures, &call_scope, content)
    } else {
        Err(DesmosifyError::new(
            format!("got unexpected signature '{}' for action '{}'", signature.variant_name(), signature.name()),
            None,
            None,
        ))
    }
}

pub fn analyze_public_element(signatures: &Signatures, scope: &Scope, element: &mut Expression) -> Result<(), DesmosifyError> {
    // TODO: check element type?
    analyze_expression(signatures, scope, element)
}

pub fn analyze_ticker(signatures: &Signatures, scope: &Scope, ticker: &mut Ticker) -> Result<(), DesmosifyError> {
    if let Some(interval_ms) = &mut ticker.interval_ms {
        analyze_expression(signatures, scope, interval_ms.as_mut())?;
    }

    let mut ticker_scope = scope.clone();
    ticker_scope.can_use_dt = true;

    analyze_action(signatures, &ticker_scope, ticker.tick_action.as_mut())
}

pub fn analyze_display_element(signatures: &Signatures, scope: &Scope, element: &mut display::Element) -> Result<(), DesmosifyError> {
    Ok(())
}