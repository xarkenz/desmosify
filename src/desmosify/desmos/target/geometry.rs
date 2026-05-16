use crate::desmos::*;
use json::JsonValue;

pub struct GeometryTarget;

impl GeometryTarget {
    pub const VALID_COMMANDS: &'static [&'static str] = &[
        // Trig Functions
        "sin",
        "cos",
        "tan",
        "csc",
        "sec",
        "cot",
        // Inverse Trig Functions
        "arcsin",
        "arccos",
        "arctan",
        "arccsc",
        "arcsec",
        "arccot",
        // Statistics
        "mean",
        "median",
        "min",
        "max",
        "quartile",
        "quantile",
        "stdev",
        "stdevp",
        "var",
        "mad",
        "cov",
        "covp",
        "corr",
        "spearman",
        "stats",
        "count",
        "total",
        // List Operations
        "join",
        "sort",
        "shuffle",
        "unique",
        // Visualizations
        "histogram",
        "dotplot",
        "boxplot",
        // Distributions
        "normaldist",
        "tdist",
        "poissondist",
        "binomialdist",
        "uniformdist",
        "pdf",
        "cdf",
        "inversecdf",
        "random",
        // Statistical Tests
        "ttest",
        "tscore",
        "ittest",
        // Calculus
        "exp",
        "ln",
        "log",
        "log_base",
        "derivative",
        "integral",
        "sum",
        "product",
        // Hyperbolic Trig Functions
        "sinh",
        "cosh",
        "tanh",
        "csch",
        "sech",
        "coth",
        // Geometry Tools
        "midpoint",
        "intersection",
        "segment",
        "line",
        "ray",
        "vector",
        "parallel",
        "perpendicular",
        "circle",
        "arc",
        "angle",
        "directedangle",
        "polygon",
        "glider",
        // Properties & Measurements
        "distance",
        "length",
        "area",
        "perimeter",
        "vertices",
        "angles",
        "directedangles",
        "segments",
        "radius",
        "center",
        "coterminal",
        "supplement",
        "start",
        "end",
        // Transformations
        "dilate",
        "rotate",
        "reflect",
        "translate",
        // Custom Colors
        "rgb",
        "hsv",
        // Sound (Beta)
        "tone",
        // Number Theory
        "lcm",
        "gcd",
        "mod",
        "ceil",
        "floor",
        "round",
        "sign",
        "sqrt",
        "cbrt",
        "nthroot",
        "nPr",
        "nCr",
    ];

    pub fn translate_constant_value(&self, value: &ConstantValue) -> Box<GraphExpression> {
        Box::new(match value {
            ConstantValue::Real(value) => GraphExpression::Decimal(*value),
            ConstantValue::Int(value) => GraphExpression::Decimal(*value as f64),
            ConstantValue::Bool(value) => GraphExpression::Decimal(if *value { 1.0 } else { 0.0 }),
            ConstantValue::Point(x_value, y_value) => GraphExpression::Parentheses {
                body: Box::new(GraphExpression::Sequence {
                    elements: vec![
                        GraphExpression::Decimal(*x_value),
                        GraphExpression::Decimal(*y_value),
                    ]
                })
            },
            ConstantValue::IPoint(x_value, y_value) => GraphExpression::Parentheses {
                body: Box::new(GraphExpression::Sequence {
                    elements: vec![
                        GraphExpression::Decimal(*x_value as f64),
                        GraphExpression::Decimal(*y_value as f64),
                    ]
                })
            },
            ConstantValue::Color(color) => match color {
                &crate::Color::Rgb { red, green, blue } => GraphExpression::Call {
                    callee: Box::new(GraphExpression::OperatorName("rgb".into())),
                    arguments: Box::new(GraphExpression::Sequence {
                        elements: vec![
                            GraphExpression::Decimal(red),
                            GraphExpression::Decimal(green),
                            GraphExpression::Decimal(blue),
                        ]
                    })
                },
                &crate::Color::Hsv { hue, saturation, value } => GraphExpression::Call {
                    callee: Box::new(GraphExpression::OperatorName("hsv".into())),
                    arguments: Box::new(GraphExpression::Sequence {
                        elements: vec![
                            GraphExpression::Decimal(hue),
                            GraphExpression::Decimal(saturation),
                            GraphExpression::Decimal(value),
                        ]
                    })
                },
            },
            ConstantValue::Polygon(points) => GraphExpression::Call {
                callee: Box::new(GraphExpression::OperatorName("polygon".into())),
                arguments: Box::new(GraphExpression::Sequence {
                    elements: Vec::from_iter(points.iter().map(|&(x_value, y_value)| {
                        GraphExpression::Parentheses {
                            body: Box::new(GraphExpression::Sequence(vec![
                                GraphExpression::Decimal(x_value),
                                GraphExpression::Decimal(y_value),
                            ]))
                        }
                    }))
                })
            },
            ConstantValue::Segment((x1_value, y1_value), (x2_value, y2_value)) => GraphExpression::Call {
                callee: Box::new(GraphExpression::OperatorName("segment".into())),
                arguments: Box::new(GraphExpression::Sequence {
                    elements: vec![
                        GraphExpression::Parentheses {
                            body: Box::new(GraphExpression::Sequence(vec![
                                GraphExpression::Decimal(*x1_value),
                                GraphExpression::Decimal(*y1_value),
                            ]))
                        },
                        GraphExpression::Parentheses {
                            body: Box::new(GraphExpression::Sequence(vec![
                                GraphExpression::Decimal(*x2_value),
                                GraphExpression::Decimal(*y2_value),
                            ]))
                        },
                    ]
                })
            },
            ConstantValue::Str(content) => GraphExpression::Alphanumeric(content.clone()),
            ConstantValue::List { items, .. } => GraphExpression::List {
                body: Box::new(GraphExpression::Sequence {
                    elements: items
                        .iter()
                        .map(|value| *self.translate_constant_value(value))
                        .collect()
                })
            },
            ConstantValue::EnumVariant { .. } => todo!(),
        })
    }

    pub fn translate_name(&self, name: &str) -> Box<GraphExpression> {
        Box::new(GraphExpression::Subscript {
            base: Box::new(GraphExpression::Letter('X')),
            script: Box::new(GraphExpression::Alphanumeric(name.chars().filter(|&ch| ch != '_').collect()))
        })
    }

    pub fn translate_operator(&self, operation: crate::Operation, operands: &[GraphExpression]) -> Box<GraphExpression> {
        let raw_operands = operands;
        let mut operands = Vec::from_iter(raw_operands.iter()
            .rev()
            .map(|operand| self.translate_expression(operand)));

        Box::new(match operation {
            crate::Operation::PointLiteral => GraphExpression::Parentheses {
                body: Box::new(GraphExpression::Sequence { elements: operands.into_iter().map(|component| *component).collect() })
            },
            crate::Operation::ListLiteral => GraphExpression::List {
                body: Box::new(GraphExpression::Sequence { elements: operands.into_iter().map(|item| *item).collect() })
            },
            crate::Operation::ListFill => GraphExpression::List {
                body: Box::new(GraphExpression::For {
                    lhs: operands.pop().unwrap(),
                    rhs: Box::new(GraphExpression::Equality {
                        lhs: Box::new(GraphExpression::Letter('x')),
                        rhs: Box::new(GraphExpression::SquareBrackets(
                            Box::new(GraphExpression::Range {
                                start: Box::new(GraphExpression::Decimal(1.0)),
                                end: Some(operands.pop().unwrap())
                            }),
                        ))
                    })
                })
            },
            crate::Operation::ListMap => GraphExpression::List {
                body: Box::new(GraphExpression::For {
                    lhs: match *operands.pop().unwrap() {
                        GraphExpression::SquareBrackets(content) => match content.as_ref() {
                            GraphExpression::For(_, _) => content,
                            _ => Box::new(GraphExpression::SquareBrackets(content))
                        },
                        operand => Box::new(operand)
                    },
                    rhs: Box::new(GraphExpression::Equality {
                        lhs: operands.pop().unwrap(),
                        rhs: operands.pop().unwrap()
                    })
                })
            },
            crate::Operation::ListFilter => todo!(),
            crate::Operation::MemberAccess => todo!(),
            crate::Operation::BuiltIn => GraphExpression::OperatorName(
                match &raw_operands[0].value {
                    ExpressionValue::Name(name) => name.clone(),
                    _ => panic!()
                },
            ),
            crate::Operation::Call => GraphExpression::Call {
                callee: operands.pop().unwrap(),
                arguments: Box::new(GraphExpression::Sequence { elements: operands.into_iter().map(|argument| *argument).collect() })
            },
            crate::Operation::ActionCall => GraphExpression::Call {
                callee: operands.pop().unwrap(),
                arguments: Box::new(GraphExpression::Sequence { elements: operands.into_iter().map(|argument| *argument).collect() })
            },
            crate::Operation::Index => GraphExpression::Parentheses {
                body: Box::new(GraphExpression::Index {
                    indexee: operands.pop().unwrap(),
                    index: operands.pop().unwrap()
                })
            },
            crate::Operation::Posate => GraphExpression::Parentheses { body: Box::new(GraphExpression::Positive { operand: operands.pop().unwrap() }) },
            crate::Operation::Negate => GraphExpression::Parentheses { body: Box::new(GraphExpression::Negative { operand: operands.pop().unwrap() }) },
            crate::Operation::Not => GraphExpression::Piecewise {
                body: Box::new(GraphExpression::Sequence {
                    elements: vec![
                        GraphExpression::Equality {
                            lhs: operands.pop().unwrap(),
                            rhs: Box::new(GraphExpression::Decimal(0.0))
                        },
                        GraphExpression::Decimal(0.0),
                    ]
                })
            },
            crate::Operation::Exponent => GraphExpression::Superscript {
                base: operands.pop().unwrap(),
                script: operands.pop().unwrap()
            },
            crate::Operation::Multiply => GraphExpression::Parentheses {
                body: Box::new(GraphExpression::Multiply {
                    lhs: operands.pop().unwrap(),
                    rhs: operands.pop().unwrap()
                })
            },
            crate::Operation::Divide => GraphExpression::Fraction {
                numerator: operands.pop().unwrap(),
                denominator: operands.pop().unwrap()
            },
            crate::Operation::Modulus => GraphExpression::Call {
                callee: Box::new(GraphExpression::OperatorName("mod".into())),
                arguments: Box::new(GraphExpression::Sequence {
                    elements: vec![
                        *operands.pop().unwrap(),
                        *operands.pop().unwrap(),
                    ]
                })
            },
            crate::Operation::Add => GraphExpression::Parentheses {
                body: Box::new(GraphExpression::Add {
                    lhs: operands.pop().unwrap(),
                    rhs: operands.pop().unwrap()
                })
            },
            crate::Operation::Subtract => GraphExpression::Parentheses {
                body: Box::new(GraphExpression::Subtract {
                    lhs: operands.pop().unwrap(),
                    rhs: operands.pop().unwrap()
                })
            },
            crate::Operation::LessThan => GraphExpression::Piecewise {
                body: Box::new(GraphExpression::Sequence {
                    elements: vec![
                        GraphExpression::InequalityChain {
                            lhs: operands.pop().unwrap(),
                            first_kind: GraphInequalityKind::LessThan,
                            rhs: operands.pop().unwrap(),
                            chain: Vec::new()
                        },
                        GraphExpression::Decimal(0.0),
                    ]
                })
            },
            crate::Operation::GreaterThan => GraphExpression::Piecewise {
                body: Box::new(GraphExpression::Sequence {
                    elements: vec![
                        GraphExpression::InequalityChain {
                            lhs: operands.pop().unwrap(),
                            first_kind: GraphInequalityKind::GreaterThan,
                            rhs: operands.pop().unwrap(),
                            chain: Vec::new()
                        },
                        GraphExpression::Decimal(0.0),
                    ]
                })
            },
            crate::Operation::LessEqual => GraphExpression::Piecewise {
                body: Box::new(GraphExpression::Sequence {
                    elements: vec![
                        GraphExpression::InequalityChain {
                            lhs: operands.pop().unwrap(),
                            first_kind: GraphInequalityKind::LessEqual,
                            rhs: operands.pop().unwrap(),
                            chain: Vec::new()
                        },
                        GraphExpression::Decimal(0.0),
                    ]
                })
            },
            crate::Operation::GreaterEqual => GraphExpression::Piecewise {
                body: Box::new(GraphExpression::Sequence {
                    elements: vec![
                        GraphExpression::InequalityChain {
                            lhs: operands.pop().unwrap(),
                            first_kind: GraphInequalityKind::GreaterEqual,
                            rhs: operands.pop().unwrap(),
                            chain: Vec::new()
                        },
                        GraphExpression::Decimal(0.0),
                    ]
                })
            },
            crate::Operation::Equal => GraphExpression::Piecewise {
                body: Box::new(GraphExpression::Sequence {
                    elements: vec![
                        GraphExpression::Equality {
                            lhs: operands.pop().unwrap(),
                            rhs: operands.pop().unwrap()
                        },
                        GraphExpression::Decimal(0.0),
                    ]
                })
            },
            // Desmos doesn't have != built-in, so we have to negate ==
            crate::Operation::NotEqual => GraphExpression::Piecewise {
                body: Box::new(GraphExpression::Sequence {
                    elements: vec![
                        GraphExpression::Colon {
                            lhs: Box::new(GraphExpression::Equality {
                                lhs: operands.pop().unwrap(),
                                rhs: operands.pop().unwrap()
                            }),
                            rhs: Box::new(GraphExpression::Decimal(0.0))
                        },
                        GraphExpression::Decimal(1.0),
                    ]
                })
            },
            crate::Operation::And => GraphExpression::Piecewise {
                body: Box::new(GraphExpression::Sequence {
                    elements: vec![
                        GraphExpression::Colon {
                            lhs: Box::new(GraphExpression::Equality {
                                lhs: operands.pop().unwrap(),
                                rhs: Box::new(GraphExpression::Decimal(0.0))
                            }),
                            rhs: Box::new(GraphExpression::Decimal(0.0))
                        },
                        *operands.pop().unwrap(),
                    ]
                })
            },
            crate::Operation::Or => GraphExpression::Piecewise {
                body: Box::new(GraphExpression::Sequence {
                    elements: vec![
                        GraphExpression::Colon {
                            lhs: Box::new(GraphExpression::Equality {
                                lhs: operands.pop().unwrap(),
                                rhs: Box::new(GraphExpression::Decimal(1.0))
                            }),
                            rhs: Box::new(GraphExpression::Decimal(1.0))
                        },
                        *operands.pop().unwrap(),
                    ]
                })
            },
            crate::Operation::ExclusiveRange => GraphExpression::List {
                body: Box::new(GraphExpression::Range {
                    start: operands.pop().unwrap(),
                    end: operands.pop().map(|operand| Box::new(GraphExpression::Parentheses {
                        body: Box::new(GraphExpression::Subtract {
                            lhs: operand,
                            rhs: Box::new(GraphExpression::Decimal(1.0))
                        })
                    }))
                })
            },
            crate::Operation::InclusiveRange => GraphExpression::List {
                body: Box::new(GraphExpression::Range {
                    start: operands.pop().unwrap(),
                    end: operands.pop()
                })
            },
            crate::Operation::Conditional => {
                let mut branches = Vec::new();
                while operands.len() > 1 {
                    branches.push(GraphExpression::Colon {
                        lhs: Box::new(GraphExpression::Equality {
                            lhs: operands.pop().unwrap(),
                            rhs: Box::new(GraphExpression::Decimal(1.0))
                        }),
                        rhs: operands.pop().unwrap()
                    });
                }
                if let Some(operand) = operands.pop() {
                    branches.push(*operand);
                }
                GraphExpression::Piecewise { body: Box::new(GraphExpression::Sequence { elements: branches }) }
            },
            crate::Operation::Assignment => todo!(),
            crate::Operation::Update => GraphExpression::RightArrow {
                lhs: operands.pop().unwrap(),
                rhs: operands.pop().unwrap()
            },
            crate::Operation::With => todo!(),
        })
    }

    pub fn translate_expression(&self, expression: &GraphExpression) -> Box<GraphExpression> {
        match &expression.value {
            ExpressionValue::Literal(value) => self.translate_constant_value(value),
            ExpressionValue::Name(name) => self.translate_name(name),
            ExpressionValue::Operator(operation, operands) => self.translate_operator(*operation, operands),
        }
    }

    pub fn translate_action(&self, action: &Action) -> Box<GraphExpression> {
        match action {
            Action::Block(subactions) => Box::new(GraphExpression::Parentheses { body: Box::new(GraphExpression::Sequence { elements: subactions.iter().map(|subaction| *self.translate_action(subaction)).collect() }) }),
            Action::Update(target, value) => Box::new(GraphExpression::RightArrow {
                lhs: self.translate_expression(target),
                rhs: self.translate_expression(value)
            }),
            Action::Call(name, arguments) => Box::new(GraphExpression::Call {
                callee: self.translate_expression(name),
                arguments: Box::new(GraphExpression::Sequence { elements: arguments.iter().map(|argument| *self.translate_expression(argument)).collect() })
            }),
            Action::Conditional(branches, default_branch) => {
                let mut piecewise_branches = Vec::from_iter(branches.iter().map(|(condition, consequent)| {
                    GraphExpression::Colon {
                        lhs: Box::new(GraphExpression::Equality {
                            lhs: self.translate_expression(condition),
                            rhs: Box::new(GraphExpression::Decimal(1.0))
                        }),
                        rhs: self.translate_action(consequent)
                    }
                }));
                if let Some(default_branch) = default_branch {
                    piecewise_branches.push(*self.translate_action(default_branch));
                }
                else {
                    piecewise_branches.push(GraphExpression::Subscript {
                        base: Box::new(GraphExpression::OperatorName("delta".into())),
                        script: Box::new(GraphExpression::Alphanumeric("noaction".into()))
                    });
                }

                Box::new(GraphExpression::Piecewise { body: Box::new(GraphExpression::Sequence { elements: piecewise_branches }) })
            },
        }
    }
}

impl crate::target::Target for GeometryTarget {
    type Output = JsonValue;

    fn name(&self) -> &'static str {
        "desmos-geometry"
    }

    fn compile(&self, definitions: &Definitions, signatures: &Signatures) -> Self::Output {
        let mut state = GraphState {
            version: 11,
            graph: GraphSettings {
                product_name: "geometry-calculator".into()
            },
            expressions: GraphExpressionList {
                entries: Vec::new(),
                ticker: None,
            },
        };
        let mut next_id: usize = 0;
        let mut get_next_id = || {
            let id = next_id.to_string();
            next_id += 1;
            id
        };

        state.expressions.entries.push(Box::new(GraphFolderEntry {
            id: "**dcg_geo_folder**".into(),
            title: "geometry".into(),
            collapsed: true,
            secret: true,
        }));

        if let Some(public) = &definitions.public {
            for expression in public {
                let entry: Box<dyn GraphEntry> = match *self.translate_expression(expression) {
                    GraphExpression::Alphanumeric(content) => {
                        Box::new(GraphTextEntry {
                            id: get_next_id(),
                            folder_id: None,
                            text: content.into(),
                        })
                    },
                    content => {
                        Box::new(GraphExpressionEntry {
                            id: get_next_id(),
                            folder_id: None,
                            expression: Some(Box::new(content)),
                            hidden: false,
                        })
                    },
                };

                state.expressions.entries.push(entry);
            }
        }

        state.expressions.entries.push(Box::new(GraphFolderEntry {
            id: "desmosify:actions".into(),
            title: "Actions".into(),
            collapsed: true,
            secret: false,
        }));

        for (name, action) in &definitions.actions {
            let signature = signatures.user_defined.get(name).unwrap();

            state.expressions.entries.push(Box::new(GraphExpressionEntry {
                id: get_next_id(),
                folder_id: Some("desmosify:actions".into()),
                expression: Some(Box::new(GraphExpression::Equality {
                    lhs: signature.parameters().map_or_else(|| self.translate_name(name), |parameters| Box::new(GraphExpression::Call {
                        callee: self.translate_name(name),
                        arguments: Box::new(GraphExpression::Sequence { elements: parameters.iter().map(|parameter| *self.translate_name(&parameter.name)).collect() })
                    })),
                    rhs: self.translate_action(action)
                })),
                hidden: false,
            }));
        }

        state.expressions.entries.push(Box::new(GraphFolderEntry {
            id: "desmosify:defs".into(),
            title: "Definitions".into(),
            collapsed: true,
            secret: false,
        }));

        for (name, expression) in &definitions.identifiers {
            let signature = signatures.user_defined.get(name).unwrap();

            state.expressions.entries.push(Box::new(GraphExpressionEntry {
                id: get_next_id(),
                folder_id: Some("desmosify:defs".into()),
                expression: Some(Box::new(GraphExpression::Equality {
                    lhs: signature.parameters().map_or_else(|| self.translate_name(name), |parameters| Box::new(GraphExpression::Call {
                        callee: self.translate_name(name),
                        arguments: Box::new(GraphExpression::Sequence { elements: parameters.iter().map(|parameter| *self.translate_name(&parameter.name)).collect() })
                    })),
                    rhs: self.translate_expression(expression)
                })),
                hidden: true,
            }));
        }

        state.expressions.entries.push(Box::new(GraphFolderEntry {
            id: "desmosify:utils".into(),
            title: "Utilities".into(),
            collapsed: true,
            secret: false,
        }));

        state.expressions.entries.push(Box::new(GraphExpressionEntry {
            id: get_next_id(),
            folder_id: Some("desmosify:utils".into()),
            expression: Some(Box::new(GraphExpression::Equality {
                lhs: Box::new(GraphExpression::Subscript {
                    base: Box::new(GraphExpression::OperatorName("delta".into())),
                    script: Box::new(GraphExpression::Alphanumeric("dummyvar".into()))
                }),
                rhs: Box::new(GraphExpression::Decimal(0.0))
            })),
            hidden: false,
        }));
        state.expressions.entries.push(Box::new(GraphExpressionEntry {
            id: get_next_id(),
            folder_id: Some("desmosify:utils".into()),
            expression: Some(Box::new(GraphExpression::Equality {
                lhs: Box::new(GraphExpression::Subscript {
                    base: Box::new(GraphExpression::OperatorName("delta".into())),
                    script: Box::new(GraphExpression::Alphanumeric("noaction".into()))
                }),
                rhs: Box::new(GraphExpression::RightArrow {
                    lhs: Box::new(GraphExpression::Subscript {
                        base: Box::new(GraphExpression::OperatorName("delta".into())),
                        script: Box::new(GraphExpression::Alphanumeric("dummyvar".into()))
                    }),
                    rhs: Box::new(GraphExpression::Decimal(0.0))
                })
            })),
            hidden: false,
        }));

        state.to_json()
    }
}
