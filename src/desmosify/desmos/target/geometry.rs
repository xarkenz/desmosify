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

    pub fn translate_constant_value(&self, value: &ConstantValue) -> Box<Expression> {
        Box::new(match value {
            ConstantValue::Real(value) => Expression::Decimal(*value),
            ConstantValue::Int(value) => Expression::Decimal(*value as f64),
            ConstantValue::Bool(value) => Expression::Decimal(if *value { 1.0 } else { 0.0 }),
            ConstantValue::Point(x_value, y_value) => Expression::Parentheses {
                body: Box::new(Expression::Sequence {
                    elements: vec![
                        Expression::Decimal(*x_value),
                        Expression::Decimal(*y_value),
                    ]
                })
            },
            ConstantValue::IPoint(x_value, y_value) => Expression::Parentheses {
                body: Box::new(Expression::Sequence {
                    elements: vec![
                        Expression::Decimal(*x_value as f64),
                        Expression::Decimal(*y_value as f64),
                    ]
                })
            },
            ConstantValue::Color(color) => match color {
                &crate::Color::Rgb { red, green, blue } => Expression::Call {
                    callee: Box::new(Expression::Command("rgb".into())),
                    arguments: Box::new(Expression::Sequence {
                        elements: vec![
                            Expression::Decimal(red),
                            Expression::Decimal(green),
                            Expression::Decimal(blue),
                        ]
                    })
                },
                &crate::Color::Hsv { hue, saturation, value } => Expression::Call {
                    callee: Box::new(Expression::Command("hsv".into())),
                    arguments: Box::new(Expression::Sequence {
                        elements: vec![
                            Expression::Decimal(hue),
                            Expression::Decimal(saturation),
                            Expression::Decimal(value),
                        ]
                    })
                },
            },
            ConstantValue::Polygon(points) => Expression::Call {
                callee: Box::new(Expression::Command("polygon".into())),
                arguments: Box::new(Expression::Sequence {
                    elements: Vec::from_iter(points.iter().map(|&(x_value, y_value)| {
                        Expression::Parentheses {
                            body: Box::new(Expression::Sequence(vec![
                                Expression::Decimal(x_value),
                                Expression::Decimal(y_value),
                            ]))
                        }
                    }))
                })
            },
            ConstantValue::Segment((x1_value, y1_value), (x2_value, y2_value)) => Expression::Call {
                callee: Box::new(Expression::Command("segment".into())),
                arguments: Box::new(Expression::Sequence {
                    elements: vec![
                        Expression::Parentheses {
                            body: Box::new(Expression::Sequence(vec![
                                Expression::Decimal(*x1_value),
                                Expression::Decimal(*y1_value),
                            ]))
                        },
                        Expression::Parentheses {
                            body: Box::new(Expression::Sequence(vec![
                                Expression::Decimal(*x2_value),
                                Expression::Decimal(*y2_value),
                            ]))
                        },
                    ]
                })
            },
            ConstantValue::Str(content) => Expression::Alphanumeric(content.clone()),
            ConstantValue::List { items, .. } => Expression::List {
                body: Box::new(Expression::Sequence {
                    elements: items
                        .iter()
                        .map(|value| *self.translate_constant_value(value))
                        .collect()
                })
            },
            ConstantValue::EnumVariant { .. } => todo!(),
        })
    }

    pub fn translate_name(&self, name: &str) -> Box<Expression> {
        Box::new(Expression::Subscript {
            base: Box::new(Expression::Letter('X')),
            script: Box::new(Expression::Alphanumeric(name.chars().filter(|&ch| ch != '_').collect()))
        })
    }

    pub fn translate_operator(&self, operation: crate::Operation, operands: &[Expression]) -> Box<Expression> {
        let raw_operands = operands;
        let mut operands = Vec::from_iter(raw_operands.iter()
            .rev()
            .map(|operand| self.translate_expression(operand)));

        Box::new(match operation {
            crate::Operation::PointLiteral => Expression::Parentheses {
                body: Box::new(Expression::Sequence { elements: operands.into_iter().map(|component| *component).collect() })
            },
            crate::Operation::ListLiteral => Expression::List {
                body: Box::new(Expression::Sequence { elements: operands.into_iter().map(|item| *item).collect() })
            },
            crate::Operation::ListFill => Expression::List {
                body: Box::new(Expression::For {
                    lhs: operands.pop().unwrap(),
                    rhs: Box::new(Expression::Equality {
                        lhs: Box::new(Expression::Letter('x')),
                        rhs: Box::new(Expression::SquareBrackets(
                            Box::new(Expression::Range {
                                start: Box::new(Expression::Decimal(1.0)),
                                end: Some(operands.pop().unwrap())
                            }),
                        ))
                    })
                })
            },
            crate::Operation::ListMap => Expression::List {
                body: Box::new(Expression::For {
                    lhs: match *operands.pop().unwrap() {
                        Expression::SquareBrackets(content) => match content.as_ref() {
                            Expression::For(_, _) => content,
                            _ => Box::new(Expression::SquareBrackets(content))
                        },
                        operand => Box::new(operand)
                    },
                    rhs: Box::new(Expression::Equality {
                        lhs: operands.pop().unwrap(),
                        rhs: operands.pop().unwrap()
                    })
                })
            },
            crate::Operation::ListFilter => todo!(),
            crate::Operation::MemberAccess => todo!(),
            crate::Operation::BuiltIn => Expression::Command(
                match &raw_operands[0].value {
                    ExpressionValue::Name(name) => name.clone(),
                    _ => panic!()
                },
            ),
            crate::Operation::Call => Expression::Call {
                callee: operands.pop().unwrap(),
                arguments: Box::new(Expression::Sequence { elements: operands.into_iter().map(|argument| *argument).collect() })
            },
            crate::Operation::ActionCall => Expression::Call {
                callee: operands.pop().unwrap(),
                arguments: Box::new(Expression::Sequence { elements: operands.into_iter().map(|argument| *argument).collect() })
            },
            crate::Operation::Index => Expression::Parentheses {
                body: Box::new(Expression::Index {
                    indexee: operands.pop().unwrap(),
                    index: operands.pop().unwrap()
                })
            },
            crate::Operation::Posate => Expression::Parentheses { body: Box::new(Expression::Positive { operand: operands.pop().unwrap() }) },
            crate::Operation::Negate => Expression::Parentheses { body: Box::new(Expression::Negative { operand: operands.pop().unwrap() }) },
            crate::Operation::Not => Expression::Piecewise {
                body: Box::new(Expression::Sequence {
                    elements: vec![
                        Expression::Equality {
                            lhs: operands.pop().unwrap(),
                            rhs: Box::new(Expression::Decimal(0.0))
                        },
                        Expression::Decimal(0.0),
                    ]
                })
            },
            crate::Operation::Exponent => Expression::Superscript {
                base: operands.pop().unwrap(),
                script: operands.pop().unwrap()
            },
            crate::Operation::Multiply => Expression::Parentheses {
                body: Box::new(Expression::Multiply {
                    lhs: operands.pop().unwrap(),
                    rhs: operands.pop().unwrap()
                })
            },
            crate::Operation::Divide => Expression::Fraction {
                numerator: operands.pop().unwrap(),
                denominator: operands.pop().unwrap()
            },
            crate::Operation::Modulus => Expression::Call {
                callee: Box::new(Expression::Command("mod".into())),
                arguments: Box::new(Expression::Sequence {
                    elements: vec![
                        *operands.pop().unwrap(),
                        *operands.pop().unwrap(),
                    ]
                })
            },
            crate::Operation::Add => Expression::Parentheses {
                body: Box::new(Expression::Add {
                    lhs: operands.pop().unwrap(),
                    rhs: operands.pop().unwrap()
                })
            },
            crate::Operation::Subtract => Expression::Parentheses {
                body: Box::new(Expression::Subtract {
                    lhs: operands.pop().unwrap(),
                    rhs: operands.pop().unwrap()
                })
            },
            crate::Operation::LessThan => Expression::Piecewise {
                body: Box::new(Expression::Sequence {
                    elements: vec![
                        Expression::InequalityChain {
                            lhs: operands.pop().unwrap(),
                            first_kind: InequalityKind::Less,
                            rhs: operands.pop().unwrap(),
                            chain: Vec::new()
                        },
                        Expression::Decimal(0.0),
                    ]
                })
            },
            crate::Operation::GreaterThan => Expression::Piecewise {
                body: Box::new(Expression::Sequence {
                    elements: vec![
                        Expression::InequalityChain {
                            lhs: operands.pop().unwrap(),
                            first_kind: InequalityKind::Greater,
                            rhs: operands.pop().unwrap(),
                            chain: Vec::new()
                        },
                        Expression::Decimal(0.0),
                    ]
                })
            },
            crate::Operation::LessEqual => Expression::Piecewise {
                body: Box::new(Expression::Sequence {
                    elements: vec![
                        Expression::InequalityChain {
                            lhs: operands.pop().unwrap(),
                            first_kind: InequalityKind::LessEqual,
                            rhs: operands.pop().unwrap(),
                            chain: Vec::new()
                        },
                        Expression::Decimal(0.0),
                    ]
                })
            },
            crate::Operation::GreaterEqual => Expression::Piecewise {
                body: Box::new(Expression::Sequence {
                    elements: vec![
                        Expression::InequalityChain {
                            lhs: operands.pop().unwrap(),
                            first_kind: InequalityKind::GreaterEqual,
                            rhs: operands.pop().unwrap(),
                            chain: Vec::new()
                        },
                        Expression::Decimal(0.0),
                    ]
                })
            },
            crate::Operation::Equal => Expression::Piecewise {
                body: Box::new(Expression::Sequence {
                    elements: vec![
                        Expression::Equality {
                            lhs: operands.pop().unwrap(),
                            rhs: operands.pop().unwrap()
                        },
                        Expression::Decimal(0.0),
                    ]
                })
            },
            // Desmos doesn't have != built-in, so we have to negate ==
            crate::Operation::NotEqual => Expression::Piecewise {
                body: Box::new(Expression::Sequence {
                    elements: vec![
                        Expression::Colon {
                            lhs: Box::new(Expression::Equality {
                                lhs: operands.pop().unwrap(),
                                rhs: operands.pop().unwrap()
                            }),
                            rhs: Box::new(Expression::Decimal(0.0))
                        },
                        Expression::Decimal(1.0),
                    ]
                })
            },
            crate::Operation::And => Expression::Piecewise {
                body: Box::new(Expression::Sequence {
                    elements: vec![
                        Expression::Colon {
                            lhs: Box::new(Expression::Equality {
                                lhs: operands.pop().unwrap(),
                                rhs: Box::new(Expression::Decimal(0.0))
                            }),
                            rhs: Box::new(Expression::Decimal(0.0))
                        },
                        *operands.pop().unwrap(),
                    ]
                })
            },
            crate::Operation::Or => Expression::Piecewise {
                body: Box::new(Expression::Sequence {
                    elements: vec![
                        Expression::Colon {
                            lhs: Box::new(Expression::Equality {
                                lhs: operands.pop().unwrap(),
                                rhs: Box::new(Expression::Decimal(1.0))
                            }),
                            rhs: Box::new(Expression::Decimal(1.0))
                        },
                        *operands.pop().unwrap(),
                    ]
                })
            },
            crate::Operation::ExclusiveRange => Expression::List {
                body: Box::new(Expression::Range {
                    start: operands.pop().unwrap(),
                    end: operands.pop().map(|operand| Box::new(Expression::Parentheses {
                        body: Box::new(Expression::Subtract {
                            lhs: operand,
                            rhs: Box::new(Expression::Decimal(1.0))
                        })
                    }))
                })
            },
            crate::Operation::InclusiveRange => Expression::List {
                body: Box::new(Expression::Range {
                    start: operands.pop().unwrap(),
                    end: operands.pop()
                })
            },
            crate::Operation::Conditional => {
                let mut branches = Vec::new();
                while operands.len() > 1 {
                    branches.push(Expression::Colon {
                        lhs: Box::new(Expression::Equality {
                            lhs: operands.pop().unwrap(),
                            rhs: Box::new(Expression::Decimal(1.0))
                        }),
                        rhs: operands.pop().unwrap()
                    });
                }
                if let Some(operand) = operands.pop() {
                    branches.push(*operand);
                }
                Expression::Piecewise { body: Box::new(Expression::Sequence { elements: branches }) }
            },
            crate::Operation::Assignment => todo!(),
            crate::Operation::Update => Expression::RightArrow {
                lhs: operands.pop().unwrap(),
                rhs: operands.pop().unwrap()
            },
            crate::Operation::With => todo!(),
        })
    }

    pub fn translate_expression(&self, expression: &Expression) -> Box<Expression> {
        match &expression.value {
            ExpressionValue::Literal(value) => self.translate_constant_value(value),
            ExpressionValue::Name(name) => self.translate_name(name),
            ExpressionValue::Operator(operation, operands) => self.translate_operator(*operation, operands),
        }
    }

    pub fn translate_action(&self, action: &Action) -> Box<Expression> {
        match action {
            Action::Block(subactions) => Box::new(Expression::Parentheses { body: Box::new(Expression::Sequence { elements: subactions.iter().map(|subaction| *self.translate_action(subaction)).collect() }) }),
            Action::Update(target, value) => Box::new(Expression::RightArrow {
                lhs: self.translate_expression(target),
                rhs: self.translate_expression(value)
            }),
            Action::Call(name, arguments) => Box::new(Expression::Call {
                callee: self.translate_expression(name),
                arguments: Box::new(Expression::Sequence { elements: arguments.iter().map(|argument| *self.translate_expression(argument)).collect() })
            }),
            Action::Conditional(branches, default_branch) => {
                let mut piecewise_branches = Vec::from_iter(branches.iter().map(|(condition, consequent)| {
                    Expression::Colon {
                        lhs: Box::new(Expression::Equality {
                            lhs: self.translate_expression(condition),
                            rhs: Box::new(Expression::Decimal(1.0))
                        }),
                        rhs: self.translate_action(consequent)
                    }
                }));
                if let Some(default_branch) = default_branch {
                    piecewise_branches.push(*self.translate_action(default_branch));
                }
                else {
                    piecewise_branches.push(Expression::Subscript {
                        base: Box::new(Expression::Command("delta".into())),
                        script: Box::new(Expression::Alphanumeric("noaction".into()))
                    });
                }

                Box::new(Expression::Piecewise { body: Box::new(Expression::Sequence { elements: piecewise_branches }) })
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
                product: "geometry-calculator".into()
            },
            expressions: Expressions {
                list: Vec::new(),
                ticker: None,
            },
        };
        let mut next_id: usize = 0;
        let mut get_next_id = || {
            let id = next_id.to_string();
            next_id += 1;
            id
        };

        state.expressions.list.push(Box::new(FolderEntry {
            id: "**dcg_geo_folder**".into(),
            title: "geometry".into(),
            collapsed: true,
            secret: true,
        }));

        if let Some(public) = &definitions.public {
            for expression in public {
                let entry: Box<dyn Entry> = match *self.translate_expression(expression) {
                    Expression::Alphanumeric(content) => {
                        Box::new(TextEntry {
                            id: get_next_id(),
                            folder_id: None,
                            content: content.into(),
                        })
                    },
                    content => {
                        Box::new(ExpressionEntry {
                            id: get_next_id(),
                            folder_id: None,
                            content: Some(Box::new(content)),
                            hidden: false,
                        })
                    },
                };

                state.expressions.list.push(entry);
            }
        }

        state.expressions.list.push(Box::new(FolderEntry {
            id: "desmosify:actions".into(),
            title: "Actions".into(),
            collapsed: true,
            secret: false,
        }));

        for (name, action) in &definitions.actions {
            let signature = signatures.user_defined.get(name).unwrap();

            state.expressions.list.push(Box::new(ExpressionEntry {
                id: get_next_id(),
                folder_id: Some("desmosify:actions".into()),
                content: Some(Box::new(Expression::Equality {
                    lhs: signature.parameters().map_or_else(|| self.translate_name(name), |parameters| Box::new(Expression::Call {
                        callee: self.translate_name(name),
                        arguments: Box::new(Expression::Sequence { elements: parameters.iter().map(|parameter| *self.translate_name(&parameter.name)).collect() })
                    })),
                    rhs: self.translate_action(action)
                })),
                hidden: false,
            }));
        }

        state.expressions.list.push(Box::new(FolderEntry {
            id: "desmosify:defs".into(),
            title: "Definitions".into(),
            collapsed: true,
            secret: false,
        }));

        for (name, expression) in &definitions.identifiers {
            let signature = signatures.user_defined.get(name).unwrap();

            state.expressions.list.push(Box::new(ExpressionEntry {
                id: get_next_id(),
                folder_id: Some("desmosify:defs".into()),
                content: Some(Box::new(Expression::Equality {
                    lhs: signature.parameters().map_or_else(|| self.translate_name(name), |parameters| Box::new(Expression::Call {
                        callee: self.translate_name(name),
                        arguments: Box::new(Expression::Sequence { elements: parameters.iter().map(|parameter| *self.translate_name(&parameter.name)).collect() })
                    })),
                    rhs: self.translate_expression(expression)
                })),
                hidden: true,
            }));
        }

        state.expressions.list.push(Box::new(FolderEntry {
            id: "desmosify:utils".into(),
            title: "Utilities".into(),
            collapsed: true,
            secret: false,
        }));

        state.expressions.list.push(Box::new(ExpressionEntry {
            id: get_next_id(),
            folder_id: Some("desmosify:utils".into()),
            content: Some(Box::new(Expression::Equality {
                lhs: Box::new(Expression::Subscript {
                    base: Box::new(Expression::Command("delta".into())),
                    script: Box::new(Expression::Alphanumeric("dummyvar".into()))
                }),
                rhs: Box::new(Expression::Decimal(0.0))
            })),
            hidden: false,
        }));
        state.expressions.list.push(Box::new(ExpressionEntry {
            id: get_next_id(),
            folder_id: Some("desmosify:utils".into()),
            content: Some(Box::new(Expression::Equality {
                lhs: Box::new(Expression::Subscript {
                    base: Box::new(Expression::Command("delta".into())),
                    script: Box::new(Expression::Alphanumeric("noaction".into()))
                }),
                rhs: Box::new(Expression::RightArrow {
                    lhs: Box::new(Expression::Subscript {
                        base: Box::new(Expression::Command("delta".into())),
                        script: Box::new(Expression::Alphanumeric("dummyvar".into()))
                    }),
                    rhs: Box::new(Expression::Decimal(0.0))
                })
            })),
            hidden: false,
        }));

        state.to_json()
    }
}
