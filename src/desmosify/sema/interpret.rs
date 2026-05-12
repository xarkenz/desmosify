use crate::ast::{ActionExpression, ActionExpressionKind, BinaryOperation, DefinitionKind, DisplayAttributeValue, Expression, ExpressionIndexOperation, ExpressionKind, ParameterList, PublicLineKind, TypeDefinition, UnaryOperation, ValueDefinition, VariableKind};
use crate::sema::{Program, ProgramAction, ProgramDisplay, ProgramDisplayAttribute, ProgramDisplayAttributeValue, ProgramDisplayElement, ProgramLet, ProgramPublic, ProgramPublicLine, ProgramTicker, ProgramVariable, ProgramVariableKind};
use crate::sema::context::{GlobalContext, LocalContext};
use crate::sema::intrinsic::IntrinsicValue;
use crate::sema::types::Type;
use crate::sema::values::{ActionValue, GlobalReference, LocalReference, Value, ValueIndexOperation, ValueListMapLoop};
use crate::token::Literal;

pub fn interpret_program(context: &GlobalContext) -> crate::Result<Program> {
    let mut lets = Vec::new();
    let mut variables = Vec::new();
    let mut actions = Vec::new();

    let mut next_local_id = 0;

    for definition in context.definitions().chain(context.action_definitions()) {
        match &definition.definition.kind {
            DefinitionKind::Value(ValueDefinition::Let { parameters, value, .. }) => {
                lets.push(interpret_let_definition(context, &mut next_local_id, parameters.as_ref(), &definition.value_type, value)?);
            }
            DefinitionKind::Value(ValueDefinition::Variable { kind, value, .. }) => {
                variables.push(interpret_variable_definition(context, &mut next_local_id, kind, &definition.value_type, value)?);
            }
            DefinitionKind::Value(ValueDefinition::Action { parameters, action }) => {
                actions.push(interpret_action_definition(context, &mut next_local_id, parameters, &definition.value_type, action)?);
            }
            _ => {}
        }
    }

    Ok(Program {
        lets,
        variables,
        actions,
        ticker: interpret_ticker_declarations(context, &mut next_local_id)?,
        public: interpret_public_declarations(context, &mut next_local_id)?,
        display: interpret_display_declarations(context, &mut next_local_id)?,
    })
}

pub fn interpret_let_definition(context: &GlobalContext, next_local_id: &mut u64, parameters: Option<&ParameterList>, value_type: &Type, value: &Expression) -> crate::Result<ProgramLet> {
    let mut local_context = LocalContext::new();

    let (typed_parameters, expected_type) = if let Some(parameters) = parameters {
        let Type::UserFunction { signature } = value_type else {
            panic!("parameter list is present but value type is not a function")
        };

        let typed_parameters = process_parameters(next_local_id, &mut local_context, parameters, &signature.parameter_types);

        (Some(typed_parameters), &signature.return_type)
    }
    else {
        (None, value_type)
    };

    let value = interpret_expression(context, next_local_id, &local_context, value)?
        .coerce_to(expected_type, false)?;

    Ok(ProgramLet {
        parameters: typed_parameters,
        value,
    })
}

pub fn interpret_variable_definition(context: &GlobalContext, next_local_id: &mut u64, kind: &VariableKind, value_type: &Type, value: &Expression) -> crate::Result<ProgramVariable> {
    let local_context = LocalContext::new();

    let kind = match kind {
        VariableKind::Default => ProgramVariableKind::Default,
        VariableKind::Timer => ProgramVariableKind::Timer,
    };

    let value = interpret_expression(context, next_local_id, &local_context, value)?
        .coerce_to(value_type, false)?;

    Ok(ProgramVariable {
        kind,
        value,
    })
}

pub fn interpret_action_definition(context: &GlobalContext, next_local_id: &mut u64, parameters: &ParameterList, action_type: &Type, action: &ActionExpression) -> crate::Result<ProgramAction> {
    let mut local_context = LocalContext::new();

    let Type::Action { parameter_types } = action_type else {
        panic!("action definition has invalid type")
    };
    let typed_parameters = process_parameters(next_local_id, &mut local_context, parameters, parameter_types);

    let action = interpret_action_expression(context, next_local_id, &local_context, action)?;

    Ok(ProgramAction {
        parameters: typed_parameters,
        action,
    })
}

pub fn process_parameters(next_local_id: &mut u64, local_context: &mut LocalContext, parameters: &ParameterList, parameter_types: &[Type]) -> Box<[LocalReference]> {
    std::iter::zip(&parameters.0, parameter_types)
        .map(|((identifier, _), parameter_type)| {
            local_context.add_local_variable(identifier.clone(), next_local_id, parameter_type.clone())
        })
        .collect()
}

pub fn interpret_ticker_declarations(context: &GlobalContext, next_local_id: &mut u64) -> crate::Result<Option<ProgramTicker>> {
    let (interval_ms, tick_action) = context
        .ticker_declarations()
        .iter()
        .enumerate()
        .try_fold::<_, _, crate::Result<_>>(
            (None, ActionValue::empty()),
            |(mut interval_ms, mut tick_action), (index, declaration)| {
                let mut local_context = LocalContext::new();

                let new_interval_ms = match declaration.interval_ms.as_ref() {
                    Some(interval_expression) => Some(interpret_expression(context, next_local_id, &local_context, interval_expression)?),
                    None => None,
                };
                if index != 0 && new_interval_ms != interval_ms {
                    return Err(Box::new(crate::Error {
                        kind: crate::ErrorKind::IncompatibleTickerIntervals,
                        span: Some(declaration.span),
                    }));
                }
                interval_ms = new_interval_ms;

                local_context.add_scoped_intrinsic("dt", IntrinsicValue::Dt.into());

                let new_tick_action = interpret_action_expression(context, next_local_id, &local_context, &declaration.tick_action)?;
                tick_action = tick_action.merge(new_tick_action);

                Ok((interval_ms, tick_action))
            },
        )?;

    if tick_action.is_empty() {
        Ok(None)
    }
    else {
        Ok(Some(ProgramTicker {
            interval_ms,
            tick_action,
        }))
    }
}

pub fn interpret_public_declarations(context: &GlobalContext, next_local_id: &mut u64) -> crate::Result<Option<ProgramPublic>> {
    // It's fine to create the local context here because it never changes.
    let local_context = LocalContext::new();
    let mut lines = Vec::new();

    for declaration in context.public_declarations() {
        for line in &declaration.lines {
            lines.push(match &line.kind {
                PublicLineKind::Text(text) => {
                    ProgramPublicLine::Text(text.clone())
                }
                PublicLineKind::Expression(expression) => {
                    ProgramPublicLine::Expression(interpret_expression(context, next_local_id, &local_context, expression)?)
                }
                PublicLineKind::Action(action) => {
                    ProgramPublicLine::Action(interpret_action_expression(context, next_local_id, &local_context, action)?)
                }
            });
        }
    }

    if lines.is_empty() {
        Ok(None)
    }
    else {
        Ok(Some(ProgramPublic {
            lines: lines.into_boxed_slice(),
        }))
    }
}

pub fn interpret_display_declarations(context: &GlobalContext, next_local_id: &mut u64) -> crate::Result<Option<ProgramDisplay>> {
    // It's fine to create the local context here because it never changes.
    // (For click actions, just create a fresh context.)
    let local_context = LocalContext::new();
    let mut elements = Vec::new();

    for declaration in context.display_declarations() {
        for element in &declaration.elements {
            elements.push(ProgramDisplayElement {
                expression: interpret_expression(context, next_local_id, &local_context, &element.expression)?,
                attributes: element.attributes
                    .iter()
                    .map(|attribute| Ok(ProgramDisplayAttribute {
                        key: attribute.key.clone(),
                        value: match &attribute.value {
                            DisplayAttributeValue::Arguments(arguments) => {
                                ProgramDisplayAttributeValue::Arguments(arguments
                                    .iter()
                                    .map(|argument| interpret_expression(context, next_local_id, &local_context, argument))
                                    .collect::<crate::Result<_>>()?)
                            }
                            DisplayAttributeValue::Action(action) => {
                                let mut action_context = local_context.new_inner();
                                // TODO: figure out how to do this better
                                action_context.add_scoped_intrinsic("index", IntrinsicValue::Index.into());
                                ProgramDisplayAttributeValue::Action(interpret_action_expression(context, next_local_id, &local_context, action)?)
                            }
                        },
                    }))
                    .collect::<crate::Result<_>>()?,
            });
        }
    }

    if elements.is_empty() {
        Ok(None)
    }
    else {
        Ok(Some(ProgramDisplay {
            elements: elements.into_boxed_slice(),
        }))
    }
}

// TODO: detect multiple updates of same variable
pub fn interpret_action_expression(context: &GlobalContext, next_local_id: &mut u64, local_context: &LocalContext, action: &ActionExpression) -> crate::Result<ActionValue> {
    match &action.kind {
        ActionExpressionKind::Disable => {
            Ok(ActionValue::Disable)
        }
        ActionExpressionKind::Compound { actions } => {
            Ok(ActionValue::Compound {
                actions: actions
                    .iter()
                    .map(|action| interpret_action_expression(context, next_local_id, local_context, action))
                    .collect::<crate::Result<_>>()?,
            })
        }
        ActionExpressionKind::Update { variable, value } => {
            let invalid_update_lhs_error = || Box::new(crate::Error {
                kind: crate::ErrorKind::InvalidUpdateLhs,
                span: Some(variable.span),
            });

            let variable = interpret_expression(context, next_local_id, &local_context, variable)?;
            let Value::Global(variable) = variable else {
                return Err(invalid_update_lhs_error());
            };
            let DefinitionKind::Value(ValueDefinition::Variable { .. }) = context.find_definition(&variable.identifier).unwrap().definition.kind else {
                return Err(invalid_update_lhs_error());
            };

            let value = interpret_expression(context, next_local_id, &local_context, value)?
                .coerce_to(&variable.value_type, false)?;

            Ok(ActionValue::Update {
                variable,
                value: Box::new(value),
            })
        }
        ActionExpressionKind::ActionCall { identifier, arguments } => {
            let Some(definition) = context.find_action_definition(identifier) else {
                return Err(Box::new(crate::Error {
                    kind: crate::ErrorKind::UndefinedAction {
                        identifier: identifier.as_ref().into(),
                    },
                    span: Some(action.span),
                }));
            };
            let Type::Action { parameter_types } = &definition.value_type else {
                panic!("action definition has invalid type")
            };

            if arguments.len() != parameter_types.len() {
                return Err(Box::new(crate::Error {
                    kind: crate::ErrorKind::InvalidArity {
                        expected: parameter_types.len(),
                        got: arguments.len(),
                    },
                    span: Some(action.span),
                }))
            }

            Ok(ActionValue::ActionCall {
                identifier: identifier.clone(),
                arguments: std::iter::zip(arguments, parameter_types)
                    .map(|(argument, parameter_type)| {
                        interpret_expression(context, next_local_id, local_context, argument)?
                            .coerce_to(parameter_type, false)
                    })
                    .collect::<crate::Result<_>>()?,
            })
        }
        ActionExpressionKind::Conditional { condition_consequents, alternative } => {
            Ok(ActionValue::Conditional {
                condition_consequents: condition_consequents
                    .iter()
                    .map(|(condition, consequent)| {
                        let condition = interpret_expression(context, next_local_id, local_context, condition)?
                            .coerce_to(&Type::Bool, false)?;
                        let consequent = interpret_action_expression(context, next_local_id, local_context, consequent)?;

                        Ok((condition, consequent))
                    })
                    .collect::<crate::Result<_>>()?,
                alternative: match alternative {
                    Some(alternative) => Box::new(interpret_action_expression(context, next_local_id, local_context, alternative)?),
                    None => Box::new(ActionValue::empty()),
                },
            })
        }
    }
}

pub fn interpret_expression(context: &GlobalContext, next_local_id: &mut u64, local_context: &LocalContext, expression: &Expression) -> crate::Result<Value> {
    match &expression.kind {
        ExpressionKind::Literal(Literal::Identifier(identifier)) => {
            if let Some(value) = local_context.find_local(identifier) {
                Ok(value.clone())
            }
            else if let Some(definition) = context.find_definition(identifier) {
                Ok(Value::Global(GlobalReference {
                    identifier: identifier.clone(),
                    value_type: definition.value_type.clone(),
                }))
            }
            else {
                Err(Box::new(crate::Error {
                    kind: crate::ErrorKind::UndefinedIdentifier {
                        identifier: identifier.as_ref().into(),
                    },
                    span: Some(expression.span),
                }))
            }
        }
        ExpressionKind::Literal(Literal::Real(value)) => {
            Ok(Value::Real(*value))
        }
        ExpressionKind::Literal(Literal::Integer(value)) => {
            let value = i64::try_from(*value).map_err(|_| Box::new(crate::Error {
                kind: crate::ErrorKind::IntegerTooLarge,
                span: Some(expression.span),
            }))?;

            Ok(Value::Int(value))
        }
        ExpressionKind::Literal(Literal::Boolean(value)) => {
            Ok(Value::Bool(*value))
        }
        ExpressionKind::Literal(Literal::String(value)) => {
            Ok(Value::Str(value.clone()))
        }
        ExpressionKind::Intrinsic(identifier) => {
            if let Some(intrinsic) = local_context.find_scoped_intrinsic(identifier) {
                Ok(intrinsic.clone())
            }
            else if let Some(intrinsic) = context.find_intrinsic(identifier) {
                Ok(intrinsic.clone())
            }
            else {
                Err(Box::new(crate::Error {
                    kind: crate::ErrorKind::UndefinedIntrinsic {
                        identifier: identifier.as_ref().into(),
                    },
                    span: Some(expression.span),
                }))
            }
        }
        ExpressionKind::Grouping { expression } => {
            interpret_expression(context, next_local_id, local_context, expression)
        }
        ExpressionKind::Unary { operation, operand } => {
            todo!()
        }
        ExpressionKind::Binary { operation, lhs, rhs } => {
            todo!()
        }
        ExpressionKind::Point2 { x, y } => {
            let x = interpret_expression(context, next_local_id, local_context, x)?;
            let y = interpret_expression(context, next_local_id, local_context, y)?;

            let (x_is_list, x_type) = x.get_type().into_flatten_list();
            let (y_is_list, y_type) = y.get_type().into_flatten_list();

            Ok(Value::Point2 {
                x: Box::new(x),
                y: Box::new(y),
                point_type: Type::Point2 {
                    x_type: Box::new(x_type),
                    y_type: Box::new(y_type),
                }.unflatten_list(x_is_list || y_is_list),
            })
        }
        ExpressionKind::Point3 { x, y, z } => {
            let x = interpret_expression(context, next_local_id, local_context, x)?;
            let y = interpret_expression(context, next_local_id, local_context, y)?;
            let z = interpret_expression(context, next_local_id, local_context, z)?;

            let (x_is_list, x_type) = x.get_type().into_flatten_list();
            let (y_is_list, y_type) = y.get_type().into_flatten_list();
            let (z_is_list, z_type) = z.get_type().into_flatten_list();

            Ok(Value::Point3 {
                x: Box::new(x),
                y: Box::new(y),
                z: Box::new(z),
                point_type: Type::Point3 {
                    x_type: Box::new(x_type),
                    y_type: Box::new(y_type),
                    z_type: Box::new(z_type),
                }.unflatten_list(x_is_list || y_is_list || z_is_list),
            })
        }
        ExpressionKind::List { items } => {
            let items: Vec<_> = items
                .iter()
                .map(|item| interpret_expression(context, next_local_id, local_context, item))
                .collect::<crate::Result<_>>()?;

            let item_type = items
                .iter()
                .try_fold(Type::Any, |current_type, item| current_type.merge(&item.get_type()))?;

            Ok(Value::List {
                items: items
                    .into_iter()
                    .map(|item| item.coerce_to(&item_type, false))
                    .collect::<crate::Result<_>>()?,
                item_type,
            })
        }
        ExpressionKind::ListRange { kind, start, end, step } => {
            let start = interpret_expression(context, next_local_id, local_context, start)?;
            let end = interpret_expression(context, next_local_id, local_context, end)?;
            let step = match step {
                Some(step) => interpret_expression(context, next_local_id, local_context, step)?,
                None => Value::Int(1),
            };

            let item_type = start.get_type()
                .merge(&end.get_type())?
                .merge(&step.get_type())?;

            Ok(Value::ListRange {
                kind: *kind,
                start: Box::new(start.coerce_to(&item_type, false)?),
                end: Box::new(end.coerce_to(&item_type, false)?),
                step: Box::new(step.coerce_to(&item_type, false)?),
                item_type,
            })
        }
        ExpressionKind::ListFill { value, count } => {
            let value = interpret_expression(context, next_local_id, local_context, value)?;
            let count = interpret_expression(context, next_local_id, local_context, count)?
                .coerce_to(&Type::Int, false)?;

            Ok(Value::ListFill {
                value: Box::new(value),
                count: Box::new(count),
            })
        }
        ExpressionKind::ListMap { loops, expression: map_expression } => {
            let mut map_context = local_context.new_inner();

            let loops = loops
                .iter()
                .map(|map_loop| {
                    let list = interpret_expression(context, next_local_id, local_context, &map_loop.list)?;
                    let item_type = list.get_type().require_flatten_list(Some(expression.span))?;

                    Ok(ValueListMapLoop {
                        local: map_context.add_local_variable(map_loop.identifier.clone(), next_local_id, item_type),
                        list,
                    })
                })
                .collect::<crate::Result<_>>()?;

            let value = interpret_expression(context, next_local_id, &map_context, map_expression)?;

            Ok(Value::ListMap {
                loops,
                value: Box::new(value),
            })
        }
        ExpressionKind::ListFilter { list, condition } => {
            let list = interpret_expression(context, next_local_id, local_context, list)?;
            let item_type = list.get_type().require_flatten_list(Some(expression.span))?;

            let condition = interpret_expression(context, next_local_id, local_context, condition)?
                .coerce_to(&Type::Bool, true)?;

            Ok(Value::ListFilter {
                list: Box::new(list),
                condition: Box::new(condition),
                item_type,
            })
        }
        ExpressionKind::Index { list, operation } => {
            let list = interpret_expression(context, next_local_id, local_context, list)?;
            let item_type = list.get_type().require_flatten_list(Some(expression.span))?;

            let operation = interpret_index_operation(context, next_local_id, local_context, operation)?;

            Ok(Value::Index {
                list: Box::new(list),
                operation,
                item_type,
            })
        }
        ExpressionKind::FunctionCall { function, arguments } => {
            let function_value = interpret_expression(context, next_local_id, local_context, function)?;
            let arguments: Box<[_]> = arguments
                .iter()
                .map(|argument| interpret_expression(context, next_local_id, local_context, argument))
                .collect::<crate::Result<_>>()?;

            match function_value.get_type() {
                Type::IntrinsicFunction(function) => {
                    function.interpret_call(context, arguments)
                }
                Type::UserFunction { signature } => {
                    if arguments.len() != signature.parameter_types.len() {
                        return Err(Box::new(crate::Error {
                            kind: crate::ErrorKind::InvalidArity {
                                expected: signature.parameter_types.len(),
                                got: arguments.len(),
                            },
                            span: Some(expression.span),
                        }));
                    }

                    // FIXME: this is not type-safe in terms of broadcasting
                    let arguments = std::iter::zip(arguments, &signature.parameter_types)
                        .map(|(argument, parameter_type)| argument.coerce_to(parameter_type, true))
                        .collect::<crate::Result<_>>()?;

                    Ok(Value::UserFunctionCall {
                        function: Box::new(function_value),
                        arguments,
                        return_type: signature.return_type,
                    })
                }
                got_type => {
                    Err(Box::new(crate::Error {
                        kind: crate::ErrorKind::ExpectedFunctionType {
                            got_type: got_type.to_string(),
                        },
                        span: Some(function.span),
                    }))
                }
            }
        }
        ExpressionKind::Conditional { condition_consequents, alternative } => {
            let mut result_is_list = false;
            let mut result_type = Type::Any;

            let condition_consequents: Box<[_]> = condition_consequents
                .iter()
                .map(|(condition, consequent)| {
                    let condition = interpret_expression(context, next_local_id, local_context, condition)?
                        .coerce_to(&Type::Bool, true)?;
                    let consequent = interpret_expression(context, next_local_id, local_context, consequent)?;
                    let (consequent_is_list, consequent_type) = consequent.get_type().into_flatten_list();
                    result_is_list |= condition.get_type().is_list() || consequent_is_list;
                    result_type = result_type.merge(&consequent_type)?;

                    Ok((condition, consequent))
                })
                .collect::<crate::Result<_>>()?;

            let alternative = alternative
                .as_ref()
                .map_or(Ok(Value::Undefined), |alternative| {
                    let alternative = interpret_expression(context, next_local_id, local_context, alternative)?;
                    let (alternative_is_list, alternative_type) = alternative.get_type().into_flatten_list();
                    result_is_list |= alternative_is_list;
                    result_type = result_type.merge(&alternative_type)?;

                    alternative.coerce_to(&result_type, true)
                })?;

            let condition_consequents = condition_consequents
                .into_iter()
                .map(|(condition, consequent)| {
                    Ok((condition, consequent.coerce_to(&result_type, true)?))
                })
                .collect::<crate::Result<_>>()?;

            Ok(Value::Conditional {
                condition_consequents,
                alternative: Box::new(alternative),
                result_type: result_type.unflatten_list(result_is_list),
            })
        }
        ExpressionKind::Let { identifier, value_type, value, expression } => {
            let mut value = interpret_expression(context, next_local_id, local_context, value)?;

            let value_type = match value_type {
                Some(type_expression) => {
                    let value_type = context.resolve_type(type_expression)?;
                    value = value.coerce_to(&value_type, false)?;
                    value_type
                }
                None => {
                    value.get_type()
                }
            };

            let mut inner_context = local_context.new_inner();
            let local = inner_context.add_local_variable(identifier.clone(), next_local_id, value_type);

            let inner = interpret_expression(context, next_local_id, local_context, expression)?;

            Ok(Value::Let {
                local,
                value: Box::new(value),
                inner: Box::new(inner),
            })
        }
        _ => {
            Err(Box::new(crate::Error {
                kind: crate::ErrorKind::UnexpectedExpressionKind,
                span: Some(expression.span),
            }))
        }
    }
}

pub fn interpret_unary_operation(context: &GlobalContext, next_local_id: &mut u64, local_context: &LocalContext, operation: UnaryOperation, operand: &Expression) -> crate::Result<Value> {
    let mut operand = interpret_expression(context, next_local_id, local_context, operand)?;
    let (is_list, mut operand_type) = operand.get_type().into_flatten_list();

    match operation {
        UnaryOperation::Positive | UnaryOperation::Negative => {
            // The result must be arithmetic
            (operand, operand_type) = operand.coerce_to_arithmetic(Type::require_numeric_or_point)?;
        }
        UnaryOperation::LogicalNot => {
            operand = operand.coerce_to(&Type::Bool, true)?;
            operand_type = Type::Bool;
        }
    }

    Ok(Value::Unary {
        operation,
        operand: Box::new(operand),
        result_type: operand_type.unflatten_list(is_list),
    })
}

pub fn interpret_binary_operation(context: &GlobalContext, next_local_id: &mut u64, local_context: &LocalContext, operation: BinaryOperation, lhs: &Expression, rhs: &Expression) -> crate::Result<Value> {
    if let BinaryOperation::MemberAccess = operation {
        return interpret_access_operation(context, next_local_id, local_context, lhs, rhs);
    }

    let mut lhs = interpret_expression(context, next_local_id, local_context, lhs)?;
    let mut rhs = interpret_expression(context, next_local_id, local_context, rhs)?;
    let (lhs_is_list, mut lhs_type) = lhs.get_type().into_flatten_list();
    let (rhs_is_list, mut rhs_type) = rhs.get_type().into_flatten_list();

    let result_type = match operation {
        BinaryOperation::MemberAccess => {
            // This case has already been dealt with above
            unreachable!()
        }
        BinaryOperation::Exponent |
        BinaryOperation::Remainder => {
            // The result must be arithmetic and cannot be a point
            (lhs, lhs_type) = lhs.coerce_to_arithmetic(Type::require_numeric)?;
            (rhs, rhs_type) = rhs.coerce_to_arithmetic(Type::require_numeric)?;
            Type::merge(&lhs_type, &rhs_type)?
        }
        BinaryOperation::Multiply => {
            // The result must be arithmetic, but at most one operand may be a point
            (lhs, lhs_type) = lhs.coerce_to_arithmetic(Type::require_numeric_or_point)?;
            if matches!(lhs_type, Type::Point2 { .. } | Type::Point3 { .. }) {
                (rhs, rhs_type) = rhs.coerce_to_arithmetic(Type::require_numeric)?;
                match &lhs_type {
                    Type::Point2 { x_type, y_type } => Type::Point2 {
                        x_type: Box::new(x_type.merge(&rhs_type)?),
                        y_type: Box::new(y_type.merge(&rhs_type)?),
                    },
                    Type::Point3 { x_type, y_type, z_type } => Type::Point3 {
                        x_type: Box::new(x_type.merge(&rhs_type)?),
                        y_type: Box::new(y_type.merge(&rhs_type)?),
                        z_type: Box::new(z_type.merge(&rhs_type)?),
                    },
                    _ => unreachable!()
                }
            }
            else {
                (rhs, rhs_type) = rhs.coerce_to_arithmetic(Type::require_numeric_or_point)?;
                match &rhs_type {
                    Type::Point2 { x_type, y_type } => Type::Point2 {
                        x_type: Box::new(x_type.merge(&lhs_type)?),
                        y_type: Box::new(y_type.merge(&lhs_type)?),
                    },
                    Type::Point3 { x_type, y_type, z_type } => Type::Point3 {
                        x_type: Box::new(x_type.merge(&lhs_type)?),
                        y_type: Box::new(y_type.merge(&lhs_type)?),
                        z_type: Box::new(z_type.merge(&lhs_type)?),
                    },
                    _ => Type::merge(&lhs_type, &rhs_type)?
                }
            }
        }
        BinaryOperation::Divide => {
            // The result is always assumed to be real, but lhs may be a point
            let result_type = match &lhs_type {
                Type::Point2 { .. } => Type::Point2 {
                    x_type: Box::new(Type::Real),
                    y_type: Box::new(Type::Real),
                },
                Type::Point3 { .. } => Type::Point3 {
                    x_type: Box::new(Type::Real),
                    y_type: Box::new(Type::Real),
                    z_type: Box::new(Type::Real),
                },
                _ => Type::Real
            };
            lhs = lhs.coerce_to(&result_type, true)?;
            rhs = rhs.coerce_to(&Type::Real, true)?;
            result_type
        }
        BinaryOperation::Add |
        BinaryOperation::Subtract => {
            // The result must be arithmetic, but may be a point
            (lhs, lhs_type) = lhs.coerce_to_arithmetic(Type::require_numeric_or_point)?;
            (rhs, rhs_type) = rhs.coerce_to_arithmetic(Type::require_numeric_or_point)?;
            Type::merge(&lhs_type, &rhs_type)?
        }
        BinaryOperation::LessThan |
        BinaryOperation::LessEqual |
        BinaryOperation::GreaterThan |
        BinaryOperation::GreaterEqual => {
            // The operands must merge into a numeric type, but the result is always a bool
            Type::merge(&lhs_type, &rhs_type)?.require_numeric()?;
            Type::Bool
        }
        BinaryOperation::Equal |
        BinaryOperation::NotEqual => {
            // The operands must merge into a numeric or point type, but the result is always a bool
            Type::merge(&lhs_type, &rhs_type)?.require_numeric_or_point()?;
            Type::Bool
        }
        BinaryOperation::LogicalAnd |
        BinaryOperation::LogicalOr => {
            lhs = lhs.coerce_to(&Type::Bool, true)?;
            rhs = rhs.coerce_to(&Type::Bool, true)?;
            Type::Bool
        }
    };

    Ok(Value::Binary {
        operation,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
        result_type: result_type.unflatten_list(lhs_is_list || rhs_is_list),
    })
}

fn interpret_access_operation(context: &GlobalContext, next_local_id: &mut u64, local_context: &LocalContext, lhs: &Expression, rhs: &Expression) -> crate::Result<Value> {
    let lhs = interpret_expression(context, next_local_id, local_context, lhs)?;
    let (lhs_is_list, lhs_type) = lhs.get_type().into_flatten_list();

    let ExpressionKind::Literal(Literal::Identifier(member_identifier)) = &rhs.kind else {
        return Err(Box::new(crate::Error {
            kind: crate::ErrorKind::ExpectedIdentifier,
            span: Some(rhs.span),
        }));
    };

    let invalid_access_error = || Box::new(crate::Error {
        kind: crate::ErrorKind::InvalidAccessOperation {
            lhs_type: lhs_type.to_string(),
            rhs: member_identifier.as_ref().into(),
        },
        span: Some(rhs.span),
    });

    match &lhs_type {
        Type::Meta { identifier } => {
            let definition = context.find_definition(&identifier).unwrap();
            let DefinitionKind::Type(definition) = &definition.definition.kind else {
                panic!("known type does not have a type definition")
            };

            match definition {
                TypeDefinition::Enumeration { variants } => {
                    let ordinal = variants
                        .iter()
                        .enumerate()
                        .find_map(|(ordinal, variant)| {
                            (member_identifier == &variant.identifier).then_some(ordinal)
                        })
                        .ok_or_else(|| Box::new(crate::Error {
                            kind: crate::ErrorKind::UndefinedEnumVariant {
                                enum_identifier: identifier.as_ref().into(),
                                variant_identifier: member_identifier.as_ref().into(),
                            },
                            span: Some(rhs.span),
                        }))?;

                    Ok(Value::EnumVariant {
                        type_identifier: identifier.clone(),
                        variant_ordinal: ordinal as i64,
                    })
                }
            }
        }
        Type::Point2 { x_type, y_type } => {
            match member_identifier.as_ref() {
                "x" => Ok(Value::GetX {
                    point: Box::new(lhs),
                    x_type: x_type.as_ref().clone().unflatten_list(lhs_is_list),
                }),
                "y" => Ok(Value::GetY {
                    point: Box::new(lhs),
                    y_type: y_type.as_ref().clone().unflatten_list(lhs_is_list),
                }),
                _ => Err(invalid_access_error())
            }
        }
        Type::Point3 { x_type, y_type, z_type } => {
            match member_identifier.as_ref() {
                "x" => Ok(Value::GetX {
                    point: Box::new(lhs),
                    x_type: x_type.as_ref().clone().unflatten_list(lhs_is_list),
                }),
                "y" => Ok(Value::GetY {
                    point: Box::new(lhs),
                    y_type: y_type.as_ref().clone().unflatten_list(lhs_is_list),
                }),
                "z" => Ok(Value::GetZ {
                    point: Box::new(lhs),
                    z_type: z_type.as_ref().clone().unflatten_list(lhs_is_list),
                }),
                _ => Err(invalid_access_error())
            }
        }
        _ => Err(invalid_access_error())
    }
}

pub fn interpret_index_operation(context: &GlobalContext, next_local_id: &mut u64, local_context: &LocalContext, operation: &ExpressionIndexOperation) -> crate::Result<ValueIndexOperation> {
    match operation {
        ExpressionIndexOperation::Single { index } => {
            let index = interpret_expression(context, next_local_id, local_context, index)?
                .coerce_to(&Type::Int, true)?;

            Ok(ValueIndexOperation::Single {
                index: Box::new(index),
            })
        }
        ExpressionIndexOperation::Range { kind, from_index, to_index, step } => {
            let from_index = interpret_expression(context, next_local_id, local_context, from_index)?
                .coerce_to(&Type::Int, false)?;
            let to_index = interpret_expression(context, next_local_id, local_context, to_index)?
                .coerce_to(&Type::Int, false)?;
            let step = match step {
                Some(step) => interpret_expression(context, next_local_id, local_context, step)?
                    .coerce_to(&Type::Int, false)?,
                None => Value::Int(1),
            };

            Ok(ValueIndexOperation::Range {
                kind: *kind,
                from_index: Box::new(from_index),
                to_index: Box::new(to_index),
                step: Box::new(step),
            })
        }
        ExpressionIndexOperation::RangeFrom { from_index, step } => {
            let from_index = interpret_expression(context, next_local_id, local_context, from_index)?
                .coerce_to(&Type::Int, false)?;
            let step = match step {
                Some(step) => interpret_expression(context, next_local_id, local_context, step)?
                    .coerce_to(&Type::Int, false)?,
                None => Value::Int(1),
            };

            Ok(ValueIndexOperation::RangeFrom {
                from_index: Box::new(from_index),
                step: Box::new(step),
            })
        }
        ExpressionIndexOperation::RangeTo { kind, to_index } => {
            let to_index = interpret_expression(context, next_local_id, local_context, to_index)?
                .coerce_to(&Type::Int, false)?;

            Ok(ValueIndexOperation::RangeTo {
                kind: *kind,
                to_index: Box::new(to_index),
            })
        }
    }
}
