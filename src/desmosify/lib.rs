use syntax::*;
use token::*;

use std::collections::BTreeMap;

pub mod semantics;
pub mod display;
pub mod syntax;
pub mod target;
pub mod token;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct SourceLocation {
    pub index: usize,
    pub line: usize,
    pub column: usize,
}

impl SourceLocation {
    pub const fn start() -> Self {
        Self {
            index: 0,
            line: 1,
            column: 1,
        }
    }
}

impl std::fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "(line {}:{})", self.line, self.column)
    }
}

#[derive(Clone, Debug)]
pub struct DesmosifyError {
    message: String,
    start: Option<SourceLocation>,
    end: Option<SourceLocation>,
}

impl DesmosifyError {
    pub fn new(message: String, start: Option<SourceLocation>, end: Option<SourceLocation>) -> Self {
        Self {
            message,
            start,
            end,
        }
    }
}

impl std::fmt::Display for DesmosifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(location) = self.start {
            write!(f, "{} ", location)?;
        }
        write!(f, "{}", self.message)
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Color {
    Rgb { red: f64, green: f64, blue: f64 },
    Hsv { hue: f64, saturation: f64, value: f64 },
}

impl Color {
    pub fn rgb(red: f64, green: f64, blue: f64) -> Self {
        Self::Rgb { red, green, blue }
    }

    pub fn hsv(hue: f64, saturation: f64, value: f64) -> Self {
        Self::Hsv { hue, saturation, value }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Parameter {
    name: String,
    data_type: DataType,
}

#[derive(Clone, PartialEq, Debug)]
pub enum DataType {
    Unknown,
    Real,
    Int,
    Bool,
    Point,
    IPoint,
    Color,
    Polygon,
    Segment,
    Str,
    List(Box<DataType>),
    Function,
    Action,
    User(String),
}

impl DataType {
    pub fn from_name(name: &str) -> Self {
        match name {
            "real" => Self::Real,
            "int" => Self::Int,
            "bool" => Self::Bool,
            "point" => Self::Point,
            "ipoint" => Self::IPoint,
            "color" => Self::Color,
            "polygon" => Self::Polygon,
            "segment" => Self::Segment,
            "str" => Self::Str,
            "action" => Self::Action, // FIXME: action is a keyword
            _ => Self::User(String::from(name))
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum DataValue {
    Real(f64),
    Int(i64),
    Bool(bool),
    Point(f64, f64),
    IPoint(i64, i64),
    Color(Color),
    Polygon(Vec<(f64, f64)>),
    Segment((f64, f64), (f64, f64)),
    Str(String),
    List(DataType, Vec<DataValue>),
    Function(String),
}

impl DataValue {
    pub fn from_token_value(value: &TokenValue) -> Option<Self> {
        Some(match value {
            &TokenValue::Integer(value) => Self::Int(value),
            &TokenValue::Real(value) => Self::Real(value),
            &TokenValue::Boolean(value) => Self::Bool(value),
            TokenValue::String(value) => Self::Str(value.clone()),
            _ => return None,
        })
    }

    pub fn data_type(&self) -> DataType {
        match self {
            Self::Real(_) => DataType::Real,
            Self::Int(_) => DataType::Int,
            Self::Bool(_) => DataType::Bool,
            Self::Point(_, _) => DataType::Point,
            Self::IPoint(_, _) => DataType::IPoint,
            Self::Color(_) => DataType::Color,
            Self::Polygon(_) => DataType::Polygon,
            Self::Segment(_, _) => DataType::Segment,
            Self::Str(_) => DataType::Str,
            Self::List(inner, _) => DataType::List(Box::new(inner.clone())),
            Self::Function(_) => DataType::Function,
        }
    }
}

#[derive(Debug)]
pub enum Action {
    Block(Vec<Action>),
    Update(Box<Expression>, Box<Expression>),
    Call(Box<Expression>, Vec<Expression>),
    Conditional(Vec<(Expression, Action)>, Option<Box<Action>>),
}

#[derive(Copy, Clone, Debug)]
pub enum VariableQualifier {
    Timer,
}

#[derive(Debug)]
pub enum Identifier {
    Const {
        name: String,
        parameters: Vec<Parameter>,
        value_type: DataType,
        value: Box<Expression>,
    },
    Let {
        name: String,
        parameters: Vec<Parameter>,
        value_type: DataType,
        value: Box<Expression>,
    },
    Var {
        name: String,
        qualifier: Option<VariableQualifier>,
        value_type: DataType,
        value: Box<Expression>,
    },
    Action {
        name: String,
        parameters: Vec<Parameter>,
        content: Box<Action>,
    },
    Enum {
        name: String,
        variants: Vec<String>,
    }
}

impl Identifier {
    pub fn variant_name(&self) -> &'static str {
        match self {
            Self::Const { .. } => "const",
            Self::Let { .. } => "let",
            Self::Var { .. } => "var",
            Self::Action { .. } => "action",
            Self::Enum { .. } => "enum",
        }
    }

    pub fn name(&self) -> &str {
        // TODO: necessary?
        match self {
            Self::Const { name, .. } => name,
            Self::Let { name, .. } => name,
            Self::Var { name, .. } => name,
            Self::Action { name, .. } => name,
            Self::Enum { name, .. } => name,
        }
    }
}

#[derive(Debug)]
pub struct Ticker {
    interval_ms: Option<Box<Expression>>,
    tick_action: Box<Action>,
}

#[derive(Debug)]
pub struct Definitions {
    public: Option<Vec<Expression>>,
    ticker: Option<Ticker>,
    display: Option<Vec<display::Element>>,
    identifiers: BTreeMap<String, Identifier>,
}

impl Definitions {
    pub fn new() -> Self {
        Self {
            public: None,
            ticker: None,
            display: None,
            identifiers: BTreeMap::new(),
        }
    }

    pub fn get_identifier(&self, name: &str) -> Option<&Identifier> {
        self.identifiers.get(name)
    }
}
