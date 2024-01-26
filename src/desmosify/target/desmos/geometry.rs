use super::*;

use crate::{Action, ConstantValue, Definitions, Signatures};
use crate::syntax::{Expression, ExpressionValue};

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
        "var", // FIXME: this probably doesn't work lol
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

    pub fn translate_constant_value(&self, value: &ConstantValue) -> Box<SyntaxNode> {
        Box::new(match value {
            ConstantValue::Real(value) => SyntaxNode::Decimal(*value),
            ConstantValue::Int(value) => SyntaxNode::Decimal(*value as f64),
            ConstantValue::Bool(value) => SyntaxNode::Decimal(if *value { 1.0 } else { 0.0 }),
            ConstantValue::Point(x_value, y_value) => SyntaxNode::Paren(Box::new(SyntaxNode::Sequence(vec![
                SyntaxNode::Decimal(*x_value),
                SyntaxNode::Decimal(*y_value),
            ]))),
            ConstantValue::IPoint(x_value, y_value) => SyntaxNode::Paren(Box::new(SyntaxNode::Sequence(vec![
                SyntaxNode::Decimal(*x_value as f64),
                SyntaxNode::Decimal(*y_value as f64),
            ]))),
            ConstantValue::Color(color) => match color {
                &crate::Color::Rgb { red, green, blue } => SyntaxNode::Call(
                    Box::new(SyntaxNode::Command("rgb".into())),
                    Box::new(SyntaxNode::Sequence(vec![
                        SyntaxNode::Decimal(red),
                        SyntaxNode::Decimal(green),
                        SyntaxNode::Decimal(blue),
                    ])),
                ),
                &crate::Color::Hsv { hue, saturation, value } => SyntaxNode::Call(
                    Box::new(SyntaxNode::Command("hsv".into())),
                    Box::new(SyntaxNode::Sequence(vec![
                        SyntaxNode::Decimal(hue),
                        SyntaxNode::Decimal(saturation),
                        SyntaxNode::Decimal(value),
                    ])),
                ),
            },
            ConstantValue::Polygon(points) => SyntaxNode::Call(
                Box::new(SyntaxNode::Command("polygon".into())),
                Box::new(SyntaxNode::Sequence(Vec::from_iter(points.iter().map(|&(x_value, y_value)| {
                    SyntaxNode::Paren(Box::new(SyntaxNode::Sequence(vec![
                        SyntaxNode::Decimal(x_value),
                        SyntaxNode::Decimal(y_value),
                    ])))
                })))),
            ),
            ConstantValue::Segment((x1_value, y1_value), (x2_value, y2_value)) => SyntaxNode::Call(
                Box::new(SyntaxNode::Command("segment".into())),
                Box::new(SyntaxNode::Sequence(vec![
                    SyntaxNode::Paren(Box::new(SyntaxNode::Sequence(vec![
                        SyntaxNode::Decimal(*x1_value),
                        SyntaxNode::Decimal(*y1_value),
                    ]))),
                    SyntaxNode::Paren(Box::new(SyntaxNode::Sequence(vec![
                        SyntaxNode::Decimal(*x2_value),
                        SyntaxNode::Decimal(*y2_value),
                    ]))),
                ])),
            ),
            ConstantValue::Str(content) => SyntaxNode::Alphanumeric(content.clone()),
            ConstantValue::List(_, values) => SyntaxNode::List(
                Box::new(SyntaxNode::Sequence(Vec::from_iter(values.iter().map(|value| {
                    *self.translate_constant_value(value)
                }))))
            ),
            ConstantValue::EnumVariant(_, _) => todo!(),
        })
    }
    
    pub fn translate_name(&self, name: &str) -> Box<SyntaxNode> {
        Box::new(SyntaxNode::Subscript(
            Box::new(SyntaxNode::Letter('X')),
            Box::new(SyntaxNode::Alphanumeric(name.chars().filter(|&ch| ch != '_').collect())),
        ))
    }

    pub fn translate_operator(&self, operation: crate::Operation, operands: &[Expression]) -> Box<SyntaxNode> {
        let raw_operands = operands;
        let mut operands = Vec::from_iter(raw_operands.iter()
            .rev()
            .map(|operand| self.translate_expression(operand)));

        Box::new(match operation {
            crate::Operation::PointLiteral => SyntaxNode::Paren(
                Box::new(SyntaxNode::Sequence(
                    operands.into_iter().map(|component| *component).collect(),
                )),
            ),
            crate::Operation::ListLiteral => SyntaxNode::List(
                Box::new(SyntaxNode::Sequence(
                    operands.into_iter().map(|item| *item).collect(),
                )),
            ),
            crate::Operation::ListFill => todo!(),
            crate::Operation::ListMap => todo!(),
            crate::Operation::ListFilter => todo!(),
            crate::Operation::MemberAccess => todo!(),
            crate::Operation::BuiltIn => SyntaxNode::Command(
                match &raw_operands[0].value {
                    ExpressionValue::Name(name) => name.clone(),
                    _ => panic!()
                },
            ),
            crate::Operation::Call => SyntaxNode::Call(
                operands.pop().unwrap(),
                Box::new(SyntaxNode::Sequence(
                    operands.into_iter().map(|argument| *argument).collect(),
                )),
            ),
            crate::Operation::ActionCall => SyntaxNode::Call(
                operands.pop().unwrap(),
                Box::new(SyntaxNode::Sequence(
                    operands.into_iter().map(|argument| *argument).collect(),
                )),
            ),
            crate::Operation::Index => todo!(),
            crate::Operation::Posate => SyntaxNode::Paren(
                Box::new(SyntaxNode::Pos(
                    operands.pop().unwrap(),
                )),
            ),
            crate::Operation::Negate => SyntaxNode::Paren(
                Box::new(SyntaxNode::Neg(
                    operands.pop().unwrap(),
                )),
            ),
            crate::Operation::Not => SyntaxNode::Piecewise(
                Box::new(SyntaxNode::Sequence(vec![
                    SyntaxNode::Equality(
                        operands.pop().unwrap(),
                        Box::new(SyntaxNode::Decimal(0.0)),
                    ),
                    SyntaxNode::Decimal(0.0),
                ])),
            ),
            crate::Operation::Exponent => SyntaxNode::Superscript(
                operands.pop().unwrap(),
                operands.pop().unwrap(),
            ),
            crate::Operation::Multiply => SyntaxNode::Paren(
                Box::new(SyntaxNode::Mul(
                    operands.pop().unwrap(),
                    operands.pop().unwrap(),
                ))
            ),
            crate::Operation::Divide => SyntaxNode::Frac(
                operands.pop().unwrap(),
                operands.pop().unwrap(),
            ),
            crate::Operation::Modulus => SyntaxNode::Call(
                Box::new(SyntaxNode::Command("mod".into())),
                Box::new(SyntaxNode::Sequence(vec![
                    *operands.pop().unwrap(),
                    *operands.pop().unwrap(),
                ])),
            ),
            crate::Operation::Add => SyntaxNode::Paren(
                Box::new(SyntaxNode::Add(
                    operands.pop().unwrap(),
                    operands.pop().unwrap(),
                ))
            ),
            crate::Operation::Subtract => SyntaxNode::Paren(
                Box::new(SyntaxNode::Sub(
                    operands.pop().unwrap(),
                    operands.pop().unwrap(),
                ))
            ),
            crate::Operation::LessThan => SyntaxNode::Piecewise(
                Box::new(SyntaxNode::Sequence(vec![
                    SyntaxNode::InequalityChain(
                        operands.pop().unwrap(),
                        InequalityType::Less,
                        operands.pop().unwrap(),
                        Vec::new(),
                    ),
                    SyntaxNode::Decimal(0.0),
                ])),
            ),
            crate::Operation::GreaterThan => SyntaxNode::Piecewise(
                Box::new(SyntaxNode::Sequence(vec![
                    SyntaxNode::InequalityChain(
                        operands.pop().unwrap(),
                        InequalityType::Greater,
                        operands.pop().unwrap(),
                        Vec::new(),
                    ),
                    SyntaxNode::Decimal(0.0),
                ])),
            ),
            crate::Operation::LessEqual => SyntaxNode::Piecewise(
                Box::new(SyntaxNode::Sequence(vec![
                    SyntaxNode::InequalityChain(
                        operands.pop().unwrap(),
                        InequalityType::LessEqual,
                        operands.pop().unwrap(),
                        Vec::new(),
                    ),
                    SyntaxNode::Decimal(0.0),
                ])),
            ),
            crate::Operation::GreaterEqual => SyntaxNode::Piecewise(
                Box::new(SyntaxNode::Sequence(vec![
                    SyntaxNode::InequalityChain(
                        operands.pop().unwrap(),
                        InequalityType::GreaterEqual,
                        operands.pop().unwrap(),
                        Vec::new(),
                    ),
                    SyntaxNode::Decimal(0.0),
                ])),
            ),
            crate::Operation::Equal => SyntaxNode::Piecewise(
                Box::new(SyntaxNode::Sequence(vec![
                    SyntaxNode::Equality(
                        operands.pop().unwrap(),
                        operands.pop().unwrap(),
                    ),
                    SyntaxNode::Decimal(0.0),
                ])),
            ),
            // Desmos doesn't have != built-in, so we have to negate ==
            crate::Operation::NotEqual => SyntaxNode::Piecewise(
                Box::new(SyntaxNode::Sequence(vec![
                    SyntaxNode::Colon(
                        Box::new(SyntaxNode::Equality(
                            operands.pop().unwrap(),
                            operands.pop().unwrap(),
                        )),
                        Box::new(SyntaxNode::Decimal(0.0))
                    ),
                    SyntaxNode::Decimal(1.0),
                ])),
            ),
            crate::Operation::And => SyntaxNode::Piecewise(
                Box::new(SyntaxNode::Sequence(vec![
                    SyntaxNode::Colon(
                        Box::new(SyntaxNode::Equality(
                            operands.pop().unwrap(),
                            Box::new(SyntaxNode::Decimal(0.0)),
                        )),
                        Box::new(SyntaxNode::Decimal(0.0)),
                    ),
                    *operands.pop().unwrap(),
                ])),
            ),
            crate::Operation::Or => SyntaxNode::Piecewise(
                Box::new(SyntaxNode::Sequence(vec![
                    SyntaxNode::Colon(
                        Box::new(SyntaxNode::Equality(
                            operands.pop().unwrap(),
                            Box::new(SyntaxNode::Decimal(1.0)),
                        )),
                        Box::new(SyntaxNode::Decimal(1.0)),
                    ),
                    *operands.pop().unwrap(),
                ])),
            ),
            crate::Operation::ExclusiveRange => todo!(),
            crate::Operation::InclusiveRange => todo!(),
            crate::Operation::Conditional => {
                let mut branches = Vec::new();
                while operands.len() > 1 {
                    branches.push(SyntaxNode::Colon(
                        Box::new(SyntaxNode::Equality(
                            operands.pop().unwrap(),
                            Box::new(SyntaxNode::Decimal(1.0)),
                        )),
                        operands.pop().unwrap(),
                    ));
                }
                if let Some(operand) = operands.pop() {
                    branches.push(*operand);
                }
                SyntaxNode::Piecewise(
                    Box::new(SyntaxNode::Sequence(branches)),
                )
            },
            crate::Operation::Assignment => todo!(),
            crate::Operation::Update => SyntaxNode::RightArrow(
                operands.pop().unwrap(),
                operands.pop().unwrap(),
            ),
            crate::Operation::With => todo!(),
        })
    }

    pub fn translate_expression(&self, expression: &Expression) -> Box<SyntaxNode> {
        match &expression.value {
            ExpressionValue::Literal(value) => self.translate_constant_value(value),
            ExpressionValue::Name(name) => self.translate_name(name),
            ExpressionValue::Operator(operation, operands) => self.translate_operator(*operation, operands),
        }
    }

    pub fn translate_action(&self, action: &Action) -> Box<SyntaxNode> {
        match action {
            Action::Block(subactions) => Box::new(SyntaxNode::Paren(
                Box::new(SyntaxNode::Sequence(subactions.iter().map(|subaction| *self.translate_action(subaction)).collect())),
            )),
            Action::Update(target, value) => Box::new(SyntaxNode::RightArrow(
               self.translate_expression(target),
               self.translate_expression(value),
            )),
            Action::Call(name, arguments) => Box::new(SyntaxNode::Call(
                self.translate_expression(name),
                Box::new(SyntaxNode::Sequence(arguments.iter().map(|argument| *self.translate_expression(argument)).collect())),
            )),
            Action::Conditional(branches, default_branch) => todo!(),
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
                    SyntaxNode::Alphanumeric(content) => {
                        Box::new(TextEntry {
                            id: get_next_id(),
                            folder_id: None,
                            content,
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
                content: Some(Box::new(SyntaxNode::Equality(
                    signature.parameters().map_or_else(|| self.translate_name(name), |parameters| Box::new(SyntaxNode::Call(
                        self.translate_name(name),
                        Box::new(SyntaxNode::Sequence(parameters.iter().map(|parameter| *self.translate_name(&parameter.name)).collect())),
                    ))),
                    self.translate_action(action),
                ))),
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
                content: Some(Box::new(SyntaxNode::Equality(
                    signature.parameters().map_or_else(|| self.translate_name(name), |parameters| Box::new(SyntaxNode::Call(
                        self.translate_name(name),
                        Box::new(SyntaxNode::Sequence(parameters.iter().map(|parameter| *self.translate_name(&parameter.name)).collect())),
                    ))),
                    self.translate_expression(expression),
                ))),
                hidden: true,
            }));
        }

        state.to_json()
    }
}