use crate::ast::RangeKind;
use crate::desmos::{GraphBinaryKind, GraphEntry, GraphExpression, GraphExpressionEntry, GraphExpressionList, GraphFolderEntry, GraphImageEntry, GraphInequalityKind, GraphSlider, GraphSliderLoopMode, GraphTextEntry, GraphTicker, GraphUnaryKind};
use crate::desmos::symbol::SymbolTable;
use crate::sema::{Program, ProgramAction, ProgramLet, ProgramPublicLine, ProgramTicker, ProgramVariable, ProgramVariableKind};
use crate::sema::display::{ImageValue, ProgramDisplayAttributeKind, ProgramDisplayElement};
use crate::sema::types::Type;
use crate::sema::values::{ActionValue, ActionValueKind, MathematicalConstant, Value, IndexKind, ValueKind, UnaryKind, BinaryKind, ReducerKind, DoubleReducerKind, ParameterizedReducerKind, ColorKind};

mod geometry;
mod graphing;
mod graphing3d;

pub use geometry::DesmosGeometryTarget;
pub use graphing::DesmosGraphingTarget;
pub use graphing3d::DesmosGraphing3DTarget;

pub fn new_target_by_name(name: &str) -> crate::Result<Box<dyn crate::target::Target>> {
    match name {
        "desmos-geometry" => Ok(Box::new(DesmosGeometryTarget::default())),
        "desmos-graphing" => Ok(Box::new(DesmosGraphingTarget::default())),
        "desmos-graphing3d" => Ok(Box::new(DesmosGraphing3DTarget::default())),
        _ => Err(Box::new(crate::Error {
            kind: crate::ErrorKind::UnsupportedTarget {
                name: name.into(),
            },
            span: None,
        }))
    }
}

#[derive(Debug)]
pub struct DesmosTargetInfo {
    next_local_id: u64,
    next_entry_id: u64,
    global_symbols: SymbolTable,
    action_symbols: SymbolTable,
}

impl DesmosTargetInfo {
    pub fn new() -> Self {
        Self {
            next_local_id: 0,
            next_entry_id: 0,
            global_symbols: SymbolTable::new(GraphExpression::Letter('G')),
            action_symbols: SymbolTable::new(GraphExpression::Letter('A')),
        }
    }

    pub fn create_local_id(&mut self) -> u64 {
        let id = self.next_local_id;
        self.next_local_id += 1;
        id
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
            kind: GraphBinaryKind::Subscript,
            lhs: Box::new(GraphExpression::Letter('l')),
            rhs: Box::new(GraphExpression::Alphanumeric(id.to_string())),
        }
    }

    pub fn create_local_symbol(&mut self) -> GraphExpression {
        let id = self.create_local_id();
        self.get_local_symbol(id)
    }
}

pub const INTRINSICS_FOLDER_ID: &str = "desmosify_intrinsics";
pub const GLOBALS_FOLDER_ID: &str = "desmosify_globals";
pub const ACTIONS_FOLDER_ID: &str = "desmosify_actions";
pub const DISPLAY_FOLDER_ID: &str = "desmosify_display";

pub struct GraphExpressionListBuilder<'target> {
    target_info: &'target mut DesmosTargetInfo,
    ticker: Option<GraphTicker>,
    public_entries: Vec<Box<dyn GraphEntry>>,
    intrinsic_entries: Vec<Box<dyn GraphEntry>>,
    global_entries: Vec<Box<dyn GraphEntry>>,
    action_entries: Vec<Box<dyn GraphEntry>>,
    display_entries: Vec<Box<dyn GraphEntry>>,
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

impl<'target> GraphExpressionListBuilder<'target> {
    pub fn build_program(program: &Program, target_info: &'target mut DesmosTargetInfo) -> crate::Result<GraphExpressionList> {
        let mut builder = Self::new(target_info);
        builder.set_program(program)?;
        Ok(builder.finish())
    }

    pub fn new(target_info: &'target mut DesmosTargetInfo) -> Self {
        Self {
            target_info,
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

    pub fn finish(mut self) -> GraphExpressionList {
        self.public_entries.push(Box::new(GraphExpressionEntry {
            id: self.target_info.create_entry_id(),
            ..Default::default()
        }));

        GraphExpressionList {
            ticker: self.ticker,
            entries: self.public_entries
                .into_iter()
                .chain(self.intrinsic_entries)
                .chain(self.global_entries)
                .chain(self.action_entries)
                .chain(self.display_entries)
                .collect(),
        }
    }

    pub fn create_dummy_noop(&mut self) -> GraphExpression {
        let dummy_noop_id = self.next_dummy_noop_id;
        self.next_dummy_noop_id += 1;
        let symbol = GraphExpression::Binary {
            kind: GraphBinaryKind::Subscript,
            lhs: Box::new(GraphExpression::Letter('D')),
            rhs: Box::new(GraphExpression::Alphanumeric(format!("Noop{dummy_noop_id}"))),
        };

        self.intrinsic_entries.push(Box::new(GraphExpressionEntry {
            id: self.target_info.create_entry_id(),
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
            self.intrinsic_entries.push(Box::new(GraphExpressionEntry {
                id: self.target_info.create_entry_id(),
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
            let local_a = self.target_info.create_local_symbol();
            let local_b = self.target_info.create_local_symbol();
            let local_s = self.target_info.create_local_symbol();

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
                id: self.target_info.create_entry_id(),
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
            let local_a = self.target_info.create_local_symbol();
            let local_b = self.target_info.create_local_symbol();
            let local_s = self.target_info.create_local_symbol();
            let local_x = self.target_info.create_local_symbol();

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
                id: self.target_info.create_entry_id(),
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
            let local_l = self.target_info.create_local_symbol();
            let local_a = self.target_info.create_local_symbol();
            let local_b = self.target_info.create_local_symbol();
            let local_s = self.target_info.create_local_symbol();
            let local_i = self.target_info.create_local_symbol();

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
                id: self.target_info.create_entry_id(),
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
            let local_l = self.target_info.create_local_symbol();
            let local_a = self.target_info.create_local_symbol();
            let local_b = self.target_info.create_local_symbol();
            let local_s = self.target_info.create_local_symbol();
            let local_i = self.target_info.create_local_symbol();

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
                id: self.target_info.create_entry_id(),
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
            let local_l = self.target_info.create_local_symbol();
            let local_a = self.target_info.create_local_symbol();
            let local_s = self.target_info.create_local_symbol();

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
                id: self.target_info.create_entry_id(),
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
            let local_l = self.target_info.create_local_symbol();
            let local_b = self.target_info.create_local_symbol();

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
                id: self.target_info.create_entry_id(),
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
            let local_l = self.target_info.create_local_symbol();
            let local_b = self.target_info.create_local_symbol();

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
                id: self.target_info.create_entry_id(),
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
                // Create undefined using 0 / 0
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Divide,
                    lhs: Box::new(match value_type {
                        Type::Point2 { .. } => GraphExpression::Unary {
                            kind: GraphUnaryKind::Parentheses,
                            inner: Box::new(GraphExpression::Sequence {
                                elements: Vec::from([
                                    GraphExpression::Integer(0),
                                    GraphExpression::Integer(0),
                                ]),
                            }),
                        },
                        Type::Point3 { .. } => GraphExpression::Unary {
                            kind: GraphUnaryKind::Parentheses,
                            inner: Box::new(GraphExpression::Sequence {
                                elements: Vec::from([
                                    GraphExpression::Integer(0),
                                    GraphExpression::Integer(0),
                                    GraphExpression::Integer(0),
                                ]),
                            }),
                        },
                        _ => GraphExpression::Integer(0)
                    }),
                    rhs: Box::new(GraphExpression::Integer(0)),
                })
            }
            ValueKind::Infinity(..) => {
                Ok(GraphExpression::Escape("infty".into()))
            }
            ValueKind::Real(value) => {
                Ok(GraphExpression::Decimal(*value))
            }
            ValueKind::Mathematical(kind) => {
                Ok(match kind {
                    MathematicalConstant::Pi => GraphExpression::Escape("pi".into()),
                    MathematicalConstant::Tau => GraphExpression::Escape("tau".into()),
                    MathematicalConstant::E => GraphExpression::Letter('e'),
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
            ValueKind::Global(reference) => {
                Ok(self.target_info.get_global_symbol(&reference.identifier))
            }
            ValueKind::Action(reference) => {
                Ok(self.target_info.get_action_symbol(&reference.identifier))
            }
            ValueKind::Local(reference) => {
                Ok(self.target_info.get_local_symbol(reference.id))
            }
            ValueKind::ViewportWidth => {
                Ok(GraphExpression::OperatorName("width".into()))
            }
            ValueKind::ViewportHeight => {
                Ok(GraphExpression::OperatorName("height".into()))
            }
            ValueKind::TickerDt => {
                Ok(GraphExpression::OperatorName("dt".into()))
            }
            ValueKind::ClickIndex => {
                Ok(GraphExpression::OperatorName("index".into()))
            }
            ValueKind::Unary { kind, operand, .. } => {
                self.translate_unary(*kind, operand, unsupported_error)
            }
            ValueKind::Binary { kind, lhs, rhs, .. } => {
                self.translate_binary(*kind, lhs, rhs, unsupported_error)
            }
            ValueKind::Reducer { kind, list, .. } => {
                self.translate_reducer(*kind, [list.as_ref()], unsupported_error)
            }
            ValueKind::ArgumentsReducer { kind, arguments, .. } => {
                self.translate_reducer(*kind, arguments, unsupported_error)
            }
            ValueKind::DoubleReducer { kind, list_1, list_2, .. } => {
                self.translate_double_reducer(*kind, list_1, list_2, unsupported_error)
            }
            ValueKind::ParameterizedReducer { kind, list, parameter, .. } => {
                self.translate_parameterized_reducer(*kind, list, parameter, unsupported_error)
            }
            ValueKind::Color { kind, value_1, value_2, value_3, .. } => {
                self.translate_color(*kind, value_1, value_2, value_3, unsupported_error)
            }
            ValueKind::Join { values, .. } => {
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(GraphExpression::OperatorName("join".into())),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: values
                            .iter()
                            .map(|argument| self.translate_value(argument))
                            .collect::<crate::Result<_>>()?,
                    }),
                })
            }
            ValueKind::Sort { list, key_list } => {
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(GraphExpression::OperatorName("sort".into())),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: [list.as_ref()]
                            .into_iter()
                            .chain(key_list.as_deref())
                            .map(|argument| self.translate_value(argument))
                            .collect::<crate::Result<_>>()?,
                    }),
                })
            }
            ValueKind::Shuffle { list, seed } => {
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(GraphExpression::OperatorName("shuffle".into())),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: [list.as_ref()]
                            .into_iter()
                            .chain(seed.as_deref())
                            .map(|argument| self.translate_value(argument))
                            .collect::<crate::Result<_>>()?,
                    }),
                })
            }
            ValueKind::Unique { list } => {
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(GraphExpression::OperatorName("unique".into())),
                    rhs: Box::new(self.translate_value(list)?),
                })
            }
            ValueKind::Rotation { object, point, angle, .. } => {
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
                                    lhs: Box::new(self.target_info.get_local_symbol(map_loop.local.id)),
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
                    rhs: Box::new(self.translate_condition(condition)?),
                })
            }
            ValueKind::Index { list, kind: operation, .. } => match operation {
                IndexKind::Single { index } => {
                    Ok(GraphExpression::Binary {
                        kind: GraphBinaryKind::Index,
                        lhs: Box::new(self.translate_value(list)?),
                        rhs: Box::new(self.translate_value(index)?),
                    })
                }
                IndexKind::Range { kind, from_index, to_index, step} => {
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
                IndexKind::RangeFrom { from_index, step } => {
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
                IndexKind::RangeTo { kind, to_index } => {
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
            _ => {
                Err(unsupported_error())
            }
        }
    }

    fn translate_unary(
        &mut self,
        kind: UnaryKind,
        operand: &Value,
        unsupported_error: impl Fn() -> Box<crate::Error>,
    ) -> crate::Result<GraphExpression> {
        let _ = unsupported_error; // We'll use you soon enough

        fn unary_operator(kind: GraphUnaryKind, inner: GraphExpression) -> GraphExpression {
            GraphExpression::Unary {
                kind: GraphUnaryKind::Parentheses,
                inner: Box::new(GraphExpression::Unary {
                    kind,
                    inner: Box::new(inner),
                }),
            }
        }
        fn unary_function(name: &str, argument: GraphExpression) -> GraphExpression {
            GraphExpression::Binary {
                kind: GraphBinaryKind::Call,
                lhs: Box::new(GraphExpression::OperatorName(name.into())),
                rhs: Box::new(argument),
            }
        }

        match kind {
            UnaryKind::AssumeType => {
                self.translate_value(operand)
            }
            UnaryKind::Positive => {
                Ok(unary_operator(
                    GraphUnaryKind::Positive,
                    self.translate_value(operand)?,
                ))
            }
            UnaryKind::Negative => {
                Ok(unary_operator(
                    GraphUnaryKind::Negative,
                    self.translate_value(operand)?,
                ))
            }
            UnaryKind::LogicalNot => {
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
            UnaryKind::GetX => {
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Dot,
                    lhs: Box::new(self.translate_value(operand)?),
                    rhs: Box::new(GraphExpression::Letter('x')),
                })
            }
            UnaryKind::GetY => {
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Dot,
                    lhs: Box::new(self.translate_value(operand)?),
                    rhs: Box::new(GraphExpression::Letter('y')),
                })
            }
            UnaryKind::GetZ => {
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Dot,
                    lhs: Box::new(self.translate_value(operand)?),
                    rhs: Box::new(GraphExpression::Letter('z')),
                })
            }
            UnaryKind::Sin => {
                Ok(unary_function(
                    "sin",
                    self.translate_value(operand)?,
                ))
            }
            UnaryKind::Cos => {
                Ok(unary_function(
                    "cos",
                    self.translate_value(operand)?,
                ))
            }
            UnaryKind::Tan => {
                Ok(unary_function(
                    "tan",
                    self.translate_value(operand)?,
                ))
            }
            UnaryKind::Csc => {
                Ok(unary_function(
                    "csc",
                    self.translate_value(operand)?,
                ))
            }
            UnaryKind::Sec => {
                Ok(unary_function(
                    "sec",
                    self.translate_value(operand)?,
                ))
            }
            UnaryKind::Cot => {
                Ok(unary_function(
                    "cot",
                    self.translate_value(operand)?,
                ))
            }
            UnaryKind::Arcsin => {
                Ok(unary_function(
                    "arcsin",
                    self.translate_value(operand)?,
                ))
            }
            UnaryKind::Arccos => {
                Ok(unary_function(
                    "arccos",
                    self.translate_value(operand)?,
                ))
            }
            UnaryKind::Arctan => {
                Ok(unary_function(
                    "arctan",
                    self.translate_value(operand)?,
                ))
            }
            UnaryKind::Arccsc => {
                Ok(unary_function(
                    "arccsc",
                    self.translate_value(operand)?,
                ))
            }
            UnaryKind::Arcsec => {
                Ok(unary_function(
                    "arcsec",
                    self.translate_value(operand)?,
                ))
            }
            UnaryKind::Arccot => {
                Ok(unary_function(
                    "arccot",
                    self.translate_value(operand)?,
                ))
            }
            UnaryKind::Sinh => {
                Ok(unary_function(
                    "sinh",
                    self.translate_value(operand)?,
                ))
            }
            UnaryKind::Cosh => {
                Ok(unary_function(
                    "cosh",
                    self.translate_value(operand)?,
                ))
            }
            UnaryKind::Tanh => {
                Ok(unary_function(
                    "tanh",
                    self.translate_value(operand)?,
                ))
            }
            UnaryKind::Csch => {
                Ok(unary_function(
                    "csch",
                    self.translate_value(operand)?,
                ))
            }
            UnaryKind::Sech => {
                Ok(unary_function(
                    "sech",
                    self.translate_value(operand)?,
                ))
            }
            UnaryKind::Coth => {
                Ok(unary_function(
                    "coth",
                    self.translate_value(operand)?,
                ))
            }
            UnaryKind::Exp => {
                Ok(unary_function(
                    "exp",
                    self.translate_value(operand)?,
                ))
            }
            UnaryKind::Ln => {
                Ok(unary_function(
                    "exp",
                    self.translate_value(operand)?,
                ))
            }
            UnaryKind::Ceil => {
                Ok(unary_function(
                    "ceil",
                    self.translate_value(operand)?,
                ))
            }
            UnaryKind::Floor => {
                Ok(unary_function(
                    "floor",
                    self.translate_value(operand)?,
                ))
            }
            UnaryKind::Round => {
                Ok(unary_function(
                    "round",
                    self.translate_value(operand)?,
                ))
            }
            UnaryKind::Abs => {
                Ok(GraphExpression::Unary {
                    kind: GraphUnaryKind::Pipes,
                    inner: Box::new(self.translate_value(operand)?),
                })
            }
            UnaryKind::Sign => {
                Ok(unary_function(
                    "sign",
                    self.translate_value(operand)?,
                ))
            }
            UnaryKind::Sqrt => {
                Ok(GraphExpression::Radical {
                    index: None,
                    radicand: Box::new(self.translate_value(operand)?),
                })
            }
            UnaryKind::Cbrt => {
                Ok(GraphExpression::Radical {
                    index: Some(Box::new(GraphExpression::Integer(3))),
                    radicand: Box::new(self.translate_value(operand)?),
                })
            }
            UnaryKind::Factorial => {
                Ok(unary_operator(
                    GraphUnaryKind::Factorial,
                    self.translate_value(operand)?,
                ))
            }
            UnaryKind::MidpointOfSegment => {
                Ok(unary_function(
                    "midpoint",
                    self.translate_value(operand)?,
                ))
            }
            UnaryKind::VectorStart => {
                Ok(unary_function(
                    "start",
                    self.translate_value(operand)?,
                ))
            }
            UnaryKind::VectorEnd => {
                Ok(unary_function(
                    "end",
                    self.translate_value(operand)?,
                ))
            }
        }
    }

    fn translate_binary(
        &mut self,
        kind: BinaryKind,
        lhs: &Value,
        rhs: &Value,
        unsupported_error: impl Fn() -> Box<crate::Error>,
    ) -> crate::Result<GraphExpression> {
        let _ = unsupported_error; // We'll use you soon enough

        fn binary_operator(kind: GraphBinaryKind, lhs: GraphExpression, rhs: GraphExpression) -> GraphExpression {
            GraphExpression::Unary {
                kind: GraphUnaryKind::Parentheses,
                inner: Box::new(GraphExpression::Binary {
                    kind,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }),
            }
        }
        fn binary_function(name: &str, lhs: GraphExpression, rhs: GraphExpression) -> GraphExpression {
            GraphExpression::Binary {
                kind: GraphBinaryKind::Call,
                lhs: Box::new(GraphExpression::OperatorName(name.into())),
                rhs: Box::new(GraphExpression::Sequence {
                    elements: [lhs, rhs].into(),
                }),
            }
        }

        match kind {
            BinaryKind::Exponent => {
                Ok(binary_operator(
                    GraphBinaryKind::Superscript,
                    self.translate_value(lhs)?,
                    self.translate_value(rhs)?,
                ))
            }
            BinaryKind::Multiply => {
                Ok(binary_operator(
                    GraphBinaryKind::Multiply,
                    self.translate_value(lhs)?,
                    self.translate_value(rhs)?,
                ))
            }
            BinaryKind::DotProduct => {
                Ok(binary_operator(
                    GraphBinaryKind::DotMultiply,
                    self.translate_value(lhs)?,
                    self.translate_value(rhs)?,
                ))
            }
            BinaryKind::CrossProduct => {
                Ok(binary_operator(
                    GraphBinaryKind::CrossMultiply,
                    self.translate_value(lhs)?,
                    self.translate_value(rhs)?,
                ))
            }
            BinaryKind::Divide => {
                Ok(binary_operator(
                    GraphBinaryKind::Divide,
                    self.translate_value(lhs)?,
                    self.translate_value(rhs)?,
                ))
            }
            BinaryKind::Remainder => {
                Ok(binary_function(
                    "mod",
                    self.translate_value(lhs)?,
                    self.translate_value(rhs)?,
                ))
            }
            BinaryKind::Add => {
                Ok(binary_operator(
                    GraphBinaryKind::Add,
                    self.translate_value(lhs)?,
                    self.translate_value(rhs)?,
                ))
            }
            BinaryKind::Subtract => {
                Ok(binary_operator(
                    GraphBinaryKind::Subtract,
                    self.translate_value(lhs)?,
                    self.translate_value(rhs)?,
                ))
            }
            BinaryKind::LessThan => {
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
            BinaryKind::LessEqual => {
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
            BinaryKind::GreaterThan => {
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
            BinaryKind::GreaterEqual => {
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
            BinaryKind::Equal => {
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
            BinaryKind::NotEqual => {
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
            BinaryKind::LogicalAnd => {
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
            BinaryKind::LogicalOr => {
                Ok(GraphExpression::Unary {
                    kind: GraphUnaryKind::Piecewise,
                    inner: Box::new(GraphExpression::Sequence {
                        elements: Vec::from([
                            self.translate_condition(lhs)?,
                            self.translate_condition(rhs)?,
                            GraphExpression::Integer(0),
                        ]),
                    }),
                })
            }
            BinaryKind::Log => {
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(GraphExpression::Binary {
                        kind: GraphBinaryKind::Subscript,
                        lhs: Box::new(GraphExpression::OperatorName("log".into())),
                        rhs: Box::new(self.translate_value(lhs)?),
                    }),
                    rhs: Box::new(self.translate_value(rhs)?),
                })
            }
            BinaryKind::Lcm => {
                Ok(binary_function(
                    "lcm",
                    self.translate_value(lhs)?,
                    self.translate_value(rhs)?,
                ))
            }
            BinaryKind::Gcd => {
                Ok(binary_function(
                    "gcd",
                    self.translate_value(lhs)?,
                    self.translate_value(rhs)?,
                ))
            }
            BinaryKind::NthRoot => {
                Ok(GraphExpression::Radical {
                    index: Some(Box::new(self.translate_value(rhs)?)),
                    radicand: Box::new(self.translate_value(lhs)?),
                })
            }
            BinaryKind::MidpointOfPoints => {
                Ok(binary_function(
                    "midpoint",
                    self.translate_value(lhs)?,
                    self.translate_value(rhs)?,
                ))
            }
            BinaryKind::Segment | BinaryKind::Segment3D => {
                Ok(binary_function(
                    "segment",
                    self.translate_value(lhs)?,
                    self.translate_value(rhs)?,
                ))
            }
            BinaryKind::Line => {
                Ok(binary_function(
                    "line",
                    self.translate_value(lhs)?,
                    self.translate_value(rhs)?,
                ))
            }
            BinaryKind::Ray => {
                Ok(binary_function(
                    "ray",
                    self.translate_value(lhs)?,
                    self.translate_value(rhs)?,
                ))
            }
            BinaryKind::Vector | BinaryKind::Vector3D => {
                Ok(binary_function(
                    "vector",
                    self.translate_value(lhs)?,
                    self.translate_value(rhs)?,
                ))
            }
            BinaryKind::Circle => {
                Ok(binary_function(
                    "circle",
                    self.translate_value(lhs)?,
                    self.translate_value(rhs)?,
                ))
            }
            BinaryKind::Sphere3D => {
                Ok(binary_function(
                    "sphere",
                    self.translate_value(lhs)?,
                    self.translate_value(rhs)?,
                ))
            }
        }
    }

    fn translate_reducer<'a>(
        &mut self,
        kind: ReducerKind,
        arguments: impl IntoIterator<Item = &'a Value>,
        unsupported_error: impl Fn() -> Box<crate::Error>,
    ) -> crate::Result<GraphExpression> {
        let _ = unsupported_error; // We'll use you soon enough

        fn get_name(kind: ReducerKind) -> &'static str {
            match kind {
                ReducerKind::Mean => "mean",
                ReducerKind::Median => "median",
                ReducerKind::Min => "min",
                ReducerKind::Max => "max",
                ReducerKind::Stdev => "stdev",
                ReducerKind::Stdevp => "stdevp",
                ReducerKind::Var => "var",
                ReducerKind::Varp => "varp",
                ReducerKind::Mad => "mad",
                ReducerKind::Count => "count",
                ReducerKind::Total => "total",
                ReducerKind::Polygon => "polygon",
                ReducerKind::Lcm => "lcm",
                ReducerKind::Gcd => "gcd",
            }
        }

        Ok(GraphExpression::Binary {
            kind: GraphBinaryKind::Call,
            lhs: Box::new(GraphExpression::OperatorName(get_name(kind).into())),
            rhs: Box::new(GraphExpression::Sequence {
                elements: arguments
                    .into_iter()
                    .map(|argument| self.translate_value(argument))
                    .collect::<crate::Result<_>>()?,
            }),
        })
    }

    fn translate_double_reducer<'a>(
        &mut self,
        kind: DoubleReducerKind,
        list_1: &Value,
        list_2: &Value,
        unsupported_error: impl Fn() -> Box<crate::Error>,
    ) -> crate::Result<GraphExpression> {
        let _ = unsupported_error; // We'll use you soon enough

        fn get_name(kind: DoubleReducerKind) -> &'static str {
            match kind {
                DoubleReducerKind::Cov => "cov",
                DoubleReducerKind::Covp => "covp",
                DoubleReducerKind::Corr => "corr",
                DoubleReducerKind::Spearman => "spearman",
            }
        }

        Ok(GraphExpression::Binary {
            kind: GraphBinaryKind::Call,
            lhs: Box::new(GraphExpression::OperatorName(get_name(kind).into())),
            rhs: Box::new(GraphExpression::Sequence {
                elements: Vec::from([
                    self.translate_value(list_1)?,
                    self.translate_value(list_2)?,
                ]),
            }),
        })
    }

    fn translate_parameterized_reducer<'a>(
        &mut self,
        kind: ParameterizedReducerKind,
        list: &Value,
        parameter: &Value,
        unsupported_error: impl Fn() -> Box<crate::Error>,
    ) -> crate::Result<GraphExpression> {
        let _ = unsupported_error; // We'll use you soon enough

        fn get_name(kind: ParameterizedReducerKind) -> &'static str {
            match kind {
                ParameterizedReducerKind::Quartile => "quartile",
                ParameterizedReducerKind::Quantile => "quantile",
                ParameterizedReducerKind::Tscore => "tscore",
            }
        }

        Ok(GraphExpression::Binary {
            kind: GraphBinaryKind::Call,
            lhs: Box::new(GraphExpression::OperatorName(get_name(kind).into())),
            rhs: Box::new(GraphExpression::Sequence {
                elements: Vec::from([
                    self.translate_value(list)?,
                    self.translate_value(parameter)?,
                ]),
            }),
        })
    }

    fn translate_color<'a>(
        &mut self,
        kind: ColorKind,
        value_1: &Value,
        value_2: &Value,
        value_3: &Value,
        unsupported_error: impl Fn() -> Box<crate::Error>,
    ) -> crate::Result<GraphExpression> {
        let _ = unsupported_error; // We'll use you soon enough

        fn get_name(kind: ColorKind) -> &'static str {
            match kind {
                ColorKind::Rgb => "rgb",
                ColorKind::Hsv => "hsv",
                ColorKind::Okhsv => "okhsv",
                ColorKind::Oklab => "oklab",
                ColorKind::Oklch => "oklch",
            }
        }

        Ok(GraphExpression::Binary {
            kind: GraphBinaryKind::Call,
            lhs: Box::new(GraphExpression::OperatorName(get_name(kind).into())),
            rhs: Box::new(GraphExpression::Sequence {
                elements: Vec::from([
                    self.translate_value(value_1)?,
                    self.translate_value(value_2)?,
                    self.translate_value(value_3)?,
                ]),
            }),
        })
    }

    pub fn translate_condition(&mut self, value: &Value) -> crate::Result<GraphExpression> {
        match &value.kind {
            ValueKind::Bool(true) => {
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Equal,
                    lhs: Box::new(GraphExpression::Integer(0)),
                    rhs: Box::new(GraphExpression::Integer(0)),
                })
            }
            ValueKind::Bool(false) => {
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Equal,
                    lhs: Box::new(GraphExpression::Integer(0)),
                    rhs: Box::new(GraphExpression::Integer(1)),
                })
            }
            ValueKind::Unary { kind: UnaryKind::LogicalNot, operand, .. } => {
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Equal,
                    lhs: Box::new(self.translate_value(operand)?),
                    rhs: Box::new(GraphExpression::Integer(0)),
                })
            }
            ValueKind::Binary { kind: BinaryKind::Equal, lhs, rhs, .. } => {
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Equal,
                    lhs: Box::new(self.translate_value(lhs)?),
                    rhs: Box::new(self.translate_value(rhs)?),
                })
            }
            ValueKind::Binary { kind: BinaryKind::NotEqual, lhs, rhs, .. } => {
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
            ValueKind::Binary { kind: BinaryKind::LessThan, lhs, rhs, .. } => {
                Ok(GraphExpression::InequalityChain {
                    first_kind: GraphInequalityKind::LessThan,
                    lhs: Box::new(self.translate_value(lhs)?),
                    rhs: Box::new(self.translate_value(rhs)?),
                    chain: Vec::new(),
                })
            }
            ValueKind::Binary { kind: BinaryKind::GreaterThan, lhs, rhs, .. } => {
                Ok(GraphExpression::InequalityChain {
                    first_kind: GraphInequalityKind::GreaterThan,
                    lhs: Box::new(self.translate_value(lhs)?),
                    rhs: Box::new(self.translate_value(rhs)?),
                    chain: Vec::new(),
                })
            }
            ValueKind::Binary { kind: BinaryKind::LessEqual, lhs, rhs, .. } => {
                Ok(GraphExpression::InequalityChain {
                    first_kind: GraphInequalityKind::LessEqual,
                    lhs: Box::new(self.translate_value(lhs)?),
                    rhs: Box::new(self.translate_value(rhs)?),
                    chain: Vec::new(),
                })
            }
            ValueKind::Binary { kind: BinaryKind::GreaterEqual, lhs, rhs, .. } => {
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
                    lhs: Box::new(self.target_info.get_global_symbol(&variable.identifier)),
                    rhs: Box::new(self.translate_value(value)?),
                })
            }
            ActionValueKind::ActionCall { action, arguments, .. } => {
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(self.translate_value(action)?),
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
        let value = self.translate_value(program_let.value())?;

        self.global_entries.push(Box::new(GraphExpressionEntry {
            id: self.target_info.create_entry_id(),
            folder_id: Some(GLOBALS_FOLDER_ID.into()),
            expression: GraphExpression::Binary {
                kind: GraphBinaryKind::Equal,
                lhs: Box::new(match program_let.parameters() {
                    Some(parameters) => GraphExpression::Binary {
                        kind: GraphBinaryKind::Call,
                        lhs: Box::new(self.target_info.get_global_symbol(&program_let.identifier())),
                        rhs: Box::new(GraphExpression::Sequence {
                            elements: parameters
                                .iter()
                                .map(|parameter| self.target_info.get_local_symbol(parameter.id))
                                .collect(),
                        }),
                    },
                    None => self.target_info.get_global_symbol(&program_let.identifier()),
                }),
                rhs: Box::new(value),
            },
            ..Default::default()
        }));

        Ok(())
    }

    pub fn add_program_variable(&mut self, program_variable: &ProgramVariable) -> crate::Result<()> {
        let value = self.translate_value(program_variable.value())?;

        self.global_entries.push(Box::new(GraphExpressionEntry {
            id: self.target_info.create_entry_id(),
            folder_id: Some(GLOBALS_FOLDER_ID.into()),
            expression: GraphExpression::Binary {
                kind: GraphBinaryKind::Equal,
                lhs: Box::new(self.target_info.get_global_symbol(&program_variable.identifier())),
                rhs: Box::new(value),
            },
            slider: match program_variable.kind() {
                ProgramVariableKind::Default => None,
                ProgramVariableKind::Timer => Some(GraphSlider {
                    loop_mode: GraphSliderLoopMode::PlayIndefinitely,
                    is_playing: true,
                    ..Default::default()
                }),
            },
            ..Default::default()
        }));

        Ok(())
    }

    pub fn add_program_action(&mut self, program_action: &ProgramAction) -> crate::Result<()> {
        let action = self.translate_action_value(program_action.action())?;

        self.action_entries.push(Box::new(GraphExpressionEntry {
            id: self.target_info.create_entry_id(),
            folder_id: Some(ACTIONS_FOLDER_ID.into()),
            expression: GraphExpression::Binary {
                kind: GraphBinaryKind::Equal,
                lhs: Box::new(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(self.target_info.get_action_symbol(&program_action.identifier())),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: program_action
                            .parameters()
                            .iter()
                            .map(|parameter| self.target_info.get_local_symbol(parameter.id))
                            .collect(),
                    }),
                }),
                rhs: Box::new(action),
            },
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
        let id = self.target_info.create_entry_id();
        let entry: Box<dyn GraphEntry> = match public_line {
            ProgramPublicLine::Expression(value) => match &value.kind {
                ValueKind::Str(text) => {
                    let text = text.trim();
                    if text.is_empty() {
                        Box::new(GraphExpressionEntry {
                            id,
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
                _ => {
                    Box::new(GraphExpressionEntry {
                        id,
                        expression: self.translate_value(value)?,
                        ..Default::default()
                    })
                }
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
        match &element.value.kind {
            ValueKind::Image(image, _) => {
                self.add_image_display_element(element, image)
            }
            _ => {
                self.add_expression_display_element(element)
            }
        }
    }

    fn add_image_display_element(&mut self, element: &ProgramDisplayElement, image: &ImageValue) -> crate::Result<()> {
        let mut entry = GraphImageEntry {
            id: self.target_info.create_entry_id(),
            folder_id: Some(DISPLAY_FOLDER_ID.into()),
            image_url: image.url.to_string(),
            name: image.name.to_string(),
            background: image.background,
            center: self.translate_value(&image.center)?,
            width: self.translate_value(&image.width)?,
            height: self.translate_value(&image.height)?,
            opacity: self.translate_value(&image.opacity)?,
            angle: self.translate_value(&image.angle)?,
            ..Default::default()
        };

        for attribute in &element.attributes {
            match &attribute.kind {
                ProgramDisplayAttributeKind::Click { action } => {
                    entry.clickable.enabled = true;
                    entry.clickable.expression = self.translate_action_value(action)?;
                }
                ProgramDisplayAttributeKind::Hovered { url } => {
                    entry.clickable.hovered_image_url = url.to_string();
                }
                ProgramDisplayAttributeKind::Pressed { url } => {
                    entry.clickable.depressed_image_url = url.to_string();
                }
                ProgramDisplayAttributeKind::Description { text } => {
                    entry.clickable.description = text.to_string();
                }
                _ => panic!("given attribute is invalid for an image: {attribute:?}")
            }
        }

        self.display_entries.push(Box::new(entry));

        Ok(())
    }

    fn add_expression_display_element(&mut self, element: &ProgramDisplayElement) -> crate::Result<()> {
        let mut entry = GraphExpressionEntry {
            id: self.target_info.create_entry_id(),
            folder_id: Some(DISPLAY_FOLDER_ID.into()),
            expression: self.translate_value(&element.value)?,
            ..Default::default()
        };

        for attribute in &element.attributes {
            match &attribute.kind {
                ProgramDisplayAttributeKind::Color { value } => {
                    // TODO: constant => set entry.color
                    entry.color_expression = self.translate_value(value)?;
                }
                ProgramDisplayAttributeKind::Point { opacity, size, style, outline } => {
                    entry.display = true;
                    entry.point.display = true;
                    if let Some(opacity) = opacity {
                        entry.point.opacity = self.translate_value(opacity)?;
                    }
                    if let Some(size) = size {
                        entry.point.size = self.translate_value(size)?;
                    }
                    entry.point.style = *style;
                    entry.point.outline = *outline;
                }
                ProgramDisplayAttributeKind::Drag { mode } => {
                    entry.point.drag_mode = *mode;
                }
                ProgramDisplayAttributeKind::Label { text, opacity, size, angle, orientation, outline } => {
                    // Don't set entry.display = true, that will show points as well
                    entry.label.display = true;
                    entry.label.text = text.to_string();
                    if let Some(opacity) = opacity {
                        entry.label.opacity = self.translate_value(opacity)?;
                    }
                    if let Some(size) = size {
                        entry.label.size = self.translate_value(size)?;
                    }
                    if let Some(angle) = angle {
                        entry.label.angle = self.translate_value(angle)?;
                    }
                    entry.label.orientation = *orientation;
                    entry.label.outline = *outline;
                }
                ProgramDisplayAttributeKind::Line { opacity, width, style } => {
                    entry.display = true;
                    entry.line.display = true;
                    if let Some(opacity) = opacity {
                        entry.line.opacity = self.translate_value(opacity)?;
                    }
                    if let Some(width) = width {
                        entry.line.width = self.translate_value(width)?;
                    }
                    entry.line.style = *style;
                }
                ProgramDisplayAttributeKind::Fill { opacity } => {
                    entry.display = true;
                    entry.fill.display = true;
                    if let Some(opacity) = opacity {
                        entry.fill.opacity = self.translate_value(opacity)?;
                    }
                }
                ProgramDisplayAttributeKind::Click { action } => {
                    entry.clickable.enabled = true;
                    entry.clickable.expression = self.translate_action_value(action)?;
                }
                ProgramDisplayAttributeKind::Description { text } => {
                    entry.clickable.description = text.to_string();
                }
                _ => panic!("given attribute is invalid for an expression: {attribute:?}")
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
            for display_element in &program_display.elements {
                self.add_display_element(display_element)?;
            }
        }

        Ok(())
    }
}
