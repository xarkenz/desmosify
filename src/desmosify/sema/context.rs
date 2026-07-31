use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use crate::ast::{Declaration, Definition, DefinitionKind, TypeDefinition, TypeExpression, TypeExpressionKind, ValueDefinition};
use crate::sema::intrinsic::{get_core_intrinsics, Intrinsic};
use crate::sema::ProgramAction;
use crate::sema::types::{Type, FunctionSignature, ListState, TypeRegistry, TypeHandle};
use crate::sema::values::{BinaryKind, Value, ValueEntry, ValueHandle, ValueRegistry, ValueTag, ValueTagKind};
use crate::target::Target;

#[derive(Debug)]
pub struct GlobalContext {
    types: TypeRegistry,
    values: ValueRegistry,
    globals: HashMap<Rc<str>, ValueHandle>,
    globals_order: Vec<Rc<str>>,
    actions: HashMap<Rc<str>, ProgramAction>,
    action_definitions_order: Vec<Rc<str>>,
    intrinsics: HashMap<&'static str, ValueHandle>,
}

impl GlobalContext {
    pub fn from_declarations(declarations: &[Declaration], target: &dyn Target) -> crate::Result<Self> {
        let mut context = Self {
            types: TypeRegistry::new(),
            values: ValueRegistry::new(),
            globals: HashMap::new(),
            globals_order: Vec::new(),
            actions: HashMap::new(),
            action_definitions_order: Vec::new(),
            intrinsics: HashMap::new(),
        };

        // Load the core set of intrinsics.
        context.intrinsics = get_core_intrinsics(target)
            .map(|(identifier, intrinsic)| (identifier, match intrinsic {
                Intrinsic::Entry(entry) => {
                    context.values.register(entry)
                }
                Intrinsic::Handle(handle) => handle,
            }))
            .collect();

        // Process type definitions first so they can be used to deduce the types of values.
        for declaration in declarations {
            let Declaration::Definition(Definition {
                identifier,
                kind: DefinitionKind::Type(definition),
                span,
            }) = declaration else {
                continue
            };

            let type_handle = match definition {
                TypeDefinition::Enumeration { variants } => {
                    let type_handle = context.types.register(Type::Any);

                    let mut previous_ordinal = None;
                    let type_definition = Type::Enum {
                        identifier: identifier.clone(),
                        values: variants
                            .iter()
                            .map(|variant| {
                                let tag = Some(ValueTag {
                                    identifier: variant.identifier.clone(),
                                    kind: ValueTagKind::EnumOrdinal,
                                });
                                let ordinal = if let Some(value) = &variant.value {
                                    // Insert a placeholder for the explicit value.
                                    context.values.register(ValueEntry {
                                        value: Value::Opaque,
                                        type_handle,
                                        tag,
                                        span: Some(value.span),
                                    })
                                }
                                else if let Some(previous_ordinal) = previous_ordinal {
                                    // Use the ordinal of the previous value plus one.
                                    context.values.register(ValueEntry {
                                        value: Value::Binary {
                                            kind: BinaryKind::Add,
                                            lhs: previous_ordinal,
                                            rhs: ValueHandle::ONE_INT,
                                        },
                                        type_handle,
                                        tag,
                                        span: Some(variant.identifier_span.tail_point()),
                                    })
                                }
                                else {
                                    // No previous value, so start at zero.
                                    context.values.register(ValueEntry {
                                        value: Value::Int(0),
                                        type_handle,
                                        tag,
                                        span: Some(variant.identifier_span.tail_point()),
                                    })
                                };

                                (variant.identifier.clone(), ordinal)
                            })
                            .collect(),
                    };

                    context.types.reregister(type_handle, type_definition);
                    type_handle
                }
            };

            let value_handle = context.values.register(ValueEntry {
                value: Value::Type(type_handle),
                type_handle: TypeHandle::META,
                span: None,
            });

            context.declare_global(identifier.clone(), Some(*span), value_handle)?;
        }

        // Register placeholder values for all value definitions.
        for declaration in declarations {
            let Declaration::Definition(Definition {
                identifier,
                kind: DefinitionKind::Value(definition),
                span,
            }) = declaration else {
                continue
            };

            match definition {
                ValueDefinition::Let { parameters, value_type, .. } => {
                    let mut value_type = context.resolve_type(value_type, parameters.is_some())?;

                    if let Some(parameters) = parameters {
                        value_type = context.types.function_type(FunctionSignature {
                            parameter_types: parameters.0
                                .iter()
                                .map(|parameter| context.resolve_type(&parameter.parameter_type, true))
                                .collect::<crate::Result<_>>()?,
                            return_type: value_type,
                        });
                    }

                    context.declare_global(identifier.clone(), )?;
                }
                ValueDefinition::Variable { value_type, .. } => {
                    let value_type = context.resolve_type(value_type, false)?;

                    context.declare_global(TypedDefinition {
                        definition,
                        value_type,
                    })?;
                }
                ValueDefinition::Action { parameters, .. } => {
                    let value_type = Type::Action {
                        parameter_types: parameters.0
                            .iter()
                            .map(|parameter| context.resolve_type(&parameter.parameter_type, false))
                            .collect::<crate::Result<_>>()?,
                    };

                    context.declare_action(TypedDefinition {
                        definition,
                        value_type,
                    })?;
                }
            }
        }

        Ok(context)
    }

    pub fn declare_global(&mut self, identifier: Rc<str>, span: Option<crate::Span>, value: ValueHandle) -> crate::Result<()> {
        if TypeHandle::find_primitive(&identifier).is_some() {
            Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ReservedIdentifier {
                    identifier,
                },
                span,
            }))
        }
        else if self.globals.insert(identifier.clone(), value).is_some() {
            Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ConflictingGlobalIdentifiers {
                    identifier,
                },
                span,
            }))
        }
        else {
            self.globals_order.push(identifier);
            Ok(())
        }
    }

    pub fn declare_action(&mut self, action: ProgramAction, span: Option<crate::Span>) -> crate::Result<()> {
        let identifier = action.identifier.clone();
        let action_span = action.action.span;
        if self.actions.insert(identifier.clone(), action).is_some() {
            Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ConflictingActionIdentifiers {
                    identifier,
                },
                span: span.or(action_span),
            }))
        }
        else {
            self.action_definitions_order.push(identifier);
            Ok(())
        }
    }

    pub fn globals(&self) -> impl Iterator<Item = (Rc<str>, ValueHandle)> {
        self.globals_order
            .iter()
            .map(|identifier| (identifier.clone(), self.globals[identifier]))
    }

    pub fn find_global(&self, identifier: &str) -> Option<ValueHandle> {
        self.globals.get(identifier).copied()
    }

    pub fn actions(&self) -> impl Iterator<Item = (Rc<str>, &ProgramAction)> {
        self.action_definitions_order
            .iter()
            .map(|identifier| (identifier.clone(), &self.actions[identifier]))
    }

    pub fn find_action(&self, identifier: &str) -> Option<&ProgramAction> {
        self.actions.get(identifier)
    }

    pub fn intrinsics(&self) -> impl Iterator<Item = ValueHandle> {
        self.intrinsics.values().copied()
    }

    pub fn find_intrinsic(&self, identifier: &str) -> Option<ValueHandle> {
        self.intrinsics.get(identifier).copied()
    }

    pub fn resolve_type(&mut self, type_expression: &TypeExpression, allow_broadcastable: bool) -> crate::Result<TypeHandle> {
        match &type_expression.kind {
            TypeExpressionKind::Identifier(identifier) => {
                if let Some(primitive) = TypeHandle::find_primitive(identifier) {
                    Ok(primitive)
                }
                else if let Some(&Value::Type(type_handle)) = self.find_global(identifier)
                    .map(|value| self.values.get(value))
                {
                    Ok(type_handle)
                }
                else {
                    Err(Box::new(crate::Error {
                        kind: crate::ErrorKind::UnrecognizedType {
                            identifier: identifier.clone(),
                        },
                        span: Some(type_expression.span),
                    }))
                }
            }
            TypeExpressionKind::Grouping { expression } => {
                self.resolve_type(expression, allow_broadcastable)
            }
            TypeExpressionKind::List { item_type } => {
                // Allow broadcastable for the purposes of giving a nicer error message
                let item_type = self.resolve_type(item_type, true)?;

                self.types.list_type(ListState::IsList, item_type)
            }
            TypeExpressionKind::Broadcastable { item_type } => {
                if !allow_broadcastable {
                    return Err(Box::new(crate::Error {
                        kind: crate::ErrorKind::BroadcastableTypeNotAllowed,
                        span: Some(type_expression.span),
                    }));
                }

                // Allow broadcastable for the purposes of giving a nicer error message
                let item_type = self.resolve_type(item_type, true)?;

                self.types.list_type(ListState::MaybeList, item_type)
            }
            TypeExpressionKind::Point2 { x_type, y_type } => {
                // Allow broadcastable components for the purposes of giving a nicer error message
                let x_type = self.resolve_type(x_type, true)?;
                let y_type = self.resolve_type(y_type, true)?;

                self.types.point_2d_type(x_type, y_type)
            }
            TypeExpressionKind::Point3 { x_type, y_type, z_type } => {
                // Allow broadcastable components for the purposes of giving a nicer error message
                let x_type = self.resolve_type(x_type, true)?;
                let y_type = self.resolve_type(y_type, true)?;
                let z_type = self.resolve_type(z_type, true)?;

                self.types.point_3d_type(x_type, y_type, z_type)
            }
        }
    }
}

#[derive(Debug)]
pub struct LocalContext<'a> {
    outer_context: Option<&'a Self>,
    source_path: &'a Path,
    locals: HashMap<Rc<str>, ValueHandle>,
}

impl<'a> LocalContext<'a> {
    pub fn new(source_path: &'a Path) -> Self {
        Self {
            outer_context: None,
            source_path,
            locals: HashMap::new(),
        }
    }

    pub fn new_inner(&'a self) -> Self {
        Self {
            outer_context: Some(self),
            source_path: self.source_path,
            locals: HashMap::new(),
        }
    }

    pub fn source_path(&self) -> &Path {
        self.source_path
    }

    pub fn source_directory(&self) -> &Path {
        if self.source_path.is_dir() {
            self.source_path
        }
        else {
            self.source_path.parent().unwrap_or(self.source_path)
        }
    }

    pub fn add_local(&mut self, identifier: Rc<str>, value: ValueHandle) {
        // TODO: prevent duplicate names?
        self.locals.insert(identifier, value);
    }

    pub fn find_local(&self, identifier: &str) -> Option<ValueHandle> {
        self.locals.get(identifier).copied().or_else(|| {
            self.outer_context.and_then(|context| context.find_local(identifier))
        })
    }
}
