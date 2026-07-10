use crate::ast::RangeKind;
use crate::desmos::{GraphBinaryKind, GraphEntry, GraphExpression, GraphExpressionEntry, GraphExpressionList, GraphFolderEntry, GraphImageEntry, GraphInequalityKind, GraphSlider, GraphSliderLoopMode, GraphTextEntry, GraphTicker, GraphUnaryKind};
use crate::desmos::builder::intrinsic::IntrinsicBuilder;
use crate::desmos::target::DesmosTargetInfo;
use crate::sema::{Program, ProgramAction, ProgramImmutable, ProgramPublicEntry, ProgramPublicLine, ProgramTicker, ProgramVariable, ProgramVariableKind};
use crate::sema::display::{ImageValue, ProgramDisplayAttributeKind, ProgramDisplayElement};
use crate::sema::values::{ActionValue, ActionValueKind, BinaryKind, ColorKind, DoubleReducerKind, IndexKind, InequalityKind, MathematicalConstant, ParameterizedReducerKind, ReducerKind, UnaryKind, Value, ValueKind};

mod intrinsic;

pub const INTRINSICS_FOLDER_ID: &str = "desmosify_intrinsics";
pub const IMMUTABLES_FOLDER_ID: &str = "desmosify_immutables";
pub const VARIABLES_FOLDER_ID: &str = "desmosify_variables";
pub const ACTIONS_FOLDER_ID: &str = "desmosify_actions";
pub const DISPLAY_FOLDER_ID: &str = "desmosify_display";

pub struct GraphExpressionListBuilder<'target> {
    target_info: &'target mut DesmosTargetInfo,
    ticker: Option<GraphTicker>,
    public_entries: Vec<Box<dyn GraphEntry>>,
    intrinsic_entries: Vec<Box<dyn GraphEntry>>,
    immutable_entries: Vec<Box<dyn GraphEntry>>,
    variable_entries: Vec<Box<dyn GraphEntry>>,
    action_entries: Vec<Box<dyn GraphEntry>>,
    display_entries: Vec<Box<dyn GraphEntry>>,
    next_dummy_noop_id: u64,
    dummy_unreachable_created: bool,
    intrinsics: IntrinsicBuilder,
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
            intrinsic_entries: Vec::new(),
            immutable_entries: Vec::new(),
            variable_entries: Vec::new(),
            action_entries: Vec::new(),
            display_entries: Vec::new(),
            next_dummy_noop_id: 0,
            dummy_unreachable_created: false,
            intrinsics: IntrinsicBuilder::new(
                Some(INTRINSICS_FOLDER_ID.into()),
                GraphExpression::Letter('I'),
            ),
        }
    }

    pub fn finish(mut self) -> GraphExpressionList {
        fn finish_folder(id: &str, title: &str, mut entries: Vec<Box<dyn GraphEntry>>) -> Vec<Box<dyn GraphEntry>> {
            if !entries.is_empty() {
                entries.insert(0, Box::new(GraphFolderEntry {
                    id: id.into(),
                    title: title.into(),
                    collapsed: true,
                    secret: false,
                }));
            }
            entries
        }

        self.intrinsic_entries.extend(self.intrinsics.finish());

        if !self.public_entries.is_empty() {
            self.public_entries.push(Box::new(GraphExpressionEntry {
                id: self.target_info.create_entry_id(),
                ..Default::default()
            }));
        }

        GraphExpressionList {
            ticker: self.ticker,
            entries: self.public_entries
                .into_iter()
                .chain(finish_folder(INTRINSICS_FOLDER_ID, "desmosify: intrinsics", self.intrinsic_entries))
                .chain(finish_folder(IMMUTABLES_FOLDER_ID, "desmosify: immutables", self.immutable_entries))
                .chain(finish_folder(VARIABLES_FOLDER_ID, "desmosify: variables", self.variable_entries))
                .chain(finish_folder(ACTIONS_FOLDER_ID, "desmosify: actions", self.action_entries))
                .chain(finish_folder(DISPLAY_FOLDER_ID, "desmosify: display", self.display_entries))
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

    pub fn translate_value(&mut self, value: &Value) -> crate::Result<GraphExpression> {
        let unsupported_error = || Box::new(crate::Error {
            kind: crate::ErrorKind::UnsupportedValue,
            span: value.span,
        });

        match &value.kind {
            ValueKind::Undefined(..) => {
                // Create undefined using the alternative branch of a piecewise. This is the best
                // way to generate it reliably for any type that I can think of.
                Ok(GraphExpression::Unary {
                    kind: GraphUnaryKind::Piecewise,
                    inner: Box::new(GraphExpression::Binary {
                        kind: GraphBinaryKind::Equal,
                        lhs: Box::new(GraphExpression::Integer(0)),
                        rhs: Box::new(GraphExpression::Integer(1)),
                    }),
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
            ValueKind::InequalityChain { lhs, chain, .. } => {
                Ok(GraphExpression::Unary {
                    kind: GraphUnaryKind::Piecewise,
                    inner: Box::new(GraphExpression::Sequence {
                        elements: Vec::from([
                            GraphExpression::InequalityChain {
                                lhs: Box::new(self.translate_value(lhs)?),
                                chain: chain
                                    .iter()
                                    .map(|(kind, rhs)| Ok((
                                        match kind {
                                            InequalityKind::LessThan => GraphInequalityKind::LessThan,
                                            InequalityKind::LessEqual => GraphInequalityKind::LessEqual,
                                            InequalityKind::GreaterThan => GraphInequalityKind::GreaterThan,
                                            InequalityKind::GreaterEqual => GraphInequalityKind::GreaterEqual,
                                        },
                                        self.translate_value(rhs)?,
                                    )))
                                    .collect::<crate::Result<_>>()?,
                            },
                            GraphExpression::Integer(0),
                        ]),
                    }),
                })
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
            ValueKind::Random { source, sample_count, .. } => {
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(GraphExpression::OperatorName("random".into())),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: source.as_deref()
                            .into_iter()
                            .chain(sample_count.as_deref())
                            .map(|argument| self.translate_value(argument))
                            .collect::<crate::Result<_>>()?,
                    }),
                })
            }
            ValueKind::SeededRandom { source, sample_count, seed, .. } => {
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(GraphExpression::OperatorName("random".into())),
                    rhs: Box::new(GraphExpression::Sequence {
                        elements: source.as_deref()
                            .into_iter()
                            .chain([sample_count.as_ref(), seed.as_ref()])
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
                        RangeKind::Inclusive => self.intrinsics.range_inclusive(self.target_info),
                        RangeKind::Exclusive => self.intrinsics.range_exclusive(self.target_info),
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
                            RangeKind::Inclusive => self.intrinsics.index_range_inclusive(self.target_info),
                            RangeKind::Exclusive => self.intrinsics.index_range_exclusive(self.target_info),
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
                        lhs: Box::new(self.intrinsics.index_range_from(self.target_info)),
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
                            RangeKind::Inclusive => self.intrinsics.index_range_inclusive(self.target_info),
                            RangeKind::Exclusive => self.intrinsics.index_range_exclusive(self.target_info),
                        }),
                        rhs: Box::new(GraphExpression::Sequence {
                            elements: Vec::from([
                                self.translate_value(list)?,
                                GraphExpression::Integer(1),
                                self.translate_value(to_index)?,
                                GraphExpression::Integer(1),
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
            ValueKind::InlineAction { parameters, action } => {
                let action = self.translate_action_value(action)?;
                let action_symbol = self.target_info.create_inline_action_symbol();

                let entry = Box::new(GraphExpressionEntry {
                    id: self.target_info.create_entry_id(),
                    folder_id: Some(ACTIONS_FOLDER_ID.into()),
                    expression: GraphExpression::Binary {
                        kind: GraphBinaryKind::Equal,
                        lhs: Box::new(if parameters.is_empty() {
                            action_symbol.clone()
                        } else {
                            GraphExpression::Binary {
                                kind: GraphBinaryKind::Call,
                                lhs: Box::new(action_symbol.clone()),
                                rhs: Box::new(GraphExpression::Sequence {
                                    elements: parameters
                                        .iter()
                                        .map(|parameter| self.target_info.get_local_symbol(parameter.id))
                                        .collect(),
                                }),
                            }
                        }),
                        rhs: Box::new(action),
                    },
                    ..Default::default()
                });
                self.action_entries.push(entry);

                Ok(action_symbol)
            }
            _ => Err(unsupported_error())
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
            UnaryKind::PrefixSum => {
                Ok(GraphExpression::Binary {
                    kind: GraphBinaryKind::Call,
                    lhs: Box::new(self.intrinsics.prefix_sum(self.target_info)),
                    rhs: Box::new(self.translate_value(operand)?),
                })
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
            BinaryKind::Rectangle => {
                Ok(GraphExpression::call(
                    self.intrinsics.rectangle(self.target_info),
                    [
                        self.translate_value(lhs)?,
                        self.translate_value(rhs)?,
                    ],
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
            ValueKind::InequalityChain { lhs, chain, .. } => {
                Ok(GraphExpression::InequalityChain {
                    lhs: Box::new(self.translate_value(lhs)?),
                    chain: chain
                        .iter()
                        .map(|(kind, rhs)| Ok((
                            match kind {
                                InequalityKind::LessThan => GraphInequalityKind::LessThan,
                                InequalityKind::LessEqual => GraphInequalityKind::LessEqual,
                                InequalityKind::GreaterThan => GraphInequalityKind::GreaterThan,
                                InequalityKind::GreaterEqual => GraphInequalityKind::GreaterEqual,
                            },
                            self.translate_value(rhs)?,
                        )))
                        .collect::<crate::Result<_>>()?,
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
                if arguments.is_empty() {
                    self.translate_value(action)
                }
                else {
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

    pub fn translate_program_immutable(&mut self, immutable: &ProgramImmutable, folder_id: Option<String>) -> crate::Result<Box<dyn GraphEntry>> {
        let value = self.translate_value(&immutable.value)?;

        Ok(Box::new(GraphExpressionEntry {
            id: self.target_info.create_entry_id(),
            folder_id,
            expression: GraphExpression::Binary {
                kind: GraphBinaryKind::Equal,
                lhs: Box::new(match &immutable.parameters {
                    Some(parameters) => GraphExpression::Binary {
                        kind: GraphBinaryKind::Call,
                        lhs: Box::new(self.target_info.get_global_symbol(&immutable.identifier)),
                        rhs: Box::new(GraphExpression::Sequence {
                            elements: parameters
                                .iter()
                                .map(|parameter| self.target_info.get_local_symbol(parameter.id))
                                .collect(),
                        }),
                    },
                    None => self.target_info.get_global_symbol(&immutable.identifier),
                }),
                rhs: Box::new(value),
            },
            ..Default::default()
        }))
    }

    pub fn add_program_immutable(&mut self, immutable: &ProgramImmutable) -> crate::Result<()> {
        let entry = self.translate_program_immutable(immutable, Some(IMMUTABLES_FOLDER_ID.into()))?;
        self.immutable_entries.push(entry);
        Ok(())
    }

    pub fn translate_program_variable(&mut self, variable: &ProgramVariable, folder_id: Option<String>) -> crate::Result<Box<dyn GraphEntry>> {
        let value = self.translate_value(&variable.value)?;

        let mut slider = match &variable.kind {
            ProgramVariableKind::Default => None,
            ProgramVariableKind::Timer => Some(GraphSlider {
                loop_mode: GraphSliderLoopMode::PlayIndefinitely,
                is_playing: true,
                ..Default::default()
            }),
            ProgramVariableKind::Slider { min, max, step } => Some(GraphSlider {
                min: min.as_ref().map_or(Ok(Default::default()), |min| {
                    self.translate_value(min)
                })?,
                max: max.as_ref().map_or(Ok(Default::default()), |max| {
                    self.translate_value(max)
                })?,
                step: step.as_ref().map_or(Ok(Default::default()), |step| {
                    self.translate_value(step)
                })?,
                ..Default::default()
            }),
        };

        if let Some((min, max, step)) = variable.value.get_type().value_range() {
            let slider = slider.get_or_insert_default();
            if let (Some(min), GraphExpression::Empty) = (&min, &slider.min) {
                slider.min = self.translate_value(min)?;
            }
            if let (Some(max), GraphExpression::Empty) = (&max, &slider.max) {
                slider.max = self.translate_value(max)?;
            }
            if let (Some(step), GraphExpression::Empty) = (&step, &slider.step) {
                slider.step = self.translate_value(step)?;
            }
        }

        Ok(Box::new(GraphExpressionEntry {
            id: self.target_info.create_entry_id(),
            folder_id,
            expression: GraphExpression::Binary {
                kind: GraphBinaryKind::Equal,
                lhs: Box::new(self.target_info.get_global_symbol(&variable.identifier)),
                rhs: Box::new(value),
            },
            slider,
            ..Default::default()
        }))
    }

    pub fn add_program_variable(&mut self, variable: &ProgramVariable) -> crate::Result<()> {
        let entry = self.translate_program_variable(variable, Some(VARIABLES_FOLDER_ID.into()))?;
        self.variable_entries.push(entry);
        Ok(())
    }

    pub fn translate_program_action(&mut self, program_action: &ProgramAction, folder_id: Option<String>) -> crate::Result<Box<dyn GraphEntry>> {
        let action = self.translate_action_value(&program_action.action)?;

        Ok(Box::new(GraphExpressionEntry {
            id: self.target_info.create_entry_id(),
            folder_id,
            expression: GraphExpression::Binary {
                kind: GraphBinaryKind::Equal,
                lhs: Box::new(if program_action.parameters.is_empty() {
                    self.target_info.get_action_symbol(&program_action.identifier)
                } else {
                    GraphExpression::Binary {
                        kind: GraphBinaryKind::Call,
                        lhs: Box::new(self.target_info.get_action_symbol(&program_action.identifier)),
                        rhs: Box::new(GraphExpression::Sequence {
                            elements: program_action.parameters
                                .iter()
                                .map(|parameter| self.target_info.get_local_symbol(parameter.id))
                                .collect(),
                        }),
                    }
                }),
                rhs: Box::new(action),
            },
            ..Default::default()
        }))
    }

    pub fn add_program_action(&mut self, program_action: &ProgramAction) -> crate::Result<()> {
        let entry = self.translate_program_action(program_action, Some(ACTIONS_FOLDER_ID.into()))?;

        self.action_entries.push(entry);

        Ok(())
    }

    pub fn set_program_ticker(&mut self, program_ticker: &ProgramTicker) -> crate::Result<()> {
        if program_ticker.tick_action.is_empty() {
            self.ticker = None;
        }
        else {
            self.ticker = Some(GraphTicker {
                playing: true,
                handler: self.translate_action_value(&program_ticker.tick_action)?,
                min_step: match &program_ticker.interval_ms {
                    Some(interval_ms) => self.translate_value(interval_ms)?,
                    None => GraphExpression::Empty,
                },
            });
        }

        Ok(())
    }

    pub fn add_public_line(&mut self, public_line: &ProgramPublicLine, folder_id: Option<String>) -> crate::Result<()> {
        let id = self.target_info.create_entry_id();
        let entry: Box<dyn GraphEntry> = match public_line {
            ProgramPublicLine::Expression(value) => match &value.kind {
                ValueKind::Str(text) => {
                    let text = text.trim();
                    if text.is_empty() {
                        Box::new(GraphExpressionEntry {
                            id,
                            folder_id,
                            ..Default::default()
                        })
                    }
                    else {
                        Box::new(GraphTextEntry {
                            id,
                            folder_id,
                            text: text.to_string(),
                        })
                    }
                }
                _ => {
                    Box::new(GraphExpressionEntry {
                        id,
                        folder_id,
                        expression: self.translate_value(value)?,
                        ..Default::default()
                    })
                }
            }
            ProgramPublicLine::Action(action) => {
                Box::new(GraphExpressionEntry {
                    id,
                    folder_id,
                    expression: self.translate_action_value(action)?,
                    ..Default::default()
                })
            }
            ProgramPublicLine::Variable(variable) => {
                self.translate_program_variable(variable, folder_id)?
            }
        };
        self.public_entries.push(entry);

        Ok(())
    }

    pub fn add_public_entry(&mut self, public_entry: &ProgramPublicEntry) -> crate::Result<()> {
        match public_entry {
            ProgramPublicEntry::Line(public_line) => {
                self.add_public_line(public_line, None)
            }
            ProgramPublicEntry::Folder { label, lines } => {
                let folder_id = self.target_info.create_entry_id();
                let folder_entry = Box::new(GraphFolderEntry {
                    id: folder_id.clone(),
                    title: label.to_string(),
                    collapsed: true,
                    secret: false,
                });
                self.public_entries.push(folder_entry);

                for line in lines {
                    self.add_public_line(line, Some(folder_id.clone()))?;
                }

                Ok(())
            }
        }
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
        for immutable in &program.immutables {
            self.add_program_immutable(immutable)?;
        }
        for variable in &program.variables {
            self.add_program_variable(variable)?;
        }
        for action in &program.actions {
            self.add_program_action(action)?;
        }
        self.set_program_ticker(&program.ticker)?;
        for public_entry in &program.public.entries {
            self.add_public_entry(public_entry)?;
        }
        for display_element in &program.display.elements {
            self.add_display_element(display_element)?;
        }

        Ok(())
    }
}
