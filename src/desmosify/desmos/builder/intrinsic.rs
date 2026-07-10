use crate::desmos::{GraphBinaryKind, GraphEntry, GraphExpression, GraphExpressionEntry, GraphInequalityKind, GraphUnaryKind};
use crate::desmos::target::DesmosTargetInfo;

macro_rules! intrinsic_builder_definition {
    ($($intrinsic:ident),* $(,)?) => {
        pub struct IntrinsicBuilder {
            folder_id: Option<String>,
            prefix: GraphExpression,
            $($intrinsic: Box<[Box<dyn GraphEntry>]>,)*
        }

        impl IntrinsicBuilder {
            pub fn new(folder_id: Option<String>, prefix: GraphExpression) -> Self {
                Self {
                    folder_id,
                    prefix,
                    $($intrinsic: Default::default()),*
                }
            }

            pub fn finish(self) -> impl Iterator<Item = Box<dyn GraphEntry>> {
                [].into_iter()
                    $(.chain(self.$intrinsic))*
            }
        }
    };
}

intrinsic_builder_definition! {
    range_inclusive,
    range_exclusive,
    index_range_inclusive,
    index_range_exclusive,
    index_range_from,
    rectangle,
    prefix_sum,
}

impl IntrinsicBuilder {
    pub fn get_symbol(&self, subscript: impl Into<String>) -> GraphExpression {
        GraphExpression::Binary {
            kind: GraphBinaryKind::Subscript,
            lhs: Box::new(self.prefix.clone()),
            rhs: Box::new(GraphExpression::Alphanumeric(subscript.into())),
        }
    }

    pub fn create_expression_entry(&mut self, info: &mut DesmosTargetInfo, expression: GraphExpression) -> Box<dyn GraphEntry> {
        Box::new(GraphExpressionEntry {
            id: info.create_entry_id(),
            folder_id: self.folder_id.clone(),
            expression,
            ..Default::default()
        })
    }

    pub fn range_inclusive(&mut self, info: &mut DesmosTargetInfo) -> GraphExpression {
        let symbol = self.get_symbol("RangeInc");

        if self.range_inclusive.is_empty() {
            let local_a = info.create_local_symbol();
            let local_b = info.create_local_symbol();
            let local_s = info.create_local_symbol();

            // range_inc(a, b, s) = {
            //     a sign(s) > b sign(s): [],
            //     a + s * [0 ... floor((b - a) / s)]
            // }
            // This is terrible.
            let expression = GraphExpression::Binary {
                kind: GraphBinaryKind::Equal,
                lhs: Box::new(GraphExpression::call(
                    symbol.clone(),
                    [local_a.clone(), local_b.clone(), local_s.clone()],
                )),
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
                                    chain: Vec::from([(
                                        GraphInequalityKind::GreaterThan,
                                        GraphExpression::Binary {
                                            kind: GraphBinaryKind::ImplicitMultiply,
                                            lhs: Box::new(local_b.clone()),
                                            rhs: Box::new(GraphExpression::Binary {
                                                kind: GraphBinaryKind::Call,
                                                lhs: Box::new(GraphExpression::OperatorName("sign".into())),
                                                rhs: Box::new(local_s.clone()),
                                            }),
                                        },
                                    )]),
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

            self.range_inclusive = [
                self.create_expression_entry(info, expression),
            ].into();
        }

        symbol
    }

    pub fn range_exclusive(&mut self, info: &mut DesmosTargetInfo) -> GraphExpression {
        let symbol = self.get_symbol("RangeExc");

        if self.range_exclusive.is_empty() {
            let local_a = info.create_local_symbol();
            let local_b = info.create_local_symbol();
            let local_s = info.create_local_symbol();
            let local_x = info.create_local_symbol();

            // range_exc(a, b, s) = x[{x = b, 0} = 0] with x = range_inc(a, b, s)
            // This is also terrible, but not nearly as bad.
            let expression = GraphExpression::Binary {
                kind: GraphBinaryKind::Equal,
                lhs: Box::new(GraphExpression::call(
                    symbol.clone(),
                    [local_a.clone(), local_b.clone(), local_s.clone()],
                )),
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
                            lhs: Box::new(self.range_inclusive(info)),
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

            self.range_exclusive = [
                self.create_expression_entry(info, expression),
            ].into();
        }

        symbol
    }

    pub fn index_range_inclusive(&mut self, info: &mut DesmosTargetInfo) -> GraphExpression {
        let symbol = self.get_symbol("IdxRangeInc");

        if self.index_range_inclusive.is_empty() {
            let local_l = info.create_local_symbol();
            let local_a = info.create_local_symbol();
            let local_b = info.create_local_symbol();
            let local_s = info.create_local_symbol();
            let local_i = info.create_local_symbol();

            // idx_range_inc(l, a, b, s) = [l[i] for i = range_inc(a, b, s)]
            let expression = GraphExpression::Binary {
                kind: GraphBinaryKind::Equal,
                lhs: Box::new(GraphExpression::call(
                    symbol.clone(),
                    [local_l.clone(), local_a.clone(), local_b.clone(), local_s.clone()],
                )),
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
                                lhs: Box::new(self.range_inclusive(info)),
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

            self.index_range_inclusive = [
                self.create_expression_entry(info, expression),
            ].into();
        }

        symbol
    }

    pub fn index_range_exclusive(&mut self, info: &mut DesmosTargetInfo) -> GraphExpression {
        let symbol = self.get_symbol("IdxRangeExc");

        if self.index_range_exclusive.is_empty() {
            let local_l = info.create_local_symbol();
            let local_a = info.create_local_symbol();
            let local_b = info.create_local_symbol();
            let local_s = info.create_local_symbol();
            let local_i = info.create_local_symbol();

            // idx_range_exc(l, a, b, s) = [l[i] for i = range_exc(a, b, s)]
            let expression = GraphExpression::Binary {
                kind: GraphBinaryKind::Equal,
                lhs: Box::new(GraphExpression::call(
                    symbol.clone(),
                    [local_l.clone(), local_a.clone(), local_b.clone(), local_s.clone()],
                )),
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
                                lhs: Box::new(self.range_exclusive(info)),
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

            self.index_range_exclusive = [
                self.create_expression_entry(info, expression),
            ].into();
        }

        symbol
    }

    pub fn index_range_from(&mut self, info: &mut DesmosTargetInfo) -> GraphExpression {
        let symbol = self.get_symbol("IdxRangeFrom");

        if self.index_range_from.is_empty() {
            let local_l = info.create_local_symbol();
            let local_a = info.create_local_symbol();
            let local_s = info.create_local_symbol();

            // idx_range_from(l, a, s) = idx_range_inc(l, a, l.count, s)
            let expression = GraphExpression::Binary {
                kind: GraphBinaryKind::Equal,
                lhs: Box::new(GraphExpression::call(
                    symbol.clone(),
                    [local_l.clone(), local_a.clone(), local_s.clone()],
                )),
                rhs: Box::new(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(self.index_range_inclusive(info)),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: Vec::from([
                            local_l.clone(),
                            local_a.clone(),
                            GraphExpression::Binary {
                                kind: GraphBinaryKind::Dot,
                                lhs: Box::new(local_l.clone()),
                                rhs: Box::new(GraphExpression::OperatorName("count".into())),
                            },
                            local_s.clone(),
                        ]),
                    }),
                }),
            };

            self.index_range_from = [
                self.create_expression_entry(info, expression),
            ].into();
        }

        symbol
    }

    pub fn rectangle(&mut self, info: &mut DesmosTargetInfo) -> GraphExpression {
        let symbol = self.get_symbol("Rect");

        if self.rectangle.is_empty() {
            let local_p1 = info.create_local_symbol();
            let local_p2 = info.create_local_symbol();

            // rect(p1, p2) = polygon(p1, (p2.x, p1.y), p2, (p1.x, p2.y))
            let expression = GraphExpression::Binary {
                kind: GraphBinaryKind::Equal,
                lhs: Box::new(GraphExpression::call(
                    symbol.clone(),
                    [local_p1.clone(), local_p2.clone()],
                )),
                rhs: Box::new(GraphExpression::call(
                    GraphExpression::OperatorName("polygon".into()),
                    [
                        local_p1.clone(),
                        GraphExpression::wrap_sequence(GraphUnaryKind::Parentheses, [
                            GraphExpression::Binary {
                                kind: GraphBinaryKind::Dot,
                                lhs: Box::new(local_p2.clone()),
                                rhs: Box::new(GraphExpression::Letter('x')),
                            },
                            GraphExpression::Binary {
                                kind: GraphBinaryKind::Dot,
                                lhs: Box::new(local_p1.clone()),
                                rhs: Box::new(GraphExpression::Letter('y')),
                            },
                        ]),
                        local_p2.clone(),
                        GraphExpression::wrap_sequence(GraphUnaryKind::Parentheses, [
                            GraphExpression::Binary {
                                kind: GraphBinaryKind::Dot,
                                lhs: Box::new(local_p1.clone()),
                                rhs: Box::new(GraphExpression::Letter('x')),
                            },
                            GraphExpression::Binary {
                                kind: GraphBinaryKind::Dot,
                                lhs: Box::new(local_p2.clone()),
                                rhs: Box::new(GraphExpression::Letter('y')),
                            },
                        ]),
                    ],
                )),
            };

            self.rectangle = [
                self.create_expression_entry(info, expression),
            ].into();
        }

        symbol
    }

    /// Nevin Brackett-Rozinsky O(n) Prefix Sum (Wackscope Algorithm)
    ///
    /// https://www.desmos.com/calculator/p091kr6k84
    pub fn prefix_sum(&mut self, info: &mut DesmosTargetInfo) -> GraphExpression {
        let symbol = self.get_symbol("PrefixSumNW");

        if self.prefix_sum.is_empty() {
            let helper_symbol = self.get_symbol("PrefixSumNWHelper");
            let local_list = info.create_local_symbol();
            let local_index = info.create_local_symbol();
            let local_wackscope_list = info.create_local_symbol();

            // prefix_sum_nw(list) = {
            //     list.count <= 1: list,
            //     prefix_sum_nw_helper([1 ... list.count]) with wackscope_list = list
            // }
            let expression = GraphExpression::Binary {
                kind: GraphBinaryKind::Equal,
                lhs: Box::new(GraphExpression::call(
                    symbol.clone(),
                    [local_list.clone()],
                )),
                rhs: Box::new(GraphExpression::wrap_sequence(GraphUnaryKind::Piecewise, [
                    GraphExpression::Binary {
                        kind: GraphBinaryKind::Colon,
                        lhs: Box::new(GraphExpression::InequalityChain {
                            lhs: Box::new(GraphExpression::Binary {
                                kind: GraphBinaryKind::Dot,
                                lhs: Box::new(local_list.clone()),
                                rhs: Box::new(GraphExpression::OperatorName("count".into())),
                            }),
                            chain: Vec::from([(
                                GraphInequalityKind::LessEqual,
                                GraphExpression::Integer(1),
                            )]),
                        }),
                        rhs: Box::new(local_list.clone()),
                    },
                    GraphExpression::Binary {
                        kind: GraphBinaryKind::With,
                        lhs: Box::new(GraphExpression::call(
                            helper_symbol.clone(),
                            [GraphExpression::Unary {
                                kind: GraphUnaryKind::List,
                                inner: Box::new(GraphExpression::Binary {
                                    kind: GraphBinaryKind::Range,
                                    lhs: Box::new(GraphExpression::Integer(1)),
                                    rhs: Box::new(GraphExpression::Binary {
                                        kind: GraphBinaryKind::Dot,
                                        lhs: Box::new(local_list.clone()),
                                        rhs: Box::new(GraphExpression::OperatorName("count".into())),
                                    }),
                                }),
                            }],
                        )),
                        rhs: Box::new(GraphExpression::Binary {
                            kind: GraphBinaryKind::Equal,
                            lhs: Box::new(local_wackscope_list.clone()),
                            rhs: Box::new(local_list.clone()),
                        }),
                    },
                ])),
            };

            // prefix_sum_nw_helper(index) = {
            //     index = 1: wackscope_list[1],
            //     prefix_sum_nw_helper(index - 1) + wackscope_list[index]
            // }
            let helper_expression = GraphExpression::Binary {
                kind: GraphBinaryKind::Equal,
                lhs: Box::new(GraphExpression::call(
                    helper_symbol.clone(),
                    [local_index.clone()],
                )),
                rhs: Box::new(GraphExpression::wrap_sequence(GraphUnaryKind::Piecewise, [
                    GraphExpression::Binary {
                        kind: GraphBinaryKind::Colon,
                        lhs: Box::new(GraphExpression::Binary {
                            kind: GraphBinaryKind::Equal,
                            lhs: Box::new(local_index.clone()),
                            rhs: Box::new(GraphExpression::Integer(1)),
                        }),
                        rhs: Box::new(GraphExpression::Binary {
                            kind: GraphBinaryKind::Index,
                            lhs: Box::new(local_wackscope_list.clone()),
                            rhs: Box::new(GraphExpression::Integer(1)),
                        }),
                    },
                    GraphExpression::Binary {
                        kind: GraphBinaryKind::Add,
                        lhs: Box::new(GraphExpression::call(
                            helper_symbol.clone(),
                            [GraphExpression::Binary {
                                kind: GraphBinaryKind::Subtract,
                                lhs: Box::new(local_index.clone()),
                                rhs: Box::new(GraphExpression::Integer(1)),
                            }],
                        )),
                        rhs: Box::new(GraphExpression::Binary {
                            kind: GraphBinaryKind::Index,
                            lhs: Box::new(local_wackscope_list.clone()),
                            rhs: Box::new(local_index.clone()),
                        }),
                    },
                ])),
            };

            self.prefix_sum = [
                self.create_expression_entry(info, helper_expression),
                self.create_expression_entry(info, expression),
            ].into();
        }

        symbol
    }
}
