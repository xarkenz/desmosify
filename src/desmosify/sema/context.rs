use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use crate::ast::{Declaration, Definition, DefinitionKind, TypeDefinition, TypeExpression, TypeExpressionKind, ValueDefinition};
use crate::sema::intrinsic::{get_core_intrinsics, Intrinsic};
use crate::sema::types::{Type, FunctionSignature, ListState, TypeRegistry, TypeHandle};
use crate::sema::values::{GlobalSymbol, GlobalSymbolKind, Value, ValueEntry, ValueHandle, ValueRegistry};
use crate::target::Target;

#[derive(Debug)]
pub struct GlobalContext<'a> {
    pub source_paths: &'a [PathBuf],
    pub target: &'a mut dyn Target,
    pub types: TypeRegistry,
    pub values: ValueRegistry,
    globals: HashMap<Rc<str>, GlobalSymbol>,
    globals_order: Vec<Rc<str>>,
    actions: HashMap<Rc<str>, GlobalSymbol>,
    action_definitions_order: Vec<Rc<str>>,
    intrinsics: HashMap<Rc<str>, GlobalSymbol>,
}

impl<'a> GlobalContext<'a> {
    pub fn initialize(source_paths: &'a [PathBuf], target: &'a mut dyn Target, declarations: &[Declaration]) -> crate::Result<Self> {
        let mut context = Self {
            source_paths,
            target,
            types: TypeRegistry::new(),
            values: ValueRegistry::new(),
            globals: HashMap::new(),
            globals_order: Vec::new(),
            actions: HashMap::new(),
            action_definitions_order: Vec::new(),
            intrinsics: HashMap::new(),
        };

        // Load the core set of intrinsics.
        context.intrinsics = get_core_intrinsics(context.target)
            .map(|(identifier, intrinsic)| {
                let identifier: Rc<str> = identifier.into();
                (identifier.clone(), GlobalSymbol {
                    kind: GlobalSymbolKind::Intrinsic,
                    identifier,
                    value: match intrinsic {
                        Intrinsic::Entry(entry) => {
                            context.values.register(entry)
                        }
                        Intrinsic::Handle(handle) => handle,
                    },
                })
            })
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

                    let type_definition = Type::Enum {
                        identifier: identifier.clone(),
                        values: variants
                            .iter()
                            .map(|variant| (
                                variant.identifier.clone(),
                                context.values.register(ValueEntry {
                                    value: Value::Opaque,
                                    type_handle,
                                    span: Some(match &variant.value {
                                        Some(value) => value.span,
                                        None => variant.identifier_span.tail_point(),
                                    }),
                                }),
                            ))
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

            context.declare_global(GlobalSymbol {
                kind: GlobalSymbolKind::UserDefinedType,
                identifier: identifier.clone(),
                value: value_handle,
            }, Some(*span))?;
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
                ValueDefinition::Let { parameters, value_type, value } => {
                    let mut value_type = context.resolve_type(value_type, parameters.is_some())?;

                    if let Some(parameters) = parameters {
                        let parameter_types = parameters.0
                            .iter()
                            .map(|parameter| context.resolve_type(&parameter.parameter_type, true))
                            .collect::<crate::Result<_>>()?;
                        value_type = context.types.function_type(FunctionSignature {
                            parameter_types,
                            return_type: value_type,
                        });
                    }

                    let value_handle = context.values.register(ValueEntry {
                        value: Value::Opaque,
                        type_handle: value_type,
                        span: Some(value.span),
                    });

                    context.declare_global(GlobalSymbol {
                        kind: GlobalSymbolKind::Immutable,
                        identifier: identifier.clone(),
                        value: value_handle,
                    }, Some(*span))?;
                }
                ValueDefinition::Variable { value_type, value, .. } => {
                    let value_type = context.resolve_type(value_type, false)?;

                    let value_handle = context.values.register(ValueEntry {
                        value: Value::Opaque,
                        type_handle: value_type,
                        span: Some(value.span),
                    });

                    context.declare_global(GlobalSymbol {
                        kind: GlobalSymbolKind::Variable,
                        identifier: identifier.clone(),
                        value: value_handle,
                    }, Some(*span))?;
                }
                ValueDefinition::Action { parameters, action } => {
                    let parameter_types = parameters.0
                        .iter()
                        .map(|parameter| context.resolve_type(&parameter.parameter_type, false))
                        .collect::<crate::Result<_>>()?;
                    let value_type = context.types.action_type(parameter_types);

                    let value_handle = context.values.register(ValueEntry {
                        value: Value::Opaque,
                        type_handle: value_type,
                        span: Some(action.span),
                    });

                    context.declare_action(GlobalSymbol {
                        kind: GlobalSymbolKind::Action,
                        identifier: identifier.clone(),
                        value: value_handle,
                    }, Some(*span))?;
                }
            }
        }

        Ok(context)
    }

    pub fn declare_global(&mut self, global: GlobalSymbol, span: Option<crate::Span>) -> crate::Result<()> {
        let identifier = global.identifier.clone();

        if TypeHandle::find_primitive(&identifier).is_some() {
            Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ReservedIdentifier {
                    identifier,
                },
                span,
            }))
        } else if self.globals.insert(identifier.clone(), global).is_some() {
            Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ConflictingGlobalIdentifiers {
                    identifier,
                },
                span,
            }))
        } else {
            self.globals_order.push(identifier);
            Ok(())
        }
    }

    pub fn declare_action(&mut self, action: GlobalSymbol, span: Option<crate::Span>) -> crate::Result<()> {
        let identifier = action.identifier.clone();

        if self.actions.insert(identifier.clone(), action).is_some() {
            Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ConflictingActionIdentifiers {
                    identifier,
                },
                span,
            }))
        } else {
            self.action_definitions_order.push(identifier);
            Ok(())
        }
    }

    pub fn globals(&self) -> impl Iterator<Item = &GlobalSymbol> {
        self.globals_order
            .iter()
            .map(|identifier| &self.globals[identifier])
    }

    pub fn find_global(&self, identifier: &str) -> Option<&GlobalSymbol> {
        self.globals.get(identifier)
    }

    pub fn actions(&self) -> impl Iterator<Item = &GlobalSymbol> {
        self.action_definitions_order
            .iter()
            .map(|identifier| &self.actions[identifier])
    }

    pub fn find_action(&self, identifier: &str) -> Option<&GlobalSymbol> {
        self.actions.get(identifier)
    }

    pub fn intrinsics(&self) -> impl Iterator<Item = &GlobalSymbol> {
        self.intrinsics.values()
    }

    pub fn find_intrinsic(&self, identifier: &str) -> Option<&GlobalSymbol> {
        self.intrinsics.get(identifier)
    }

    pub fn new_local_context(&self, source_id: usize) -> LocalContext<'a> {
        LocalContext::new(&self.source_paths[source_id])
    }

    pub fn resolve_type(&mut self, type_expression: &TypeExpression, allow_broadcastable: bool) -> crate::Result<TypeHandle> {
        match &type_expression.kind {
            TypeExpressionKind::Identifier(identifier) => {
                if let Some(primitive) = TypeHandle::find_primitive(identifier) {
                    Ok(primitive)
                } else if let Some(&Value::Type(type_handle)) = self.find_global(identifier)
                    .map(|global| self.values.get(global.value))
                {
                    Ok(type_handle)
                } else {
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

                self.types.list_type(ListState::IsList, item_type, Some(type_expression.span))
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

                self.types.list_type(ListState::MaybeList, item_type, Some(type_expression.span))
            }
            TypeExpressionKind::Point2 { x_type, y_type } => {
                // Allow broadcastable components for the purposes of giving a nicer error message
                let x_type = self.resolve_type(x_type, true)?;
                let y_type = self.resolve_type(y_type, true)?;

                self.types.point_2d_type(x_type, y_type, Some(type_expression.span))
            }
            TypeExpressionKind::Point3 { x_type, y_type, z_type } => {
                // Allow broadcastable components for the purposes of giving a nicer error message
                let x_type = self.resolve_type(x_type, true)?;
                let y_type = self.resolve_type(y_type, true)?;
                let z_type = self.resolve_type(z_type, true)?;

                self.types.point_3d_type(x_type, y_type, z_type, Some(type_expression.span))
            }
        }
    }

    pub fn coerce_value(&mut self, handle: ValueHandle, to_type: TypeHandle, allow_list: bool) -> crate::Result<ValueHandle> {
        self.values.coerce(&mut self.types, handle, to_type, allow_list).ok_or_else(|| Box::new(crate::Error {
            kind: crate::ErrorKind::MismatchedTypes {
                expected_type: self.types.repr(to_type),
                got_type: self.types.repr(self.values.get_type(handle)),
            },
            span: self.values.get_span(handle),
        }))
    }

    pub fn expect_coercible(&self, from_type: TypeHandle, to_type: TypeHandle, span: Option<crate::Span>) -> crate::Result<()> {
        if !self.types.can_coerce(from_type, to_type) {
            Err(Box::new(crate::Error {
                kind: crate::ErrorKind::MismatchedTypes {
                    expected_type: self.types.repr(to_type),
                    got_type: self.types.repr(from_type),
                },
                span,
            }))
        }
        else {
            Ok(())
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
