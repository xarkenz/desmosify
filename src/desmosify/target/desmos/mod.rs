use json::JsonValue;

pub mod geometry;
pub mod graphing;
pub mod graphing_3d;

pub use geometry::GeometryTarget;
pub use graphing::GraphingTarget;
pub use graphing_3d::Graphing3DTarget;

#[derive(Copy, Clone, Debug)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
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
    pub content: Option<Box<SyntaxNode>>,
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
    pub handler: Option<Box<SyntaxNode>>,
    pub min_step: Option<Box<SyntaxNode>>,
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

#[derive(Debug)]
pub enum ValueType {
    ErrorType = 12,
    SeedType = 13,
    EmptyList = 11,
    Action = 5,
    Any = 0,
    ListOfAny = 6,
    Number = 1,
    ListOfNumber = 7,
    Bool = 2,
    ListOfBool = 8,
    Point = 3,
    ListOfPoint = 9,
    Distribution = 4,
    ListOfDistribution = 10,
    RGBColor = 14,
    ListOfColor = 15,
    Polygon = 16,
    ListOfPolygon = 17,
    Segment = 18,
    ListOfSegment = 19,
    Circle = 20,
    ListOfCircle = 21,
    Arc = 22,
    ListOfArc = 23,
    Line = 24,
    ListOfLine = 25,
    Ray = 26,
    ListOfRay = 27,
    Angle = 28,
    ListOfAngle = 29,
    DirectedAngle = 30,
    ListOfDirectedAngle = 31,
    Transformation = 32,
    ListOfTransformation = 33,
    Vector = 34,
    ListOfVector = 35,
    Point3D = 100,
    ListOfPoint3D = 101,
    Segment3D = 102,
    ListOfSegment3D = 103,
    Triangle3D = 104,
    ListOfTriangle3D = 105,
    Sphere3D = 106,
    ListOfSphere3D = 107,
}

#[derive(Copy, Clone, Debug)]
pub enum BracketType {
    Paren,
    Square,
    Curly,
    Pipe,
}

impl BracketType {
    pub fn left(&self) -> &str {
        match self {
            Self::Paren => "(",
            Self::Square => "[",
            Self::Curly => "\\{",
            Self::Pipe => "|",
        }
    }

    pub fn right(&self) -> &str {
        match self {
            Self::Paren => ")",
            Self::Square => "]",
            Self::Curly => "\\}",
            Self::Pipe => "|",
        }
    }
}

#[derive(Debug)]
pub enum LatexNode {
    Group {
        content: Latex,
    },
    Sqrt {
        index: Option<Latex>,
        radicand: Latex,
    },
    Frac {
        numerator: Latex,
        denominator: Latex,
    },
    Subscript {
        content: Latex,
    },
    Superscript {
        content: Latex,
    },
    Left {
        bracket_type: BracketType,
    },
    Right {
        bracket_type: BracketType,
    },
    OperatorName {
        content: String,
    },
    Escape {
        value: String,
    },
    Symbol {
        value: char,
    },
    Symbols {
        value: String,
    },
}

impl LatexNode {
    fn last_char_is_alphabetic(&self) -> bool {
        match self {
            Self::Group { .. } => false,
            Self::Sqrt { .. } => false,
            Self::Frac { .. } => false,
            Self::Subscript { .. } => false,
            Self::Superscript { .. } => false,
            Self::Left { .. } => false,
            Self::Right { .. } => false,
            Self::OperatorName { .. } => false,
            Self::Escape { value } => value.ends_with(|c: char| c.is_alphabetic()),
            Self::Symbol { value } => value.is_alphabetic(),
            Self::Symbols { value } => value.ends_with(|c: char| c.is_alphabetic()),
        }
    }
}

#[derive(Debug)]
pub struct Latex {
    nodes: Vec<LatexNode>,
}

impl Latex {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn from_nodes(nodes: Vec<LatexNode>) -> Self {
        Self { nodes }
    }

    pub fn add(mut self, mut latex: Latex) -> Self {
        if self.nodes.is_empty() {
            self.nodes = latex.nodes;
        } else {
            self.nodes.append(&mut latex.nodes);
        }
        self
    }

    pub fn add_node(mut self, node: LatexNode) -> Self {
        self.nodes.push(node);
        self
    }

    pub fn add_group(mut self, content: Latex) -> Self {
        self.nodes.push(LatexNode::Group { content });
        self
    }

    pub fn add_sqrt(mut self, index: Option<Latex>, radicand: Latex) -> Self {
        self.nodes.push(LatexNode::Sqrt { index, radicand });
        self
    }

    pub fn add_frac(mut self, numerator: Latex, denominator: Latex) -> Self {
        self.nodes.push(LatexNode::Frac { numerator, denominator });
        self
    }

    pub fn add_superscript(mut self, content: Latex) -> Self {
        self.nodes.push(LatexNode::Superscript { content });
        self
    }

    pub fn add_subscript(mut self, content: Latex) -> Self {
        self.nodes.push(LatexNode::Subscript { content });
        self
    }

    pub fn add_left(mut self, bracket_type: BracketType) -> Self {
        self.nodes.push(LatexNode::Left { bracket_type });
        self
    }

    pub fn add_right(mut self, bracket_type: BracketType) -> Self {
        self.nodes.push(LatexNode::Right { bracket_type });
        self
    }

    pub fn add_operator_name(mut self, content: String) -> Self {
        self.nodes.push(LatexNode::OperatorName { content });
        self
    }

    pub fn add_escape(mut self, value: String) -> Self {
        self.nodes.push(LatexNode::Escape { value });
        self
    }

    pub fn add_symbol(mut self, value: char) -> Self {
        self.nodes.push(LatexNode::Symbol { value });
        self
    }

    pub fn add_symbols(mut self, value: String) -> Self {
        self.nodes.push(LatexNode::Symbols { value });
        self
    }
}

impl std::fmt::Display for Latex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut end_is_alphabetic = false;

        for node in &self.nodes {
            match node {
                LatexNode::Group { content } => {
                    write!(f, "{{{content}}}")?;
                },
                LatexNode::Sqrt { index: Some(index), radicand } => {
                    write!(f, "\\sqrt[{index}]{{{radicand}}}")?;
                },
                LatexNode::Sqrt { index: None, radicand } => {
                    write!(f, "\\sqrt{{{radicand}}}")?;
                },
                LatexNode::Frac { numerator, denominator } => {
                    write!(f, "\\frac{{{numerator}}}{{{denominator}}}")?;
                },
                LatexNode::Superscript { content } => {
                    write!(f, "^{{{content}}}")?;
                },
                LatexNode::Subscript { content } => {
                    write!(f, "_{{{content}}}")?;
                },
                LatexNode::Left { bracket_type } => {
                    write!(f, "\\left{}", bracket_type.left())?;
                },
                LatexNode::Right { bracket_type } => {
                    write!(f, "\\right{}", bracket_type.right())?;
                },
                LatexNode::OperatorName { content } => {
                    write!(f, "\\operatorname{{{content}}}")?;
                },
                LatexNode::Escape { value } => {
                    write!(f, "\\{value}")?;
                },
                LatexNode::Symbol { value } => match *value {
                    '&' | '%' | '$' | '#' | '{' | '}' => write!(f, "\\{value}")?,
                    '~' => write!(f, "\\sim")?,
                    c if end_is_alphabetic && c.is_alphabetic() => write!(f, " {value}")?,
                    _ => write!(f, "{value}")?,
                },
                LatexNode::Symbols { value } => if end_is_alphabetic && value.starts_with(|c: char| c.is_alphabetic()) {
                    write!(f, " {value}")?;
                } else {
                    write!(f, "{value}")?;
                },
            }

            end_is_alphabetic = node.last_char_is_alphabetic();
        }

        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub enum InequalityType {
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
}

impl InequalityType {
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

#[derive(Debug)]
pub enum SyntaxNode {
    Equality(Box<SyntaxNode>, Box<SyntaxNode>),
    Inequality(Box<SyntaxNode>, InequalityType, Box<SyntaxNode>),
    InequalityChain(Box<SyntaxNode>, InequalityType, Box<SyntaxNode>, Vec<(InequalityType, SyntaxNode)>),
    Regression(Box<SyntaxNode>, Box<SyntaxNode>),
    Pos(Box<SyntaxNode>),
    Neg(Box<SyntaxNode>),
    Add(Box<SyntaxNode>, Box<SyntaxNode>),
    Sub(Box<SyntaxNode>, Box<SyntaxNode>),
    Mul(Box<SyntaxNode>, Box<SyntaxNode>),
    DotMul(Box<SyntaxNode>, Box<SyntaxNode>),
    CrossMul(Box<SyntaxNode>, Box<SyntaxNode>),
    Div(Box<SyntaxNode>, Box<SyntaxNode>),
    Factorial(Box<SyntaxNode>),
    Call(Box<SyntaxNode>, Box<SyntaxNode>),
    ImplicitCall(Box<SyntaxNode>, Box<SyntaxNode>),
    Index(Box<SyntaxNode>, Box<SyntaxNode>),
    Paren(Box<SyntaxNode>),
    List(Box<SyntaxNode>),
    Pipes(Box<SyntaxNode>),
    Subscript(Box<SyntaxNode>, Box<SyntaxNode>),
    Superscript(Box<SyntaxNode>, Box<SyntaxNode>),
    Prime(Box<SyntaxNode>),
    Sequence(Vec<SyntaxNode>),
    Sqrt(Box<SyntaxNode>),
    NthRoot(Box<SyntaxNode>, Box<SyntaxNode>),
    Frac(Box<SyntaxNode>, Box<SyntaxNode>),
    Derivative(Box<SyntaxNode>, Box<SyntaxNode>),
    Integral(Box<SyntaxNode>, Box<SyntaxNode>, Box<SyntaxNode>, Option<Box<SyntaxNode>>),
    Sum(Box<SyntaxNode>, Box<SyntaxNode>, Box<SyntaxNode>),
    Product(Box<SyntaxNode>, Box<SyntaxNode>, Box<SyntaxNode>),
    Piecewise(Box<SyntaxNode>),
    Colon(Box<SyntaxNode>, Box<SyntaxNode>),
    Ellipsis(Box<SyntaxNode>, Option<Box<SyntaxNode>>),
    For(Box<SyntaxNode>, Box<SyntaxNode>),
    With(Box<SyntaxNode>, Box<SyntaxNode>),
    Dot(Box<SyntaxNode>, Box<SyntaxNode>),
    PercentOf(Box<SyntaxNode>, Box<SyntaxNode>),
    RightArrow(Box<SyntaxNode>, Box<SyntaxNode>),
    MixedNumber(Box<SyntaxNode>, Box<SyntaxNode>, Box<SyntaxNode>),
    ImplicitMul(Box<SyntaxNode>, Box<SyntaxNode>),
    Letter(char),
    Decimal(f64),
    Command(String),
    Alphanumeric(String),
}

impl SyntaxNode {
    pub fn to_latex(&self) -> Latex {
        match self {
            Self::Equality(lhs, rhs) => lhs.to_latex().add_symbol('=').add(rhs.to_latex()),
            Self::Inequality(lhs, inequality, rhs) => lhs.to_latex().add_node(inequality.to_latex_node()).add(rhs.to_latex()),
            Self::InequalityChain(lhs, inequality, rhs, chain) => {
                let mut latex = lhs.to_latex().add_node(inequality.to_latex_node()).add(rhs.to_latex());
                for (inequality, value) in chain {
                    latex = latex.add_node(inequality.to_latex_node()).add(value.to_latex());
                }
                latex
            },
            Self::Regression(lhs, rhs) => lhs.to_latex().add_symbol('~').add(rhs.to_latex()),
            Self::Pos(value) => Latex::new().add_symbol('+').add(value.to_latex()),
            Self::Neg(value) => Latex::new().add_symbol('-').add(value.to_latex()),
            Self::Add(lhs, rhs) => lhs.to_latex().add_symbol('+').add(rhs.to_latex()),
            Self::Sub(lhs, rhs) => lhs.to_latex().add_symbol('-').add(rhs.to_latex()),
            Self::Mul(lhs, rhs) => lhs.to_latex().add_symbol('*').add(rhs.to_latex()),
            Self::DotMul(lhs, rhs) => lhs.to_latex().add_escape(String::from("cdot")).add(rhs.to_latex()),
            Self::CrossMul(lhs, rhs) => lhs.to_latex().add_escape(String::from("cross")).add(rhs.to_latex()),
            Self::Div(lhs, rhs) => lhs.to_latex().add_symbol('/').add(rhs.to_latex()),
            Self::Factorial(value) => value.to_latex().add_symbol('!'),
            Self::Call(callee, args) => callee.to_latex().add_left(BracketType::Paren).add(args.to_latex()).add_right(BracketType::Paren),
            Self::ImplicitCall(callee, arg) => callee.to_latex().add(arg.to_latex()),
            Self::Index(indexee, index) => indexee.to_latex().add_left(BracketType::Square).add(index.to_latex()).add_right(BracketType::Square),
            Self::Paren(content) => Latex::new().add_left(BracketType::Paren).add(content.to_latex()).add_right(BracketType::Paren),
            Self::List(content) => Latex::new().add_left(BracketType::Square).add(content.to_latex()).add_right(BracketType::Square),
            Self::Pipes(content) => Latex::new().add_left(BracketType::Pipe).add(content.to_latex()).add_right(BracketType::Pipe),
            Self::Subscript(base, script) => base.to_latex().add_subscript(script.to_latex()),
            Self::Superscript(base, script) => base.to_latex().add_superscript(script.to_latex()),
            Self::Prime(value) => value.to_latex().add_symbol('\''),
            Self::Sequence(elements) => if let Some(first) = elements.get(0) {
                let mut latex = first.to_latex();
                if let Some(others) = elements.get(1..) {
                    for element in others {
                        latex = latex.add_symbol(',').add(element.to_latex());
                    }
                }
                latex
            } else {
                Latex::new()
            },
            Self::Sqrt(radicand) => Latex::new().add_sqrt(None, radicand.to_latex()),
            Self::NthRoot(index, radicand) => Latex::new().add_sqrt(Some(index.to_latex()), radicand.to_latex()),
            Self::Frac(numerator, denominator) => Latex::new().add_frac(numerator.to_latex(), denominator.to_latex()),
            Self::Derivative(differential, body) => Latex::new().add_frac(Latex::new().add_symbol('d'), Latex::new().add_symbol('d').add(differential.to_latex())).add(body.to_latex()),
            Self::Integral(differential, from, to, body) => if let Some(body) = body {
                Latex::new().add_escape(String::from("int")).add_subscript(from.to_latex()).add_superscript(to.to_latex()).add(body.to_latex()).add_symbol('d').add(differential.to_latex())
            } else {
                Latex::new().add_escape(String::from("int")).add_subscript(from.to_latex()).add_superscript(to.to_latex()).add_symbol('d').add(differential.to_latex())
            },
            Self::Sum(bottom, top, body) => Latex::new().add_escape(String::from("sum")).add_subscript(bottom.to_latex()).add_superscript(top.to_latex()).add(body.to_latex()),
            Self::Product(bottom, top, body) => Latex::new().add_escape(String::from("prod")).add_subscript(bottom.to_latex()).add_superscript(top.to_latex()).add(body.to_latex()),
            Self::Piecewise(content) => Latex::new().add_left(BracketType::Curly).add(content.to_latex()).add_right(BracketType::Curly),
            Self::Colon(lhs, rhs) => lhs.to_latex().add_symbol(':').add(rhs.to_latex()),
            Self::Ellipsis(lhs, rhs) => if let Some(rhs) = rhs {
                lhs.to_latex().add_symbols(String::from("...")).add(rhs.to_latex())
            } else {
                lhs.to_latex().add_symbols(String::from("..."))
            },
            Self::For(lhs, rhs) => lhs.to_latex().add_operator_name(String::from("for")).add(rhs.to_latex()),
            Self::With(lhs, rhs) => lhs.to_latex().add_operator_name(String::from("with")).add(rhs.to_latex()),
            Self::Dot(lhs, rhs) => lhs.to_latex().add_symbol('.').add(rhs.to_latex()),
            Self::PercentOf(lhs, rhs) => lhs.to_latex().add_symbol('%').add_operator_name(String::from("of")).add(rhs.to_latex()),
            Self::RightArrow(lhs, rhs) => lhs.to_latex().add_escape(String::from("to")).add(rhs.to_latex()),
            Self::MixedNumber(whole, numerator, denominator) => whole.to_latex().add_frac(numerator.to_latex(), denominator.to_latex()),
            Self::ImplicitMul(lhs, rhs) => lhs.to_latex().add(rhs.to_latex()),
            Self::Letter(letter) => Latex::new().add_symbol(*letter),
            Self::Decimal(number) => if number.is_nan() {
                Latex::new().add_frac(Latex::new().add_symbol('0'), Latex::new().add_symbol('0'))
            } else if number.is_infinite() {
                Latex::new().add_escape(String::from("infty"))
            } else {
                Latex::new().add_symbols(number.to_string())
            },
            Self::Command(name) => Latex::new().add_operator_name(name.clone()),
            Self::Alphanumeric(value) => Latex::new().add_symbols(value.clone()),
        }
    }
}