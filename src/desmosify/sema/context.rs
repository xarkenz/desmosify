use std::collections::HashMap;
use std::rc::Rc;
use crate::ast::{Declaration, Definition, DefinitionKind, DisplayDeclaration, PublicDeclaration, TickerDeclaration, TypeExpression, TypeExpressionKind, ValueDefinition};
use crate::sema::intrinsic::Intrinsic;
use crate::sema::types::{DataType, FunctionSignature};

#[derive(Clone, Debug)]
pub struct TypedDefinition {
    pub definition: Definition,
    pub data_type: DataType,
}

#[derive(Debug)]
pub struct GlobalContext {
    definitions: HashMap<Rc<str>, TypedDefinition>,
    intrinsics: HashMap<Rc<str>, &'static Intrinsic>,
    ticker_declarations: Vec<TickerDeclaration>,
    public_declarations: Vec<PublicDeclaration>,
    display_declarations: Vec<DisplayDeclaration>,
}

impl GlobalContext {
    pub fn from_declarations(mut declarations: Vec<Declaration>) -> crate::Result<Self> {
        let mut context = Self {
            definitions: HashMap::new(),
            intrinsics: HashMap::new(),
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
                data_type: DataType::UserType {
                    identifier: type_definition.identifier.clone(),
                },
                definition: type_definition,
            })?;
        }

        // Process the remaining declarations, including value definitions.
        for declaration in declarations {
            match declaration {
                Declaration::Definition(definition) => {
                    let data_type = match &definition.kind {
                        DefinitionKind::Type(..) => {
                            // We already processed type definitions.
                            unreachable!()
                        },
                        DefinitionKind::Value(ValueDefinition::Let { parameters, value_type, .. }) => {
                            let value_type = context.resolve_type(value_type)?;
                            if let Some(parameters) = parameters {
                                DataType::UserFunction {
                                    signature: Box::new(FunctionSignature {
                                        parameter_types: parameters.0
                                            .iter()
                                            .map(|(_, parameter_type)| context.resolve_type(parameter_type))
                                            .collect::<crate::Result<_>>()?,
                                        return_type: value_type,
                                    }),
                                }
                            }
                            else {
                                value_type
                            }
                        }
                        DefinitionKind::Value(ValueDefinition::Variable { value_type, .. }) => {
                            context.resolve_type(value_type)?
                        }
                        DefinitionKind::Value(ValueDefinition::Action { parameters, .. }) => {
                            DataType::Action {
                                parameter_types: parameters.0
                                    .iter()
                                    .map(|(_, parameter_type)| context.resolve_type(parameter_type))
                                    .collect::<crate::Result<_>>()?,
                            }
                        }
                    };

                    context.add_definition(TypedDefinition {
                        definition,
                        data_type,
                    })?;
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
        if let Some(old_definition) = self.definitions.insert(identifier, definition) {
            Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ConflictingGlobalIdentifiers {
                    identifier: old_definition.definition.identifier.as_ref().into(),
                },
                span: Some(old_definition.definition.span),
            }))
        }
        else {
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

    pub fn find_definition(&self, identifier: &str) -> Option<&TypedDefinition> {
        self.definitions.get(identifier)
    }

    pub fn resolve_type(&self, type_expression: &TypeExpression) -> crate::Result<DataType> {
        match &type_expression.kind {
            TypeExpressionKind::Any => {
                Ok(DataType::Any)
            }
            TypeExpressionKind::Identifier(identifier) => {
                if let Some(primitive) = DataType::find_primitive(identifier) {
                    Ok(primitive)
                }
                else if let Some(TypedDefinition {
                                     data_type: DataType::UserType { identifier },
                                     ..
                                 }) = self.find_definition(identifier) {
                    Ok(DataType::UserValue {
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
                self.resolve_type(expression)
            }
            TypeExpressionKind::List { item_type } => {
                Ok(DataType::List {
                    item_type: Box::new(self.resolve_type(item_type)?),
                })
            }
            TypeExpressionKind::Point2 { x_type, y_type } => {
                Ok(DataType::Point2 {
                    x_type: Box::new(self.resolve_type(x_type)?),
                    y_type: Box::new(self.resolve_type(y_type)?),
                })
            }
            TypeExpressionKind::Point3 { x_type, y_type, z_type } => {
                Ok(DataType::Point3 {
                    x_type: Box::new(self.resolve_type(x_type)?),
                    y_type: Box::new(self.resolve_type(y_type)?),
                    z_type: Box::new(self.resolve_type(z_type)?),
                })
            }
        }
    }
}

#[derive(Debug)]
pub struct LocalContext<'a> {
    outer_context: Option<&'a Self>,
    locals: HashMap<Rc<str>, DataType>,
    scoped_intrinsics: Vec<&'static Intrinsic>,
}

impl<'a> LocalContext<'a> {
    pub fn new() -> Self {
        Self {
            outer_context: None,
            locals: HashMap::new(),
            scoped_intrinsics: Vec::new(),
        }
    }

    pub fn new_inner(&'a self) -> LocalContext<'a> {
        Self {
            outer_context: Some(self),
            locals: HashMap::new(),
            scoped_intrinsics: Vec::new(),
        }
    }

    pub fn add_local(&mut self, identifier: Rc<str>, data_type: DataType) {
        // TODO: prevent duplicate names?
        self.locals.insert(identifier, data_type);
    }

    pub fn find_local(&self, identifier: &str) -> Option<&DataType> {
        self.locals.get(identifier).or_else(|| {
            self.outer_context.and_then(|context| context.find_local(identifier))
        })
    }

    pub fn add_scoped_intrinsic(&mut self, intrinsic: &'static Intrinsic) {
        self.scoped_intrinsics.push(intrinsic);
    }

    pub fn find_scoped_intrinsic(&self, identifier: &str) -> Option<&'static Intrinsic> {
        self.scoped_intrinsics.iter().copied().find(|intrinsic| intrinsic.identifier == identifier).or_else(|| {
            self.outer_context.and_then(|context| context.find_scoped_intrinsic(identifier))
        })
    }
}
