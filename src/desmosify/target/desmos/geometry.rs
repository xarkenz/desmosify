use super::*;

use crate::{Definitions, ConstantValue};
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
        let mut operands = Vec::from_iter(operands.iter()
            .rev()
            .map(|operand| self.translate_expression(operand)));

        Box::new(match operation {
            crate::Operation::PointLiteral => todo!(),
            crate::Operation::ListLiteral => todo!(),
            crate::Operation::ListFill => todo!(),
            crate::Operation::ListMap => todo!(),
            crate::Operation::ListFilter => todo!(),
            crate::Operation::MemberAccess => todo!(),
            crate::Operation::BuiltIn => todo!(),
            crate::Operation::Call => todo!(),
            crate::Operation::ActionCall => SyntaxNode::Call(operands.pop().unwrap(), Box::new(SyntaxNode::Sequence(operands.into_iter().map(|arg| *arg).collect()))),
            crate::Operation::Index => todo!(),
            crate::Operation::Posate => todo!(),
            crate::Operation::Negate => todo!(),
            crate::Operation::Not => todo!(),
            crate::Operation::Exponent => todo!(),
            crate::Operation::Multiply => todo!(),
            crate::Operation::Divide => todo!(),
            crate::Operation::Modulus => todo!(),
            crate::Operation::Add => todo!(),
            crate::Operation::Subtract => todo!(),
            crate::Operation::LessThan => todo!(),
            crate::Operation::GreaterThan => todo!(),
            crate::Operation::LessEqual => todo!(),
            crate::Operation::GreaterEqual => todo!(),
            crate::Operation::Equal => todo!(),
            crate::Operation::NotEqual => todo!(),
            crate::Operation::And => todo!(),
            crate::Operation::Or => todo!(),
            crate::Operation::ExclusiveRange => todo!(),
            crate::Operation::InclusiveRange => todo!(),
            crate::Operation::Conditional => todo!(),
            crate::Operation::Assignment => todo!(),
            crate::Operation::Update => SyntaxNode::RightArrow(operands.pop().unwrap(), operands.pop().unwrap()),
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
}

impl crate::target::Target for GeometryTarget {
    type Output = JsonValue;

    fn name(&self) -> &'static str {
        "desmos-geometry"
    }

    fn compile(&self, definitions: &Definitions) -> Self::Output {
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
                            content: Some(Box::new(content))
                        })
                    },
                };

                state.expressions.list.push(entry);
            }
        }

        state.expressions.list.push(Box::new(FolderEntry {
            id: "desmosify_actions".into(),
            title: "Actions".into(),
            collapsed: true,
            secret: false,
        }));

        for (_name, _action) in &definitions.actions {
            //
        }

        state.expressions.list.push(Box::new(FolderEntry {
            id: "desmosify_variables".into(),
            title: "Variables".into(),
            collapsed: true,
            secret: false,
        }));

        for (name, expression) in &definitions.identifiers {
            state.expressions.list.push(Box::new(ExpressionEntry {
                id: get_next_id(),
                folder_id: Some("desmosify_variables".into()),
                content: Some(Box::new(SyntaxNode::Equality(
                    self.translate_name(name),
                    self.translate_expression(expression),
                ))),
            }))
        }

        state.to_json()
    }
}