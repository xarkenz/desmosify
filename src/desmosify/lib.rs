#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use std::collections::BTreeMap;

pub mod semantics;
pub mod display;
pub mod syntax;
pub mod target;
pub mod token;

use syntax::*;
use token::*;

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
    Void,
    Real,
    Int,
    Bool,
    Point,
    IPoint,
    Color,
    Polygon,
    Segment,
    Str,
    List { item_type: Box<DataType> },
    Function { name: String },
    Action { name: String },
    User { name: String },
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
            _ => Self::User { name: String::from(name) }
        }
    }
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown => write!(f, "?"),
            Self::Void => write!(f, "void"),
            Self::Real => write!(f, "real"),
            Self::Int => write!(f, "int"),
            Self::Bool => write!(f, "bool"),
            Self::Point => write!(f, "point"),
            Self::IPoint => write!(f, "ipoint"),
            Self::Color => write!(f, "color"),
            Self::Polygon => write!(f, "polygon"),
            Self::Segment => write!(f, "segment"),
            Self::Str => write!(f, "str"),
            Self::List { item_type } => write!(f, "[{item_type}]"),
            Self::Function { name } => write!(f, "<function {name}>"),
            Self::Action { name } => write!(f, "<action {name}>"),
            Self::User { name } => write!(f, "{name}"),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum ConstantValue {
    Real(f64),
    Int(i64),
    Bool(bool),
    Point(f64, f64),
    IPoint(i64, i64),
    Color(Color),
    Polygon(Vec<(f64, f64)>),
    Segment((f64, f64), (f64, f64)),
    Str(String),
    List(DataType, Vec<ConstantValue>),
    EnumVariant(String, String),
}

impl ConstantValue {
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
            Self::List(item_type, _) => DataType::List {
                item_type: Box::new(item_type.clone()),
            },
            Self::EnumVariant(name, _) => DataType::User {
                name: name.clone(),
            },
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
pub enum Signature {
    Const {
        name: String,
        parameters: Option<Vec<Parameter>>,
        value_type: DataType,
    },
    Let {
        name: String,
        parameters: Option<Vec<Parameter>>,
        value_type: DataType,
    },
    Var {
        name: String,
        qualifier: Option<VariableQualifier>,
        value_type: DataType,
    },
    Action {
        name: String,
        parameters: Vec<Parameter>,
    },
    Enum {
        name: String,
        variants: Vec<String>,
    },
}

impl Signature {
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
        match self {
            Self::Const { name, .. } => name,
            Self::Let { name, .. } => name,
            Self::Var { name, .. } => name,
            Self::Action { name, .. } => name,
            Self::Enum { name, .. } => name,
        }
    }

    pub fn parameters(&self) -> Option<&[Parameter]> {
        match self {
            Self::Const { parameters, .. } => parameters.as_deref(),
            Self::Let { parameters, .. } => parameters.as_deref(),
            Self::Action { parameters, .. } => Some(parameters),
            _ => None
        }
    }
}

#[derive(Debug)]
pub struct Ticker {
    pub interval_ms: Option<Box<Expression>>,
    pub tick_action: Box<Action>,
}

#[derive(Debug)]
pub struct Definitions {
    pub identifiers: BTreeMap<String, Box<Expression>>,
    pub actions: BTreeMap<String, Box<Action>>,
    pub public: Option<Vec<Expression>>,
    pub ticker: Option<Ticker>,
    pub display: Option<Vec<display::Element>>,
}

impl Definitions {
    pub fn new() -> Self {
        Self {
            identifiers: BTreeMap::new(),
            actions: BTreeMap::new(),
            public: None,
            ticker: None,
            display: None,
        }
    }
}

#[derive(Debug)]
pub struct Signatures {
    pub user_defined: BTreeMap<String, Signature>,
}

impl Signatures {
    pub fn new() -> Self {
        Self {
            user_defined: BTreeMap::new(),
        }
    }
}
