use crate::ast::{BinaryOperation, RangeKind, UnaryOperation};
use crate::desmos::{GraphBinaryKind, GraphExpressionEntry, GraphFolderEntry, GraphEntry, GraphExpression, GraphExpressionList, GraphTicker, GraphInequalityKind, GraphUnaryKind, GraphTextEntry, GraphPointStyle, GraphLabelOrientation, GraphLineStyle};
use crate::desmos::symbol::SymbolTable;
use crate::sema::{Program, ProgramAction, ProgramDisplayAttribute, ProgramDisplayAttributeValue, ProgramDisplayElement, ProgramLet, ProgramPublicLine, ProgramTicker, ProgramVariable};
use crate::sema::intrinsic::{IntrinsicBinaryKind, IntrinsicColorKind, IntrinsicDoubleReducerKind, IntrinsicParameterizedReducerKind, IntrinsicReducerKind, IntrinsicUnaryKind, IntrinsicValue};
use crate::sema::types::Type;
use crate::sema::values::{ActionValue, ActionValueKind, MathematicalConstant, Value, ValueIndexOperation, ValueKind};

mod geometry;
mod graphing;
mod graphing_3d;

pub use geometry::DesmosGeometryTarget;
pub use graphing::DesmosGraphingTarget;
pub use graphing_3d::DesmosGraphing3DTarget;

pub const INTRINSICS_FOLDER_ID: &str = "desmosify_intrinsics";
pub const GLOBALS_FOLDER_ID: &str = "desmosify_globals";
pub const ACTIONS_FOLDER_ID: &str = "desmosify_actions";
pub const DISPLAY_FOLDER_ID: &str = "desmosify_display";

pub struct GraphExpressionListBuilder {
    ticker: Option<GraphTicker>,
    public_entries: Vec<Box<dyn GraphEntry>>,
    intrinsic_entries: Vec<Box<dyn GraphEntry>>,
    global_entries: Vec<Box<dyn GraphEntry>>,
    action_entries: Vec<Box<dyn GraphEntry>>,
    display_entries: Vec<Box<dyn GraphEntry>>,
    next_local_id: u64,
    next_entry_id: u64,
    global_symbols: SymbolTable,
    action_symbols: SymbolTable,
    next_dummy_noop_id: u64,
    dummy_unreachable_created: bool,
    intrinsic_range_inclusive_created: bool,
    intrinsic_range_exclusive_created: bool,
    intrinsic_index_range_inclusive_created: bool,
    intrinsic_index_range_exclusive_created: bool,
    intrinsic_index_range_from_created: bool,
    intrinsic_index_range_to_inclusive_created: bool,
    intrinsic_index_range_to_exclusive_created: bool,
}

impl GraphExpressionListBuilder {
    pub fn new() -> Self {
        Self {
            ticker: None,
            public_entries: Vec::new(),
            intrinsic_entries: vec![Box::new(GraphFolderEntry {
                id: INTRINSICS_FOLDER_ID.into(),
                title: "desmosify:intrinsics".into(),
                collapsed: true,
                secret: false,
            })],
            global_entries: vec![Box::new(GraphFolderEntry {
                id: GLOBALS_FOLDER_ID.into(),
                title: "desmosify:globals".into(),
                collapsed: true,
                secret: false,
            })],
            action_entries: vec![Box::new(GraphFolderEntry {
                id: ACTIONS_FOLDER_ID.into(),
                title: "desmosify:actions".into(),
                collapsed: true,
                secret: false,
            })],
            display_entries: vec![Box::new(GraphFolderEntry {
                id: DISPLAY_FOLDER_ID.into(),
                title: "desmosify:display".into(),
                collapsed: true,
                secret: false,
            })],
            next_local_id: 0,
            next_entry_id: 0,
            global_symbols: SymbolTable::new(GraphExpression::Letter('G')),
            action_symbols: SymbolTable::new(GraphExpression::Letter('A')),
            next_dummy_noop_id: 0,
            dummy_unreachable_created: false,
            intrinsic_range_inclusive_created: false,
            intrinsic_range_exclusive_created: false,
            intrinsic_index_range_inclusive_created: false,
            intrinsic_index_range_exclusive_created: false,
            intrinsic_index_range_from_created: false,
            intrinsic_index_range_to_inclusive_created: false,
            intrinsic_index_range_to_exclusive_created: false,
        }
    }

    pub fn finish(self) -> GraphExpressionList {
        GraphExpressionList {
            ticker: self.ticker,
            entries: self.public_entries
                .into_iter()
                .chain(self.intrinsic_entries)
                .chain(self.global_entries)
                .chain(self.action_entries)
                .chain(self.display_entries)
                .collect(),
            next_local_id: self.next_local_id,
        }
    }

    pub fn build_program(program: &Program) -> crate::Result<GraphExpressionList> {
        let mut builder = Self::new();
        builder.next_local_id = program.next_local_id();
        builder.set_program(program)?;
        Ok(builder.finish())
    }

    pub fn create_entry_id(&mut self) -> String {
        let id = self.next_entry_id;
        self.next_entry_id += 1;
        id.to_string()
    }

    pub fn create_local_id(&mut self) -> u64 {
        let id = self.next_local_id;
        self.next_local_id += 1;
        id
    }

    pub fn get_global_symbol(&mut self, identifier: &str) -> GraphExpression {
        self.global_symbols.get_symbol(identifier)
    }

    pub fn get_action_symbol(&mut self, identifier: &str) -> GraphExpression {
        self.action_symbols.get_symbol(identifier)
    }

    pub fn get_local_symbol(&mut self, id: u64) -> GraphExpression {
        GraphExpression::Binary {
            kind: GraphBinaryKind::Subscript,
            lhs: Box::new(GraphExpression::Letter('l')),
            rhs: Box::new(GraphExpression::Alphanumeric(id.to_string())),
        }
    }

    pub fn create_local_symbol(&mut self) -> GraphExpression {
        let id = self.create_local_id();
        self.get_local_symbol(id)
    }

    pub fn create_dummy_noop(&mut self) -> GraphExpression {
        let dummy_noop_id = self.next_dummy_noop_id;
        self.next_dummy_noop_id += 1;
        let symbol = GraphExpression::Binary {
            kind: GraphBinaryKind::Subscript,
            lhs: Box::new(GraphExpression::Letter('D')),
            rhs: Box::new(GraphExpression::Alphanumeric(format!("Noop{dummy_noop_id}"))),
        };

        let id = self.create_entry_id();
        self.intrinsic_entries.push(Box::new(GraphExpressionEntry {
            id,
            folder_id: Some(INTRINSICS_FOLDER_ID.into()),
            expression: GraphExpression::Binary {
                kind: GraphBinaryKind::Equal,
                lhs: Box::new(symbol.clone()),
                rhs: Box::new(GraphExpression::Integer(0)),
            },
            ..Default::default()
        }));

        symbol
    }

    pub fn get_dummy_unreachable(&mut self) -> GraphExpression {
        let symbol = GraphExpression::Binary {
            kind: GraphBinaryKind::Subscript,
            lhs: Box::new(GraphExpression::Letter('D')),
            rhs: Box::new(GraphExpression::Alphanumeric("Unreachable".into())),
        };

        if !self.dummy_unreachable_created {
            let id = self.create_entry_id();
            self.intrinsic_entries.push(Box::new(GraphExpressionEntry {
                id,
                folder_id: Some(INTRINSICS_FOLDER_ID.into()),
                expression: GraphExpression::Binary {
                    kind: GraphBinaryKind::Equal,
                    lhs: Box::new(symbol.clone()),
                    rhs: Box::new(GraphExpression::Integer(0)),
                },
                ..Default::default()
            }));

            self.dummy_unreachable_created = true;
        }

        symbol
    }

    pub fn get_intrinsic_range_inclusive(&mut self) -> GraphExpression {
        let symbol = GraphExpression::Binary {
            kind: GraphBinaryKind::Subscript,
            lhs: Box::new(GraphExpression::Letter('I')),
            rhs: Box::new(GraphExpression::Alphanumeric("RangeInc".into())),
        };

        if !self.intrinsic_range_inclusive_created {
            let entry_id = self.create_entry_id();

            let local_a = self.create_local_symbol();
            let local_b = self.create_local_symbol();
            let local_s = self.create_local_symbol();

            // range_inc(a, b, s) = {
            //     a sign(s) > b sign(s): [],
            //     a + s * [0 ... floor((b - a) / s)]
            // }
            // This is terrible.
            let expression = GraphExpression::Binary {
                kind: GraphBinaryKind::Equal,
                lhs: Box::new(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(symbol.clone()),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: Vec::from([
                            local_a.clone(),
                            local_b.clone(),
                            local_s.clone(),
                        ]),
                    }),
                }),
                rhs: Box::new(GraphExpression::Unary {
                    kind: GraphUnaryKind::Piecewise,
                    inner: Box::new(GraphExpression::Sequence {
                        elements: Vec::from([
                            GraphExpression::Binary {
                                kind: GraphBinaryKind::Colon,
                                lhs: Box::new(GraphExpression::InequalityChain {
                                    lhs: Box::new(GraphExpression::Binary {
                                        kind: GraphBinaryKind::ImplicitMultiply,
                                        lhs: Box::new(local_a.clone()),
                                        rhs: Box::new(GraphExpression::Binary {
                                            kind: GraphBinaryKind::Call,
                                            lhs: Box::new(GraphExpression::OperatorName("sign".into())),
                                            rhs: Box::new(local_s.clone()),
                                        }),
                                    }),
                                    first_kind: GraphInequalityKind::GreaterThan,
                                    rhs: Box::new(GraphExpression::Binary {
                                        kind: GraphBinaryKind::ImplicitMultiply,
                                        lhs: Box::new(local_b.clone()),
                                        rhs: Box::new(GraphExpression::Binary {
                                            kind: GraphBinaryKind::Call,
                                            lhs: Box::new(GraphExpression::OperatorName("sign".into())),
                                            rhs: Box::new(local_s.clone()),
                                        }),
                                    }),
                                    chain: Vec::new(),
                                }),
                                rhs: Box::new(GraphExpression::Unary {
                                    kind: GraphUnaryKind::List,
                                    inner: Box::new(GraphExpression::Empty),
                                }),
                            },
                            GraphExpression::Binary {
                                kind: GraphBinaryKind::Add,
                                lhs: Box::new(local_a.clone()),
                                rhs: Box::new(GraphExpression::Binary {
                                    kind: GraphBinaryKind::Multiply,
                                    lhs: Box::new(local_s.clone()),
                                    rhs: Box::new(GraphExpression::Unary {
                                        kind: GraphUnaryKind::List,
                                        inner: Box::new(GraphExpression::Binary {
                                            kind: GraphBinaryKind::Range,
                                            lhs: Box::new(GraphExpression::Integer(0)),
                                            rhs: Box::new(GraphExpression::Binary {
                                                kind: GraphBinaryKind::Call,
                                                lhs: Box::new(GraphExpression::OperatorName("floor".into())),
                                                rhs: Box::new(GraphExpression::Binary {
                                                    kind: GraphBinaryKind::Fraction,
                                                    lhs: Box::new(GraphExpression::Binary {
                                                        kind: GraphBinaryKind::Subtract,
                                                        lhs: Box::new(local_b.clone()),
                                                        rhs: Box::new(local_a.clone()),
                                                    }),
                                                    rhs: Box::new(local_s.clone()),
                                                }),
                                            }),
                                        }),
                                    }),
                                }),
                            },
                        ]),
                    }),
                }),
            };

            self.intrinsic_entries.push(Box::new(GraphExpressionEntry {
                id: entry_id,
                folder_id: Some(INTRINSICS_FOLDER_ID.into()),
                expression,
                ..Default::default()
            }));

            self.intrinsic_range_inclusive_created = true;
        }

        symbol
    }

    pub fn get_intrinsic_range_exclusive(&mut self) -> GraphExpression {
        let symbol = GraphExpression::Binary {
            kind: GraphBinaryKind::Subscript,
            lhs: Box::new(GraphExpression::Letter('I')),
            rhs: Box::new(GraphExpression::Alphanumeric("RangeExc".into())),
        };

        if !self.intrinsic_range_exclusive_created {
            let entry_id = self.create_entry_id();

            let local_a = self.create_local_symbol();
            let local_b = self.create_local_symbol();
            let local_s = self.create_local_symbol();
            let local_x = self.create_local_symbol();

            // range_exc(a, b, s) = x[{x = b, 0} = 0] with x = range_inc(a, b, s)
            // This is also terrible, but not nearly as bad.
            let expression = GraphExpression::Binary {
                kind: GraphBinaryKind::Equal,
                lhs: Box::new(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(symbol.clone()),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: Vec::from([
                            local_a.clone(),
                            local_b.clone(),
                            local_s.clone(),
                        ]),
                    }),
                }),
                rhs: Box::new(GraphExpression::Binary {
                    kind: GraphBinaryKind::With,
                    lhs: Box::new(GraphExpression::Binary {
                        kind: GraphBinaryKind::Index,
                        lhs: Box::new(local_x.clone()),
                        rhs: Box::new(GraphExpression::Binary {
                            kind: GraphBinaryKind::Equal,
                            lhs: Box::new(GraphExpression::Unary {
                                kind: GraphUnaryKind::Piecewise,
                                inner: Box::new(GraphExpression::Sequence {
                                    elements: Vec::from([
                                        GraphExpression::Binary {
                                            kind: GraphBinaryKind::Equal,
                                            lhs: Box::new(local_x.clone()),
                                            rhs: Box::new(local_b.clone()),
                                        },
                                        GraphExpression::Integer(0),
                                    ]),
                                }),
                            }),
                            rhs: Box::new(GraphExpression::Integer(0)),
                        }),
                    }),
                    rhs: Box::new(GraphExpression::Binary {
                        kind: GraphBinaryKind::Equal,
                        lhs: Box::new(local_x.clone()),
                        rhs: Box::new(GraphExpression::Binary {
                            kind: GraphBinaryKind::Call,
                            lhs: Box::new(self.get_intrinsic_range_inclusive()),
                            rhs: Box::new(GraphExpression::Sequence {
                                elements: Vec::from([
                                    local_a.clone(),
                                    local_b.clone(),
                                    local_s.clone(),
                                ]),
                            }),
                        }),
                    }),
                }),
            };

            self.intrinsic_entries.push(Box::new(GraphExpressionEntry {
                id: entry_id,
                folder_id: Some(INTRINSICS_FOLDER_ID.into()),
                expression,
                ..Default::default()
            }));

            self.intrinsic_range_exclusive_created = true;
        }

        symbol
    }

    pub fn get_intrinsic_index_range_inclusive(&mut self) -> GraphExpression {
        let symbol = GraphExpression::Binary {
            kind: GraphBinaryKind::Subscript,
            lhs: Box::new(GraphExpression::Letter('I')),
            rhs: Box::new(GraphExpression::Alphanumeric("IdxRangeInc".into())),
        };

        if !self.intrinsic_index_range_inclusive_created {
            let entry_id = self.create_entry_id();

            let local_l = self.create_local_symbol();
            let local_a = self.create_local_symbol();
            let local_b = self.create_local_symbol();
            let local_s = self.create_local_symbol();
            let local_i = self.create_local_symbol();

            // idx_range_inc(l, a, b, s) = [l[i] for i = range_inc(a, b, s)]
            let expression = GraphExpression::Binary {
                kind: GraphBinaryKind::Equal,
                lhs: Box::new(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(symbol.clone()),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: Vec::from([
                            local_l.clone(),
                            local_a.clone(),
                            local_b.clone(),
                            local_s.clone(),
                        ]),
                    }),
                }),
                rhs: Box::new(GraphExpression::Unary {
                    kind: GraphUnaryKind::List,
                    inner: Box::new(GraphExpression::Binary {
                        kind: GraphBinaryKind::For,
                        lhs: Box::new(GraphExpression::Binary {
                            kind: GraphBinaryKind::Index,
                            lhs: Box::new(local_l.clone()),
                            rhs: Box::new(local_i.clone()),
                        }),
                        rhs: Box::new(GraphExpression::Binary {
                            kind: GraphBinaryKind::Equal,
                            lhs: Box::new(local_i.clone()),
                            rhs: Box::new(GraphExpression::Binary {
                                kind: GraphBinaryKind::Call,
                                lhs: Box::new(self.get_intrinsic_range_inclusive()),
                                rhs: Box::new(GraphExpression::Sequence {
                                    elements: Vec::from([
                                        local_a.clone(),
                                        local_b.clone(),
                                        local_s.clone(),
                                    ]),
                                }),
                            }),
                        }),
                    }),
                }),
            };

            self.intrinsic_entries.push(Box::new(GraphExpressionEntry {
                id: entry_id,
                folder_id: Some(INTRINSICS_FOLDER_ID.into()),
                expression,
                ..Default::default()
            }));

            self.intrinsic_index_range_inclusive_created = true;
        }

        symbol
    }

    pub fn get_intrinsic_index_range_exclusive(&mut self) -> GraphExpression {
        let symbol = GraphExpression::Binary {
            kind: GraphBinaryKind::Subscript,
            lhs: Box::new(GraphExpression::Letter('I')),
            rhs: Box::new(GraphExpression::Alphanumeric("IdxRangeExc".into())),
        };

        if !self.intrinsic_index_range_exclusive_created {
            let entry_id = self.create_entry_id();

            let local_l = self.create_local_symbol();
            let local_a = self.create_local_symbol();
            let local_b = self.create_local_symbol();
            let local_s = self.create_local_symbol();
            let local_i = self.create_local_symbol();

            // idx_range_exc(l, a, b, s) = [l[i] for i = range_exc(a, b, s)]
            let expression = GraphExpression::Binary {
                kind: GraphBinaryKind::Equal,
                lhs: Box::new(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(symbol.clone()),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: Vec::from([
                            local_l.clone(),
                            local_a.clone(),
                            local_b.clone(),
                            local_s.clone(),
                        ]),
                    }),
                }),
                rhs: Box::new(GraphExpression::Unary {
                    kind: GraphUnaryKind::List,
                    inner: Box::new(GraphExpression::Binary {
                        kind: GraphBinaryKind::For,
                        lhs: Box::new(GraphExpression::Binary {
                            kind: GraphBinaryKind::Index,
                            lhs: Box::new(local_l.clone()),
                            rhs: Box::new(local_i.clone()),
                        }),
                        rhs: Box::new(GraphExpression::Binary {
                            kind: GraphBinaryKind::Equal,
                            lhs: Box::new(local_i.clone()),
                            rhs: Box::new(GraphExpression::Binary {
                                kind: GraphBinaryKind::Call,
                                lhs: Box::new(self.get_intrinsic_range_exclusive()),
                                rhs: Box::new(GraphExpression::Sequence {
                                    elements: Vec::from([
                                        local_a.clone(),
                                        local_b.clone(),
                                        local_s.clone(),
                                    ]),
                                }),
                            }),
                        }),
                    }),
                }),
            };

            self.intrinsic_entries.push(Box::new(GraphExpressionEntry {
                id: entry_id,
                folder_id: Some(INTRINSICS_FOLDER_ID.into()),
                expression,
                ..Default::default()
            }));

            self.intrinsic_index_range_exclusive_created = true;
        }

        symbol
    }

    pub fn get_intrinsic_index_range_from(&mut self) -> GraphExpression {
        let symbol = GraphExpression::Binary {
            kind: GraphBinaryKind::Subscript,
            lhs: Box::new(GraphExpression::Letter('I')),
            rhs: Box::new(GraphExpression::Alphanumeric("IdxRangeFrom".into())),
        };

        if !self.intrinsic_index_range_from_created {
            let entry_id = self.create_entry_id();

            let local_l = self.create_local_symbol();
            let local_a = self.create_local_symbol();
            let local_s = self.create_local_symbol();

            // idx_range_from(l, a, s) = idx_range_inc(l, a, count(l), s)
            let expression = GraphExpression::Binary {
                kind: GraphBinaryKind::Equal,
                lhs: Box::new(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(symbol.clone()),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: Vec::from([
                            local_l.clone(),
                            local_a.clone(),
                            local_s.clone(),
                        ]),
                    }),
                }),
                rhs: Box::new(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(self.get_intrinsic_index_range_inclusive()),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: Vec::from([
                            local_l.clone(),
                            local_a.clone(),
                            GraphExpression::Binary {
                                kind: GraphBinaryKind::Call,
                                lhs: Box::new(GraphExpression::OperatorName("count".into())),
                                rhs: Box::new(local_l.clone()),
                            },
                            local_s.clone(),
                        ]),
                    }),
                }),
            };

            self.intrinsic_entries.push(Box::new(GraphExpressionEntry {
                id: entry_id,
                folder_id: Some(INTRINSICS_FOLDER_ID.into()),
                expression,
                ..Default::default()
            }));

            self.intrinsic_index_range_from_created = true;
        }

        symbol
    }

    pub fn get_intrinsic_index_range_to_inclusive(&mut self) -> GraphExpression {
        let symbol = GraphExpression::Binary {
            kind: GraphBinaryKind::Subscript,
            lhs: Box::new(GraphExpression::Letter('I')),
            rhs: Box::new(GraphExpression::Alphanumeric("IdxRangeToInc".into())),
        };

        if !self.intrinsic_index_range_to_inclusive_created {
            let entry_id = self.create_entry_id();

            let local_l = self.create_local_symbol();
            let local_b = self.create_local_symbol();

            // idx_range_to_inc(l, b) = idx_range_inc(l, 1, b, 1)
            let expression = GraphExpression::Binary {
                kind: GraphBinaryKind::Equal,
                lhs: Box::new(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(symbol.clone()),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: Vec::from([
                            local_l.clone(),
                            local_b.clone(),
                        ]),
                    }),
                }),
                rhs: Box::new(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(self.get_intrinsic_index_range_inclusive()),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: Vec::from([
                            local_l.clone(),
                            GraphExpression::Integer(1),
                            local_b.clone(),
                            GraphExpression::Integer(1),
                        ]),
                    }),
                }),
            };

            self.intrinsic_entries.push(Box::new(GraphExpressionEntry {
                id: entry_id,
                folder_id: Some(INTRINSICS_FOLDER_ID.into()),
                expression,
                ..Default::default()
            }));

            self.intrinsic_index_range_to_inclusive_created = true;
        }

        symbol
    }

    pub fn get_intrinsic_index_range_to_exclusive(&mut self) -> GraphExpression {
        let symbol = GraphExpression::Binary {
            kind: GraphBinaryKind::Subscript,
            lhs: Box::new(GraphExpression::Letter('I')),
            rhs: Box::new(GraphExpression::Alphanumeric("IdxRangeToExc".into())),
        };

        if !self.intrinsic_index_range_to_exclusive_created {
            let entry_id = self.create_entry_id();

            let local_l = self.create_local_symbol();
            let local_b = self.create_local_symbol();

            // idx_range_to_exc(l, b) = idx_range_exc(l, 1, b, 1)
            let expression = GraphExpression::Binary {
                kind: GraphBinaryKind::Equal,
                lhs: Box::new(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(symbol.clone()),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: Vec::from([
                            local_l.clone(),
                            local_b.clone(),
                        ]),
                    }),
                }),
                rhs: Box::new(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(self.get_intrinsic_index_range_exclusive()),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: Vec::from([
                            local_l.clone(),
                            GraphExpression::Integer(1),
                            local_b.clone(),
                            GraphExpression::Integer(1),
                        ]),
                    }),
                }),
            };

            self.intrinsic_entries.push(Box::new(GraphExpressionEntry {
                id: entry_id,
                folder_id: Some(INTRINSICS_FOLDER_ID.into()),
                expression,
                ..Default::default()
            }));

            self.intrinsic_index_range_to_exclusive_created = true;
        }

        symbol
    }

    pub fn translate_value(&mut self, value: &Value) -> crate::Result<GraphExpression> {
        let unsupported_error = || Box::new(crate::Error {
            kind: crate::ErrorKind::UnsupportedValue,
            span: value.span,
        });

        match &value.kind {
            ValueKind::Undefined(value_type) => {
                let undefined_scalar = GraphExpression::Binary {
                    kind: GraphBinaryKind::Fraction,
                    lhs: Box::new(GraphExpression::Integer(0)),
                    rhs: Box::new(GraphExpression::Integer(0)),
                };
                match value_type {
                    Type::Point2 { .. } => Ok(GraphExpression::Unary {
                        kind: GraphUnaryKind::Parentheses,
                        inner: Box::new(GraphExpression::Sequence {
                            elements: Vec::from([
                                undefined_scalar.clone(),
                                undefined_scalar,
                            ]),
                        }),
                    }),
                    Type::Point3 { .. } => Ok(GraphExpression::Unary {
                        kind: GraphUnaryKind::Parentheses,
                        inner: Box::new(GraphExpression::Sequence {
                            elements: Vec::from([
                                undefined_scalar.clone(),
                                undefined_scalar.clone(),
                                undefined_scalar,
                            ]),
                        }),
                    }),
                    _ => Ok(undefined_scalar)
                }
            }
            ValueKind::Real(value) => {
                Ok(GraphExpression::Decimal(*value))
            }
            ValueKind::Mathematical { kind, coefficient } => {
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::ImplicitMultiply,
                    lhs: Box::new(GraphExpression::Decimal(*coefficient)),
                    rhs: Box::new(match kind {
                        MathematicalConstant::Pi => GraphExpression::Escape("pi".into()),
                        MathematicalConstant::Tau => GraphExpression::Escape("tau".into()),
                        MathematicalConstant::E => GraphExpression::Letter('e'),
                    }),
                })
            }
            ValueKind::Int(value) => {
                Ok(GraphExpression::Integer(*value))
            }
            ValueKind::Bool(value) => {
                Ok(GraphExpression::Integer(*value as i64))
            }
            ValueKind::EnumVariant { variant_ordinal, .. } => {
                Ok(GraphExpression::Integer(*variant_ordinal))
            }
            ValueKind::Intrinsic(value) => {
                self.translate_intrinsic_value(value)
            }
            ValueKind::Global(reference) => {
                Ok(self.get_global_symbol(&reference.identifier))
            }
            ValueKind::Local(reference) => {
                Ok(self.get_local_symbol(reference.id))
            }
            ValueKind::AssumeType(value, _) => {
                self.translate_value(value)
            }
            ValueKind::Unary { operation, operand, .. } => {
                self.translate_unary(*operation, operand)
            }
            ValueKind::Binary { operation, lhs, rhs, .. } => {
                self.translate_binary(*operation, lhs, rhs, unsupported_error)
            }
            ValueKind::Point2 { x, y, .. } => {
                Ok(GraphExpression::Unary {
                    kind: GraphUnaryKind::Parentheses,
                    inner: Box::new(GraphExpression::Sequence {
                        elements: Vec::from([
                            self.translate_value(x)?,
                            self.translate_value(y)?,
                        ]),
                    }),
                })
            }
            ValueKind::Point3 { x, y, z, .. } => {
                Ok(GraphExpression::Unary {
                    kind: GraphUnaryKind::Parentheses,
                    inner: Box::new(GraphExpression::Sequence {
                        elements: Vec::from([
                            self.translate_value(x)?,
                            self.translate_value(y)?,
                            self.translate_value(z)?,
                        ]),
                    }),
                })
            }
            ValueKind::GetX { point, .. } => {
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Dot,
                    lhs: Box::new(self.translate_value(point)?),
                    rhs: Box::new(GraphExpression::Letter('x')),
                })
            }
            ValueKind::GetY { point, .. } => {
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Dot,
                    lhs: Box::new(self.translate_value(point)?),
                    rhs: Box::new(GraphExpression::Letter('y')),
                })
            }
            ValueKind::GetZ { point, .. } => {
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Dot,
                    lhs: Box::new(self.translate_value(point)?),
                    rhs: Box::new(GraphExpression::Letter('z')),
                })
            }
            ValueKind::List { items, .. } => {
                Ok(GraphExpression::Unary {
                    kind: GraphUnaryKind::List,
                    inner: Box::new(GraphExpression::Sequence {
                        elements: items
                            .iter()
                            .map(|item| self.translate_value(item))
                            .collect::<crate::Result<_>>()?,
                    }),
                })
            }
            ValueKind::ListRange { kind, start, end, step, .. } => {
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(match kind {
                        RangeKind::Inclusive => self.get_intrinsic_range_inclusive(),
                        RangeKind::Exclusive => self.get_intrinsic_range_exclusive(),
                    }),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: Vec::from([
                            self.translate_value(start)?,
                            self.translate_value(end)?,
                            self.translate_value(step)?,
                        ]),
                    }),
                })
            }
            ValueKind::ListFill { value, count } => {
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(GraphExpression::OperatorName("repeat".into())),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: Vec::from([
                            self.translate_value(value)?,
                            self.translate_value(count)?,
                        ]),
                    }),
                })
            }
            ValueKind::ListMap { loops, value } => {
                Ok(GraphExpression::Unary {
                    kind: GraphUnaryKind::List,
                    inner: Box::new(GraphExpression::Binary {
                        kind: GraphBinaryKind::For,
                        lhs: Box::new(self.translate_value(value)?),
                        rhs: Box::new(GraphExpression::Sequence {
                            elements: loops
                                .iter()
                                .rev()
                                .map(|map_loop| Ok(GraphExpression::Binary {
                                    kind: GraphBinaryKind::Equal,
                                    lhs: Box::new(self.get_local_symbol(map_loop.local.id)),
                                    rhs: Box::new(self.translate_value(&map_loop.list)?),
                                }))
                                .collect::<crate::Result<_>>()?,
                        }),
                    }),
                })
            }
            ValueKind::ListFilter { list, condition, .. } => {
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Index,
                    lhs: Box::new(self.translate_value(list)?),
                    rhs: Box::new(self.translate_value(condition)?),
                })
            }
            ValueKind::Index { list, operation, .. } => match operation {
                ValueIndexOperation::Single { index } => {
                    Ok(GraphExpression::Binary {
                        kind: GraphBinaryKind::Index,
                        lhs: Box::new(self.translate_value(list)?),
                        rhs: Box::new(self.translate_value(index)?),
                    })
                }
                ValueIndexOperation::Range { kind, from_index, to_index, step} => {
                    Ok(GraphExpression::Binary {
                        kind: GraphBinaryKind::Call,
                        lhs: Box::new(match kind {
                            RangeKind::Inclusive => self.get_intrinsic_index_range_inclusive(),
                            RangeKind::Exclusive => self.get_intrinsic_index_range_exclusive(),
                        }),
                        rhs: Box::new(GraphExpression::Sequence {
                            elements: Vec::from([
                                self.translate_value(list)?,
                                self.translate_value(from_index)?,
                                self.translate_value(to_index)?,
                                self.translate_value(step)?,
                            ]),
                        }),
                    })
                }
                ValueIndexOperation::RangeFrom { from_index, step } => {
                    Ok(GraphExpression::Binary {
                        kind: GraphBinaryKind::Call,
                        lhs: Box::new(self.get_intrinsic_index_range_from()),
                        rhs: Box::new(GraphExpression::Sequence {
                            elements: Vec::from([
                                self.translate_value(list)?,
                                self.translate_value(from_index)?,
                                self.translate_value(step)?,
                            ]),
                        }),
                    })
                }
                ValueIndexOperation::RangeTo { kind, to_index } => {
                    Ok(GraphExpression::Binary {
                        kind: GraphBinaryKind::Call,
                        lhs: Box::new(match kind {
                            RangeKind::Inclusive => self.get_intrinsic_index_range_to_inclusive(),
                            RangeKind::Exclusive => self.get_intrinsic_index_range_to_exclusive(),
                        }),
                        rhs: Box::new(GraphExpression::Sequence {
                            elements: Vec::from([
                                self.translate_value(list)?,
                                self.translate_value(to_index)?,
                            ]),
                        }),
                    })
                }
            }
            ValueKind::Conditional { condition_consequents, alternative, .. } => {
                Ok(GraphExpression::Unary {
                    kind: GraphUnaryKind::Piecewise,
                    inner: Box::new(GraphExpression::Sequence {
                        elements: {
                            let mut elements: Vec<_> = condition_consequents
                                .iter()
                                .map(|(condition, consequent)| Ok(GraphExpression::Binary {
                                    kind: GraphBinaryKind::Colon,
                                    lhs: Box::new(self.translate_condition(condition)?),
                                    rhs: Box::new(self.translate_value(consequent)?),
                                }))
                                .collect::<crate::Result<_>>()?;
                            if !alternative.is_undefined() {
                                elements.push(self.translate_value(alternative)?);
                            }
                            elements
                        },
                    }),
                })
            }
            ValueKind::UserFunctionCall { function, arguments, .. } => {
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(self.translate_value(function)?),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: arguments
                            .iter()
                            .map(|argument| self.translate_value(argument))
                            .collect::<crate::Result<_>>()?,
                    }),
                })
            }
            ValueKind::Let { local, value, inner, .. } => {
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::With,
                    lhs: Box::new(self.translate_value(inner)?),
                    rhs: Box::new(GraphExpression::Binary {
                        kind: GraphBinaryKind::Equal,
                        lhs: Box::new(self.get_local_symbol(local.id)),
                        rhs: Box::new(self.translate_value(value)?),
                    }),
                })
            }
            _ => {
                Err(unsupported_error())
            }
        }
    }

    pub fn translate_unary(&mut self, operation: UnaryOperation, operand: &Value) -> crate::Result<GraphExpression> {
        match operation {
            UnaryOperation::Positive => {
                Ok(GraphExpression::Unary {
                    kind: GraphUnaryKind::Parentheses,
                    inner: Box::new(GraphExpression::Unary {
                        kind: GraphUnaryKind::Positive,
                        inner: Box::new(self.translate_value(operand)?),
                    }),
                })
            }
            UnaryOperation::Negative => {
                Ok(GraphExpression::Unary {
                    kind: GraphUnaryKind::Parentheses,
                    inner: Box::new(GraphExpression::Unary {
                        kind: GraphUnaryKind::Negative,
                        inner: Box::new(self.translate_value(operand)?),
                    }),
                })
            }
            UnaryOperation::LogicalNot => {
                Ok(GraphExpression::Unary {
                    kind: GraphUnaryKind::Piecewise,
                    inner: Box::new(GraphExpression::Sequence {
                        elements: vec![
                            GraphExpression::Binary {
                                kind: GraphBinaryKind::Equal,
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

    pub fn translate_binary(
        &mut self,
        operation: BinaryOperation,
        lhs: &Value,
        rhs: &Value,
        unsupported_error: impl Fn() -> Box<crate::Error>,
    ) -> crate::Result<GraphExpression> {
        match operation {
            BinaryOperation::Exponent => {
                Ok(GraphExpression::Unary {
                    kind: GraphUnaryKind::Parentheses,
                    inner: Box::new(GraphExpression::Binary {
                        kind: GraphBinaryKind::Superscript,
                        lhs: Box::new(self.translate_value(lhs)?),
                        rhs: Box::new(self.translate_value(rhs)?),
                    }),
                })
            }
            BinaryOperation::Multiply => {
                Ok(GraphExpression::Unary {
                    kind: GraphUnaryKind::Parentheses,
                    inner: Box::new(GraphExpression::Binary {
                        kind: GraphBinaryKind::Multiply,
                        lhs: Box::new(self.translate_value(lhs)?),
                        rhs: Box::new(self.translate_value(rhs)?),
                    }),
                })
            }
            BinaryOperation::Divide => {
                Ok(GraphExpression::Unary {
                    kind: GraphUnaryKind::Parentheses,
                    inner: Box::new(GraphExpression::Binary {
                        kind: GraphBinaryKind::Divide,
                        lhs: Box::new(self.translate_value(lhs)?),
                        rhs: Box::new(self.translate_value(rhs)?),
                    }),
                })
            }
            BinaryOperation::Remainder => {
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(GraphExpression::OperatorName("mod".into())),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: Vec::from([
                            self.translate_value(lhs)?,
                            self.translate_value(rhs)?,
                        ]),
                    }),
                })
            }
            BinaryOperation::Add => {
                Ok(GraphExpression::Unary {
                    kind: GraphUnaryKind::Parentheses,
                    inner: Box::new(GraphExpression::Binary {
                        kind: GraphBinaryKind::Add,
                        lhs: Box::new(self.translate_value(lhs)?),
                        rhs: Box::new(self.translate_value(rhs)?),
                    }),
                })
            }
            BinaryOperation::Subtract => {
                Ok(GraphExpression::Unary {
                    kind: GraphUnaryKind::Parentheses,
                    inner: Box::new(GraphExpression::Binary {
                        kind: GraphBinaryKind::Subtract,
                        lhs: Box::new(self.translate_value(lhs)?),
                        rhs: Box::new(self.translate_value(rhs)?),
                    }),
                })
            }
            BinaryOperation::LessThan => {
                Ok(GraphExpression::Unary {
                    kind: GraphUnaryKind::Piecewise,
                    inner: Box::new(GraphExpression::Sequence {
                        elements: Vec::from([
                            GraphExpression::InequalityChain {
                                lhs: Box::new(self.translate_value(lhs)?),
                                first_kind: GraphInequalityKind::LessThan,
                                rhs: Box::new(self.translate_value(rhs)?),
                                chain: Vec::new(),
                            },
                            GraphExpression::Integer(0),
                        ]),
                    }),
                })
            }
            BinaryOperation::LessEqual => {
                Ok(GraphExpression::Unary {
                    kind: GraphUnaryKind::Piecewise,
                    inner: Box::new(GraphExpression::Sequence {
                        elements: Vec::from([
                            GraphExpression::InequalityChain {
                                lhs: Box::new(self.translate_value(lhs)?),
                                first_kind: GraphInequalityKind::LessEqual,
                                rhs: Box::new(self.translate_value(rhs)?),
                                chain: Vec::new(),
                            },
                            GraphExpression::Integer(0),
                        ]),
                    }),
                })
            }
            BinaryOperation::GreaterThan => {
                Ok(GraphExpression::Unary {
                    kind: GraphUnaryKind::Piecewise,
                    inner: Box::new(GraphExpression::Sequence {
                        elements: Vec::from([
                            GraphExpression::InequalityChain {
                                lhs: Box::new(self.translate_value(lhs)?),
                                first_kind: GraphInequalityKind::GreaterThan,
                                rhs: Box::new(self.translate_value(rhs)?),
                                chain: Vec::new(),
                            },
                            GraphExpression::Integer(0),
                        ]),
                    }),
                })
            }
            BinaryOperation::GreaterEqual => {
                Ok(GraphExpression::Unary {
                    kind: GraphUnaryKind::Piecewise,
                    inner: Box::new(GraphExpression::Sequence {
                        elements: Vec::from([
                            GraphExpression::InequalityChain {
                                lhs: Box::new(self.translate_value(lhs)?),
                                first_kind: GraphInequalityKind::GreaterEqual,
                                rhs: Box::new(self.translate_value(rhs)?),
                                chain: Vec::new(),
                            },
                            GraphExpression::Integer(0),
                        ]),
                    }),
                })
            }
            BinaryOperation::Equal => {
                Ok(GraphExpression::Unary {
                    kind: GraphUnaryKind::Piecewise,
                    inner: Box::new(GraphExpression::Sequence {
                        elements: Vec::from([
                            GraphExpression::Binary {
                                kind: GraphBinaryKind::Equal,
                                lhs: Box::new(self.translate_value(lhs)?),
                                rhs: Box::new(self.translate_value(rhs)?),
                            },
                            GraphExpression::Integer(0),
                        ]),
                    }),
                })
            }
            BinaryOperation::NotEqual => {
                Ok(GraphExpression::Unary {
                    kind: GraphUnaryKind::Piecewise,
                    inner: Box::new(GraphExpression::Sequence {
                        elements: Vec::from([
                            GraphExpression::Binary {
                                kind: GraphBinaryKind::Colon,
                                lhs: Box::new(GraphExpression::Binary {
                                    kind: GraphBinaryKind::Equal,
                                    lhs: Box::new(self.translate_value(lhs)?),
                                    rhs: Box::new(self.translate_value(rhs)?),
                                }),
                                rhs: Box::new(GraphExpression::Integer(0)),
                            },
                            GraphExpression::Integer(1),
                        ]),
                    }),
                })
            }
            BinaryOperation::LogicalAnd => {
                Ok(GraphExpression::Unary {
                    kind: GraphUnaryKind::Piecewise,
                    inner: Box::new(GraphExpression::Sequence {
                        elements: Vec::from([
                            GraphExpression::Binary {
                                kind: GraphBinaryKind::Colon,
                                lhs: Box::new(self.translate_condition(lhs)?),
                                rhs: Box::new(self.translate_value(rhs)?),
                            },
                            GraphExpression::Integer(0),
                        ]),
                    }),
                })
            }
            BinaryOperation::LogicalOr => {
                Ok(GraphExpression::Unary {
                    kind: GraphUnaryKind::Piecewise,
                    inner: Box::new(GraphExpression::Sequence {
                        elements: Vec::from([
                            self.translate_condition(lhs)?,
                            self.translate_value(rhs)?,
                        ]),
                    }),
                })
            }
            _ => {
                Err(unsupported_error())
            }
        }
    }

    pub fn translate_condition(&mut self, value: &Value) -> crate::Result<GraphExpression> {
        match &value.kind {
            ValueKind::Binary { operation: BinaryOperation::Equal, lhs, rhs, .. } => {
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Equal,
                    lhs: Box::new(self.translate_value(lhs)?),
                    rhs: Box::new(self.translate_value(rhs)?),
                })
            }
            ValueKind::Binary { operation: BinaryOperation::NotEqual, lhs, rhs, .. } => {
                // If only Desmos had an operator for this... substitute with {lhs = rhs, 0} = 0
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Equal,
                    lhs: Box::new(GraphExpression::Unary {
                        kind: GraphUnaryKind::Piecewise,
                        inner: Box::new(GraphExpression::Sequence {
                            elements: vec![
                                GraphExpression::Binary {
                                    kind: GraphBinaryKind::Equal,
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
            ValueKind::Binary { operation: BinaryOperation::LessThan, lhs, rhs, .. } => {
                Ok(GraphExpression::InequalityChain {
                    first_kind: GraphInequalityKind::LessThan,
                    lhs: Box::new(self.translate_value(lhs)?),
                    rhs: Box::new(self.translate_value(rhs)?),
                    chain: Vec::new(),
                })
            }
            ValueKind::Binary { operation: BinaryOperation::GreaterThan, lhs, rhs, .. } => {
                Ok(GraphExpression::InequalityChain {
                    first_kind: GraphInequalityKind::GreaterThan,
                    lhs: Box::new(self.translate_value(lhs)?),
                    rhs: Box::new(self.translate_value(rhs)?),
                    chain: Vec::new(),
                })
            }
            ValueKind::Binary { operation: BinaryOperation::LessEqual, lhs, rhs, .. } => {
                Ok(GraphExpression::InequalityChain {
                    first_kind: GraphInequalityKind::LessEqual,
                    lhs: Box::new(self.translate_value(lhs)?),
                    rhs: Box::new(self.translate_value(rhs)?),
                    chain: Vec::new(),
                })
            }
            ValueKind::Binary { operation: BinaryOperation::GreaterEqual, lhs, rhs, .. } => {
                Ok(GraphExpression::InequalityChain {
                    first_kind: GraphInequalityKind::GreaterEqual,
                    lhs: Box::new(self.translate_value(lhs)?),
                    rhs: Box::new(self.translate_value(rhs)?),
                    chain: Vec::new(),
                })
            }
            _ => {
                // Do a general "!= 0" check to evaluate a boolean: {value = 0, 0} = 0
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Equal,
                    lhs: Box::new(GraphExpression::Unary {
                        kind: GraphUnaryKind::Piecewise,
                        inner: Box::new(GraphExpression::Sequence {
                            elements: Vec::from([
                                GraphExpression::Binary {
                                    kind: GraphBinaryKind::Equal,
                                    lhs: Box::new(self.translate_value(value)?),
                                    rhs: Box::new(GraphExpression::Integer(0)),
                                },
                                GraphExpression::Integer(0),
                            ]),
                        }),
                    }),
                    rhs: Box::new(GraphExpression::Integer(0)),
                })
            }
        }
    }

    pub fn translate_intrinsic_value(&mut self, value: &IntrinsicValue) -> crate::Result<GraphExpression> {
        fn unary_function(name: &str, argument: GraphExpression) -> GraphExpression {
            GraphExpression::Binary {
                kind: GraphBinaryKind::Call,
                lhs: Box::new(GraphExpression::OperatorName(name.into())),
                rhs: Box::new(argument),
            }
        }
        fn binary_infix(kind: GraphBinaryKind, lhs: GraphExpression, rhs: GraphExpression) -> GraphExpression {
            GraphExpression::Unary {
                kind: GraphUnaryKind::Parentheses,
                inner: Box::new(GraphExpression::Binary {
                    kind,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }),
            }
        }
        fn get_reducer_name(kind: &IntrinsicReducerKind) -> &'static str {
            match kind {
                IntrinsicReducerKind::Mean => "mean",
                IntrinsicReducerKind::Median => "median",
                IntrinsicReducerKind::Min => "min",
                IntrinsicReducerKind::Max => "max",
                IntrinsicReducerKind::Stdev => "stdev",
                IntrinsicReducerKind::Stdevp => "stdevp",
                IntrinsicReducerKind::Var => "var",
                IntrinsicReducerKind::Varp => "varp",
                IntrinsicReducerKind::Mad => "mad",
                IntrinsicReducerKind::Count => "count",
                IntrinsicReducerKind::Total => "total",
                IntrinsicReducerKind::Polygon => "polygon",
                IntrinsicReducerKind::Lcm => "lcm",
                IntrinsicReducerKind::Gcd => "gcd",
            }
        }
        fn get_double_reducer_name(kind: &IntrinsicDoubleReducerKind) -> &'static str {
            match kind {
                IntrinsicDoubleReducerKind::Cov => "cov",
                IntrinsicDoubleReducerKind::Covp => "covp",
                IntrinsicDoubleReducerKind::Corr => "corr",
                IntrinsicDoubleReducerKind::Spearman => "spearman",
            }
        }
        fn get_parameterized_reducer_name(kind: &IntrinsicParameterizedReducerKind) -> &'static str {
            match kind {
                IntrinsicParameterizedReducerKind::Quartile => "quartile",
                IntrinsicParameterizedReducerKind::Quantile => "quantile",
                IntrinsicParameterizedReducerKind::Tscore => "tscore",
            }
        }
        fn get_color_name(kind: &IntrinsicColorKind) -> &'static str {
            match kind {
                IntrinsicColorKind::Rgb => "rgb",
                IntrinsicColorKind::Hsv => "hsv",
                IntrinsicColorKind::Okhsv => "okhsv",
                IntrinsicColorKind::Oklab => "oklab",
                IntrinsicColorKind::Oklch => "oklch",
            }
        }

        match value {
            IntrinsicValue::Unary { kind, argument, .. } => {
                let argument = self.translate_value(argument)?;

                Ok(match kind {
                    IntrinsicUnaryKind::Sin => unary_function("sin", argument),
                    IntrinsicUnaryKind::Cos => unary_function("cos", argument),
                    IntrinsicUnaryKind::Tan => unary_function("tan", argument),
                    IntrinsicUnaryKind::Csc => unary_function("csc", argument),
                    IntrinsicUnaryKind::Sec => unary_function("sec", argument),
                    IntrinsicUnaryKind::Cot => unary_function("cot", argument),
                    IntrinsicUnaryKind::Arcsin => unary_function("arcsin", argument),
                    IntrinsicUnaryKind::Arccos => unary_function("arccos", argument),
                    IntrinsicUnaryKind::Arctan => unary_function("arctan", argument),
                    IntrinsicUnaryKind::Arccsc => unary_function("arccsc", argument),
                    IntrinsicUnaryKind::Arcsec => unary_function("arcsec", argument),
                    IntrinsicUnaryKind::Arccot => unary_function("arccot", argument),
                    IntrinsicUnaryKind::Sinh => unary_function("sinh", argument),
                    IntrinsicUnaryKind::Cosh => unary_function("cosh", argument),
                    IntrinsicUnaryKind::Tanh => unary_function("tanh", argument),
                    IntrinsicUnaryKind::Csch => unary_function("csch", argument),
                    IntrinsicUnaryKind::Sech => unary_function("sech", argument),
                    IntrinsicUnaryKind::Coth => unary_function("coth", argument),
                })
            }
            IntrinsicValue::Binary { kind, lhs, rhs, .. } => {
                let lhs = self.translate_value(lhs)?;
                let rhs = self.translate_value(rhs)?;

                Ok(match kind {
                    IntrinsicBinaryKind::Dot => binary_infix(GraphBinaryKind::DotMultiply, lhs, rhs),
                    IntrinsicBinaryKind::Cross => binary_infix(GraphBinaryKind::CrossMultiply, lhs, rhs),
                })
            }
            IntrinsicValue::Reducer { kind, list, .. } => {
                let name = get_reducer_name(kind);

                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(GraphExpression::OperatorName(name.into())),
                    rhs: Box::new(self.translate_value(list)?),
                })
            }
            IntrinsicValue::ArgumentsReducer { kind, arguments, .. } => {
                let name = get_reducer_name(kind);

                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(GraphExpression::OperatorName(name.into())),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: arguments
                            .iter()
                            .map(|argument| self.translate_value(argument))
                            .collect::<crate::Result<_>>()?,
                    }),
                })
            }
            IntrinsicValue::DoubleReducer { kind, list_1, list_2, .. } => {
                let name = get_double_reducer_name(kind);

                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(GraphExpression::OperatorName(name.into())),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: Vec::from([
                            self.translate_value(list_1)?,
                            self.translate_value(list_2)?,
                        ]),
                    }),
                })
            }
            IntrinsicValue::ParameterizedReducer { kind, list, parameter, .. } => {
                let name = get_parameterized_reducer_name(kind);

                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(GraphExpression::OperatorName(name.into())),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: Vec::from([
                            self.translate_value(list)?,
                            self.translate_value(parameter)?,
                        ]),
                    }),
                })
            }
            IntrinsicValue::Color { kind, value_1, value_2, value_3, .. } => {
                let name = get_color_name(kind);

                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(GraphExpression::OperatorName(name.into())),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: Vec::from([
                            self.translate_value(value_1)?,
                            self.translate_value(value_2)?,
                            self.translate_value(value_3)?,
                        ]),
                    }),
                })
            }
            IntrinsicValue::Segment { point_1, point_2, .. } => {
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(GraphExpression::OperatorName("segment".into())),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: Vec::from([
                            self.translate_value(point_1)?,
                            self.translate_value(point_2)?,
                        ]),
                    }),
                })
            }
            IntrinsicValue::Rotate { object, point, angle, .. } => {
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(GraphExpression::OperatorName("rotate".into())),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: Vec::from([
                            self.translate_value(object)?,
                            self.translate_value(point)?,
                            self.translate_value(angle)?,
                        ]),
                    }),
                })
            }
            IntrinsicValue::Join { arguments, .. } => {
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(GraphExpression::OperatorName("join".into())),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: arguments
                            .iter()
                            .map(|argument| self.translate_value(argument))
                            .collect::<crate::Result<_>>()?,
                    }),
                })
            }
            IntrinsicValue::Width => {
                Ok(GraphExpression::OperatorName("width".into()))
            }
            IntrinsicValue::Height => {
                Ok(GraphExpression::OperatorName("height".into()))
            }
            IntrinsicValue::Dt => {
                Ok(GraphExpression::OperatorName("dt".into()))
            }
            IntrinsicValue::Index => {
                Ok(GraphExpression::OperatorName("index".into()))
            }
        }
    }

    pub fn translate_action_value(&mut self, action: &ActionValue) -> crate::Result<GraphExpression> {
        if action.is_empty() {
            // Usually omitting the action expression is not an option, so update a dummy
            // variable instead.
            return Ok(GraphExpression::Binary {
                kind: GraphBinaryKind::RightArrow,
                lhs: Box::new(self.create_dummy_noop()),
                rhs: Box::new(GraphExpression::Integer(0)),
            });
        }

        match &action.kind {
            ActionValueKind::Disable => {
                // Generate an expression like {0 = 1: unreachable -> 0} since the missing
                // conditional default case is what causes the "disable" behavior.
                Ok(GraphExpression::Unary {
                    kind: GraphUnaryKind::Piecewise,
                    inner: Box::new(GraphExpression::Binary {
                        kind: GraphBinaryKind::Colon,
                        lhs: Box::new(GraphExpression::Binary {
                            kind: GraphBinaryKind::Equal,
                            lhs: Box::new(GraphExpression::Integer(0)),
                            rhs: Box::new(GraphExpression::Integer(1)),
                        }),
                        rhs: Box::new(GraphExpression::Binary {
                            kind: GraphBinaryKind::RightArrow,
                            lhs: Box::new(self.get_dummy_unreachable()),
                            rhs: Box::new(GraphExpression::Integer(0)),
                        }),
                    }),
                })
            }
            ActionValueKind::Compound { actions } => match actions.as_ref() {
                [] => {
                    // This case was already handled above with the is_empty() check
                    unreachable!()
                }
                [action] => {
                    self.translate_action_value(action)
                }
                _ => {
                    Ok(GraphExpression::Unary {
                        kind: GraphUnaryKind::Parentheses,
                        inner: Box::new(GraphExpression::Sequence {
                            elements: actions
                                .iter()
                                .map(|action| self.translate_action_value(action))
                                .collect::<crate::Result<_>>()?,
                        }),
                    })
                }
            }
            ActionValueKind::Update { variable, value, .. } => {
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::RightArrow,
                    lhs: Box::new(self.get_global_symbol(&variable.identifier)),
                    rhs: Box::new(self.translate_value(value)?),
                })
            }
            ActionValueKind::ActionCall { identifier, arguments, .. } => {
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(self.get_action_symbol(identifier)),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: arguments
                            .iter()
                            .map(|argument| self.translate_value(argument))
                            .collect::<crate::Result<_>>()?,
                    }),
                })
            }
            ActionValueKind::Conditional { condition_consequents, alternative } => {
                Ok(GraphExpression::Unary {
                    kind: GraphUnaryKind::Piecewise,
                    inner: Box::new(GraphExpression::Sequence {
                        elements: {
                            let mut elements: Vec<_> = condition_consequents
                                .iter()
                                .map(|(condition, consequent)| Ok(GraphExpression::Binary {
                                    kind: GraphBinaryKind::Colon,
                                    lhs: Box::new(self.translate_condition(condition)?),
                                    rhs: Box::new(self.translate_action_value(consequent)?),
                                }))
                                .collect::<crate::Result<_>>()?;
                            // TODO: propagate disable
                            elements.push(self.translate_action_value(alternative)?);
                            elements
                        },
                    }),
                })
            }
        }
    }

    pub fn add_program_let(&mut self, program_let: &ProgramLet) -> crate::Result<()> {
        let id = self.create_entry_id();
        let expression = GraphExpression::Binary {
            kind: GraphBinaryKind::Equal,
            lhs: Box::new(match program_let.parameters() {
                Some(parameters) => GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(self.get_global_symbol(&program_let.identifier())),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: parameters
                            .iter()
                            .map(|parameter| self.get_local_symbol(parameter.id))
                            .collect(),
                    }),
                },
                None => self.get_global_symbol(&program_let.identifier()),
            }),
            rhs: Box::new(self.translate_value(program_let.value())?),
        };

        self.global_entries.push(Box::new(GraphExpressionEntry {
            id,
            folder_id: Some(GLOBALS_FOLDER_ID.into()),
            expression,
            ..Default::default()
        }));

        Ok(())
    }

    pub fn add_program_variable(&mut self, program_variable: &ProgramVariable) -> crate::Result<()> {
        let id = self.create_entry_id();
        let expression = GraphExpression::Binary {
            kind: GraphBinaryKind::Equal,
            lhs: Box::new(self.get_global_symbol(&program_variable.identifier())),
            rhs: Box::new(self.translate_value(program_variable.value())?),
        };

        self.global_entries.push(Box::new(GraphExpressionEntry {
            id,
            folder_id: Some(GLOBALS_FOLDER_ID.into()),
            expression,
            ..Default::default()
        }));

        Ok(())
    }

    pub fn add_program_action(&mut self, program_action: &ProgramAction) -> crate::Result<()> {
        let id = self.create_entry_id();
        let expression = GraphExpression::Binary {
            kind: GraphBinaryKind::Equal,
            lhs: Box::new(GraphExpression::Binary {
                kind: GraphBinaryKind::Call,
                lhs: Box::new(self.get_action_symbol(&program_action.identifier())),
                rhs: Box::new(GraphExpression::Sequence {
                    elements: program_action
                        .parameters()
                        .iter()
                        .map(|parameter| self.get_local_symbol(parameter.id))
                        .collect(),
                }),
            }),
            rhs: Box::new(self.translate_action_value(program_action.action())?),
        };

        self.action_entries.push(Box::new(GraphExpressionEntry {
            id,
            folder_id: Some(ACTIONS_FOLDER_ID.into()),
            expression,
            ..Default::default()
        }));

        Ok(())
    }

    pub fn set_program_ticker(&mut self, program_ticker: Option<&ProgramTicker>) -> crate::Result<()> {
        self.ticker = match program_ticker {
            Some(program_ticker) => Some(GraphTicker {
                playing: false,
                handler: self.translate_action_value(program_ticker.tick_action())?,
                min_step: match program_ticker.interval_ms() {
                    Some(interval_ms) => self.translate_value(interval_ms)?,
                    None => GraphExpression::Empty,
                },
            }),
            None => None,
        };

        Ok(())
    }

    pub fn add_public_line(&mut self, public_line: &ProgramPublicLine) -> crate::Result<()> {
        let id = self.create_entry_id();
        let entry: Box<dyn GraphEntry> = match public_line {
            ProgramPublicLine::Text(text) => {
                let text = text.trim();
                if text.is_empty() {
                    Box::new(GraphExpressionEntry {
                        id,
                        expression: GraphExpression::Empty,
                        ..Default::default()
                    })
                }
                else {
                    Box::new(GraphTextEntry {
                        id,
                        folder_id: None,
                        text: text.to_string(),
                    })
                }
            }
            ProgramPublicLine::Expression(value) => {
                Box::new(GraphExpressionEntry {
                    id,
                    expression: self.translate_value(value)?,
                    ..Default::default()
                })
            }
            ProgramPublicLine::Action(action) => {
                Box::new(GraphExpressionEntry {
                    id,
                    expression: self.translate_action_value(action)?,
                    ..Default::default()
                })
            }
        };
        self.public_entries.push(entry);

        Ok(())
    }

    pub fn add_display_element(&mut self, element: &ProgramDisplayElement) -> crate::Result<()> {
        let mut entry = GraphExpressionEntry {
            id: self.create_entry_id(),
            folder_id: Some(DISPLAY_FOLDER_ID.into()),
            expression: self.translate_value(element.expression())?,
            display: true,
            ..Default::default()
        };

        let unsupported_error = |value: &Value| Box::new(crate::Error {
            kind: crate::ErrorKind::UnsupportedValue,
            span: value.span,
        });

        fn prevent_duplicate(attribute: &ProgramDisplayAttribute, has_attribute: &mut bool) -> crate::Result<()> {
            if *has_attribute {
                Err(Box::new(crate::Error {
                    kind: crate::ErrorKind::DuplicatedDisplayAttribute {
                        key: attribute.key().into(),
                    },
                    span: attribute.key_span(),
                }))
            }
            else {
                *has_attribute = true;
                Ok(())
            }
        }
        fn get_arguments(attribute: &ProgramDisplayAttribute, min_arity: usize, max_arity: usize) -> crate::Result<&[Value]> {
            let ProgramDisplayAttributeValue::Arguments(arguments) = attribute.value() else {
                return Err(Box::new(crate::Error {
                    kind: crate::ErrorKind::DisplayAttributeExpectedArguments {
                        key: attribute.key().into(),
                    },
                    span: attribute.key_span(),
                }));
            };

            if (min_arity ..= max_arity).contains(&arguments.len()) {
                Ok(arguments)
            }
            else {
                Err(Box::new(crate::Error {
                    kind: crate::ErrorKind::InvalidDisplayAttributeArity {
                        key: attribute.key().into(),
                        min: min_arity,
                        max: max_arity,
                        got: arguments.len(),
                    },
                    span: attribute.key_span(),
                }))
            }
        }
        fn get_action(attribute: &ProgramDisplayAttribute) -> crate::Result<&ActionValue> {
            if let ProgramDisplayAttributeValue::Action(action) = attribute.value() {
                Ok(action)
            }
            else {
                Err(Box::new(crate::Error {
                    kind: crate::ErrorKind::DisplayAttributeExpectedAction {
                        key: attribute.key().into(),
                    },
                    span: attribute.key_span(),
                }))
            }
        }

        let mut has_color = false;
        let mut has_point = false;
        let mut has_label = false;
        let mut has_line = false;
        let mut has_fill = false;
        let mut has_click = false;

        for attribute in element.attributes() {
            match attribute.key() {
                "color" => {
                    // color(<color>: color)
                    prevent_duplicate(attribute, &mut has_color)?;
                    let arguments = get_arguments(attribute, 1, 1)?;

                    // TODO: constant => set entry.color
                    entry.color_expression = self.translate_value(&arguments[0])?;
                }
                "point" => {
                    // point([opacity]: real, [size]: real, [style]: str, [outline]: bool)
                    prevent_duplicate(attribute, &mut has_point)?;
                    let arguments = get_arguments(attribute, 0, 4)?;

                    entry.point.display = true;
                    if let Some(opacity) = arguments.get(0) {
                        entry.point.opacity = self.translate_value(opacity)?;
                    }
                    if let Some(size) = arguments.get(1) {
                        entry.point.size = self.translate_value(size)?;
                    }
                    if let Some(style_value) = arguments.get(2) {
                        let ValueKind::Str(style_name) = &style_value.kind else {
                            return Err(unsupported_error(style_value))
                        };
                        let Some(style) = GraphPointStyle::from_name(style_name) else {
                            return Err(unsupported_error(style_value))
                        };
                        entry.point.style = style;
                    }
                    if let Some(outline_value) = arguments.get(3) {
                        let ValueKind::Bool(outline) = outline_value.kind else {
                            return Err(unsupported_error(outline_value))
                        };
                        entry.point.outline = outline;
                    }
                }
                "label" => {
                    // label([text]: str, [opacity]: real, [size]: real, [angle]: real,
                    //       [orientation]: str, [outline]: bool)
                    prevent_duplicate(attribute, &mut has_label)?;
                    let arguments = get_arguments(attribute, 0, 6)?;

                    entry.label.display = true;
                    if let Some(text_value) = arguments.get(0) {
                        let ValueKind::Str(text) = &text_value.kind else {
                            return Err(unsupported_error(text_value))
                        };
                        entry.label.text = text.to_string();
                    }
                    if let Some(opacity) = arguments.get(1) {
                        entry.label.opacity = self.translate_value(opacity)?;
                    }
                    if let Some(size) = arguments.get(2) {
                        entry.label.size = self.translate_value(size)?;
                    }
                    if let Some(angle) = arguments.get(3) {
                        entry.label.angle = self.translate_value(angle)?;
                    }
                    if let Some(orientation_value) = arguments.get(4) {
                        let ValueKind::Str(orientation_name) = &orientation_value.kind else {
                            return Err(unsupported_error(orientation_value))
                        };
                        let Some(orientation) = GraphLabelOrientation::from_name(orientation_name) else {
                            return Err(unsupported_error(orientation_value))
                        };
                        entry.label.orientation = orientation;
                    }
                    if let Some(outline_value) = arguments.get(5) {
                        let ValueKind::Bool(outline) = outline_value.kind else {
                            return Err(unsupported_error(outline_value))
                        };
                        entry.label.outline = outline;
                    }
                }
                "line" => {
                    // line([opacity]: real, [width]: real, [style]: str)
                    prevent_duplicate(attribute, &mut has_line)?;
                    let arguments = get_arguments(attribute, 0, 3)?;

                    entry.line.display = true;
                    if let Some(opacity) = arguments.get(0) {
                        entry.line.opacity = self.translate_value(opacity)?;
                    }
                    if let Some(width) = arguments.get(1) {
                        entry.line.width = self.translate_value(width)?;
                    }
                    if let Some(style_value) = arguments.get(2) {
                        let ValueKind::Str(style_name) = &style_value.kind else {
                            return Err(unsupported_error(style_value))
                        };
                        let Some(style) = GraphLineStyle::from_name(style_name) else {
                            return Err(unsupported_error(style_value))
                        };
                        entry.line.style = style;
                    }
                }
                "fill" => {
                    // fill([opacity]: real)
                    prevent_duplicate(attribute, &mut has_fill)?;
                    let arguments = get_arguments(attribute, 0, 1)?;

                    entry.fill.display = true;
                    if let Some(opacity) = arguments.get(0) {
                        entry.fill.opacity = self.translate_value(opacity)?;
                    }
                }
                "click" => {
                    // TODO: click(action(index) { ... }) to allow other attributes?
                    // click { ... }
                    prevent_duplicate(attribute, &mut has_click)?;
                    let action = get_action(attribute)?;

                    entry.clickable.enabled = true;
                    entry.clickable.expression = self.translate_action_value(action)?;
                }
                _ => {
                    return Err(Box::new(crate::Error {
                        kind: crate::ErrorKind::UnsupportedDisplayAttribute {
                            key: attribute.key().into(),
                        },
                        span: attribute.key_span(),
                    }));
                }
            }
        }

        self.display_entries.push(Box::new(entry));

        Ok(())
    }

    pub fn set_program(&mut self, program: &Program) -> crate::Result<()> {
        for program_let in program.lets() {
            self.add_program_let(program_let)?;
        }
        for program_variable in program.variables() {
            self.add_program_variable(program_variable)?;
        }
        for program_action in program.actions() {
            self.add_program_action(program_action)?;
        }
        self.set_program_ticker(program.ticker())?;
        if let Some(program_public) = program.public() {
            for public_line in program_public.lines() {
                self.add_public_line(public_line)?;
            }
        }
        if let Some(program_display) = program.display() {
            for display_element in program_display.elements() {
                self.add_display_element(display_element)?;
            }
        }

        Ok(())
    }
}
