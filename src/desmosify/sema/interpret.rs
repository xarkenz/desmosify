use std::path::PathBuf;
use std::rc::Rc;
use crate::ast::{ActionExpression, ActionExpressionKind, BinaryOperation, DefinitionKind, DisplayAttribute, DisplayAttributeValue, Expression, IndexOperation, ExpressionKind, ParameterList, PublicLineKind, TypeDefinition, UnaryOperation, ValueDefinition, VariableKind, EnumerationVariant};
use crate::sema::{Program, ProgramAction, ProgramEnumeration, ProgramImmutable, ProgramPublic, ProgramPublicLine, ProgramTicker, ProgramVariable, ProgramVariableKind};
use crate::sema::context::{GlobalContext, LocalContext};
use crate::sema::display::{DragMode, LabelOrientation, LineStyle, PointStyle, ProgramDisplay, ProgramDisplayAttribute, ProgramDisplayAttributeKind, ProgramDisplayElement};
use crate::sema::types::{ListState, Type};
use crate::sema::values::{ActionValue, ActionValueKind, GlobalReference, LocalReference, Value, IndexKind, ValueKind, ListMapLoop, BinaryKind, UnaryKind, ActionReference, InequalityKind};
use crate::target::Target;

pub fn interpret_program(source_paths: &[PathBuf], target: &mut dyn Target, context: &GlobalContext) -> crate::Result<Program> {
    let mut enumerations = Vec::new();
    let mut immutables = Vec::new();
    let mut variables = Vec::new();
    let mut actions = Vec::new();

    for (identifier, definition) in context.definitions().chain(context.action_definitions()) {
        let local_context = LocalContext::new(&source_paths[definition.definition.span.source_id]);

        match &definition.definition.kind {
            DefinitionKind::Type(TypeDefinition::Enumeration { variants }) => {
                enumerations.push(interpret_enumeration_definition(
                    target,
                    context,
                    local_context,
                    identifier.clone(),
                    variants,
                )?);
            }
            DefinitionKind::Value(ValueDefinition::Let { parameters, value, .. }) => {
                immutables.push(interpret_let_definition(
                    target,
                    context,
                    local_context,
                    identifier.clone(),
                    parameters.as_ref(),
                    &definition.value_type,
                    value,
                )?);
            }
            DefinitionKind::Value(ValueDefinition::Variable { kind, value, .. }) => {
                variables.push(interpret_variable_definition(
                    target,
                    context,
                    local_context,
                    identifier.clone(),
                    kind,
                    &definition.value_type,
                    value,
                )?);
            }
            DefinitionKind::Value(ValueDefinition::Action { parameters, action }) => {
                actions.push(interpret_action_definition(
                    target,
                    context,
                    local_context,
                    identifier.clone(),
                    parameters,
                    &definition.value_type,
                    action,
                )?);
            }
        }
    }

    let ticker = interpret_ticker_declarations(source_paths, target, context)?;
    let public = interpret_public_declarations(source_paths, target, context, &mut variables)?;
    let display = interpret_display_declarations(source_paths, target, context)?;

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
    target: &mut dyn Target,
    context: &GlobalContext,
    local_context: LocalContext,
    identifier: Rc<str>,
    variants: &[EnumerationVariant],
) -> crate::Result<ProgramEnumeration> {
    let mut values: Vec<(Rc<str>, Value)> = Vec::with_capacity(variants.len());

    for variant in variants {
        let value = if let Some(value) = &variant.value {
            interpret_expression(target, context, &local_context, value)?
                .coerce_to(&Type::Int, false)?
        }
        else if let Some((_, previous_value)) = values.last() {
            // TODO: eliminate this once constant folding is implemented?
            if let ValueKind::Int(previous_int) = previous_value.kind {
                // TODO: should raise error if this would be too big
                ValueKind::Int(previous_int + 1).into()
            }
            else {
                ValueKind::Binary {
                    kind: BinaryKind::Add,
                    lhs: Box::new(previous_value.clone()),
                    rhs: Box::new(ValueKind::Int(1).into()),
                    result_type: Type::Int,
                }.into()
            }
        }
        else {
            ValueKind::Int(0).into()
        };

        values.push((variant.identifier.clone(), value));
    }

    Ok(ProgramEnumeration {
        identifier,
        values: values.into_boxed_slice(),
    })
}

pub fn interpret_let_definition(
    target: &mut dyn Target,
    context: &GlobalContext,
    mut local_context: LocalContext,
    identifier: Rc<str>,
    parameters: Option<&ParameterList>,
    value_type: &Type,
    value: &Expression,
) -> crate::Result<ProgramImmutable> {
    let (typed_parameters, expected_type) = if let Some(parameters) = parameters {
        let Type::UserFunction { signature } = value_type else {
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
    target: &mut dyn Target,
    context: &GlobalContext,
    local_context: LocalContext,
    identifier: Rc<str>,
    kind: &VariableKind,
    value_type: &Type,
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
    target: &mut dyn Target,
    context: &GlobalContext,
    mut local_context: LocalContext,
    identifier: Rc<str>,
    parameters: &ParameterList,
    action_type: &Type,
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
    target: &mut dyn Target,
    local_context: &mut LocalContext,
    parameters: &ParameterList,
    parameter_types: &[Type],
) -> Box<[LocalReference]> {
    std::iter::zip(&parameters.0, parameter_types)
        .map(|(parameter, parameter_type)| {
            local_context.add_local_variable(parameter.identifier.clone(), target.create_local_id(), parameter_type.clone())
        })
        .collect()
}

pub fn interpret_ticker_declarations(
    source_paths: &[PathBuf],
    target: &mut dyn Target,
    context: &GlobalContext,
) -> crate::Result<Option<ProgramTicker>> {
    let mut tick_actions = Vec::with_capacity(context.ticker_declarations().len());

    let interval_ms = context
        .ticker_declarations()
        .iter()
        .enumerate()
        .try_fold::<_, _, crate::Result<_>>(
            None,
            |interval_ms, (index, declaration)| {
                let mut local_context = LocalContext::new(&source_paths[declaration.span.source_id]);

                let new_interval_ms = match declaration.interval_ms.as_ref() {
                    Some(interval_expression) => Some(interpret_expression(target, context, &local_context, interval_expression)?),
                    None => None,
                };
                if index != 0 && new_interval_ms != interval_ms {
                    return Err(Box::new(crate::Error {
                        kind: crate::ErrorKind::IncompatibleTickerIntervals,
                        span: Some(declaration.span),
                    }))
                }

                local_context.add_scoped_intrinsic("dt", ValueKind::TickerDt);

                let new_tick_action = interpret_action_expression(target, context, &local_context, &declaration.tick_action)?;
                tick_actions.push(new_tick_action);

                Ok(new_interval_ms)
            },
        )?;

    let tick_action = match tick_actions.len() {
        0 => return Ok(None),
        1 => tick_actions.into_iter().next().unwrap(),
        2.. => ActionValueKind::Compound {
            actions: tick_actions.into_boxed_slice(),
        }.into()
    };

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

pub fn interpret_public_declarations(
    source_paths: &[PathBuf],
    target: &mut dyn Target,
    context: &GlobalContext,
    variables: &mut Vec<ProgramVariable>,
) -> crate::Result<Option<ProgramPublic>> {
    let mut lines = Vec::new();

    for declaration in context.public_declarations() {
        let local_context = LocalContext::new(&source_paths[declaration.span.source_id]);

        for line in &declaration.lines {
            lines.push(match &line.kind {
                PublicLineKind::Expression(expression) => {
                    ProgramPublicLine::Expression(interpret_expression(target, context, &local_context, expression)?)
                }
                PublicLineKind::Action(action) => {
                    ProgramPublicLine::Action(interpret_action_expression(target, context, &local_context, action)?)
                }
                PublicLineKind::Slider { var_identifier } => {
                    let var_index = variables
                        .iter()
                        .position(|variable| &variable.identifier == var_identifier)
                        .ok_or_else(|| {
                            // We can provide some pretty good diagnostics for this error.
                            let Some(definition) = context.find_definition(var_identifier) else {
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
                    ProgramPublicLine::Variable(variables.remove(var_index))
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

pub fn interpret_display_declarations(
    source_paths: &[PathBuf],
    target: &mut dyn Target,
    context: &GlobalContext,
) -> crate::Result<Option<ProgramDisplay>> {
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

    fn require_arguments(attribute: &DisplayAttribute, min_arity: usize, max_arity: usize) -> crate::Result<&[Expression]> {
        let DisplayAttributeValue::Arguments(arguments) = &attribute.value else {
            return Err(Box::new(crate::Error {
                kind: crate::ErrorKind::DisplayAttributeExpectedArguments {
                    key: attribute.key.as_ref().into(),
                },
                span: Some(attribute.key_span),
            }))
        };

        if (min_arity ..= max_arity).contains(&arguments.len()) {
            Ok(arguments)
        }
        else {
            Err(Box::new(crate::Error {
                kind: crate::ErrorKind::InvalidDisplayAttributeArity {
                    key: attribute.key.as_ref().into(),
                    min: min_arity,
                    max: max_arity,
                    got: arguments.len(),
                },
                span: Some(attribute.key_span),
            }))
        }
    }

    fn require_action(attribute: &DisplayAttribute) -> crate::Result<&ActionExpression> {
        if let DisplayAttributeValue::Action(action) = &attribute.value {
            Ok(action)
        }
        else {
            Err(Box::new(crate::Error {
                kind: crate::ErrorKind::DisplayAttributeExpectedAction {
                    key: attribute.key.as_ref().into(),
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
                        // color(<color>: color)
                        prevent_duplicate(attribute, &mut has_color)?;
                        let arguments = require_arguments(attribute, 1, 1)?;

                        ProgramDisplayAttributeKind::Color {
                            value: interpret_expression(target, context, &local_context, &arguments[0])?
                                .coerce_to(&Type::Color, true)?,
                        }
                    }
                    "point" => {
                        // point([opacity]: real, [size]: real, [style]: str, [outline]: bool)
                        prevent_duplicate(attribute, &mut has_point)?;
                        let arguments = require_arguments(attribute, 0, 4)?;

                        ProgramDisplayAttributeKind::Point {
                            opacity: interpret_option!(arguments.get(0), &Type::Real)?,
                            size: interpret_option!(arguments.get(1), &Type::Real)?,
                            style: interpret_option_named!(arguments.get(2), PointStyle)?,
                            outline: interpret_option_bool!(arguments.get(3), false)?,
                        }
                    }
                    "drag" => {
                        // drag([mode]: str)
                        prevent_duplicate(attribute, &mut has_drag)?;
                        let arguments = require_arguments(attribute, 0, 1)?;

                        ProgramDisplayAttributeKind::Drag {
                            mode: interpret_option_named!(arguments.get(0), DragMode)?,
                        }
                    }
                    "label" => {
                        // label(<text>: str, [opacity]: real, [size]: real, [angle]: real,
                        //       [orientation]: str, [outline]: bool)
                        prevent_duplicate(attribute, &mut has_label)?;
                        let arguments = require_arguments(attribute, 1, 6)?;

                        ProgramDisplayAttributeKind::Label {
                            text: interpret_expression(target, context, &local_context, &arguments[0])?
                                .get_const_str()?,
                            opacity: interpret_option!(arguments.get(1), &Type::Real)?,
                            size: interpret_option!(arguments.get(2), &Type::Real)?,
                            angle: interpret_option!(arguments.get(3), &Type::Real)?,
                            orientation: interpret_option_named!(arguments.get(4), LabelOrientation)?,
                            outline: interpret_option_bool!(arguments.get(5), true)?,
                        }
                    }
                    "line" => {
                        // line([opacity]: real, [width]: real, [style]: str)
                        prevent_duplicate(attribute, &mut has_line)?;
                        let arguments = require_arguments(attribute, 0, 3)?;

                        ProgramDisplayAttributeKind::Line {
                            opacity: interpret_option!(arguments.get(0), &Type::Real)?,
                            width: interpret_option!(arguments.get(1), &Type::Real)?,
                            style: interpret_option_named!(arguments.get(2), LineStyle)?,
                        }
                    }
                    "fill" => {
                        // fill([opacity]: real)
                        prevent_duplicate(attribute, &mut has_fill)?;
                        let arguments = require_arguments(attribute, 0, 1)?;

                        ProgramDisplayAttributeKind::Fill {
                            opacity: interpret_option!(arguments.get(0), &Type::Real)?,
                        }
                    }
                    "click" => {
                        // TODO: click(action(index) { ... }) to allow other attributes?
                        // click { ... }
                        prevent_duplicate(attribute, &mut has_click)?;
                        let action = require_action(attribute)?;

                        let mut action_context = local_context.new_inner();
                        // TODO: only available when list?
                        action_context.add_scoped_intrinsic("index", ValueKind::ClickIndex);

                        ProgramDisplayAttributeKind::Click {
                            action: interpret_action_expression(target, context, &action_context, action)?,
                        }
                    }
                    "hovered" => {
                        // hovered(<url>: str)
                        prevent_duplicate(attribute, &mut has_hovered)?;
                        let arguments = require_arguments(attribute, 1, 1)?;

                        ProgramDisplayAttributeKind::Hovered {
                            url: interpret_expression(target, context, &local_context, &arguments[0])?
                                .get_const_str()?,
                        }
                    }
                    "pressed" => {
                        // pressed(<url>: str)
                        prevent_duplicate(attribute, &mut has_pressed)?;
                        let arguments = require_arguments(attribute, 1, 1)?;

                        ProgramDisplayAttributeKind::Pressed {
                            url: interpret_expression(target, context, &local_context, &arguments[0])?
                                .get_const_str()?,
                        }
                    }
                    "description" => {
                        // description(<text>: str)
                        prevent_duplicate(attribute, &mut has_description)?;
                        let arguments = require_arguments(attribute, 1, 1)?;

                        ProgramDisplayAttributeKind::Description {
                            text: interpret_expression(target, context, &local_context, &arguments[0])?
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
pub fn interpret_action_expression(
    target: &mut dyn Target,
    context: &GlobalContext,
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
            let ValueKind::Global(variable) = variable.kind else {
                return Err(invalid_update_lhs_error())
            };
            let DefinitionKind::Value(ValueDefinition::Variable { .. }) = context.find_definition(&variable.identifier).unwrap().definition.kind else {
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
    target: &mut dyn Target,
    context: &GlobalContext,
    local_context: &LocalContext,
    expression: &Expression,
) -> crate::Result<Value> {
    let kind = match &expression.kind {
        ExpressionKind::Identifier(identifier) => {
            if let Some(local) = local_context.find_local(identifier) {
                local.clone()
            }
            else if let Some(definition) = context.find_definition(identifier) {
                ValueKind::Global(GlobalReference {
                    identifier: identifier.clone(),
                    value_type: definition.value_type.clone(),
                })
            }
            else {
                return Err(Box::new(crate::Error {
                    kind: crate::ErrorKind::UndefinedIdentifier {
                        identifier: identifier.as_ref().into(),
                    },
                    span: Some(expression.span),
                }))
            }
        }
        ExpressionKind::Real(value) => {
            ValueKind::Real(*value)
        }
        ExpressionKind::Integer(value) => {
            let value = i64::try_from(*value).map_err(|_| Box::new(crate::Error {
                kind: crate::ErrorKind::IntegerTooLarge,
                span: Some(expression.span),
            }))?;

            ValueKind::Int(value)
        }
        ExpressionKind::Boolean(value) => {
            ValueKind::Bool(*value)
        }
        ExpressionKind::String(value) => {
            ValueKind::Str(value.clone())
        }
        ExpressionKind::ActionIdentifier(identifier) => {
            if let Some(definition) = context.find_action_definition(identifier) {
                ValueKind::Action(ActionReference {
                    identifier: identifier.clone(),
                    action_type: definition.value_type.clone(),
                })
            }
            else {
                return Err(Box::new(crate::Error {
                    kind: crate::ErrorKind::UndefinedAction {
                        identifier: identifier.as_ref().into(),
                    },
                    span: Some(expression.span),
                }))
            }
        }
        ExpressionKind::IntrinsicIdentifier(identifier) => {
            if let Some(intrinsic) = local_context.find_scoped_intrinsic(identifier) {
                intrinsic.clone()
            }
            else if let Some(intrinsic) = context.find_intrinsic(identifier) {
                intrinsic.clone()
            }
            else {
                return Err(Box::new(crate::Error {
                    kind: crate::ErrorKind::UndefinedIntrinsic {
                        identifier: identifier.as_ref().into(),
                    },
                    span: Some(expression.span),
                }))
            }
        }
        ExpressionKind::Grouping { expression } => {
            interpret_expression(target, context, local_context, expression)?.kind
        }
        ExpressionKind::Unary { operation, operand } => {
            interpret_unary_operation(target, context, local_context, *operation, operand)?
        }
        ExpressionKind::Binary { operation, lhs, rhs } => {
            interpret_binary_operation(target, context, local_context, *operation, lhs, rhs, Some(expression.span))?
        }
        ExpressionKind::Point2 { x, y } => {
            let x = interpret_expression(target, context, local_context, x)?;
            let y = interpret_expression(target, context, local_context, y)?;

            let (x_list, x_type) = x.get_type().into_flatten_list();
            let (y_list, y_type) = y.get_type().into_flatten_list();

            ValueKind::Point2 {
                x: Box::new(x),
                y: Box::new(y),
                point_type: Type::Point2 {
                    x_type: Box::new(x_type),
                    y_type: Box::new(y_type),
                }.unflatten_list(ListState::merge(x_list, y_list)),
            }
        }
        ExpressionKind::Point3 { x, y, z } => {
            let x = interpret_expression(target, context, local_context, x)?;
            let y = interpret_expression(target, context, local_context, y)?;
            let z = interpret_expression(target, context, local_context, z)?;

            let (x_list, x_type) = x.get_type().into_flatten_list();
            let (y_list, y_type) = y.get_type().into_flatten_list();
            let (z_list, z_type) = z.get_type().into_flatten_list();

            ValueKind::Point3 {
                x: Box::new(x),
                y: Box::new(y),
                z: Box::new(z),
                point_type: Type::Point3 {
                    x_type: Box::new(x_type),
                    y_type: Box::new(y_type),
                    z_type: Box::new(z_type),
                }.unflatten_list(ListState::merge(ListState::merge(x_list, y_list), z_list)),
            }
        }
        ExpressionKind::List { items } => {
            let items: Vec<_> = items
                .iter()
                .map(|item| interpret_expression(target, context, local_context, item))
                .collect::<crate::Result<_>>()?;

            let item_type = items
                .iter()
                .try_fold(Type::Any, |current_type, item| {
                    current_type.merge(&item.get_type())
                        .map_err(|error| error.with_span(item.span))
                })?;

            ValueKind::List {
                items: items
                    .into_iter()
                    .map(|item| item.coerce_to(&item_type, false))
                    .collect::<crate::Result<_>>()?,
                item_type,
            }
        }
        ExpressionKind::ListRange { kind, start, end, step } => {
            let start = interpret_expression(target, context, local_context, start)?;
            let end = interpret_expression(target, context, local_context, end)?;
            let step = match step {
                Some(step) => interpret_expression(target, context, local_context, step)?,
                None => ValueKind::Int(1).into(),
            };

            let item_type = start.get_type()
                .merge(&end.get_type())
                .map_err(|error| error.with_span(end.span))?
                .merge(&step.get_type())
                .map_err(|error| error.with_span(step.span))?;

            ValueKind::ListRange {
                kind: *kind,
                start: Box::new(start.coerce_to(&item_type, false)?),
                end: Box::new(end.coerce_to(&item_type, false)?),
                step: Box::new(step.coerce_to(&item_type, false)?),
                item_type,
            }
        }
        ExpressionKind::ListFill { value, count } => {
            let value = interpret_expression(target, context, local_context, value)?;
            let count = interpret_expression(target, context, local_context, count)?
                .coerce_to(&Type::Int, false)?;

            ValueKind::ListFill {
                value: Box::new(value),
                count: Box::new(count),
            }
        }
        ExpressionKind::ListMap { loops, expression: map_expression } => {
            let mut map_context = local_context.new_inner();

            let loops = loops
                .iter()
                .map(|map_loop| {
                    let list = interpret_expression(target, context, local_context, &map_loop.list)?;
                    let item_type = list.get_type().require_flatten_list()
                        .map_err(|error| error.with_span(Some(expression.span)))?;

                    Ok(ListMapLoop {
                        local: map_context.add_local_variable(map_loop.identifier.clone(), target.create_local_id(), item_type),
                        local_span: Some(map_loop.identifier_span),
                        list,
                    })
                })
                .collect::<crate::Result<_>>()?;

            let value = interpret_expression(target, context, &map_context, map_expression)?;

            ValueKind::ListMap {
                loops,
                value: Box::new(value),
            }
        }
        ExpressionKind::ListFilter { list, condition } => {
            let list = interpret_expression(target, context, local_context, list)?;
            let item_type = list.get_type().require_flatten_list()
                .map_err(|error| error.with_span(Some(expression.span)))?;

            let condition = interpret_expression(target, context, local_context, condition)?
                .coerce_to(&Type::Bool, true)?;

            ValueKind::ListFilter {
                list: Box::new(list),
                condition: Box::new(condition),
                item_type,
            }
        }
        ExpressionKind::Index { list, operation } => {
            let list = interpret_expression(target, context, local_context, list)?;
            let item_type = list.get_type().require_flatten_list()
                .map_err(|error| error.with_span(Some(expression.span)))?;

            let operation = interpret_index_operation(target, context, local_context, operation)?;

            ValueKind::Index {
                list: Box::new(list),
                kind: operation,
                item_type,
            }
        }
        ExpressionKind::FunctionCall { function, arguments } => {
            let function = interpret_expression(target, context, local_context, function)?;
            let arguments: Box<[_]> = arguments
                .iter()
                .map(|argument| interpret_expression(target, context, local_context, argument))
                .collect::<crate::Result<_>>()?;

            match function.get_type() {
                Type::IntrinsicFunction(intrinsic_function) => {
                    intrinsic_function.interpret_call(target, context, local_context, function.span, arguments)?
                }
                Type::UserFunction { signature } => {
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
                    let arguments = std::iter::zip(arguments, &signature.parameter_types)
                        .map(|(argument, parameter_type)| {
                            if let Type::List { state: ListState::MaybeList, .. } = parameter_type {
                                result_list_state = ListState::merge(result_list_state, argument.get_type().list_state())
                            }
                            argument.coerce_to(parameter_type, false)
                        })
                        .collect::<crate::Result<_>>()?;

                    let return_type = match signature.return_type {
                        Type::List { state: ListState::MaybeList, item_type } => {
                            item_type.unflatten_list(result_list_state)
                        }
                        other => other
                    };

                    ValueKind::UserFunctionCall {
                        function: Box::new(function),
                        arguments,
                        return_type,
                    }
                }
                got_type => {
                    return Err(Box::new(crate::Error {
                        kind: crate::ErrorKind::ExpectedFunctionType {
                            got_type: got_type.to_string(),
                        },
                        span: function.span,
                    }))
                }
            }
        }
        ExpressionKind::Conditional { condition_consequents, alternative } => {
            let mut result_type = Type::Any;

            let condition_consequents: Box<[_]> = condition_consequents
                .iter()
                .map(|(condition, consequent)| {
                    let condition = interpret_expression(target, context, local_context, condition)?
                        .coerce_to(&Type::Bool, true)?;
                    // A list condition should cause the whole expression to broadcast
                    let (result_list, inner_type) = result_type.flatten_list();
                    result_type = inner_type.clone().unflatten_list(ListState::merge(result_list, condition.get_type().list_state()));

                    let consequent = interpret_expression(target, context, local_context, consequent)?;
                    result_type = result_type.merge(&consequent.get_type())
                        .map_err(|error| error.with_span(consequent.span))?;

                    Ok((condition, consequent))
                })
                .collect::<crate::Result<_>>()?;

            let alternative = alternative
                .as_ref()
                .map_or(Ok(ValueKind::Undefined(result_type.clone()).into()), |alternative| {
                    let alternative = interpret_expression(target, context, local_context, alternative)?;
                    result_type = result_type.merge(&alternative.get_type())
                        .map_err(|error| error.with_span(alternative.span))?;

                    alternative.coerce_to(result_type.flatten_list().1, true)
                })?;

            let result_inner_type = result_type.flatten_list().1;
            let condition_consequents = condition_consequents
                .into_iter()
                .map(|(condition, consequent)| {
                    Ok((condition, consequent.coerce_to(result_inner_type, true)?))
                })
                .collect::<crate::Result<_>>()?;

            ValueKind::Conditional {
                condition_consequents,
                alternative: Box::new(alternative),
                result_type,
            }
        }
        ExpressionKind::Let { identifier, value_type, value, inner, .. } => {
            let mut value = interpret_expression(target, context, local_context, value)?;

            if let Some(value_type) = value_type {
                let value_type = context.resolve_type(value_type, true)?;
                value = value.coerce_to(&value_type, false)?;
            }

            let mut inner_context = local_context.new_inner();
            inner_context.add_local(identifier.clone(), value.kind);

            interpret_expression(target, context, &inner_context, inner)?.kind
        }
        _ => {
            return Err(Box::new(crate::Error {
                kind: crate::ErrorKind::UnexpectedExpressionKind,
                span: Some(expression.span),
            }))
        }
    };

    Ok(Value {
        kind,
        span: Some(expression.span),
    })
}

pub fn interpret_unary_operation(
    target: &mut dyn Target,
    context: &GlobalContext,
    local_context: &LocalContext,
    operation: UnaryOperation,
    operand: &Expression,
) -> crate::Result<ValueKind> {
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

    Ok(ValueKind::Unary {
        kind,
        operand: Box::new(operand),
        result_type,
    })
}

pub fn interpret_binary_operation(
    target: &mut dyn Target,
    context: &GlobalContext,
    local_context: &LocalContext,
    operation: BinaryOperation,
    lhs: &Expression,
    rhs: &Expression,
    span: Option<crate::Span>,
) -> crate::Result<ValueKind> {
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
            if matches!(lhs_type, Type::Point2 { .. } | Type::Point3 { .. }) {
                rhs = rhs.coerce_to_arithmetic(Type::require_numeric)?;
                let rhs_type = rhs.get_type().into_flatten_list().1;
                match &lhs_type {
                    Type::Point2 { x_type, y_type } => Type::Point2 {
                        x_type: Box::new(x_type.merge(&rhs_type)
                            .map_err(|error| error.with_span(span))?),
                        y_type: Box::new(y_type.merge(&rhs_type)
                            .map_err(|error| error.with_span(span))?),
                    },
                    Type::Point3 { x_type, y_type, z_type } => Type::Point3 {
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
                    Type::Point2 { x_type, y_type } => Type::Point2 {
                        x_type: Box::new(x_type.merge(&lhs_type)
                            .map_err(|error| error.with_span(span))?),
                        y_type: Box::new(y_type.merge(&lhs_type)
                            .map_err(|error| error.with_span(span))?),
                    },
                    Type::Point3 { x_type, y_type, z_type } => Type::Point3 {
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

    Ok(ValueKind::Binary {
        kind,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
        result_type,
    })
}

fn interpret_access_operation(
    target: &mut dyn Target,
    context: &GlobalContext,
    local_context: &LocalContext,
    lhs: &Expression,
    rhs: &Expression,
) -> crate::Result<ValueKind> {
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

                    Ok(ValueKind::EnumVariant {
                        type_identifier: identifier.clone(),
                        variant_ordinal: ordinal as i64,
                    })
                }
            }
        }
        Type::Point2 { x_type, y_type } => {
            match member_identifier.as_ref() {
                "x" => Ok(ValueKind::Unary {
                    kind: UnaryKind::GetX,
                    operand: Box::new(lhs),
                    result_type: x_type.as_ref().clone().unflatten_list(lhs_list),
                }),
                "y" => Ok(ValueKind::Unary {
                    kind: UnaryKind::GetY,
                    operand: Box::new(lhs),
                    result_type: y_type.as_ref().clone().unflatten_list(lhs_list),
                }),
                _ => Err(invalid_access_error())
            }
        }
        Type::Point3 { x_type, y_type, z_type } => {
            match member_identifier.as_ref() {
                "x" => Ok(ValueKind::Unary {
                    kind: UnaryKind::GetX,
                    operand: Box::new(lhs),
                    result_type: x_type.as_ref().clone().unflatten_list(lhs_list),
                }),
                "y" => Ok(ValueKind::Unary {
                    kind: UnaryKind::GetY,
                    operand: Box::new(lhs),
                    result_type: y_type.as_ref().clone().unflatten_list(lhs_list),
                }),
                "z" => Ok(ValueKind::Unary {
                    kind: UnaryKind::GetZ,
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
    target: &mut dyn Target,
    context: &GlobalContext,
    local_context: &LocalContext,
    rightmost_kind: InequalityKind,
    lhs: &Expression,
    rhs: &Expression,
    span: Option<crate::Span>,
) -> crate::Result<ValueKind> {
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

    Ok(ValueKind::InequalityChain {
        lhs: Box::new(lhs_value),
        chain: chain_rev.into_iter().rev().collect(),
        result_type: Type::Bool.unflatten_list(list_state),
    })
}

pub fn interpret_index_operation(
    target: &mut dyn Target,
    context: &GlobalContext,
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
                None => ValueKind::Int(1).into(),
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
                None => ValueKind::Int(1).into(),
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
