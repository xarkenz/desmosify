use crate::desmos::{BinaryKind, ExpressionEntry, FolderEntry, GraphEntry, GraphExpression, GraphExpressionList, GraphTicker, InequalityKind, UnaryKind};
use crate::desmos::error::{DesmosError, DesmosErrorKind, DesmosResult};
use crate::desmos::symbol::SymbolTable;
use crate::sema::intrinsic::IntrinsicValue;
use crate::sema::types::Type;
use crate::sema::values::{ActionValue, MathematicalConstant, Value, ValueIndexOperation};

pub mod geometry;
pub mod graphing;
pub mod graphing_3d;

pub use geometry::GeometryTarget;
pub use graphing::GraphingTarget;
pub use graphing_3d::Graphing3DTarget;
use crate::ast::{BinaryOperation, RangeKind, UnaryOperation};

pub const GLOBALS_FOLDER_ID: &str = "desmosify_globals";
pub const ACTIONS_FOLDER_ID: &str = "desmosify_actions";
pub const DISPLAY_FOLDER_ID: &str = "desmosify_display";

pub struct GraphExpressionListBuilder {
    ticker: Option<GraphTicker>,
    public_entries: Vec<Box<dyn GraphEntry>>,
    global_entries: Vec<Box<dyn GraphEntry>>,
    action_entries: Vec<Box<dyn GraphEntry>>,
    display_entries: Vec<Box<dyn GraphEntry>>,
    next_entry_id: u64,
    global_symbols: SymbolTable,
    action_symbols: SymbolTable,
    next_dummy_noop_id: u64,
    dummy_unreachable_created: bool,
}

impl GraphExpressionListBuilder {
    pub fn new() -> Self {
        Self {
            ticker: None,
            public_entries: Vec::new(),
            global_entries: vec![Box::new(FolderEntry {
                id: GLOBALS_FOLDER_ID.into(),
                title: "desmosify:globals".into(),
                collapsed: true,
                secret: false,
            })],
            action_entries: vec![Box::new(FolderEntry {
                id: ACTIONS_FOLDER_ID.into(),
                title: "desmosify:actions".into(),
                collapsed: true,
                secret: false,
            })],
            display_entries: vec![Box::new(FolderEntry {
                id: DISPLAY_FOLDER_ID.into(),
                title: "desmosify:display".into(),
                collapsed: true,
                secret: false,
            })],
            next_entry_id: 0,
            global_symbols: SymbolTable::new(GraphExpression::Letter('G')),
            action_symbols: SymbolTable::new(GraphExpression::Letter('A')),
            next_dummy_noop_id: 0,
            dummy_unreachable_created: false,
        }
    }

    pub fn finish(self) -> GraphExpressionList {
        GraphExpressionList {
            ticker: self.ticker,
            entries: self.public_entries
                .into_iter()
                .chain(self.global_entries)
                .chain(self.action_entries)
                .chain(self.display_entries)
                .collect(),
        }
    }

    pub fn create_entry_id(&mut self) -> String {
        let id = self.next_entry_id;
        self.next_entry_id += 1;
        id.to_string()
    }

    pub fn get_global_symbol(&mut self, identifier: &str) -> GraphExpression {
        self.global_symbols.get_symbol(identifier)
    }

    pub fn get_action_symbol(&mut self, identifier: &str) -> GraphExpression {
        self.action_symbols.get_symbol(identifier)
    }

    pub fn get_local_symbol(&mut self, id: u64) -> GraphExpression {
        GraphExpression::Binary {
            kind: BinaryKind::Subscript,
            lhs: Box::new(GraphExpression::Letter('l')),
            rhs: Box::new(GraphExpression::Alphanumeric(id.to_string())),
        }
    }

    pub fn create_dummy_noop(&mut self) -> GraphExpression {
        let dummy_noop_id = self.next_dummy_noop_id;
        self.next_dummy_noop_id += 1;
        let symbol = GraphExpression::Binary {
            kind: BinaryKind::Subscript,
            lhs: Box::new(GraphExpression::Letter('D')),
            rhs: Box::new(GraphExpression::Alphanumeric(format!("Noop{dummy_noop_id}"))),
        };

        let id = self.create_entry_id();
        self.action_entries.push(Box::new(ExpressionEntry {
            id,
            folder_id: Some(ACTIONS_FOLDER_ID.into()),
            expression: Some(GraphExpression::Binary {
                kind: BinaryKind::Equal,
                lhs: Box::new(symbol.clone()),
                rhs: Box::new(GraphExpression::Integer(0)),
            }),
            hidden: false,
        }));

        symbol
    }

    pub fn get_dummy_unreachable(&mut self) -> GraphExpression {
        let symbol = GraphExpression::Binary {
            kind: BinaryKind::Subscript,
            lhs: Box::new(GraphExpression::Letter('D')),
            rhs: Box::new(GraphExpression::Alphanumeric("Unreachable".into())),
        };

        if !self.dummy_unreachable_created {
            let id = self.create_entry_id();
            self.action_entries.push(Box::new(ExpressionEntry {
                id,
                folder_id: Some(ACTIONS_FOLDER_ID.into()),
                expression: Some(GraphExpression::Binary {
                    kind: BinaryKind::Equal,
                    lhs: Box::new(symbol.clone()),
                    rhs: Box::new(GraphExpression::Integer(0)),
                }),
                hidden: false,
            }));

            self.dummy_unreachable_created = true;
        }

        symbol
    }

    pub fn translate_value(&mut self, value: &Value) -> DesmosResult<GraphExpression> {
        match value {
            Value::Undefined(value_type) => {
                let undefined_scalar = GraphExpression::Binary {
                    kind: BinaryKind::Fraction,
                    lhs: Box::new(GraphExpression::Integer(0)),
                    rhs: Box::new(GraphExpression::Integer(0)),
                };
                match value_type {
                    Type::Point2 { .. } => Ok(GraphExpression::Unary {
                        kind: UnaryKind::Parentheses,
                        inner: Box::new(GraphExpression::Sequence {
                            elements: vec![
                                undefined_scalar.clone(),
                                undefined_scalar,
                            ],
                        }),
                    }),
                    Type::Point3 { .. } => Ok(GraphExpression::Unary {
                        kind: UnaryKind::Parentheses,
                        inner: Box::new(GraphExpression::Sequence {
                            elements: vec![
                                undefined_scalar.clone(),
                                undefined_scalar.clone(),
                                undefined_scalar,
                            ],
                        }),
                    }),
                    _ => Ok(undefined_scalar)
                }
            }
            Value::Real(value) => {
                Ok(GraphExpression::Decimal(*value))
            }
            Value::Mathematical { kind, coefficient } => {
                Ok(GraphExpression::Binary {
                    kind: BinaryKind::ImplicitMultiply,
                    lhs: Box::new(GraphExpression::Decimal(*coefficient)),
                    rhs: Box::new(match kind {
                        MathematicalConstant::Pi => GraphExpression::Escape("pi".into()),
                        MathematicalConstant::Tau => GraphExpression::Escape("tau".into()),
                        MathematicalConstant::E => GraphExpression::Letter('e'),
                    }),
                })
            }
            Value::Int(value) => {
                Ok(GraphExpression::Integer(*value))
            }
            Value::Bool(value) => {
                Ok(GraphExpression::Integer(*value as i64))
            }
            Value::EnumVariant { variant_ordinal, .. } => {
                Ok(GraphExpression::Integer(*variant_ordinal))
            }
            Value::Intrinsic(value) => {
                self.translate_intrinsic_value(value)
            }
            Value::Global(reference) => {
                Ok(self.get_global_symbol(&reference.identifier))
            }
            Value::Local(reference) => {
                Ok(self.get_local_symbol(reference.id))
            }
            Value::Unary { operation, operand, .. } => {
                self.translate_unary(*operation, operand)
            }
            Value::Binary { operation, lhs, rhs, .. } => {
                self.translate_binary(*operation, lhs, rhs)
            }
            Value::Point2 { x, y, .. } => {
                Ok(GraphExpression::Unary {
                    kind: UnaryKind::Parentheses,
                    inner: Box::new(GraphExpression::Sequence {
                        elements: vec![
                            self.translate_value(x)?,
                            self.translate_value(y)?,
                        ],
                    }),
                })
            }
            Value::Point3 { x, y, z, .. } => {
                Ok(GraphExpression::Unary {
                    kind: UnaryKind::Parentheses,
                    inner: Box::new(GraphExpression::Sequence {
                        elements: vec![
                            self.translate_value(x)?,
                            self.translate_value(y)?,
                            self.translate_value(z)?,
                        ],
                    }),
                })
            }
            Value::GetX { point, .. } => {
                Ok(GraphExpression::Binary {
                    kind: BinaryKind::Dot,
                    lhs: Box::new(self.translate_value(point)?),
                    rhs: Box::new(GraphExpression::Letter('x')),
                })
            }
            Value::GetY { point, .. } => {
                Ok(GraphExpression::Binary {
                    kind: BinaryKind::Dot,
                    lhs: Box::new(self.translate_value(point)?),
                    rhs: Box::new(GraphExpression::Letter('y')),
                })
            }
            Value::GetZ { point, .. } => {
                Ok(GraphExpression::Binary {
                    kind: BinaryKind::Dot,
                    lhs: Box::new(self.translate_value(point)?),
                    rhs: Box::new(GraphExpression::Letter('z')),
                })
            }
            Value::List { items, .. } => {
                Ok(GraphExpression::Unary {
                    kind: UnaryKind::List,
                    inner: Box::new(GraphExpression::Sequence {
                        elements: items
                            .iter()
                            .map(|item| self.translate_value(item))
                            .collect::<DesmosResult<_>>()?,
                    }),
                })
            }
            Value::ListRange { kind, start, end, step, .. } => {
                self.translate_list_range(*kind, start, end, step)
            }
            Value::ListFill { value, count } => {
                Ok(GraphExpression::Binary {
                    kind: BinaryKind::Call,
                    lhs: Box::new(GraphExpression::OperatorName("repeat".into())),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: vec![
                            self.translate_value(value)?,
                            self.translate_value(count)?,
                        ],
                    }),
                })
            }
            Value::ListMap { loops, value } => {
                Ok(GraphExpression::Unary {
                    kind: UnaryKind::List,
                    inner: Box::new(GraphExpression::Binary {
                        kind: BinaryKind::For,
                        lhs: Box::new(self.translate_value(value)?),
                        rhs: Box::new(GraphExpression::Sequence {
                            elements: loops
                                .iter()
                                .rev()
                                .map(|map_loop| Ok(GraphExpression::Binary {
                                    kind: BinaryKind::Equal,
                                    lhs: Box::new(self.get_local_symbol(map_loop.local.id)),
                                    rhs: Box::new(self.translate_value(&map_loop.list)?),
                                }))
                                .collect::<DesmosResult<_>>()?,
                        }),
                    }),
                })
            }
            Value::ListFilter { list, condition, .. } => {
                Ok(GraphExpression::Binary {
                    kind: BinaryKind::Index,
                    lhs: Box::new(self.translate_value(list)?),
                    rhs: Box::new(self.translate_value(condition)?),
                })
            }
            Value::Index { list, operation, .. } => match operation {
                ValueIndexOperation::Single { index } => {
                    Ok(GraphExpression::Binary {
                        kind: BinaryKind::Index,
                        lhs: Box::new(self.translate_value(list)?),
                        rhs: Box::new(self.translate_value(index)?),
                    })
                }
                ValueIndexOperation::Range { kind, from_index, to_index, step} => {
                    todo!()
                }
                ValueIndexOperation::RangeFrom { from_index, step } => {
                    todo!()
                }
                ValueIndexOperation::RangeTo { kind, to_index } => {
                    todo!()
                }
            }
            Value::Conditional { condition_consequents, alternative, .. } => {
                Ok(GraphExpression::Unary {
                    kind: UnaryKind::Piecewise,
                    inner: Box::new(GraphExpression::Sequence {
                        elements: {
                            let mut elements: Vec<_> = condition_consequents
                                .iter()
                                .map(|(condition, consequent)| Ok(GraphExpression::Binary {
                                    kind: BinaryKind::Colon,
                                    lhs: Box::new(self.translate_condition(condition)?),
                                    rhs: Box::new(self.translate_value(consequent)?),
                                }))
                                .collect::<DesmosResult<_>>()?;
                            if !alternative.is_undefined() {
                                elements.push(self.translate_value(alternative)?);
                            }
                            elements
                        },
                    }),
                })
            }
            Value::UserFunctionCall { function, arguments, .. } => {
                Ok(GraphExpression::Binary {
                    kind: BinaryKind::Call,
                    lhs: Box::new(self.translate_value(function)?),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: arguments
                            .iter()
                            .map(|argument| self.translate_value(argument))
                            .collect::<DesmosResult<_>>()?,
                    }),
                })
            }
            Value::Let { local, value, inner } => {
                Ok(GraphExpression::Binary {
                    kind: BinaryKind::With,
                    lhs: Box::new(self.translate_value(inner)?),
                    rhs: Box::new(GraphExpression::Binary {
                        kind: BinaryKind::Equal,
                        lhs: Box::new(self.get_local_symbol(local.id)),
                        rhs: Box::new(self.translate_value(value)?),
                    }),
                })
            }
            _ => {
                Err(Box::new(DesmosError {
                    kind: DesmosErrorKind::UnsupportedValue,
                }))
            }
        }
    }

    pub fn translate_unary(&mut self, operation: UnaryOperation, operand: &Value) -> DesmosResult<GraphExpression> {
        match operation {
            UnaryOperation::Positive => {
                Ok(GraphExpression::Unary {
                    kind: UnaryKind::Parentheses,
                    inner: Box::new(GraphExpression::Unary {
                        kind: UnaryKind::Positive,
                        inner: Box::new(self.translate_value(operand)?),
                    }),
                })
            }
            UnaryOperation::Negative => {
                Ok(GraphExpression::Unary {
                    kind: UnaryKind::Parentheses,
                    inner: Box::new(GraphExpression::Unary {
                        kind: UnaryKind::Negative,
                        inner: Box::new(self.translate_value(operand)?),
                    }),
                })
            }
            UnaryOperation::LogicalNot => {
                Ok(GraphExpression::Unary {
                    kind: UnaryKind::Piecewise,
                    inner: Box::new(GraphExpression::Sequence {
                        elements: vec![
                            GraphExpression::Binary {
                                kind: BinaryKind::Equal,
                                lhs: Box::new(self.translate_value(operand)?),
                                rhs: Box::new(GraphExpression::Integer(0)),
                            },
                            GraphExpression::Integer(0),
                        ],
                    }),
                })
            }
        }
    }

    pub fn translate_binary(&mut self, operation: BinaryOperation, lhs: &Value, rhs: &Value) -> DesmosResult<GraphExpression> {
        todo!()
    }

    pub fn translate_list_range(&mut self, kind: RangeKind, start: &Value, end: &Value, step: &Value) -> DesmosResult<GraphExpression> {
        todo!()
    }

    pub fn translate_condition(&mut self, value: &Value) -> DesmosResult<GraphExpression> {
        match value {
            Value::Binary { operation: BinaryOperation::Equal, lhs, rhs, .. } => {
                Ok(GraphExpression::Binary {
                    kind: BinaryKind::Equal,
                    lhs: Box::new(self.translate_value(lhs)?),
                    rhs: Box::new(self.translate_value(rhs)?),
                })
            }
            Value::Binary { operation: BinaryOperation::NotEqual, lhs, rhs, .. } => {
                // If only Desmos had an operator for this... substitute with {lhs = rhs, 0} = 0
                Ok(GraphExpression::Binary {
                    kind: BinaryKind::Equal,
                    lhs: Box::new(GraphExpression::Unary {
                        kind: UnaryKind::Piecewise,
                        inner: Box::new(GraphExpression::Sequence {
                            elements: vec![
                                GraphExpression::Binary {
                                    kind: BinaryKind::Equal,
                                    lhs: Box::new(self.translate_value(lhs)?),
                                    rhs: Box::new(self.translate_value(rhs)?),
                                },
                                GraphExpression::Integer(0),
                            ],
                        }),
                    }),
                    rhs: Box::new(GraphExpression::Integer(0)),
                })
            }
            Value::Binary { operation: BinaryOperation::LessThan, lhs, rhs, .. } => {
                Ok(GraphExpression::InequalityChain {
                    first_kind: InequalityKind::LessThan,
                    lhs: Box::new(self.translate_value(lhs)?),
                    rhs: Box::new(self.translate_value(rhs)?),
                    chain: Vec::new(),
                })
            }
            Value::Binary { operation: BinaryOperation::GreaterThan, lhs, rhs, .. } => {
                Ok(GraphExpression::InequalityChain {
                    first_kind: InequalityKind::GreaterThan,
                    lhs: Box::new(self.translate_value(lhs)?),
                    rhs: Box::new(self.translate_value(rhs)?),
                    chain: Vec::new(),
                })
            }
            Value::Binary { operation: BinaryOperation::LessEqual, lhs, rhs, .. } => {
                Ok(GraphExpression::InequalityChain {
                    first_kind: InequalityKind::LessEqual,
                    lhs: Box::new(self.translate_value(lhs)?),
                    rhs: Box::new(self.translate_value(rhs)?),
                    chain: Vec::new(),
                })
            }
            Value::Binary { operation: BinaryOperation::GreaterEqual, lhs, rhs, .. } => {
                Ok(GraphExpression::InequalityChain {
                    first_kind: InequalityKind::GreaterEqual,
                    lhs: Box::new(self.translate_value(lhs)?),
                    rhs: Box::new(self.translate_value(rhs)?),
                    chain: Vec::new(),
                })
            }
            _ => {
                // Do a general "!= 0" check to evaluate a boolean: {value = 0, 0} = 0
                Ok(GraphExpression::Binary {
                    kind: BinaryKind::Equal,
                    lhs: Box::new(GraphExpression::Unary {
                        kind: UnaryKind::Piecewise,
                        inner: Box::new(GraphExpression::Sequence {
                            elements: vec![
                                GraphExpression::Binary {
                                    kind: BinaryKind::Equal,
                                    lhs: Box::new(self.translate_value(value)?),
                                    rhs: Box::new(GraphExpression::Integer(0)),
                                },
                                GraphExpression::Integer(0),
                            ],
                        }),
                    }),
                    rhs: Box::new(GraphExpression::Integer(0)),
                })
            }
        }
    }

    pub fn translate_intrinsic_value(&mut self, value: &IntrinsicValue) -> DesmosResult<GraphExpression> {
        match value {
            _ => {
                Err(Box::new(DesmosError {
                    kind: DesmosErrorKind::UnsupportedValue,
                }))
            }
        }
    }

    pub fn translate_action_value(&mut self, action: &ActionValue) -> DesmosResult<GraphExpression> {
        if action.is_empty() {
            // Usually omitting the action expression is not an option, so update a dummy
            // variable instead.
            return Ok(GraphExpression::Binary {
                kind: BinaryKind::RightArrow,
                lhs: Box::new(self.create_dummy_noop()),
                rhs: Box::new(GraphExpression::Integer(0)),
            });
        }

        match action {
            ActionValue::Disable => {
                // Generate an expression like {0 = 1: unreachable -> 0} since the missing
                // conditional default case is what causes the "disable" behavior.
                Ok(GraphExpression::Unary {
                    kind: UnaryKind::Piecewise,
                    inner: Box::new(GraphExpression::Binary {
                        kind: BinaryKind::Colon,
                        lhs: Box::new(GraphExpression::Binary {
                            kind: BinaryKind::Equal,
                            lhs: Box::new(GraphExpression::Integer(0)),
                            rhs: Box::new(GraphExpression::Integer(1)),
                        }),
                        rhs: Box::new(GraphExpression::Binary {
                            kind: BinaryKind::RightArrow,
                            lhs: Box::new(self.get_dummy_unreachable()),
                            rhs: Box::new(GraphExpression::Integer(0)),
                        }),
                    }),
                })
            }
            ActionValue::Compound { actions } => match actions.as_ref() {
                [] => {
                    // This case was already handled above with the is_empty() check
                    unreachable!()
                }
                [action] => {
                    self.translate_action_value(action)
                }
                _ => {
                    Ok(GraphExpression::Unary {
                        kind: UnaryKind::Parentheses,
                        inner: Box::new(GraphExpression::Sequence {
                            elements: actions
                                .iter()
                                .map(|action| self.translate_action_value(action))
                                .collect::<DesmosResult<_>>()?,
                        }),
                    })
                }
            }
            ActionValue::Update { variable, value } => {
                Ok(GraphExpression::Binary {
                    kind: BinaryKind::RightArrow,
                    lhs: Box::new(self.get_global_symbol(&variable.identifier)),
                    rhs: Box::new(self.translate_value(value)?),
                })
            }
            ActionValue::ActionCall { identifier, arguments } => {
                Ok(GraphExpression::Binary {
                    kind: BinaryKind::Call,
                    lhs: Box::new(self.get_action_symbol(identifier)),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: arguments
                            .iter()
                            .map(|argument| self.translate_value(argument))
                            .collect::<DesmosResult<_>>()?,
                    }),
                })
            }
            ActionValue::Conditional { condition_consequents, alternative } => {
                Ok(GraphExpression::Unary {
                    kind: UnaryKind::Piecewise,
                    inner: Box::new(GraphExpression::Sequence {
                        elements: {
                            let mut elements: Vec<_> = condition_consequents
                                .iter()
                                .map(|(condition, consequent)| Ok(GraphExpression::Binary {
                                    kind: BinaryKind::Colon,
                                    lhs: Box::new(self.translate_condition(condition)?),
                                    rhs: Box::new(self.translate_action_value(consequent)?),
                                }))
                                .collect::<DesmosResult<_>>()?;
                            // TODO: propagate disable
                            elements.push(self.translate_action_value(alternative)?);
                            elements
                        },
                    }),
                })
            }
        }
    }
}
