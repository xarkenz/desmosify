use std::rc::Rc;
use crate::ast::{ActionExpression, ActionExpressionKind, BinaryOperation, DefinitionKind, DisplayAttribute, Expression, IndexOperation, ExpressionKind, ParameterList, PublicLineKind, TypeDefinition, UnaryOperation, ValueDefinition, VariableKind, EnumerationVariant, PublicLine, Declaration, TickerDeclaration, PublicDeclaration, DisplayDeclaration};
use crate::sema::{Program, ProgramAction, ProgramEnumeration, ProgramImmutable, ProgramPublicList, ProgramPublicEntry, ProgramPublicLine, ProgramTicker, ProgramVariable, ProgramVariableKind};
use crate::sema::context::{GlobalContext, LocalContext};
use crate::sema::display::{DragMode, LabelOrientation, LineStyle, PointStyle, ProgramDisplayList, ProgramDisplayAttribute, ProgramDisplayAttributeKind, ProgramDisplayElement};
use crate::sema::types::{ListState, Type, TypeHandle};
use crate::sema::values::{ActionValue, ActionValueKind, GlobalSymbol, IndexKind, Value, ListMapLoop, BinaryKind, UnaryKind, InequalityKind, TernaryKind, ValueHandle, ValueEntry, GlobalSymbolKind};

pub fn interpret_program(context: &mut GlobalContext, declarations: &[Declaration]) -> crate::Result<Program> {
    let mut enumerations = Vec::new();
    let mut immutables = Vec::new();
    let mut variables = Vec::new();
    let mut actions = Vec::new();

    let mut tickers = Vec::new();
    let mut public_lists = Vec::new();
    let mut display_lists = Vec::new();

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
                tickers.push(interpret_ticker_declaration(
                    context,
                    ticker_declaration,
                )?);
            }
            Declaration::Public(public_declaration) => {
                public_lists.push(interpret_public_declaration(
                    context,
                    &mut variables,
                    public_declaration,
                )?);
            }
            Declaration::Display(display_declaration) => {
                display_lists.push(interpret_display_declaration(
                    context,
                    display_declaration,
                )?);
            }
        }
    }

    Ok(Program {
        enumerations: enumerations.into_boxed_slice(),
        immutables: immutables.into_boxed_slice(),
        variables: variables.into_boxed_slice(),
        actions: actions.into_boxed_slice(),
        tickers: tickers.into_boxed_slice(),
        public_lists: public_lists.into_boxed_slice(),
        display_lists: display_lists.into_boxed_slice(),
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
        panic!("invalid definition for enum '{identifier}'")
    };
    let ordinals: Vec<_> = values
        .iter()
        .map(|&(_, ordinal)| ordinal)
        .collect();

    let mut previous_value: Option<(Rc<str>, ValueHandle)> = None;
    for (variant, ordinal) in std::iter::zip(variants, ordinals) {
        let ordinal_value = if let Some(value) = &variant.value {
            // Use the explicit value as the ordinal.
            let entry = interpret_expression(context, &local_context, value)?;
            context.expect_coercible(entry.type_handle, TypeHandle::INT, entry.span)?;
            entry.value
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
    let Some(value_handle) = context.find_global(&identifier)
        .map(|global| global.value)
    else {
        panic!("no immutable '{identifier}' found in context")
    };
    let mut expected_type = context.values.get_type(value_handle);

    let parameter_values = parameters.map(|parameters| {
        let Type::Function { signature } = context.types.get(expected_type) else {
            panic!("parameter list is present but value type is not a function")
        };
        expected_type = signature.return_type;
        let parameter_types = signature.parameter_types.clone();

        process_parameters(context, &mut local_context, parameters, &parameter_types)
    });

    let entry = interpret_expression(context, &local_context, value)?;
    context.expect_coercible(entry.type_handle, expected_type, entry.span)?;
    context.values.replace(value_handle, entry.value);

    Ok(ProgramImmutable {
        identifier,
        parameters: parameter_values,
        value: value_handle,
    })
}

pub fn interpret_variable_definition(
    context: &mut GlobalContext,
    local_context: LocalContext,
    identifier: Rc<str>,
    kind: &VariableKind,
    value: &Expression,
) -> crate::Result<ProgramVariable> {
    let Some(value_handle) = context.find_global(&identifier)
        .map(|global| global.value)
    else {
        panic!("no variable '{identifier}' found in context")
    };
    let expected_type = context.values.get_type(value_handle);

    // TODO: timer and slider should be restricted to certain types, should also affect slider step
    let kind = match kind {
        VariableKind::Default => ProgramVariableKind::Default,
        VariableKind::Timer => ProgramVariableKind::Timer,
        VariableKind::Slider { min, max, step } => {
            let mut interpret = |option: Option<&Expression>| {
                option.map(|expression| {
                    interpret_expression(context, &local_context, expression)?
                        .register(&mut context.values)
                        .coerce(context, expected_type, false)
                }).transpose()
            };
            ProgramVariableKind::Slider {
                min: interpret(min.as_deref())?,
                max: interpret(max.as_deref())?,
                step: interpret(step.as_deref())?,
            }
        }
    };

    let entry = interpret_expression(context, &local_context, value)?;
    context.expect_coercible(entry.type_handle, expected_type, entry.span)?;
    context.values.replace(value_handle, entry.value);

    Ok(ProgramVariable {
        identifier,
        kind,
        value: value_handle,
    })
}

pub fn interpret_action_definition(
    context: &mut GlobalContext,
    mut local_context: LocalContext,
    identifier: Rc<str>,
    parameters: &ParameterList,
    action: &ActionExpression,
) -> crate::Result<ProgramAction> {
    let Some(type_handle) = context.find_global(&identifier)
        .map(|global| context.values.get_type(global.value))
    else {
        panic!("no action '{identifier}' found in context")
    };
    let Type::Action { parameter_types } = context.types.get(type_handle) else {
        panic!("invalid definition for action '{identifier}'")
    };
    let parameter_types = parameter_types.clone();

    let parameter_values = process_parameters(context, &mut local_context, parameters, &parameter_types);

    let action = interpret_action_expression(context, &local_context, action)?;

    Ok(ProgramAction {
        identifier,
        parameters: parameter_values,
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

pub fn interpret_ticker_declaration(
    context: &mut GlobalContext,
    declaration: &TickerDeclaration,
) -> crate::Result<ProgramTicker> {
    let local_context = context.new_local_context(declaration.span.source_id);

    let interval_ms = declaration.interval_ms.as_ref().map(|interval_ms| {
        interpret_expression(context, &local_context, interval_ms)?
            .register(&mut context.values)
            .coerce(context, TypeHandle::REAL, false)
    }).transpose()?;

    let tick_action = interpret_expression(context, &local_context, &declaration.tick_action)?;

    let tick_arguments: Box<[_]> = match context.types.expect_action_type(
        tick_action.type_handle,
        0,
        &[TypeHandle::REAL],
        tick_action.span,
    )? {
        &[] => Box::new([]),
        &[dt_type, ..] => Box::new([
            context.coerce_value(ValueHandle::TICKER_DT, dt_type, false)?,
        ]),
    };

    Ok(ProgramTicker {
        interval_ms,
        tick_action: ActionValueKind::ActionCall {
            action: tick_action.register(&mut context.values),
            arguments: tick_arguments,
        }.into(),
    })
}

pub fn interpret_public_declaration(
    context: &mut GlobalContext,
    variables: &mut Vec<ProgramVariable>,
    declaration: &PublicDeclaration,
) -> crate::Result<ProgramPublicList> {
    let local_context = context.new_local_context(declaration.span.source_id);

    Ok(ProgramPublicList {
        entries: declaration.lines
            .iter()
            .map(|line| match &line.kind {
                PublicLineKind::Expression(..) |
                PublicLineKind::Action(..) |
                PublicLineKind::Slider { .. } => {
                    interpret_public_line(context, variables, &local_context, line)
                        .map(ProgramPublicEntry::Line)
                }
                PublicLineKind::Folder { label, lines } => {
                    Ok(ProgramPublicEntry::Folder {
                        label: label.clone(),
                        lines: lines
                            .iter()
                            .map(|line| interpret_public_line(context, variables, &local_context, line))
                            .collect::<crate::Result<_>>()?,
                    })
                }
            })
            .collect::<crate::Result<_>>()?,
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
            Ok(ProgramPublicLine::Expression(interpret_expression(context, local_context, expression)?
                .register(&mut context.values)))
        }
        PublicLineKind::Action(action) => {
            Ok(ProgramPublicLine::Action(interpret_action_expression(context, local_context, action)?))
        }
        PublicLineKind::Slider { var_identifier } => {
            let var_index = variables
                .iter()
                .position(|variable| &variable.identifier == var_identifier)
                .ok_or_else(|| {
                    // We can provide some pretty good diagnostics for this error.
                    if let Some(global) = context.find_global(var_identifier) {
                        if matches!(global.kind, GlobalSymbolKind::Variable) {
                            Box::new(crate::Error {
                                kind: crate::ErrorKind::MultipleSlidersForVariable {
                                    identifier: var_identifier.clone(),
                                },
                                span: Some(line.span),
                            })
                        }
                        else {
                            Box::new(crate::Error {
                                kind: crate::ErrorKind::InvalidSliderReference {
                                    identifier: var_identifier.clone(),
                                },
                                span: Some(line.span),
                            })
                        }
                    }
                    else {
                        Box::new(crate::Error {
                            kind: crate::ErrorKind::UndefinedIdentifier {
                                identifier: var_identifier.clone(),
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

pub fn interpret_display_declaration(
    context: &mut GlobalContext,
    declaration: &DisplayDeclaration,
) -> crate::Result<ProgramDisplayList> {
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

    let local_context = context.new_local_context(declaration.span.source_id);

    let interpret_option = |context: &mut GlobalContext, option: Option<&Expression>, to_type: TypeHandle| {
        option.map(|expression| {
            interpret_expression(context, &local_context, expression)?
                .register(&mut context.values)
                .coerce(context, to_type, true)
        }).transpose()
    };

    macro_rules! interpret_option_named {
        ($opt:expr, $e:ty) => {
            $opt.map_or(Ok(Default::default()), |expression| {
                let entry = interpret_expression(context, &local_context, expression)?;
                entry.value
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
                        span: entry.span,
                    }))
            })
        };
    }

    let interpret_option_bool = |context: &mut GlobalContext, option: Option<&Expression>, default: bool| {
        option.map_or(Ok(default), |expression| {
            let entry = interpret_expression(context, &local_context, expression)?;
            entry.value
                .as_const_bool()
                .ok_or_else(|| Box::new(crate::Error {
                    kind: crate::ErrorKind::ExpectedConstant {
                        type_identifier: context.types.repr(TypeHandle::BOOL),
                    },
                    span: entry.span,
                }))
        })
    };

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
                        value: interpret_expression(context, &local_context, &attribute.arguments[0])?
                            .register(&mut context.values)
                            .coerce(context, TypeHandle::COLOR, true)?,
                    }
                }
                "point" => {
                    // point(opacity?: real, size?: real, style?: str, outline?: bool)
                    prevent_duplicate(attribute, &mut has_point)?;
                    check_arity(attribute, 0, 4)?;

                    ProgramDisplayAttributeKind::Point {
                        opacity: interpret_option(context, attribute.arguments.get(0), TypeHandle::REAL)?,
                        size: interpret_option(context, attribute.arguments.get(1), TypeHandle::REAL)?,
                        style: interpret_option_named!(attribute.arguments.get(2), PointStyle)?,
                        outline: interpret_option_bool(context, attribute.arguments.get(3), false)?,
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
                        text: interpret_expression(context, &local_context, &attribute.arguments[0])?
                            .expect_const_str(&context.types)?,
                        opacity: interpret_option(context, attribute.arguments.get(1), TypeHandle::REAL)?,
                        size: interpret_option(context, attribute.arguments.get(2), TypeHandle::REAL)?,
                        angle: interpret_option(context, attribute.arguments.get(3), TypeHandle::REAL)?,
                        orientation: interpret_option_named!(attribute.arguments.get(4), LabelOrientation)?,
                        outline: interpret_option_bool(context, attribute.arguments.get(5), true)?,
                    }
                }
                "line" => {
                    // line(opacity?: real, width?: real, style?: str)
                    prevent_duplicate(attribute, &mut has_line)?;
                    check_arity(attribute, 0, 3)?;

                    ProgramDisplayAttributeKind::Line {
                        opacity: interpret_option(context, attribute.arguments.get(0), TypeHandle::REAL)?,
                        width: interpret_option(context, attribute.arguments.get(1), TypeHandle::REAL)?,
                        style: interpret_option_named!(attribute.arguments.get(2), LineStyle)?,
                    }
                }
                "fill" => {
                    // fill(opacity?: real)
                    prevent_duplicate(attribute, &mut has_fill)?;
                    check_arity(attribute, 0, 1)?;

                    ProgramDisplayAttributeKind::Fill {
                        opacity: interpret_option(context, attribute.arguments.get(0), TypeHandle::REAL)?,
                    }
                }
                "click" => {
                    // click(on_click: action(int?))
                    prevent_duplicate(attribute, &mut has_click)?;
                    check_arity(attribute, 1, 1)?;

                    let on_click = interpret_expression(context, &local_context, &attribute.arguments[0])?;

                    let on_click_arguments: Box<[_]> = match context.types.expect_action_type(
                        on_click.type_handle,
                        0,
                        &[TypeHandle::INT],
                        on_click.span,
                    )? {
                        &[] => Box::new([]),
                        &[index_type, ..] => Box::new([
                            context.coerce_value(ValueHandle::CLICK_INDEX, index_type, false)?,
                        ]),
                    };

                    ProgramDisplayAttributeKind::Click {
                        action: ActionValueKind::ActionCall {
                            action: on_click.register(&mut context.values),
                            arguments: on_click_arguments,
                        }.into(),
                    }
                }
                "hovered" => {
                    // hovered(url: str)
                    prevent_duplicate(attribute, &mut has_hovered)?;
                    check_arity(attribute, 1, 1)?;

                    ProgramDisplayAttributeKind::Hovered {
                        url: interpret_expression(context, &local_context, &attribute.arguments[0])?
                            .expect_const_str(&context.types)?,
                    }
                }
                "pressed" => {
                    // pressed(url: str)
                    prevent_duplicate(attribute, &mut has_pressed)?;
                    check_arity(attribute, 1, 1)?;

                    ProgramDisplayAttributeKind::Pressed {
                        url: interpret_expression(context, &local_context, &attribute.arguments[0])?
                            .expect_const_str(&context.types)?,
                    }
                }
                "description" => {
                    // description(text: str)
                    prevent_duplicate(attribute, &mut has_description)?;
                    check_arity(attribute, 1, 1)?;

                    ProgramDisplayAttributeKind::Description {
                        text: interpret_expression(context, &local_context, &attribute.arguments[0])?
                            .expect_const_str(&context.types)?,
                    }
                }
                _ => return Err(Box::new(crate::Error {
                    kind: crate::ErrorKind::UnsupportedDisplayAttribute {
                        key: attribute.key.as_ref().into(),
                    },
                    span: Some(attribute.key_span),
                }))
            };

            attributes.push(ProgramDisplayAttribute {
                kind,
                key_span: Some(attribute.key_span),
            });
        }

        elements.push(ProgramDisplayElement {
            value: interpret_expression(context, &local_context, &element.expression)?,
            span: Some(element.span),
            attributes: attributes.into_boxed_slice(),
        });
    }

    Ok(ProgramDisplayList {
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
                        interpret_action_expression(context, local_context, action)
                    })
                    .collect::<crate::Result<_>>()?,
            }
        }
        ActionExpressionKind::Update { variable, value } => {
            let invalid_update_lhs_error = || Box::new(crate::Error {
                kind: crate::ErrorKind::InvalidUpdateLhs,
                span: Some(variable.span),
            });

            let variable = interpret_expression(context, &local_context, variable)?;
            let Value::GlobalReference(global) = variable.value else {
                return Err(invalid_update_lhs_error())
            };
            let GlobalSymbolKind::Variable = global.kind else {
                return Err(invalid_update_lhs_error())
            };

            let value = interpret_expression(context, &local_context, value)?
                .register(&mut context.values)
                .coerce(context, variable.type_handle, false)?;

            ActionValueKind::Update {
                variable_identifier: global.identifier,
                variable_span: variable.span,
                value,
            }
        }
        ActionExpressionKind::ActionCall { action: callee, arguments } => {
            let callee = interpret_expression(context, &local_context, callee)?;

            let Type::Action { parameter_types } = context.types.get(callee.type_handle) else {
                return Err(Box::new(crate::Error {
                    kind: crate::ErrorKind::ExpectedAction,
                    span: callee.span,
                }))
            };
            let parameter_types = parameter_types.clone();

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
                action: callee.register(&mut context.values),
                arguments: std::iter::zip(arguments, parameter_types)
                    .map(|(argument, parameter_type)| {
                        interpret_expression(context, local_context, argument)?
                            .register(&mut context.values)
                            .coerce(context, parameter_type, false)
                    })
                    .collect::<crate::Result<_>>()?,
            }
        }
        ActionExpressionKind::Conditional { condition_consequents, alternative } => {
            ActionValueKind::Conditional {
                condition_consequents: condition_consequents
                    .iter()
                    .map(|(condition, consequent)| {
                        let condition = interpret_expression(context, local_context, condition)?
                            .register(&mut context.values)
                            .coerce(context, TypeHandle::BOOL, false)?;
                        let consequent = interpret_action_expression(context, local_context, consequent)?;

                        Ok((condition, consequent))
                    })
                    .collect::<crate::Result<_>>()?,
                alternative: match alternative {
                    Some(alternative) => {
                        Box::new(interpret_action_expression(context, local_context, alternative)?)
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
                    lhs: x.register(&mut context.values),
                    rhs: y.register(&mut context.values),
                },
                type_handle: context.types.point_2d_type(x_type, y_type, Some(expression.span))
                    .and_then(|point_type| context.types.unflatten_list(
                        ListState::merge(x_list, y_list),
                        point_type,
                        Some(expression.span),
                    ))?,
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
                    first: x.register(&mut context.values),
                    second: y.register(&mut context.values),
                    third: z.register(&mut context.values),
                },
                type_handle: context.types.point_3d_type(x_type, y_type, z_type, Some(expression.span))
                    .and_then(|point_type| context.types.unflatten_list(
                        ListState::merge_all([x_list, y_list, z_list]),
                        point_type,
                        Some(expression.span),
                    ))?,
                span: Some(expression.span),
            })
        }
        ExpressionKind::List { items } => {
            let mut items: Box<[_]> = items
                .iter()
                .map(|item| {
                    Ok(interpret_expression(context, local_context, item)?
                        .register(&mut context.values))
                })
                .collect::<crate::Result<_>>()?;

            let item_type = items
                .iter()
                .try_fold(TypeHandle::ANY, |current_type, &item| context.types.merge(
                    current_type,
                    context.values.get_type(item),
                    context.values.get_span(item),
                ))?;
            let list_type = context.types.list_type(ListState::IsList, item_type, Some(expression.span))?;

            for item in &mut items {
                *item = item.coerce(context, item_type, false)?;
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

            let item_type = context.types.merge(start_type, end_type, Some(expression.span))
                .and_then(|item_type| context.types.merge(item_type, step_type, Some(expression.span)))?;
            let list_type = context.types.list_type(ListState::IsList, item_type, Some(expression.span))?;

            Ok(ValueEntry {
                value: Value::ListRange {
                    kind: *kind,
                    start: start.coerce(context, item_type, false)?,
                    end: end.coerce(context, item_type, false)?,
                    step: step.coerce(context, item_type, false)?,
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

            let list_type = context.types.list_type(
                ListState::IsList,
                context.values.get_type(value),
                context.values.get_span(value),
            )?;

            Ok(ValueEntry {
                value: Value::ListFill {
                    value,
                    count: count.coerce(context, TypeHandle::INT, false)?,
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
                    let item_type = context.types.expect_list_type(list.type_handle, list.span)?;

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
                        list: list.register(&mut context.values),
                    })
                })
                .collect::<crate::Result<_>>()?;

            let value = interpret_expression(context, &map_context, map_expression)?;

            Ok(ValueEntry {
                type_handle: context.types.list_type(ListState::IsList, value.type_handle, value.span)?,
                value: Value::ListMap {
                    loops,
                    value: value.register(&mut context.values),
                },
                span: Some(expression.span),
            })
        }
        ExpressionKind::ListFilter { list, condition } => {
            let list = interpret_expression(context, local_context, list)?;
            context.types.expect_list_type(list.type_handle, list.span)?;

            let condition = interpret_expression(context, local_context, condition)?
                .register(&mut context.values);

            Ok(ValueEntry {
                type_handle: list.type_handle,
                value: Value::ListFilter {
                    list: list.register(&mut context.values),
                    condition: condition.coerce(context, TypeHandle::BOOL, true)?,
                },
                span: Some(expression.span),
            })
        }
        ExpressionKind::Index { list, operation } => {
            let list = interpret_expression(context, local_context, list)?;
            let item_type = context.types.expect_list_type(list.type_handle, list.span)?;

            let (kind, list_state) = interpret_index_operation(context, local_context, operation)?;

            Ok(ValueEntry {
                value: Value::Index {
                    list: list.register(&mut context.values),
                    kind,
                },
                type_handle: context.types.unflatten_list(list_state, item_type, Some(expression.span))?,
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

                    *argument = argument.coerce(context, parameter_type, false)?;
                }

                let return_type = match context.types.get(signature.return_type) {
                    &Type::List { state: ListState::MaybeList, item_type } => {
                        context.types.unflatten_list(result_list_state, item_type, None)
                            .expect("list item type should never be a list type")
                    }
                    _ => signature.return_type
                };

                Ok(ValueEntry {
                    value: Value::UserFunctionCall {
                        function: function.register(&mut context.values),
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
                    let condition = condition.coerce(context, TypeHandle::BOOL, true)?;
                    // A list condition should cause the whole expression to broadcast.
                    let (result_list, inner_type) = context.types.flatten_list(result_type);
                    result_type = context.types.unflatten_list(
                        ListState::merge(
                            result_list,
                            condition.get_type(&context.values).flatten_list(&context.types).0,
                        ),
                        inner_type,
                        condition.get_span(&context.values),
                    )?;

                    let consequent = interpret_expression(context, local_context, consequent)?;
                    result_type = context.types.merge(result_type, consequent.type_handle, consequent.span)?;

                    Ok((condition, consequent.register(&mut context.values)))
                })
                .collect::<crate::Result<_>>()?;

            let alternative = alternative
                .as_ref()
                .map_or(Ok(ValueHandle::UNDEFINED), |alternative| {
                    let alternative = interpret_expression(context, local_context, alternative)?;
                    result_type = context.types.merge(result_type, alternative.type_handle, alternative.span)?;

                    let alternative = alternative.register(&mut context.values);
                    alternative.coerce(context, context.types.flatten_list(result_type).1, true)
                })?;

            let result_inner_type = context.types.flatten_list(result_type).1;
            for (_, consequent) in &mut condition_consequents {
                *consequent = consequent.coerce(context, result_inner_type, true)?;
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
                value = value.coerce(context, value_type, false)?;
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

    let operand = interpret_expression(context, local_context, operand)?;
    let mut operand = operand.register(&mut context.values);

    let result_type: Option<TypeHandle> = match kind {
        UnaryKind::Positive |
        UnaryKind::Negative => {
            operand = operand.coerce(context, TypeHandle::ARITHMETIC_SCALAR_OR_POINT, true)?;
            None
        }
        UnaryKind::LogicalNot => {
            operand = operand.coerce(context, TypeHandle::BOOL, true)?;
            None
        }
        _ => unreachable!("all cases from the previous match should be covered")
    };

    Ok(ValueEntry {
        value: Value::Unary {
            kind,
            operand,
        },
        type_handle: result_type.unwrap_or(context.values.get_type(operand)),
        span,
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
            return interpret_access_operation(context, local_context, lhs, rhs, span)
        }
        BinaryOperation::Exponent => BinaryKind::Exponent,
        BinaryOperation::Multiply => BinaryKind::Multiply,
        BinaryOperation::Divide => BinaryKind::Divide,
        BinaryOperation::Remainder => BinaryKind::Remainder,
        BinaryOperation::Add => BinaryKind::Add,
        BinaryOperation::Subtract => BinaryKind::Subtract,
        BinaryOperation::LessThan => {
            return interpret_inequality_chain(context, local_context, InequalityKind::LessThan, lhs, rhs, span)
        }
        BinaryOperation::LessEqual => {
            return interpret_inequality_chain(context, local_context, InequalityKind::LessEqual, lhs, rhs, span)
        }
        BinaryOperation::GreaterThan => {
            return interpret_inequality_chain(context, local_context, InequalityKind::GreaterThan, lhs, rhs, span)
        }
        BinaryOperation::GreaterEqual => {
            return interpret_inequality_chain(context, local_context, InequalityKind::GreaterEqual, lhs, rhs, span)
        }
        BinaryOperation::Equal => BinaryKind::Equal,
        BinaryOperation::NotEqual => BinaryKind::NotEqual,
        BinaryOperation::LogicalAnd => BinaryKind::LogicalAnd,
        BinaryOperation::LogicalOr => BinaryKind::LogicalOr,
    };

    let lhs = interpret_expression(context, local_context, lhs)?;
    let rhs = interpret_expression(context, local_context, rhs)?;
    let mut lhs_type = lhs.type_handle;
    let mut rhs_type = rhs.type_handle;
    let mut lhs = lhs.register(&mut context.values);
    let mut rhs = rhs.register(&mut context.values);

    let result_type = match kind {
        BinaryKind::Exponent |
        BinaryKind::Remainder => {
            // The result must be arithmetic and cannot be a point
            lhs = lhs.coerce(context, TypeHandle::ARITHMETIC_SCALAR, true)?;
            rhs = rhs.coerce(context, TypeHandle::ARITHMETIC_SCALAR, true)?;
            lhs_type = context.values.get_type(lhs);
            rhs_type = context.values.get_type(rhs);
            context.types.merge(lhs_type, rhs_type, span)?
        }
        BinaryKind::Multiply => {
            // The result must be arithmetic, but at most one operand may be a point
            lhs = lhs.coerce(context, TypeHandle::ARITHMETIC_SCALAR_OR_POINT, true)?;
            lhs_type = context.values.get_type(lhs);
            let (lhs_list, lhs_inner) = context.types.flatten_list(lhs_type);
            if matches!(context.types.get(lhs_inner), Type::Point2D { .. } | Type::Point3D { .. }) {
                rhs = rhs.coerce(context, TypeHandle::ARITHMETIC_SCALAR, true)?;
                rhs_type = context.values.get_type(rhs);
                let (rhs_list, rhs_inner) = context.types.flatten_list(rhs_type);
                let result_inner = match context.types.get(lhs_inner) {
                    &Type::Point2D { x_type, y_type } => {
                        let x_type = context.types.merge(x_type, rhs_inner, span)?;
                        let y_type = context.types.merge(y_type, rhs_inner, span)?;
                        context.types.point_2d_type(x_type, y_type, span)?
                    }
                    &Type::Point3D { x_type, y_type, z_type } => {
                        let x_type = context.types.merge(x_type, rhs_inner, span)?;
                        let y_type = context.types.merge(y_type, rhs_inner, span)?;
                        let z_type = context.types.merge(z_type, rhs_inner, span)?;
                        context.types.point_3d_type(x_type, y_type, z_type, span)?
                    }
                    _ => unreachable!()
                };
                context.types.unflatten_list(ListState::merge(lhs_list, rhs_list), result_inner, span)?
            }
            else {
                rhs = rhs.coerce(context, TypeHandle::ARITHMETIC_SCALAR_OR_POINT, true)?;
                rhs_type = context.values.get_type(rhs);
                let (rhs_list, rhs_inner) = context.types.flatten_list(rhs_type);
                let result_inner = match context.types.get(rhs_inner) {
                    &Type::Point2D { x_type, y_type } => {
                        let x_type = context.types.merge(x_type, lhs_inner, span)?;
                        let y_type = context.types.merge(y_type, lhs_inner, span)?;
                        context.types.point_2d_type(x_type, y_type, span)?
                    }
                    &Type::Point3D { x_type, y_type, z_type } => {
                        let x_type = context.types.merge(x_type, lhs_inner, span)?;
                        let y_type = context.types.merge(y_type, lhs_inner, span)?;
                        let z_type = context.types.merge(z_type, lhs_inner, span)?;
                        context.types.point_3d_type(x_type, y_type, z_type, span)?
                    }
                    _ => context.types.merge(lhs_inner, rhs_inner, span)?
                };
                context.types.unflatten_list(ListState::merge(lhs_list, rhs_list), result_inner, span)?
            }
        }
        BinaryKind::Divide => {
            // The result is always assumed to be real, but lhs may be a point
            lhs = lhs.coerce(context, TypeHandle::REAL_SCALAR_OR_POINT, true)?;
            rhs = rhs.coerce(context, TypeHandle::REAL, true)?;
            lhs_type = context.values.get_type(lhs);
            rhs_type = context.values.get_type(rhs);
            let (lhs_list, lhs_inner) = context.types.flatten_list(lhs_type);
            let (rhs_list, _) = context.types.flatten_list(rhs_type);
            context.types.unflatten_list(ListState::merge(lhs_list, rhs_list), lhs_inner, span)?
        }
        BinaryKind::Add |
        BinaryKind::Subtract => {
            // The result must be arithmetic, but may be a point
            lhs = lhs.coerce(context, TypeHandle::ARITHMETIC_SCALAR_OR_POINT, true)?;
            rhs = rhs.coerce(context, TypeHandle::ARITHMETIC_SCALAR_OR_POINT, true)?;
            lhs_type = context.values.get_type(lhs);
            rhs_type = context.values.get_type(rhs);
            context.types.merge(lhs_type, rhs_type, span)?
        }
        BinaryKind::Equal |
        BinaryKind::NotEqual => {
            // The operands must merge into a numeric or point type, but the result is always a bool
            let (lhs_list, lhs_inner) = context.types.flatten_list(lhs_type);
            let (rhs_list, rhs_inner) = context.types.flatten_list(rhs_type);
            let merged_inner = context.types.merge_inner(lhs_inner, rhs_inner, span)?;
            context.expect_coercible(merged_inner, TypeHandle::REAL_SCALAR_OR_POINT, span)?;
            context.types.unflatten_list(ListState::merge(lhs_list, rhs_list), TypeHandle::BOOL, span)?
        }
        BinaryKind::LogicalAnd |
        BinaryKind::LogicalOr => {
            lhs = lhs.coerce(context, TypeHandle::BOOL, true)?;
            rhs = rhs.coerce(context, TypeHandle::BOOL, true)?;
            lhs_type = context.values.get_type(lhs);
            rhs_type = context.values.get_type(rhs);
            context.types.merge(lhs_type, rhs_type, span)?
        }
        _ => unreachable!("all cases from the previous match should be covered")
    };

    Ok(ValueEntry {
        value: Value::Binary {
            kind,
            lhs,
            rhs,
        },
        type_handle: result_type,
        span,
    })
}

fn interpret_access_operation(
    context: &mut GlobalContext,
    local_context: &LocalContext,
    lhs: &Expression,
    rhs: &Expression,
    span: Option<crate::Span>,
) -> crate::Result<ValueEntry> {
    let lhs = interpret_expression(context, local_context, lhs)?;
    let lhs_type = lhs.type_handle;
    let (lhs_list, lhs_type) = context.types.flatten_list(lhs_type);
    let lhs = lhs.register(&mut context.values);

    let ExpressionKind::Identifier(rhs_identifier) = &rhs.kind else {
        return Err(Box::new(crate::Error {
            kind: crate::ErrorKind::ExpectedIdentifier,
            span: Some(rhs.span),
        }))
    };

    let invalid_access_error = |context: &GlobalContext| Box::new(crate::Error {
        kind: crate::ErrorKind::InvalidAccessOperation {
            lhs_type: context.types.repr(lhs_type),
            rhs: rhs_identifier.clone(),
        },
        span: Some(rhs.span),
    });

    if let &Value::Type(type_handle) = context.values.get(lhs) {
        return match context.types.get(type_handle) {
            Type::Enum { identifier, values } => {
                let value = values
                    .iter()
                    .find_map(|&(ref value_identifier, value)| {
                        (rhs_identifier == value_identifier).then_some(value)
                    })
                    .ok_or_else(|| Box::new(crate::Error {
                        kind: crate::ErrorKind::UndefinedEnumValue {
                            enum_identifier: identifier.clone(),
                            variant_identifier: rhs_identifier.clone(),
                        },
                        span: Some(rhs.span),
                    }))?;

                Ok(ValueEntry {
                    value: Value::GlobalReference(GlobalSymbol {
                        kind: GlobalSymbolKind::EnumOrdinal,
                        identifier: rhs_identifier.clone(),
                        value,
                    }),
                    type_handle,
                    span,
                })
            }
            _ => Err(invalid_access_error(context))
        }
    }

    match context.types.get(lhs_type) {
        &Type::Point2D { x_type, y_type } => {
            let (kind, result_type) = match rhs_identifier.as_ref() {
                "x" => (UnaryKind::XOfPoint2D, x_type),
                "y" => (UnaryKind::YOfPoint2D, y_type),
                _ => return Err(invalid_access_error(context))
            };

            Ok(ValueEntry {
                value: Value::Unary {
                    kind,
                    operand: lhs,
                },
                type_handle: context.types.unflatten_list(lhs_list, result_type, span)?,
                span,
            })
        }
        &Type::Point3D { x_type, y_type, z_type } => {
            let (kind, result_type) = match rhs_identifier.as_ref() {
                "x" => (UnaryKind::XOfPoint3D, x_type),
                "y" => (UnaryKind::YOfPoint3D, y_type),
                "z" => (UnaryKind::ZOfPoint3D, z_type),
                _ => return Err(invalid_access_error(context))
            };

            Ok(ValueEntry {
                value: Value::Unary {
                    kind,
                    operand: lhs,
                },
                type_handle: context.types.unflatten_list(lhs_list, result_type, span)?,
                span,
            })
        }
        _ => Err(invalid_access_error(context))
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
    let rhs = interpret_expression(context, local_context, rhs)?;
    let (mut list_state, mut compared_type) = context.types.flatten_list(rhs.type_handle);

    // Descend into the LHS subtree as long as comparison operations is found.
    // Any chained comparisons without parentheses will be in the left subtree because comparisons
    // have left-to-right associativity.
    let mut chain_rev = vec![(rightmost_kind, rhs.register(&mut context.values))];
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

            let rhs = interpret_expression(context, local_context, rhs)?;
            let (rhs_list, rhs_inner) = context.types.flatten_list(rhs.type_handle);

            list_state = ListState::merge(list_state, rhs_list);
            compared_type = context.types.merge(compared_type, rhs_inner, rhs.span)?;

            chain_rev.push((kind, rhs.register(&mut context.values)));
            current_lhs = lhs.as_ref();
        }
        else {
            break
        }
    }

    let lhs =  interpret_expression(context, local_context, current_lhs)?;
    let (lhs_list, lhs_inner) = context.types.flatten_list(lhs.type_handle);

    list_state = ListState::merge(list_state, lhs_list);
    compared_type = context.types.merge(compared_type, lhs_inner, lhs.span)?;

    context.expect_coercible(compared_type, TypeHandle::REAL, span)?;

    Ok(ValueEntry {
        value: Value::InequalityChain {
            lhs: lhs.register(&mut context.values),
            chain: chain_rev.into_iter().rev().collect(),
        },
        type_handle: context.types.unflatten_list(list_state, TypeHandle::BOOL, span)?,
        span,
    })
}

pub fn interpret_index_operation(
    context: &mut GlobalContext,
    local_context: &LocalContext,
    operation: &IndexOperation,
) -> crate::Result<(IndexKind, Option<ListState>)> {
    match operation {
        IndexOperation::Single { index } => {
            let index = interpret_expression(context, local_context, index)?;
            let list_state = context.types.flatten_list(index.type_handle).0;
            let index = index.register(&mut context.values);

            Ok((
                IndexKind::Single {
                    index: index.coerce(context, TypeHandle::INT, true)?,
                },
                list_state,
            ))
        }
        IndexOperation::Range { kind, from_index, to_index, step } => {
            let from_index = interpret_expression(context, local_context, from_index)?
                .register(&mut context.values);
            let to_index = interpret_expression(context, local_context, to_index)?
                .register(&mut context.values);
            let step = match step {
                Some(step) => interpret_expression(context, local_context, step)?
                    .register(&mut context.values),
                None => ValueHandle::ONE_INT,
            };

            Ok((
                IndexKind::Range {
                    kind: *kind,
                    from_index: from_index.coerce(context, TypeHandle::INT, false)?,
                    to_index: to_index.coerce(context, TypeHandle::INT, false)?,
                    step: step.coerce(context, TypeHandle::INT, false)?,
                },
                Some(ListState::IsList),
            ))
        }
        IndexOperation::RangeFrom { from_index, step } => {
            let from_index = interpret_expression(context, local_context, from_index)?
                .register(&mut context.values);
            let step = match step {
                Some(step) => interpret_expression(context, local_context, step)?
                    .register(&mut context.values),
                None => ValueHandle::ONE_INT,
            };

            Ok((
                IndexKind::RangeFrom {
                    from_index: from_index.coerce(context, TypeHandle::INT, false)?,
                    step: step.coerce(context, TypeHandle::INT, false)?,
                },
                Some(ListState::IsList),
            ))
        }
        IndexOperation::RangeTo { kind, to_index } => {
            let to_index = interpret_expression(context, local_context, to_index)?
                .register(&mut context.values);

            Ok((
                IndexKind::RangeTo {
                    kind: *kind,
                    to_index: to_index.coerce(context, TypeHandle::INT, false)?,
                },
                Some(ListState::IsList),
            ))
        }
    }
}
