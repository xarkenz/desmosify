use std::rc::Rc;
use crate::ast::{ActionExpression, ActionExpressionKind, BinaryOperation, DefinitionKind, DisplayAttribute, Expression, IndexOperation, ExpressionKind, ParameterList, PublicLineKind, TypeDefinition, UnaryOperation, ValueDefinition, VariableKind, EnumerationVariant, PublicLine, Declaration, TickerDeclaration, PublicDeclaration, DisplayDeclaration};
use crate::sema::{Program, ProgramAction, ProgramEnumeration, ProgramImmutable, ProgramPublic, ProgramPublicEntry, ProgramPublicLine, ProgramTicker, ProgramVariable, ProgramVariableKind};
use crate::sema::context::{GlobalContext, LocalContext};
use crate::sema::display::{DragMode, LabelOrientation, LineStyle, PointStyle, ProgramDisplay, ProgramDisplayAttribute, ProgramDisplayAttributeKind, ProgramDisplayElement};
use crate::sema::types::{ListState, Type, TypeHandle};
use crate::sema::values::{ActionValue, ActionValueKind, GlobalSymbol, IndexKind, Value, ListMapLoop, BinaryKind, UnaryKind, InequalityKind, TernaryKind, ValueHandle, ValueEntry, GlobalSymbolKind};
use crate::target::Target;

pub fn interpret_program(context: &mut GlobalContext, declarations: &[Declaration]) -> crate::Result<Program> {
    let mut enumerations = Vec::new();
    let mut immutables = Vec::new();
    let mut variables = Vec::new();
    let mut actions = Vec::new();

    let mut ticker_declarations = Vec::new();
    let mut public_declarations = Vec::new();
    let mut display_declarations = Vec::new();

    for declaration in declarations {
        let local_context = context.new_local_context(declaration.span().source_id);

        match declaration {
            Declaration::Definition(definition) => match &definition.kind {
                DefinitionKind::Type(TypeDefinition::Enumeration { variants }) => {
                    enumerations.push(interpret_enumeration_definition(
                        context,
                        local_context,
                        definition.identifier.clone(),
                        variants,
                    )?);
                }
                DefinitionKind::Value(ValueDefinition::Let { parameters, value, .. }) => {
                    immutables.push(interpret_let_definition(
                        context,
                        local_context,
                        definition.identifier.clone(),
                        parameters.as_ref(),
                        value,
                    )?);
                }
                DefinitionKind::Value(ValueDefinition::Variable { kind, value, .. }) => {
                    variables.push(interpret_variable_definition(
                        context,
                        local_context,
                        definition.identifier.clone(),
                        kind,
                        value,
                    )?);
                }
                DefinitionKind::Value(ValueDefinition::Action { parameters, action }) => {
                    actions.push(interpret_action_definition(
                        context,
                        local_context,
                        definition.identifier.clone(),
                        parameters,
                        action,
                    )?);
                }
            }
            Declaration::Ticker(ticker_declaration) => {
                ticker_declarations.push(ticker_declaration);
            }
            Declaration::Public(public_declaration) => {
                public_declarations.push(public_declaration);
            }
            Declaration::Display(display_declaration) => {
                display_declarations.push(display_declaration);
            }
        }
    }

    let ticker = interpret_ticker_declarations(context, ticker_declarations)?;
    let public = interpret_public_declarations(context, &mut variables, public_declarations)?;
    let display = interpret_display_declarations(context, display_declarations)?;

    Ok(Program {
        enumerations: enumerations.into_boxed_slice(),
        immutables: immutables.into_boxed_slice(),
        variables: variables.into_boxed_slice(),
        actions: actions.into_boxed_slice(),
        ticker,
        public,
        display,
    })
}

pub fn interpret_enumeration_definition(
    context: &mut GlobalContext,
    local_context: LocalContext,
    identifier: Rc<str>,
    variants: &[EnumerationVariant],
) -> crate::Result<ProgramEnumeration> {
    let Some(&Value::Type(type_handle)) = context.find_global(&identifier)
        .map(|global| context.values.get(global.value))
    else {
        panic!("no enum '{identifier}' found in context")
    };
    let Type::Enum { values, .. } = context.types.get(type_handle) else {
        panic!("invalid type definition for enum '{identifier}'")
    };
    let ordinals: Vec<_> = values
        .iter()
        .map(|&(_, ordinal)| ordinal)
        .collect();

    let mut previous_value: Option<(Rc<str>, ValueHandle)> = None;
    for (variant, ordinal) in std::iter::zip(variants, ordinals) {
        let ordinal_value = if let Some(value) = &variant.value {
            // Use the explicit value as the ordinal.
            interpret_expression(context, &local_context, value)?.value
        } else if let Some((previous_identifier, previous_ordinal)) = previous_value.take() {
            // Use the ordinal of the previous value plus one.
            let previous_reference = context.values.register(ValueEntry {
                value: Value::GlobalReference(GlobalSymbol {
                    kind: GlobalSymbolKind::EnumOrdinal,
                    identifier: previous_identifier,
                    value: previous_ordinal,
                }),
                type_handle,
                span: None,
            });
            Value::Binary {
                kind: BinaryKind::Add,
                lhs: previous_reference,
                rhs: ValueHandle::ONE_INT,
            }
        } else {
            // No previous value, so start at zero.
            Value::Int(0)
        };

        context.values.replace(ordinal, ordinal_value);
        previous_value = Some((variant.identifier.clone(), ordinal));
    }

    Ok(ProgramEnumeration {
        identifier,
        type_handle,
    })
}

pub fn interpret_let_definition(
    context: &mut GlobalContext,
    mut local_context: LocalContext,
    identifier: Rc<str>,
    parameters: Option<&ParameterList>,
    value: &Expression,
) -> crate::Result<ProgramImmutable> {
    let (typed_parameters, expected_type) = if let Some(parameters) = parameters {
        let Type::Function { signature } = value_type else {
            panic!("parameter list is present but value type is not a function")
        };

        let typed_parameters = process_parameters(target, &mut local_context, parameters, &signature.parameter_types);

        (Some(typed_parameters), &signature.return_type)
    }
    else {
        (None, value_type)
    };

    let value = interpret_expression(target, context, &local_context, value)?
        .coerce_to(expected_type, false)?;

    Ok(ProgramImmutable {
        identifier,
        parameters: typed_parameters,
        value,
    })
}

pub fn interpret_variable_definition(
    context: &mut GlobalContext,
    local_context: LocalContext,
    identifier: Rc<str>,
    kind: &VariableKind,
    value: &Expression,
) -> crate::Result<ProgramVariable> {
    // TODO: timer and slider should be restricted to certain types, should also affect slider step
    let kind = match kind {
        VariableKind::Default => ProgramVariableKind::Default,
        VariableKind::Timer => ProgramVariableKind::Timer,
        VariableKind::Slider { min, max, step } => {
            let mut interpret = |option: Option<&Expression>| {
                option.map(|expression| {
                    interpret_expression(target, context, &local_context, expression)?
                        .coerce_to(value_type, false)
                }).transpose()
            };
            ProgramVariableKind::Slider {
                min: interpret(min.as_deref())?.map(Box::new),
                max: interpret(max.as_deref())?.map(Box::new),
                step: interpret(step.as_deref())?.map(Box::new),
            }
        }
    };

    let value = interpret_expression(target, context, &local_context, value)?
        .coerce_to(value_type, false)?;

    Ok(ProgramVariable {
        identifier,
        kind,
        value,
    })
}

pub fn interpret_action_definition(
    context: &mut GlobalContext,
    mut local_context: LocalContext,
    identifier: Rc<str>,
    parameters: &ParameterList,
    action: &ActionExpression,
) -> crate::Result<ProgramAction> {
    let Type::Action { parameter_types } = action_type else {
        panic!("action definition has invalid type")
    };
    let typed_parameters = process_parameters(target, &mut local_context, parameters, parameter_types);

    let action = interpret_action_expression(target, context, &local_context, action)?;

    Ok(ProgramAction {
        identifier,
        parameters: typed_parameters,
        action,
    })
}

pub fn process_parameters(
    context: &mut GlobalContext,
    local_context: &mut LocalContext,
    parameters: &ParameterList,
    parameter_types: &[TypeHandle],
) -> Box<[ValueHandle]> {
    std::iter::zip(&parameters.0, parameter_types)
        .map(|(parameter, &parameter_type)| {
            let parameter_value = context.values.register(ValueEntry {
                value: Value::Local {
                    id: context.target.create_local_id(),
                },
                type_handle: parameter_type,
                span: Some(parameter.identifier_span),
            });
            local_context.add_local(parameter.identifier.clone(), parameter_value);
            parameter_value
        })
        .collect()
}

pub fn interpret_ticker_declarations<'a>(
    context: &mut GlobalContext,
    ticker_declarations: impl IntoIterator<Item = &'a TickerDeclaration>,
) -> crate::Result<ProgramTicker> {
    let mut interval_ms = None;

    let tick_actions = ticker_declarations
        .iter()
        .enumerate()
        .map(|(index, declaration)| {
            let local_context = context.new_local_context(declaration.span.source_id);

            let new_interval_ms = match declaration.interval_ms.as_ref() {
                Some(interval_expression) => Some(interpret_expression(context, &local_context, interval_expression)?),
                None => None,
            };
            if index != 0 && new_interval_ms != interval_ms {
                return Err(Box::new(crate::Error {
                    kind: crate::ErrorKind::IncompatibleTickerIntervals,
                    span: Some(declaration.span),
                }))
            }
            interval_ms = new_interval_ms;

            let tick_action = interpret_expression(context, &local_context, &declaration.tick_action)?;
            let tick_type = context.values.get_type(tick_action);

            let tick_arguments: Box<[_]> = match context.types.expect_action_type(tick_type, 0, &[TypeHandle::REAL])? {
                &[] => Box::new([]),
                &[dt_type, ..] => Box::new([
                    context.coerce_value(ValueHandle::TICKER_DT, dt_type, false)?,
                ]),
            };

            Ok(ActionValueKind::ActionCall {
                action: tick_action,
                arguments: tick_arguments,
            }.with_span(None))
        })
        .collect::<crate::Result<_>>()?;

    Ok(ProgramTicker {
        interval_ms,
        tick_action: ActionValueKind::Compound {
            actions: tick_actions,
        }.into(),
    })
}

pub fn interpret_public_declarations<'a>(
    context: &mut GlobalContext,
    variables: &mut Vec<ProgramVariable>,
    public_declarations: impl IntoIterator<Item = &'a PublicDeclaration>,
) -> crate::Result<ProgramPublic> {
    let mut entries = Vec::new();

    for declaration in context.public_declarations() {
        let local_context = LocalContext::new(&source_paths[declaration.span.source_id]);

        for line in &declaration.lines {
            entries.push(match &line.kind {
                PublicLineKind::Expression(..) |
                PublicLineKind::Action(..) |
                PublicLineKind::Slider { .. } => {
                    ProgramPublicEntry::Line(interpret_public_line(target, context, variables, &local_context, line)?)
                }
                PublicLineKind::Folder { label, lines } => {
                    ProgramPublicEntry::Folder {
                        label: label.clone(),
                        lines: lines
                            .iter()
                            .map(|line| interpret_public_line(target, context, variables, &local_context, line))
                            .collect::<crate::Result<_>>()?,
                    }
                }
            });
        }
    }

    Ok(ProgramPublic {
        entries: entries.into_boxed_slice(),
    })
}

fn interpret_public_line(
    context: &mut GlobalContext,
    variables: &mut Vec<ProgramVariable>,
    local_context: &LocalContext,
    line: &PublicLine,
) -> crate::Result<ProgramPublicLine> {
    match &line.kind {
        PublicLineKind::Expression(expression) => {
            Ok(ProgramPublicLine::Expression(interpret_expression(target, context, local_context, expression)?))
        }
        PublicLineKind::Action(action) => {
            Ok(ProgramPublicLine::Action(interpret_action_expression(target, context, local_context, action)?))
        }
        PublicLineKind::Slider { var_identifier } => {
            let var_index = variables
                .iter()
                .position(|variable| &variable.identifier == var_identifier)
                .ok_or_else(|| {
                    // We can provide some pretty good diagnostics for this error.
                    let Some(definition) = context.find_global(var_identifier) else {
                        return Box::new(crate::Error {
                            kind: crate::ErrorKind::UndefinedIdentifier {
                                identifier: var_identifier.as_ref().into(),
                            },
                            span: Some(line.span),
                        })
                    };
                    if matches!(definition.definition.kind, DefinitionKind::Value(ValueDefinition::Variable { .. })) {
                        Box::new(crate::Error {
                            kind: crate::ErrorKind::MultipleSlidersForVariable {
                                identifier: var_identifier.as_ref().into(),
                            },
                            span: Some(line.span),
                        })
                    }
                    else {
                        Box::new(crate::Error {
                            kind: crate::ErrorKind::InvalidSliderReference {
                                identifier: var_identifier.as_ref().into(),
                            },
                            span: Some(line.span),
                        })
                    }
                })?;

            // Steal the variable and put it in the public list
            Ok(ProgramPublicLine::Variable(variables.remove(var_index)))
        }
        PublicLineKind::Folder { .. } => {
            // If this function has been called, the assumption is that a folder is not allowed.
            Err(Box::new(crate::Error {
                kind: crate::ErrorKind::CannotNestFolders,
                span: Some(line.span),
            }))
        }
    }
}

pub fn interpret_display_declarations<'a>(
    context: &mut GlobalContext,
    display_declarations: impl IntoIterator<Item = &'a DisplayDeclaration>,
) -> crate::Result<ProgramDisplay> {
    fn prevent_duplicate(attribute: &DisplayAttribute, has_attribute: &mut bool) -> crate::Result<()> {
        if std::mem::replace(has_attribute, true) {
            Err(Box::new(crate::Error {
                kind: crate::ErrorKind::DuplicatedDisplayAttribute {
                    key: attribute.key.as_ref().into(),
                },
                span: Some(attribute.key_span),
            }))
        }
        else {
            Ok(())
        }
    }

    fn check_arity(attribute: &DisplayAttribute, min_arity: usize, max_arity: usize) -> crate::Result<()> {
        if (min_arity ..= max_arity).contains(&attribute.arguments.len()) {
            Ok(())
        }
        else {
            Err(Box::new(crate::Error {
                kind: crate::ErrorKind::InvalidDisplayAttributeArity {
                    key: attribute.key.as_ref().into(),
                    min: min_arity,
                    max: max_arity,
                    got: attribute.arguments.len(),
                },
                span: Some(attribute.key_span),
            }))
        }
    }

    let mut elements = Vec::new();

    for declaration in context.display_declarations() {
        let local_context = LocalContext::new(&source_paths[declaration.span.source_id]);

        macro_rules! interpret_option {
            ($opt:expr, $t:expr) => {
                (($opt)
                    .map(|expression| {
                        interpret_expression(target, context, &local_context, expression)?
                            .coerce_to($t, true)
                    })
                    .transpose())
            };
        }

        macro_rules! interpret_option_named {
            ($opt:expr, $e:ty) => {
                (($opt)
                    .map_or(Ok(Default::default()), |expression| {
                        let value = interpret_expression(target, context, &local_context, expression)?;
                        value.kind
                            .as_const_str()
                            .and_then(|name| <$e>::from_name(&name))
                            .ok_or_else(|| Box::new(crate::Error {
                                kind: crate::ErrorKind::ExpectedConstantStrFromList {
                                    allowed: <$e>::NAMES
                                        .iter()
                                        .copied()
                                        .map(Into::into)
                                        .collect(),
                                },
                                span: value.span,
                            }))
                    }))
            };
        }

        macro_rules! interpret_option_bool {
            ($opt:expr, $default:expr) => {
                (($opt)
                    .map_or(Ok($default), |expression| {
                        let value = interpret_expression(target, context, &local_context, expression)?;
                        value.kind
                            .as_const_bool()
                            .ok_or_else(|| Box::new(crate::Error {
                                kind: crate::ErrorKind::ExpectedConstant {
                                    type_identifier: Type::Bool.to_string(),
                                },
                                span: value.span,
                            }))
                    }))
            };
        }

        for element in &declaration.elements {
            let mut has_color = false;
            let mut has_point = false;
            let mut has_drag = false;
            let mut has_label = false;
            let mut has_line = false;
            let mut has_fill = false;
            let mut has_click = false;
            let mut has_hovered = false;
            let mut has_pressed = false;
            let mut has_description = false;

            let mut attributes = Vec::with_capacity(element.attributes.len());

            for attribute in &element.attributes {
                let kind = match attribute.key.as_ref() {
                    "color" => {
                        // color(c: color)
                        prevent_duplicate(attribute, &mut has_color)?;
                        check_arity(attribute, 1, 1)?;

                        ProgramDisplayAttributeKind::Color {
                            value: interpret_expression(target, context, &local_context, &attribute.arguments[0])?
                                .coerce_to(&Type::Color, true)?,
                        }
                    }
                    "point" => {
                        // point(opacity?: real, size?: real, style?: str, outline?: bool)
                        prevent_duplicate(attribute, &mut has_point)?;
                        check_arity(attribute, 0, 4)?;

                        ProgramDisplayAttributeKind::Point {
                            opacity: interpret_option!(attribute.arguments.get(0), &Type::Real)?,
                            size: interpret_option!(attribute.arguments.get(1), &Type::Real)?,
                            style: interpret_option_named!(attribute.arguments.get(2), PointStyle)?,
                            outline: interpret_option_bool!(attribute.arguments.get(3), false)?,
                        }
                    }
                    "drag" => {
                        // drag(mode?: str)
                        prevent_duplicate(attribute, &mut has_drag)?;
                        check_arity(attribute, 0, 1)?;

                        ProgramDisplayAttributeKind::Drag {
                            mode: interpret_option_named!(attribute.arguments.get(0), DragMode)?,
                        }
                    }
                    "label" => {
                        // label(text: str, opacity?: real, size?: real, angle?: real,
                        //       orientation?: str, outline?: bool)
                        prevent_duplicate(attribute, &mut has_label)?;
                        check_arity(attribute, 1, 6)?;

                        ProgramDisplayAttributeKind::Label {
                            text: interpret_expression(target, context, &local_context, &attribute.arguments[0])?
                                .get_const_str()?,
                            opacity: interpret_option!(attribute.arguments.get(1), &Type::Real)?,
                            size: interpret_option!(attribute.arguments.get(2), &Type::Real)?,
                            angle: interpret_option!(attribute.arguments.get(3), &Type::Real)?,
                            orientation: interpret_option_named!(attribute.arguments.get(4), LabelOrientation)?,
                            outline: interpret_option_bool!(attribute.arguments.get(5), true)?,
                        }
                    }
                    "line" => {
                        // line(opacity?: real, width?: real, style?: str)
                        prevent_duplicate(attribute, &mut has_line)?;
                        check_arity(attribute, 0, 3)?;

                        ProgramDisplayAttributeKind::Line {
                            opacity: interpret_option!(attribute.arguments.get(0), &Type::Real)?,
                            width: interpret_option!(attribute.arguments.get(1), &Type::Real)?,
                            style: interpret_option_named!(attribute.arguments.get(2), LineStyle)?,
                        }
                    }
                    "fill" => {
                        // fill(opacity?: real)
                        prevent_duplicate(attribute, &mut has_fill)?;
                        check_arity(attribute, 0, 1)?;

                        ProgramDisplayAttributeKind::Fill {
                            opacity: interpret_option!(attribute.arguments.get(0), &Type::Real)?,
                        }
                    }
                    "click" => {
                        // click(on_click: action(int?))
                        prevent_duplicate(attribute, &mut has_click)?;
                        check_arity(attribute, 1, 1)?;

                        let on_click = interpret_expression(target, context, &local_context, &attribute.arguments[0])?;
                        let on_click_type = on_click.get_type();

                        let on_click_arguments: Box<[_]> = match on_click_type.require_action(0, &[Type::Int])? {
                            [] => Box::new([]),
                            [index_type] => Box::new([
                                Value::ClickIndex.with_span(None).coerce_to(index_type, false)?
                            ]),
                            _ => unreachable!()
                        };

                        ProgramDisplayAttributeKind::Click {
                            action: ActionValueKind::ActionCall {
                                action: Box::new(on_click),
                                arguments: on_click_arguments,
                            }.into(),
                        }
                    }
                    "hovered" => {
                        // hovered(url: str)
                        prevent_duplicate(attribute, &mut has_hovered)?;
                        check_arity(attribute, 1, 1)?;

                        ProgramDisplayAttributeKind::Hovered {
                            url: interpret_expression(target, context, &local_context, &attribute.arguments[0])?
                                .get_const_str()?,
                        }
                    }
                    "pressed" => {
                        // pressed(url: str)
                        prevent_duplicate(attribute, &mut has_pressed)?;
                        check_arity(attribute, 1, 1)?;

                        ProgramDisplayAttributeKind::Pressed {
                            url: interpret_expression(target, context, &local_context, &attribute.arguments[0])?
                                .get_const_str()?,
                        }
                    }
                    "description" => {
                        // description(text: str)
                        prevent_duplicate(attribute, &mut has_description)?;
                        check_arity(attribute, 1, 1)?;

                        ProgramDisplayAttributeKind::Description {
                            text: interpret_expression(target, context, &local_context, &attribute.arguments[0])?
                                .get_const_str()?,
                        }
                    }
                    _ => {
                        return Err(Box::new(crate::Error {
                            kind: crate::ErrorKind::UnsupportedDisplayAttribute {
                                key: attribute.key.as_ref().into(),
                            },
                            span: Some(attribute.key_span),
                        }))
                    }
                };

                attributes.push(ProgramDisplayAttribute {
                    kind,
                    key_span: Some(attribute.key_span),
                });
            }

            elements.push(ProgramDisplayElement {
                value: interpret_expression(target, context, &local_context, &element.expression)?,
                span: Some(element.span),
                attributes: attributes.into_boxed_slice(),
            });
        }
    }

    Ok(ProgramDisplay {
        elements: elements.into_boxed_slice(),
    })
}

// TODO: detect multiple updates of same variable
pub fn interpret_action_expression(
    context: &mut GlobalContext,
    local_context: &LocalContext,
    action: &ActionExpression,
) -> crate::Result<ActionValue> {
    let kind = match &action.kind {
        ActionExpressionKind::Disable => {
            ActionValueKind::Disable
        }
        ActionExpressionKind::Compound { actions } => {
            ActionValueKind::Compound {
                actions: actions
                    .iter()
                    .map(|action| {
                        interpret_action_expression(target, context, local_context, action)
                    })
                    .collect::<crate::Result<_>>()?,
            }
        }
        ActionExpressionKind::Update { variable, value } => {
            let invalid_update_lhs_error = || Box::new(crate::Error {
                kind: crate::ErrorKind::InvalidUpdateLhs,
                span: Some(variable.span),
            });

            let variable = interpret_expression(target, context, &local_context, variable)?;
            let variable_span = variable.span;
            let Value::GlobalReference(variable) = variable.kind else {
                return Err(invalid_update_lhs_error())
            };
            let DefinitionKind::Value(ValueDefinition::Variable { .. }) = context.find_global(&variable.identifier).unwrap().definition.kind else {
                return Err(invalid_update_lhs_error())
            };

            let value = interpret_expression(target, context, &local_context, value)?
                .coerce_to(&variable.value_type, false)?;

            ActionValueKind::Update {
                variable,
                variable_span,
                value: Box::new(value),
            }
        }
        ActionExpressionKind::ActionCall { action: callee, arguments } => {
            let callee = interpret_expression(target, context, &local_context, callee)?;

            let Type::Action { parameter_types } = callee.get_type() else {
                return Err(Box::new(crate::Error {
                    kind: crate::ErrorKind::ExpectedAction,
                    span: callee.span,
                }))
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

            ActionValueKind::ActionCall {
                action: Box::new(callee),
                arguments: std::iter::zip(arguments, &parameter_types)
                    .map(|(argument, parameter_type)| {
                        interpret_expression(target, context, local_context, argument)?
                            .coerce_to(parameter_type, false)
                    })
                    .collect::<crate::Result<_>>()?,
            }
        }
        ActionExpressionKind::Conditional { condition_consequents, alternative } => {
            ActionValueKind::Conditional {
                condition_consequents: condition_consequents
                    .iter()
                    .map(|(condition, consequent)| {
                        let condition = interpret_expression(target, context, local_context, condition)?
                            .coerce_to(&Type::Bool, false)?;
                        let consequent = interpret_action_expression(target, context, local_context, consequent)?;

                        Ok((condition, consequent))
                    })
                    .collect::<crate::Result<_>>()?,
                alternative: match alternative {
                    Some(alternative) => {
                        Box::new(interpret_action_expression(target, context, local_context, alternative)?)
                    }
                    None => {
                        Box::new(ActionValueKind::empty().into())
                    }
                },
            }
        }
    };

    Ok(ActionValue {
        kind,
        span: Some(action.span),
    })
}

pub fn interpret_expression(
    context: &mut GlobalContext,
    local_context: &LocalContext,
    expression: &Expression,
) -> crate::Result<ValueEntry> {
    match &expression.kind {
        ExpressionKind::Undefined => {
            Ok(ValueEntry {
                value: Value::Undefined,
                type_handle: TypeHandle::ANY,
                span: Some(expression.span),
            })
        }
        ExpressionKind::Infinity => {
            Ok(ValueEntry {
                value: Value::Infinity,
                type_handle: TypeHandle::INT,
                span: Some(expression.span),
            })
        }
        ExpressionKind::Real(value) => {
            Ok(ValueEntry {
                value: Value::Real(*value),
                type_handle: TypeHandle::REAL,
                span: Some(expression.span),
            })
        }
        ExpressionKind::Integer(value) => {
            let value = i64::try_from(*value).map_err(|_| Box::new(crate::Error {
                kind: crate::ErrorKind::IntegerTooLarge,
                span: Some(expression.span),
            }))?;

            Ok(ValueEntry {
                value: Value::Int(value),
                type_handle: TypeHandle::INT,
                span: Some(expression.span),
            })
        }
        ExpressionKind::Boolean(value) => {
            Ok(ValueEntry {
                value: Value::Bool(*value),
                type_handle: TypeHandle::BOOL,
                span: Some(expression.span),
            })
        }
        ExpressionKind::String(value) => {
            Ok(ValueEntry {
                value: Value::Str(value.clone()),
                type_handle: TypeHandle::STR,
                span: Some(expression.span),
            })
        }
        ExpressionKind::Identifier(identifier) => {
            if let Some(local) = local_context.find_local(identifier) {
                Ok(ValueEntry {
                    value: Value::Alias(local),
                    type_handle: context.values.get_type(local),
                    span: Some(expression.span),
                })
            }
            else if let Some(symbol) = context.find_global(identifier) {
                Ok(ValueEntry {
                    value: Value::GlobalReference(symbol.clone()),
                    type_handle: context.values.get_type(symbol.value),
                    span: Some(expression.span),
                })
            }
            else {
                Err(Box::new(crate::Error {
                    kind: crate::ErrorKind::UndefinedIdentifier {
                        identifier: identifier.clone(),
                    },
                    span: Some(expression.span),
                }))
            }
        }
        ExpressionKind::ActionIdentifier(identifier) => {
            if let Some(symbol) = context.find_action(identifier) {
                Ok(ValueEntry {
                    value: Value::ActionReference(symbol.clone()),
                    type_handle: context.values.get_type(symbol.value),
                    span: Some(expression.span),
                })
            }
            else {
                Err(Box::new(crate::Error {
                    kind: crate::ErrorKind::UndefinedAction {
                        identifier: identifier.clone(),
                    },
                    span: Some(expression.span),
                }))
            }
        }
        ExpressionKind::IntrinsicIdentifier(identifier) => {
            if let Some(symbol) = context.find_intrinsic(identifier) {
                Ok(ValueEntry {
                    value: Value::IntrinsicReference(symbol.clone()),
                    type_handle: context.values.get_type(symbol.value),
                    span: Some(expression.span),
                })
            }
            else {
                Err(Box::new(crate::Error {
                    kind: crate::ErrorKind::UndefinedIntrinsic {
                        identifier: identifier.clone(),
                    },
                    span: Some(expression.span),
                }))
            }
        }
        ExpressionKind::Grouping { expression } => {
            interpret_expression(context, local_context, expression)
        }
        ExpressionKind::Unary { operation, operand } => {
            interpret_unary_operation(context, local_context, *operation, operand, Some(expression.span))
        }
        ExpressionKind::Binary { operation, lhs, rhs } => {
            interpret_binary_operation(context, local_context, *operation, lhs, rhs, Some(expression.span))
        }
        ExpressionKind::Point2D { x, y } => {
            let x = interpret_expression(context, local_context, x)?;
            let y = interpret_expression(context, local_context, y)?;

            let (x_list, x_type) = context.types.flatten_list(x.type_handle);
            let (y_list, y_type) = context.types.flatten_list(y.type_handle);

            Ok(ValueEntry {
                value: Value::Binary {
                    kind: BinaryKind::Point2D,
                    lhs: context.values.register(x),
                    rhs: context.values.register(y),
                },
                type_handle: context.types.point_2d_type(x_type, y_type)
                    .and_then(|point_type| context.types.unflatten_list(
                        ListState::merge(x_list, y_list),
                        point_type,
                    ))
                    .map_err(|error| error.with_span(Some(expression.span)))?,
                span: Some(expression.span),
            })
        }
        ExpressionKind::Point3D { x, y, z } => {
            let x = interpret_expression(context, local_context, x)?;
            let y = interpret_expression(context, local_context, y)?;
            let z = interpret_expression(context, local_context, z)?;

            let (x_list, x_type) = context.types.flatten_list(x.type_handle);
            let (y_list, y_type) = context.types.flatten_list(y.type_handle);
            let (z_list, z_type) = context.types.flatten_list(z.type_handle);

            Ok(ValueEntry {
                value: Value::Ternary {
                    kind: TernaryKind::Point3D,
                    first: context.values.register(x),
                    second: context.values.register(y),
                    third: context.values.register(z),
                },
                type_handle: context.types.point_3d_type(x_type, y_type, z_type)
                    .and_then(|point_type| context.types.unflatten_list(
                        ListState::merge_all([x_list, y_list, z_list]),
                        point_type,
                    ))
                    .map_err(|error| error.with_span(Some(expression.span)))?,
                span: Some(expression.span),
            })
        }
        ExpressionKind::List { items } => {
            let mut items: Box<[_]> = items
                .iter()
                .map(|item| {
                    let entry = interpret_expression(context, local_context, item)?;
                    Ok(context.values.register(entry))
                })
                .collect::<crate::Result<_>>()?;

            let item_type = items
                .iter()
                .try_fold(TypeHandle::ANY, |current_type, &item| {
                    context.types.merge(current_type, context.values.get_type(item))
                        .map_err(|error| error.with_span(context.values.get_span(item)))
                })?;
            let list_type = context.types.list_type(ListState::IsList, item_type)?;

            for item in &mut items {
                *item = context.coerce_value(*item, item_type, false)?;
            }

            Ok(ValueEntry {
                value: Value::List {
                    items,
                },
                type_handle: list_type,
                span: Some(expression.span),
            })
        }
        ExpressionKind::ListRange { kind, start, end, step } => {
            let start = interpret_expression(context, local_context, start)?
                .register(&mut context.values);
            let end = interpret_expression(context, local_context, end)?
                .register(&mut context.values);
            let step = match step {
                Some(step) => interpret_expression(context, local_context, step)?
                    .register(&mut context.values),
                None => ValueHandle::ONE_INT,
            };

            let start_type = context.values.get_type(start);
            let end_type = context.values.get_type(end);
            let step_type = context.values.get_type(step);

            let item_type = context.types.merge(start_type, end_type)
                .and_then(|item_type| context.types.merge(item_type, step_type))
                .map_err(|error| error.with_span(Some(expression.span)))?;
            let list_type = context.types.list_type(ListState::IsList, item_type)?;

            Ok(ValueEntry {
                value: Value::ListRange {
                    kind: *kind,
                    start: context.coerce_value(start, item_type, false)?,
                    end: context.coerce_value(end, item_type, false)?,
                    step: context.coerce_value(step, item_type, false)?,
                },
                type_handle: list_type,
                span: Some(expression.span),
            })
        }
        ExpressionKind::ListFill { value, count } => {
            let value = interpret_expression(context, local_context, value)?
                .register(&mut context.values);
            let count = interpret_expression(context, local_context, count)?
                .register(&mut context.values);

            let list_type = context.types.list_type(ListState::IsList, context.values.get_type(value))?;

            Ok(ValueEntry {
                value: Value::ListFill {
                    value,
                    count: context.coerce_value(count, TypeHandle::INT, false)?,
                },
                type_handle: list_type,
                span: Some(expression.span),
            })
        }
        ExpressionKind::ListMap { loops, expression: map_expression } => {
            let mut map_context = local_context.new_inner();

            let loops = loops
                .iter()
                .map(|map_loop| {
                    let list = interpret_expression(context, local_context, &map_loop.list)?;
                    let item_type = context.types.expect_list_type(list.type_handle)
                        .map_err(|error| error.with_span(list.span))?;

                    let local = context.values.register(ValueEntry {
                        value: Value::Local {
                            id: context.target.create_local_id(),
                        },
                        type_handle: item_type,
                        span: Some(map_loop.identifier_span),
                    });
                    map_context.add_local(map_loop.identifier.clone(), local);

                    Ok(ListMapLoop {
                        local,
                        list: context.values.register(list),
                    })
                })
                .collect::<crate::Result<_>>()?;

            let value = interpret_expression(context, &map_context, map_expression)?;

            Ok(ValueEntry {
                type_handle: context.types.list_type(ListState::IsList, value.type_handle)?,
                value: Value::ListMap {
                    loops,
                    value: context.values.register(value),
                },
                span: Some(expression.span),
            })
        }
        ExpressionKind::ListFilter { list, condition } => {
            let list = interpret_expression(context, local_context, list)?;
            context.types.expect_list_type(list.type_handle)
                .map_err(|error| error.with_span(list.span))?;

            let condition = interpret_expression(context, local_context, condition)?
                .register(&mut context.values);

            Ok(ValueEntry {
                type_handle: list.type_handle,
                value: Value::ListFilter {
                    list: context.values.register(list),
                    condition: context.coerce_value(condition, TypeHandle::BOOL, true)?,
                },
                span: Some(expression.span),
            })
        }
        ExpressionKind::Index { list, operation } => {
            let list = interpret_expression(context, local_context, list)?;
            let item_type = context.types.expect_list_type(list.type_handle)
                .map_err(|error| error.with_span(list.span))?;

            let kind = interpret_index_operation(context, local_context, operation)?;

            Ok(ValueEntry {
                type_handle: if kind.result_is_list() {
                    list.type_handle
                } else {
                    item_type
                },
                value: Value::Index {
                    list: context.values.register(list),
                    kind,
                },
                span: Some(expression.span),
            })
        }
        ExpressionKind::FunctionCall { function, arguments } => {
            let function = interpret_expression(context, local_context, function)?;
            let mut arguments: Box<[_]> = arguments
                .iter()
                .map(|argument| Ok(interpret_expression(context, local_context, argument)?
                    .register(&mut context.values)))
                .collect::<crate::Result<_>>()?;

            if let Value::IntrinsicFunction(intrinsic_function) = function.value {
                intrinsic_function.interpret_call(context, local_context, Some(expression.span), arguments)
            }
            else if let Type::Function { signature } = context.types.get(function.type_handle) {
                let signature = signature.clone();
                if arguments.len() != signature.parameter_types.len() {
                    return Err(Box::new(crate::Error {
                        kind: crate::ErrorKind::InvalidArity {
                            expected: signature.parameter_types.len(),
                            got: arguments.len(),
                        },
                        span: function.span,
                    }))
                }

                let mut result_list_state = None;
                for (argument, parameter_type) in std::iter::zip(&mut arguments, signature.parameter_types) {
                    if let Type::List { state: ListState::MaybeList, .. } = context.types.get(parameter_type) {
                        result_list_state = ListState::merge(
                            result_list_state,
                            argument.get_type(&context.values).flatten_list(&context.types).0,
                        )
                    }

                    *argument = context.coerce_value(*argument, parameter_type, false)?;
                }

                let return_type = match context.types.get(signature.return_type) {
                    &Type::List { state: ListState::MaybeList, item_type } => {
                        context.types.unflatten_list(result_list_state, item_type)?
                    }
                    _ => signature.return_type
                };

                Ok(ValueEntry {
                    value: Value::UserFunctionCall {
                        function: context.values.register(function),
                        arguments,
                    },
                    type_handle: return_type,
                    span: Some(expression.span),
                })
            }
            else {
                Err(Box::new(crate::Error {
                    kind: crate::ErrorKind::ExpectedFunctionType {
                        got_type: context.types.repr(function.type_handle),
                    },
                    span: function.span,
                }))
            }
        }
        ExpressionKind::Conditional { condition_consequents, alternative } => {
            let mut result_type = TypeHandle::ANY;

            let mut condition_consequents: Box<[_]> = condition_consequents
                .iter()
                .map(|(condition, consequent)| {
                    let condition = interpret_expression(context, local_context, condition)?
                        .register(&mut context.values);
                    let condition = context.coerce_value(condition, TypeHandle::BOOL, true)?;
                    // A list condition should cause the whole expression to broadcast.
                    let (result_list, inner_type) = context.types.flatten_list(result_type);
                    result_type = context.types.unflatten_list(
                        ListState::merge(
                            result_list,
                            condition.get_type(&context.values).flatten_list(&context.types).0,
                        ),
                        inner_type,
                    )?;

                    let consequent = interpret_expression(context, local_context, consequent)?;
                    result_type = context.types.merge(result_type, consequent.type_handle)
                        .map_err(|error| error.with_span(consequent.span))?;

                    Ok((condition, context.values.register(consequent)))
                })
                .collect::<crate::Result<_>>()?;

            let alternative = alternative
                .as_ref()
                .map_or(Ok(ValueHandle::UNDEFINED), |alternative| {
                    let alternative = interpret_expression(context, local_context, alternative)?;
                    result_type = context.types.merge(result_type, alternative.type_handle)
                        .map_err(|error| error.with_span(alternative.span))?;

                    let alternative = context.values.register(alternative);
                    context.coerce_value(alternative, context.types.flatten_list(result_type).1, true)
                })?;

            let result_inner_type = context.types.flatten_list(result_type).1;
            for (_, consequent) in &mut condition_consequents {
                *consequent = context.coerce_value(*consequent, result_inner_type, true)?;
            }

            Ok(ValueEntry {
                value: Value::Conditional {
                    condition_consequents,
                    alternative,
                },
                type_handle: result_type,
                span: Some(expression.span),
            })
        }
        ExpressionKind::Let { identifier, value_type, value, inner, .. } => {
            let mut value = interpret_expression(context, local_context, value)?
                .register(&mut context.values);

            if let Some(value_type) = value_type {
                let value_type = context.resolve_type(value_type, true)?;
                value = context.coerce_value(value, value_type, false)?;
            }

            let mut inner_context = local_context.new_inner();
            inner_context.add_local(identifier.clone(), value);

            interpret_expression(context, &inner_context, inner)
        }
        ExpressionKind::InlineAction { parameters, action } => {
            let mut action_context = local_context.new_inner();

            let parameter_types: Box<[_]> = parameters.0
                .iter()
                .map(|parameter| context.resolve_type(&parameter.parameter_type, false))
                .collect::<crate::Result<_>>()?;
            let typed_parameters = process_parameters(context, &mut action_context, parameters, &parameter_types);

            let action = interpret_action_expression(context, &action_context, action)?;

            Ok(ValueEntry {
                value: Value::Action {
                    parameters: typed_parameters,
                    action: Box::new(action),
                },
                type_handle: context.types.action_type(parameter_types),
                span: Some(expression.span),
            })
        }
        ExpressionKind::Character(..) => {
            Err(Box::new(crate::Error {
                kind: crate::ErrorKind::UnexpectedExpressionKind,
                span: Some(expression.span),
            }))
        }
    }
}

pub fn interpret_unary_operation(
    context: &mut GlobalContext,
    local_context: &LocalContext,
    operation: UnaryOperation,
    operand: &Expression,
    span: Option<crate::Span>,
) -> crate::Result<ValueEntry> {
    let kind = match operation {
        UnaryOperation::Positive => UnaryKind::Positive,
        UnaryOperation::Negative => UnaryKind::Negative,
        UnaryOperation::LogicalNot => UnaryKind::LogicalNot,
    };

    let mut operand = interpret_expression(target, context, local_context, operand)?;
    let operand_list = operand.get_type().list_state();

    let result_type = match kind {
        UnaryKind::Positive |
        UnaryKind::Negative => {
            // The result must be arithmetic
            operand = operand.coerce_to_arithmetic(Type::require_numeric_or_point)?;
            operand.get_type()
        }
        UnaryKind::LogicalNot => {
            operand = operand.coerce_to(&Type::Bool, true)?;
            Type::Bool.unflatten_list(operand_list)
        }
        _ => unreachable!("all cases from the previous match should be covered")
    };

    Ok(Value::Unary {
        kind,
        operand: Box::new(operand),
        result_type,
    })
}

pub fn interpret_binary_operation(
    context: &mut GlobalContext,
    local_context: &LocalContext,
    operation: BinaryOperation,
    lhs: &Expression,
    rhs: &Expression,
    span: Option<crate::Span>,
) -> crate::Result<ValueEntry> {
    let kind = match operation {
        BinaryOperation::MemberAccess => {
            // Handle this operation separately since its right hand side is not a value
            return interpret_access_operation(target, context, local_context, lhs, rhs)
        }
        BinaryOperation::Exponent => BinaryKind::Exponent,
        BinaryOperation::Multiply => BinaryKind::Multiply,
        BinaryOperation::Divide => BinaryKind::Divide,
        BinaryOperation::Remainder => BinaryKind::Remainder,
        BinaryOperation::Add => BinaryKind::Add,
        BinaryOperation::Subtract => BinaryKind::Subtract,
        BinaryOperation::LessThan => {
            return interpret_inequality_chain(target, context, local_context, InequalityKind::LessThan, lhs, rhs, span)
        }
        BinaryOperation::LessEqual => {
            return interpret_inequality_chain(target, context, local_context, InequalityKind::LessEqual, lhs, rhs, span)
        }
        BinaryOperation::GreaterThan => {
            return interpret_inequality_chain(target, context, local_context, InequalityKind::GreaterThan, lhs, rhs, span)
        }
        BinaryOperation::GreaterEqual => {
            return interpret_inequality_chain(target, context, local_context, InequalityKind::GreaterEqual, lhs, rhs, span)
        }
        BinaryOperation::Equal => BinaryKind::Equal,
        BinaryOperation::NotEqual => BinaryKind::NotEqual,
        BinaryOperation::LogicalAnd => BinaryKind::LogicalAnd,
        BinaryOperation::LogicalOr => BinaryKind::LogicalOr,
    };

    let mut lhs = interpret_expression(target, context, local_context, lhs)?;
    let mut rhs = interpret_expression(target, context, local_context, rhs)?;
    let (lhs_list, lhs_type) = lhs.get_type().into_flatten_list();
    let (rhs_list, rhs_type) = rhs.get_type().into_flatten_list();

    let result_type = match kind {
        BinaryKind::Exponent |
        BinaryKind::Remainder => {
            // The result must be arithmetic and cannot be a point
            lhs = lhs.coerce_to_arithmetic(Type::require_numeric)?;
            rhs = rhs.coerce_to_arithmetic(Type::require_numeric)?;
            Type::merge(&lhs.get_type(), &rhs.get_type())
                .map_err(|error| error.with_span(span))?
        }
        BinaryKind::Multiply => {
            // The result must be arithmetic, but at most one operand may be a point
            lhs = lhs.coerce_to_arithmetic(Type::require_numeric_or_point)?;
            let lhs_type = lhs.get_type().into_flatten_list().1;
            if matches!(lhs_type, Type::Point2D { .. } | Type::Point3D { .. }) {
                rhs = rhs.coerce_to_arithmetic(Type::require_numeric)?;
                let rhs_type = rhs.get_type().into_flatten_list().1;
                match &lhs_type {
                    Type::Point2D { x_type, y_type } => Type::Point2D {
                        x_type: Box::new(x_type.merge(&rhs_type)
                            .map_err(|error| error.with_span(span))?),
                        y_type: Box::new(y_type.merge(&rhs_type)
                            .map_err(|error| error.with_span(span))?),
                    },
                    Type::Point3D { x_type, y_type, z_type } => Type::Point3D {
                        x_type: Box::new(x_type.merge(&rhs_type)
                            .map_err(|error| error.with_span(span))?),
                        y_type: Box::new(y_type.merge(&rhs_type)
                            .map_err(|error| error.with_span(span))?),
                        z_type: Box::new(z_type.merge(&rhs_type)
                            .map_err(|error| error.with_span(span))?),
                    },
                    _ => unreachable!()
                }.unflatten_list(ListState::merge(lhs_list, rhs_list))
            }
            else {
                rhs = rhs.coerce_to_arithmetic(Type::require_numeric_or_point)?;
                let rhs_type = rhs.get_type().into_flatten_list().1;
                match &rhs_type {
                    Type::Point2D { x_type, y_type } => Type::Point2D {
                        x_type: Box::new(x_type.merge(&lhs_type)
                            .map_err(|error| error.with_span(span))?),
                        y_type: Box::new(y_type.merge(&lhs_type)
                            .map_err(|error| error.with_span(span))?),
                    },
                    Type::Point3D { x_type, y_type, z_type } => Type::Point3D {
                        x_type: Box::new(x_type.merge(&lhs_type)
                            .map_err(|error| error.with_span(span))?),
                        y_type: Box::new(y_type.merge(&lhs_type)
                            .map_err(|error| error.with_span(span))?),
                        z_type: Box::new(z_type.merge(&lhs_type)
                            .map_err(|error| error.with_span(span))?),
                    },
                    _ => Type::merge(&lhs_type, &rhs_type)
                        .map_err(|error| error.with_span(span))?
                }.unflatten_list(ListState::merge(lhs_list, rhs_list))
            }
        }
        BinaryKind::Divide => {
            // The result is always assumed to be real, but lhs may be a point
            let result_type = match lhs.get_type().flatten_list().1 {
                Type::Point2D { .. } => Type::Point2D {
                    x_type: Box::new(Type::Real),
                    y_type: Box::new(Type::Real),
                },
                Type::Point3D { .. } => Type::Point3D {
                    x_type: Box::new(Type::Real),
                    y_type: Box::new(Type::Real),
                    z_type: Box::new(Type::Real),
                },
                _ => Type::Real
            };
            lhs = lhs.coerce_to(&result_type, true)?;
            rhs = rhs.coerce_to(&Type::Real, true)?;
            result_type.unflatten_list(ListState::merge(lhs_list, rhs_list))
        }
        BinaryKind::Add |
        BinaryKind::Subtract => {
            // The result must be arithmetic, but may be a point
            lhs = lhs.coerce_to_arithmetic(Type::require_numeric_or_point)?;
            rhs = rhs.coerce_to_arithmetic(Type::require_numeric_or_point)?;
            Type::merge(&lhs.get_type(), &rhs.get_type())
                .map_err(|error| error.with_span(span))?
        }
        BinaryKind::Equal |
        BinaryKind::NotEqual => {
            // The operands must merge into a numeric or point type, but the result is always a bool
            Type::merge(&lhs_type, &rhs_type)
                .map_err(|error| error.with_span(span))?
                .require_numeric_or_point()
                .map_err(|error| error.with_span(span))?;
            Type::Bool.unflatten_list(ListState::merge(lhs_list, rhs_list))
        }
        BinaryKind::LogicalAnd |
        BinaryKind::LogicalOr => {
            lhs = lhs.coerce_to(&Type::Bool, true)?;
            rhs = rhs.coerce_to(&Type::Bool, true)?;
            Type::Bool.unflatten_list(ListState::merge(lhs_list, rhs_list))
        }
        _ => unreachable!("all cases from the previous match should be covered")
    };

    Ok(Value::Binary {
        kind,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
        result_type,
    })
}

fn interpret_access_operation(
    context: &mut GlobalContext,
    local_context: &LocalContext,
    lhs: &Expression,
    rhs: &Expression,
) -> crate::Result<ValueEntry> {
    let lhs = interpret_expression(target, context, local_context, lhs)?;
    let (lhs_list, lhs_type) = lhs.get_type().into_flatten_list();

    let ExpressionKind::Identifier(member_identifier) = &rhs.kind else {
        return Err(Box::new(crate::Error {
            kind: crate::ErrorKind::ExpectedIdentifier,
            span: Some(rhs.span),
        }))
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
            let definition = context.find_global(&identifier).unwrap();
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
                        ordinal: ordinal as i64,
                    })
                }
            }
        }
        Type::Point2D { x_type, y_type } => {
            match member_identifier.as_ref() {
                "x" => Ok(Value::Unary {
                    kind: UnaryKind::XOfPoint2D,
                    operand: Box::new(lhs),
                    result_type: x_type.as_ref().clone().unflatten_list(lhs_list),
                }),
                "y" => Ok(Value::Unary {
                    kind: UnaryKind::YOfPoint2D,
                    operand: Box::new(lhs),
                    result_type: y_type.as_ref().clone().unflatten_list(lhs_list),
                }),
                _ => Err(invalid_access_error())
            }
        }
        Type::Point3D { x_type, y_type, z_type } => {
            match member_identifier.as_ref() {
                "x" => Ok(Value::Unary {
                    kind: UnaryKind::XOfPoint3D,
                    operand: Box::new(lhs),
                    result_type: x_type.as_ref().clone().unflatten_list(lhs_list),
                }),
                "y" => Ok(Value::Unary {
                    kind: UnaryKind::YOfPoint3D,
                    operand: Box::new(lhs),
                    result_type: y_type.as_ref().clone().unflatten_list(lhs_list),
                }),
                "z" => Ok(Value::Unary {
                    kind: UnaryKind::ZOfPoint3D,
                    operand: Box::new(lhs),
                    result_type: z_type.as_ref().clone().unflatten_list(lhs_list),
                }),
                _ => Err(invalid_access_error())
            }
        }
        _ => Err(invalid_access_error())
    }
}

fn interpret_inequality_chain(
    context: &mut GlobalContext,
    local_context: &LocalContext,
    rightmost_kind: InequalityKind,
    lhs: &Expression,
    rhs: &Expression,
    span: Option<crate::Span>,
) -> crate::Result<ValueEntry> {
    let rhs_value = interpret_expression(target, context, local_context, rhs)?;
    let (mut list_state, mut compared_type) = rhs_value.get_type().into_flatten_list();

    // Descend into the LHS subtree as long as comparison operations is found.
    // Any chained comparisons without parentheses will be in the left subtree because comparisons
    // have left-to-right associativity.
    let mut chain_rev = vec![(rightmost_kind, rhs_value)];
    let mut current_lhs = lhs;
    loop {
        if let ExpressionKind::Binary { operation, lhs, rhs } = &current_lhs.kind {
            let kind = match operation {
                BinaryOperation::LessThan => InequalityKind::LessThan,
                BinaryOperation::LessEqual => InequalityKind::LessEqual,
                BinaryOperation::GreaterThan => InequalityKind::GreaterThan,
                BinaryOperation::GreaterEqual => InequalityKind::GreaterEqual,
                _ => break
            };

            let rhs_value = interpret_expression(target, context, local_context, rhs)?;
            let (rhs_list, rhs_type) = rhs_value.get_type().into_flatten_list();

            list_state = ListState::merge(list_state, rhs_list);
            compared_type = compared_type.merge(&rhs_type)
                .map_err(|error| error.with_span(rhs_value.span))?;

            chain_rev.push((kind, rhs_value));
            current_lhs = lhs.as_ref();
        }
        else {
            break
        }
    }

    let lhs_value =  interpret_expression(target, context, local_context, current_lhs)?;
    let (lhs_list, lhs_type) = lhs_value.get_type().into_flatten_list();

    list_state = ListState::merge(list_state, lhs_list);
    compared_type.merge(&lhs_type)
        .map_err(|error| error.with_span(lhs_value.span))?
        .require_numeric()
        .map_err(|error| error.with_span(span))?;

    Ok(Value::InequalityChain {
        lhs: Box::new(lhs_value),
        chain: chain_rev.into_iter().rev().collect(),
        result_type: Type::Bool.unflatten_list(list_state),
    })
}

pub fn interpret_index_operation(
    context: &mut GlobalContext,
    local_context: &LocalContext,
    operation: &IndexOperation,
) -> crate::Result<IndexKind> {
    match operation {
        IndexOperation::Single { index } => {
            let index = interpret_expression(target, context, local_context, index)?
                .coerce_to(&Type::Int, true)?;

            Ok(IndexKind::Single {
                index: Box::new(index),
            })
        }
        IndexOperation::Range { kind, from_index, to_index, step } => {
            let from_index = interpret_expression(target, context, local_context, from_index)?
                .coerce_to(&Type::Int, false)?;
            let to_index = interpret_expression(target, context, local_context, to_index)?
                .coerce_to(&Type::Int, false)?;
            let step = match step {
                Some(step) => interpret_expression(target, context, local_context, step)?
                    .coerce_to(&Type::Int, false)?,
                None => Value::Int(1).into(),
            };

            Ok(IndexKind::Range {
                kind: *kind,
                from_index: Box::new(from_index),
                to_index: Box::new(to_index),
                step: Box::new(step),
            })
        }
        IndexOperation::RangeFrom { from_index, step } => {
            let from_index = interpret_expression(target, context, local_context, from_index)?
                .coerce_to(&Type::Int, false)?;
            let step = match step {
                Some(step) => interpret_expression(target, context, local_context, step)?
                    .coerce_to(&Type::Int, false)?,
                None => Value::Int(1).into(),
            };

            Ok(IndexKind::RangeFrom {
                from_index: Box::new(from_index),
                step: Box::new(step),
            })
        }
        IndexOperation::RangeTo { kind, to_index } => {
            let to_index = interpret_expression(target, context, local_context, to_index)?
                .coerce_to(&Type::Int, false)?;

            Ok(IndexKind::RangeTo {
                kind: *kind,
                to_index: Box::new(to_index),
            })
        }
    }
}
