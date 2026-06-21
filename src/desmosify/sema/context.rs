use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use crate::ast::{Declaration, Definition, DefinitionKind, DisplayDeclaration, PublicDeclaration, TickerDeclaration, TypeExpression, TypeExpressionKind, ValueDefinition};
use crate::sema::intrinsic::get_core_intrinsics;
use crate::sema::types::{Type, FunctionSignature, ListState};
use crate::sema::values::{LocalReference, ValueKind};
use crate::target::Target;

#[derive(Clone, Debug)]
pub struct TypedDefinition {
    pub definition: Definition,
    pub value_type: Type,
}

#[derive(Debug)]
pub struct GlobalContext {
    definitions: HashMap<Rc<str>, TypedDefinition>,
    definitions_order: Vec<Rc<str>>,
    action_definitions: HashMap<Rc<str>, TypedDefinition>,
    action_definitions_order: Vec<Rc<str>>,
    intrinsics: HashMap<&'static str, ValueKind>,
    ticker_declarations: Vec<TickerDeclaration>,
    public_declarations: Vec<PublicDeclaration>,
    display_declarations: Vec<DisplayDeclaration>,
}

impl GlobalContext {
    pub fn from_declarations(mut declarations: Vec<Declaration>, target: &dyn Target) -> crate::Result<Self> {
        let mut context = Self {
            definitions: HashMap::new(),
            definitions_order: Vec::new(),
            action_definitions: HashMap::new(),
            action_definitions_order: Vec::new(),
            intrinsics: get_core_intrinsics(target).collect(),
            ticker_declarations: Vec::new(),
            public_declarations: Vec::new(),
            display_declarations: Vec::new(),
        };

        // Process type definitions first so they can be used to deduce the data types of values.
        let type_declarations = declarations
            .extract_if(.., |declaration| {
                matches!(declaration, Declaration::Definition(Definition {
                    kind: DefinitionKind::Type(..),
                    ..
                }))
            });
        for type_declaration in type_declarations {
            let Declaration::Definition(type_definition) = type_declaration else {
                // It would not have been extracted otherwise...
                unreachable!()
            };

            context.add_definition(TypedDefinition {
                value_type: Type::Meta {
                    identifier: type_definition.identifier.clone(),
                },
                definition: type_definition,
            })?;
        }

        // Process the remaining declarations, including value definitions.
        for declaration in declarations {
            match declaration {
                Declaration::Definition(definition) => match &definition.kind {
                    DefinitionKind::Type(..) => {
                        // We already processed type definitions.
                        unreachable!()
                    },
                    DefinitionKind::Value(ValueDefinition::Let { parameters, value_type, .. }) => {
                        let mut value_type = context.resolve_type(value_type, parameters.is_some())?;

                        if let Some(parameters) = parameters {
                            value_type = Type::UserFunction {
                                signature: Box::new(FunctionSignature {
                                    parameter_types: parameters.0
                                        .iter()
                                        .map(|parameter| context.resolve_type(&parameter.parameter_type, true))
                                        .collect::<crate::Result<_>>()?,
                                    return_type: value_type,
                                }),
                            };
                        }

                        context.add_definition(TypedDefinition {
                            definition,
                            value_type,
                        })?;
                    }
                    DefinitionKind::Value(ValueDefinition::Variable { value_type, .. }) => {
                        let value_type = context.resolve_type(value_type, false)?;

                        context.add_definition(TypedDefinition {
                            definition,
                            value_type,
                        })?;
                    }
                    DefinitionKind::Value(ValueDefinition::Action { parameters, .. }) => {
                        let value_type = Type::Action {
                            parameter_types: parameters.0
                                .iter()
                                .map(|parameter| context.resolve_type(&parameter.parameter_type, false))
                                .collect::<crate::Result<_>>()?,
                        };

                        context.add_action_definition(TypedDefinition {
                            definition,
                            value_type,
                        })?;
                    }
                }
                Declaration::Ticker(ticker_declaration) => {
                    context.add_ticker_declaration(ticker_declaration);
                }
                Declaration::Public(public_declaration) => {
                    context.add_public_declaration(public_declaration);
                }
                Declaration::Display(display_declaration) => {
                    context.add_display_declaration(display_declaration);
                }
            }
        }

        Ok(context)
    }

    pub fn add_definition(&mut self, definition: TypedDefinition) -> crate::Result<()> {
        let identifier = definition.definition.identifier.clone();
        if let Some(old_definition) = self.definitions.insert(identifier.clone(), definition) {
            Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ConflictingGlobalIdentifiers {
                    identifier: old_definition.definition.identifier.as_ref().into(),
                },
                span: Some(old_definition.definition.span),
            }))
        }
        else {
            self.definitions_order.push(identifier);
            Ok(())
        }
    }

    pub fn add_action_definition(&mut self, definition: TypedDefinition) -> crate::Result<()> {
        let identifier = definition.definition.identifier.clone();
        if let Some(old_definition) = self.action_definitions.insert(identifier.clone(), definition) {
            Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ConflictingActionIdentifiers {
                    identifier: old_definition.definition.identifier.as_ref().into(),
                },
                span: Some(old_definition.definition.span),
            }))
        }
        else {
            self.action_definitions_order.push(identifier);
            Ok(())
        }
    }

    pub fn add_ticker_declaration(&mut self, ticker_declaration: TickerDeclaration) {
        self.ticker_declarations.push(ticker_declaration);
    }

    pub fn add_public_declaration(&mut self, public_declaration: PublicDeclaration) {
        self.public_declarations.push(public_declaration);
    }

    pub fn add_display_declaration(&mut self, display_declaration: DisplayDeclaration) {
        self.display_declarations.push(display_declaration);
    }

    pub fn definitions(&self) -> impl Iterator<Item = (&Rc<str>, &TypedDefinition)> {
        self.definitions_order
            .iter()
            .map(|identifier| (identifier, &self.definitions[identifier]))
    }

    pub fn find_definition(&self, identifier: &str) -> Option<&TypedDefinition> {
        self.definitions.get(identifier)
    }

    pub fn action_definitions(&self) -> impl Iterator<Item = (&Rc<str>, &TypedDefinition)> {
        self.action_definitions_order
            .iter()
            .map(|identifier| (identifier, &self.action_definitions[identifier]))
    }

    pub fn find_action_definition(&self, identifier: &str) -> Option<&TypedDefinition> {
        self.action_definitions.get(identifier)
    }

    pub fn intrinsics(&self) -> impl Iterator<Item = &ValueKind> {
        self.intrinsics.values()
    }

    pub fn find_intrinsic(&self, identifier: &str) -> Option<&ValueKind> {
        self.intrinsics.get(identifier)
    }

    pub fn ticker_declarations(&self) -> &[TickerDeclaration] {
        &self.ticker_declarations
    }

    pub fn public_declarations(&self) -> &[PublicDeclaration] {
        &self.public_declarations
    }

    pub fn display_declarations(&self) -> &[DisplayDeclaration] {
        &self.display_declarations
    }

    pub fn resolve_type(&self, type_expression: &TypeExpression, allow_broadcastable: bool) -> crate::Result<Type> {
        let check_point_component = |component_type: Type| {
            if !component_type.is_numeric() {
                Err(Box::new(crate::Error {
                    kind: crate::ErrorKind::InvalidPointComponentType {
                        component_type: component_type.to_string(),
                    },
                    span: Some(type_expression.span),
                }))
            }
            else {
                Ok(component_type)
            }
        };

        match &type_expression.kind {
            TypeExpressionKind::Identifier(identifier) => {
                if let Some(primitive) = Type::find_primitive(identifier) {
                    Ok(primitive)
                }
                else if let Some(TypedDefinition {
                                     value_type: Type::Meta { identifier },
                                     ..
                                 }) = self.find_definition(identifier) {
                    Ok(Type::Enum {
                        type_identifier: identifier.clone(),
                    })
                }
                else {
                    Err(Box::new(crate::Error {
                        kind: crate::ErrorKind::UnrecognizedType {
                            identifier: identifier.as_ref().into(),
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

                if item_type.list_state().is_some() {
                    Err(Box::new(crate::Error {
                        kind: crate::ErrorKind::InvalidListItemType {
                            item_type: item_type.to_string(),
                        },
                        span: Some(type_expression.span),
                    }))
                }
                else {
                    Ok(item_type.into_list(ListState::IsList))
                }
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

                if item_type.list_state().is_some() {
                    Err(Box::new(crate::Error {
                        kind: crate::ErrorKind::InvalidBroadcastableItemType {
                            item_type: item_type.to_string(),
                        },
                        span: Some(type_expression.span),
                    }))
                }
                else {
                    Ok(item_type.into_list(ListState::MaybeList))
                }
            }
            TypeExpressionKind::Point2 { x_type, y_type } => {
                // Allow broadcastable components for the purposes of giving a nicer error message
                Ok(Type::Point2 {
                    x_type: Box::new(check_point_component(self.resolve_type(x_type, true)?)?),
                    y_type: Box::new(check_point_component(self.resolve_type(y_type, true)?)?),
                })
            }
            TypeExpressionKind::Point3 { x_type, y_type, z_type } => {
                // Allow broadcastable components for the purposes of giving a nicer error message
                Ok(Type::Point3 {
                    x_type: Box::new(check_point_component(self.resolve_type(x_type, true)?)?),
                    y_type: Box::new(check_point_component(self.resolve_type(y_type, true)?)?),
                    z_type: Box::new(check_point_component(self.resolve_type(z_type, true)?)?),
                })
            }
        }
    }
}

#[derive(Debug)]
pub struct LocalContext<'a> {
    outer_context: Option<&'a Self>,
    source_path: &'a Path,
    locals: HashMap<Rc<str>, ValueKind>,
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

    pub fn add_local(&mut self, identifier: Rc<str>, value: ValueKind) {
        // TODO: prevent duplicate names?
        self.locals.insert(identifier, value);
    }

    pub fn add_local_variable(&mut self, identifier: Rc<str>, id: u64, value_type: Type) -> LocalReference {
        let local_reference = LocalReference {
            id,
            value_type,
        };
        self.add_local(identifier, ValueKind::Local(local_reference.clone()));
        local_reference
    }

    pub fn find_local(&self, identifier: &str) -> Option<&ValueKind> {
        self.locals.get(identifier).or_else(|| {
            self.outer_context.and_then(|context| context.find_local(identifier))
        })
    }
}
