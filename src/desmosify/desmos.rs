use json::JsonValue;
use crate::desmos::latex::{BracketType, Latex, LatexNode};

pub mod latex;
pub mod target;

pub use target::geometry::GeometryTarget;
pub use target::graphing::GraphingTarget;
pub use target::graphing_3d::Graphing3DTarget;

#[derive(Copy, Clone, Debug)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            r,
            g,
            b,
        }
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

pub mod colors {
    use super::Color;

    pub const RED: Color = Color::new(0xC7, 0x44, 0x40);
    pub const BLUE: Color = Color::new(0x2D, 0x70, 0xB3);
    pub const GREEN: Color = Color::new(0x38, 0x8C, 0x46);
    pub const PURPLE: Color = Color::new(0x60, 0x42, 0xA6);
    pub const ORANGE: Color = Color::new(0xFA, 0x7E, 0x19);
    pub const BLACK: Color = Color::new(0x00, 0x00, 0x00);
}

pub trait ToJson {
    fn to_json(&self) -> JsonValue;
}

pub trait Entry: ToJson + std::fmt::Debug {
    fn type_name(&self) -> &str;
    fn id(&self) -> &str;
}

#[derive(Debug)]
pub struct FolderEntry {
    pub id: String,
    pub title: String,
    pub collapsed: bool,
    pub secret: bool,
}

impl ToJson for FolderEntry {
    fn to_json(&self) -> JsonValue {
        let mut object = json::object!{
            "type": self.type_name(),
            "id": self.id(),
            "title": self.title.as_str(),
            "collapsed": self.collapsed,
        };
        if self.secret {
            object["secret"] = true.into();
        }
        object
    }
}

impl Entry for FolderEntry {
    fn type_name(&self) -> &str {
        "folder"
    }

    fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Debug)]
pub struct ExpressionEntry {
    pub id: String,
    pub folder_id: Option<String>,
    pub content: Option<Box<Expression>>,
    pub hidden: bool,
}

impl ToJson for ExpressionEntry {
    fn to_json(&self) -> JsonValue {
        let mut object = json::object!{
            "type": self.type_name(),
            "id": self.id(),
        };
        if let Some(folder_id) = &self.folder_id {
            object["folderId"] = folder_id.as_str().into();
        }
        object["latex"] = self.content.as_ref().map_or_else(String::new, |content| content.to_latex().to_string()).into();
        if self.hidden {
            object["hidden"] = true.into();
        }
        object
    }
}

impl Entry for ExpressionEntry {
    fn type_name(&self) -> &str {
        "expression"
    }

    fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Debug)]
pub struct TextEntry {
    pub id: String,
    pub folder_id: Option<String>,
    pub content: String,
}

impl ToJson for TextEntry {
    fn to_json(&self) -> JsonValue {
        let mut object = json::object!{
            "type": self.type_name(),
            "id": self.id(),
            "text": self.content.as_str(),
        };
        if let Some(folder_id) = &self.folder_id {
            object["folderId"] = folder_id.as_str().into();
        }
        object
    }
}

impl Entry for TextEntry {
    fn type_name(&self) -> &str {
        "text"
    }

    fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Debug)]
pub struct Ticker {
    pub playing: bool,
    pub handler: Option<Box<Expression>>,
    pub min_step: Option<Box<Expression>>,
}

impl ToJson for Ticker {
    fn to_json(&self) -> JsonValue {
        let mut object = json::object!{
            "open": true,
            "playing": self.playing,
        };
        if let Some(handler) = &self.handler {
            object["handlerLatex"] = handler.to_latex().to_string().into();
        }
        if let Some(min_step) = &self.min_step {
            object["minStepLatex"] = min_step.to_latex().to_string().into();
        }
        object
    }
}

#[derive(Debug)]
pub struct Expressions {
    pub list: Vec<Box<dyn Entry>>,
    pub ticker: Option<Ticker>,
}

impl ToJson for Expressions {
    fn to_json(&self) -> JsonValue {
        let list = Vec::from_iter(self.list.iter().map(|entry| entry.to_json()));
        let mut object = json::object!{
            "list": list,
        };
        if let Some(ticker) = &self.ticker {
            object["ticker"] = ticker.to_json();
        }
        object
    }
}

#[derive(Debug)]
pub struct GraphSettings {
    pub product: String,
}

impl ToJson for GraphSettings {
    fn to_json(&self) -> JsonValue {
        json::object!{
            "product": self.product.as_str(),
        }
    }
}

#[derive(Debug)]
pub struct GraphState {
    pub version: i32,
    pub graph: GraphSettings,
    pub expressions: Expressions,
}

impl ToJson for GraphState {
    fn to_json(&self) -> JsonValue {
        json::object!{
            "version": self.version,
            "graph": self.graph.to_json(),
            "expressions": self.expressions.to_json(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum InequalityKind {
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
}

impl InequalityKind {
    pub fn to_latex_node(&self) -> LatexNode {
        LatexNode::Escape {
            value: String::from(match self {
                Self::Less => "lt",
                Self::Greater => "gt",
                Self::LessEqual => "le",
                Self::GreaterEqual => "ge",
            }),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum UnaryExpressionKind {
    Positive,
    Negative,
    Factorial,
    Prime,
    Parentheses,
    List,
    Piecewise,
    Pipes,
}

#[derive(Copy, Clone, Debug)]
pub enum BinaryExpressionKind {
    Equality,
    Regression,
    Add,
    Subtract,
    Multiply,
    DotMultiply,
    CrossMultiply,
    ImplicitMultiply,
    Divide,
    Fraction,
    Call,
    ImplicitCall,
    Index,
    Subscript,
    Superscript,
    Colon,
    For,
    With,
    Dot,
    PercentOf,
    RightArrow,
}

#[derive(Debug)]
pub enum Expression {
    Letter(char),
    Integer(i64),
    Decimal(f64),
    Command(Box<str>),
    Alphanumeric(Box<str>),
    Unary {
        kind: UnaryExpressionKind,
        inner: Box<Self>,
    },
    Binary {
        kind: BinaryExpressionKind,
        lhs: Box<Self>,
        rhs: Box<Self>,
    },
    InequalityChain {
        lhs: Box<Self>,
        first_kind: InequalityKind,
        rhs: Box<Self>,
        chain: Vec<(InequalityKind, Self)>,
    },
    Sequence {
        elements: Vec<Self>,
    },
    Radical {
        index: Option<Box<Self>>,
        radicand: Box<Self>,
    },
    Derivative {
        differential: Box<Self>,
        body: Box<Self>,
    },
    Integral {
        differential: Box<Self>,
        lower_bound: Box<Self>,
        upper_bound: Box<Self>,
        body: Option<Box<Self>>,
    },
    Sum {
        initial: Box<Self>,
        upper_bound: Box<Self>,
        body: Box<Self>,
    },
    Product {
        initial: Box<Self>,
        upper_bound: Box<Self>,
        body: Box<Self>,
    },
    Range {
        start: Box<Self>,
        end: Option<Box<Self>>,
    },
    MixedNumber {
        whole: Box<Self>,
        numerator: Box<Self>,
        denominator: Box<Self>,
    },
}

impl Expression {
    pub fn to_latex(&self) -> Latex {
        match self {
            Self::Letter(letter) => {
                Latex::new().add_symbol(*letter)
            }
            Self::Integer(value) => {
                Latex::new().add_symbols(value.to_string())
            }
            Self::Decimal(value) => {
                if value.is_nan() {
                    Latex::new().add_frac(
                        Latex::new().add_symbol('0'),
                        Latex::new().add_symbol('0'),
                    )
                }
                else if value.is_infinite() {
                    if *value > 0.0 {
                        Latex::new().add_escape("infty".into())
                    }
                    else {
                        Latex::new().add_symbol('-').add_escape("infty".into())
                    }
                }
                else {
                    Latex::new().add_symbols(value.to_string())
                }
            }
            Self::Command(name) => {
                Latex::new().add_operator_name(name.to_string())
            }
            Self::Alphanumeric(value) => {
                Latex::new().add_symbols(value.to_string())
            }
            Self::Unary { kind, inner } => match kind {
                UnaryExpressionKind::Positive => {
                    Latex::new().add_symbol('+').add(inner.to_latex())
                }
                UnaryExpressionKind::Negative => {
                    Latex::new().add_symbol('-').add(inner.to_latex())
                }
                UnaryExpressionKind::Factorial => {
                    inner.to_latex().add_symbol('!')
                }
                UnaryExpressionKind::Prime => {
                    inner.to_latex().add_symbol('\'')
                }
                UnaryExpressionKind::Parentheses => {
                    Latex::new()
                        .add_left(BracketType::Parenthesis)
                        .add(inner.to_latex())
                        .add_right(BracketType::Parenthesis)
                }
                UnaryExpressionKind::List => {
                    Latex::new()
                        .add_left(BracketType::Square)
                        .add(inner.to_latex())
                        .add_right(BracketType::Square)
                }
                UnaryExpressionKind::Piecewise => {
                    Latex::new()
                        .add_left(BracketType::Curly)
                        .add(inner.to_latex())
                        .add_right(BracketType::Curly)
                }
                UnaryExpressionKind::Pipes => {
                    Latex::new()
                        .add_left(BracketType::Pipe)
                        .add(inner.to_latex())
                        .add_right(BracketType::Pipe)
                }
            }
            Self::Binary { kind, lhs, rhs } => match kind {
                BinaryExpressionKind::Equality => {
                    lhs.to_latex().add_symbol('=').add(rhs.to_latex())
                }
                BinaryExpressionKind::Regression => {
                    lhs.to_latex().add_symbol('~').add(rhs.to_latex())
                }
                BinaryExpressionKind::Add => {
                    lhs.to_latex().add_symbol('+').add(rhs.to_latex())
                }
                BinaryExpressionKind::Subtract => {
                    lhs.to_latex().add_symbol('-').add(rhs.to_latex())
                }
                BinaryExpressionKind::Multiply => {
                    lhs.to_latex().add_symbol('*').add(rhs.to_latex())
                }
                BinaryExpressionKind::DotMultiply => {
                    lhs.to_latex().add_escape("cdot".into()).add(rhs.to_latex())
                }
                BinaryExpressionKind::CrossMultiply => {
                    lhs.to_latex().add_escape("cross".into()).add(rhs.to_latex())
                }
                BinaryExpressionKind::ImplicitMultiply => {
                    lhs.to_latex().add(rhs.to_latex())
                }
                BinaryExpressionKind::Divide => {
                    lhs.to_latex().add_symbol('/').add(rhs.to_latex())
                }
                BinaryExpressionKind::Fraction => {
                    Latex::new().add_frac(lhs.to_latex(), rhs.to_latex())
                }
                BinaryExpressionKind::Call => {
                    lhs.to_latex()
                        .add_left(BracketType::Parenthesis)
                        .add(rhs.to_latex())
                        .add_right(BracketType::Parenthesis)
                }
                BinaryExpressionKind::ImplicitCall => {
                    lhs.to_latex().add(rhs.to_latex())
                }
                BinaryExpressionKind::Index => {
                    lhs.to_latex()
                        .add_left(BracketType::Square)
                        .add(rhs.to_latex())
                        .add_right(BracketType::Square)
                }
                BinaryExpressionKind::Subscript => {
                    lhs.to_latex().add_subscript(rhs.to_latex())
                }
                BinaryExpressionKind::Superscript => {
                    lhs.to_latex().add_superscript(rhs.to_latex())
                }
                BinaryExpressionKind::Colon => {
                    lhs.to_latex().add_symbol(':').add(rhs.to_latex())
                }
                BinaryExpressionKind::For => {
                    lhs.to_latex().add_operator_name("for".into()).add(rhs.to_latex())
                }
                BinaryExpressionKind::With => {
                    lhs.to_latex().add_operator_name("with".into()).add(rhs.to_latex())
                }
                BinaryExpressionKind::Dot => {
                    lhs.to_latex().add_symbol('.').add(rhs.to_latex())
                }
                BinaryExpressionKind::PercentOf => {
                    lhs.to_latex().add_symbol('%').add_operator_name("of".into()).add(rhs.to_latex())
                }
                BinaryExpressionKind::RightArrow => {
                    lhs.to_latex().add_escape("to".into()).add(rhs.to_latex())
                }
            }
            Self::InequalityChain { lhs, first_kind, rhs, chain } => {
                let mut latex = lhs.to_latex().add_node(first_kind.to_latex_node()).add(rhs.to_latex());
                for (inequality, value) in chain {
                    latex = latex.add_node(inequality.to_latex_node()).add(value.to_latex());
                }
                latex
            }
            Self::Sequence { elements } => {
                match elements.as_slice() {
                    [] => Latex::new(),
                    [first, rest @ ..] => rest
                        .iter()
                        .fold(first.to_latex(), |latex, next| {
                            latex.add_symbol(',').add(next.to_latex())
                        })
                }
            }
            Self::Radical { index, radicand } => {
                Latex::new().add_sqrt(index.as_deref().map(Self::to_latex), radicand.to_latex())
            }
            Self::Derivative { differential, body } => {
                Latex::new()
                    .add_frac(
                        Latex::new().add_symbol('d'),
                        Latex::new().add_symbol('d').add(differential.to_latex()),
                    )
                    .add(body.to_latex())
            }
            Self::Integral { differential, lower_bound, upper_bound, body } => {
                let mut latex = Latex::new()
                    .add_escape("int".into())
                    .add_subscript(lower_bound.to_latex())
                    .add_superscript(upper_bound.to_latex());
                if let Some(body) = body {
                    latex = latex.add(body.to_latex());
                }
                latex
                    .add_symbol('d')
                    .add(differential.to_latex())
            }
            Self::Sum { initial, upper_bound, body } => {
                Latex::new()
                    .add_escape("sum".into())
                    .add_subscript(initial.to_latex())
                    .add_superscript(upper_bound.to_latex())
                    .add(body.to_latex())
            }
            Self::Product { initial, upper_bound, body } => {
                Latex::new()
                    .add_escape("prod".into())
                    .add_subscript(initial.to_latex())
                    .add_superscript(upper_bound.to_latex())
                    .add(body.to_latex())
            }
            Self::Range { start, end } => {
                if let Some(rhs) = end {
                    start.to_latex().add_symbols("...".into()).add(rhs.to_latex())
                }
                else {
                    start.to_latex().add_symbols("...".into())
                }
            }
            Self::MixedNumber { whole, numerator, denominator } => {
                whole.to_latex().add_frac(numerator.to_latex(), denominator.to_latex())
            }
        }
    }
}
