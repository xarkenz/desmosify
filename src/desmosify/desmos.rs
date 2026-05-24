use json::JsonValue;
use crate::desmos::latex::{BracketType, Latex, LatexNode};

pub mod latex;
pub mod target;
pub mod symbol;

pub trait ToJson {
    fn to_json(&self) -> JsonValue;
}

pub trait GraphEntry : ToJson + std::fmt::Debug {
    fn type_name(&self) -> &str;
    fn id(&self) -> &str;
}

#[derive(Debug)]
pub struct GraphFolderEntry {
    pub id: String,
    pub title: String,
    pub collapsed: bool,
    pub secret: bool,
}

impl ToJson for GraphFolderEntry {
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

impl GraphEntry for GraphFolderEntry {
    fn type_name(&self) -> &str {
        "folder"
    }

    fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Copy, Clone, Debug)]
pub enum GraphColor {
    IntRgb(u8, u8, u8),
}

impl GraphColor {
    pub const RED: Self = Self::IntRgb(0xC7, 0x44, 0x40);
    pub const BLUE: Self = Self::IntRgb(0x2D, 0x70, 0xB3);
    pub const GREEN: Self = Self::IntRgb(0x38, 0x8C, 0x46);
    pub const PURPLE: Self = Self::IntRgb(0x60, 0x42, 0xA6);
    pub const ORANGE: Self = Self::IntRgb(0xFA, 0x7E, 0x19);
    pub const BLACK: Self = Self::IntRgb(0x00, 0x00, 0x00);
}

impl std::fmt::Display for GraphColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IntRgb(r, g, b) => {
                write!(f, "#{r:02x}{g:02x}{b:02x}")
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub enum GraphSliderLoopMode {
    #[default]
    LoopForwardReverse,
    LoopForward,
    PlayOnce,
    PlayIndefinitely,
}

impl GraphSliderLoopMode {
    pub fn name(&self) -> &'static str {
        match self {
            Self::LoopForwardReverse => "LOOP_FORWARD_REVERSE",
            Self::LoopForward => "LOOP_FORWARD",
            Self::PlayOnce => "PLAY_ONCE",
            Self::PlayIndefinitely => "PLAY_INDEFINITELY",
        }
    }
}

#[derive(Debug)]
pub struct GraphSlider {
    pub loop_mode: GraphSliderLoopMode,
    pub animation_period: Option<f64>,
    pub is_playing: bool,
    pub min: GraphExpression,
    pub max: GraphExpression,
    pub step: GraphExpression,
}

impl ToJson for GraphSlider {
    fn to_json(&self) -> JsonValue {
        let mut object = json::object!{
            "loopMode": self.loop_mode.name(),
        };
        if let Some(animation_period) = self.animation_period {
            object["animationPeriod"] = animation_period.into();
        }
        if self.is_playing {
            object["isPlaying"] = true.into();
        }
        if !self.min.is_empty() {
            object["min"] = self.min.to_latex().to_string().into();
            object["hardMin"] = true.into();
        }
        if !self.max.is_empty() {
            object["max"] = self.max.to_latex().to_string().into();
            object["hardMax"] = true.into();
        }
        if !self.step.is_empty() {
            object["step"] = self.step.to_latex().to_string().into();
        }
        object
    }
}

impl Default for GraphSlider {
    fn default() -> Self {
        Self {
            loop_mode: Default::default(),
            animation_period: None,
            is_playing: false,
            min: Default::default(),
            max: Default::default(),
            step: Default::default(),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub enum GraphPointStyle {
    #[default]
    Point,
    Open,
    Cross,
    Square,
    Plus,
    Triangle,
    Diamond,
    Star,
}

impl GraphPointStyle {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "point" | "POINT" => Some(Self::Point),
            "open" | "OPEN" => Some(Self::Open),
            "cross" | "CROSS" => Some(Self::Cross),
            "square" | "SQUARE" => Some(Self::Square),
            "plus" | "PLUS" => Some(Self::Plus),
            "triangle" | "TRIANGLE" => Some(Self::Triangle),
            "diamond" | "DIAMOND" => Some(Self::Diamond),
            "star" | "STAR" => Some(Self::Star),
            _ => None
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Point => "POINT",
            Self::Open => "OPEN",
            Self::Cross => "CROSS",
            Self::Square => "SQUARE",
            Self::Plus => "PLUS",
            Self::Triangle => "TRIANGLE",
            Self::Diamond => "DIAMOND",
            Self::Star => "STAR",
        }
    }
}

#[derive(Debug)]
pub struct GraphPointInfo {
    pub display: bool,
    pub opacity: GraphExpression,
    pub size: GraphExpression,
    pub style: GraphPointStyle,
    pub outline: bool,
}

impl GraphPointInfo {
    pub fn add_fields(&self, object: &mut JsonValue) {
        object["points"] = self.display.into();
        if !self.opacity.is_empty() {
            object["pointOpacity"] = self.opacity.to_latex().to_string().into();
        }
        if !self.size.is_empty() {
            object["pointSize"] = self.size.to_latex().to_string().into();
            object["movablePointSize"] = object["pointSize"].clone();
        }
        if self.style != Default::default() {
            object["pointStyle"] = self.style.name().into();
        }
        if self.outline {
            object["pointOutline"] = true.into();
        }
    }
}

impl Default for GraphPointInfo {
    fn default() -> Self {
        Self {
            display: false,
            opacity: Default::default(),
            size: Default::default(),
            style: Default::default(),
            outline: false,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub enum GraphLabelOrientation {
    #[default]
    Default,
    Center,
    CenterAuto,
    AutoCenter,
    Above,
    AboveLeft,
    AboveRight,
    AboveAuto,
    Below,
    BelowLeft,
    BelowRight,
    BelowAuto,
    Left,
    AutoLeft,
    Right,
    AutoRight,
}

impl GraphLabelOrientation {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "default" => Some(Self::Default),
            "center" => Some(Self::Center),
            "center_auto" => Some(Self::CenterAuto),
            "auto_center" => Some(Self::AutoCenter),
            "above" => Some(Self::Above),
            "above_left" => Some(Self::AboveLeft),
            "above_right" => Some(Self::AboveRight),
            "above_auto" => Some(Self::AboveAuto),
            "below" => Some(Self::Below),
            "below_left" => Some(Self::BelowLeft),
            "below_right" => Some(Self::BelowRight),
            "below_auto" => Some(Self::BelowAuto),
            "left" => Some(Self::Left),
            "auto_left" => Some(Self::AutoLeft),
            "right" => Some(Self::Right),
            "auto_right" => Some(Self::AutoRight),
            _ => None
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Center => "center",
            Self::CenterAuto => "center_auto",
            Self::AutoCenter => "auto_center",
            Self::Above => "above",
            Self::AboveLeft => "above_left",
            Self::AboveRight => "above_right",
            Self::AboveAuto => "above_auto",
            Self::Below => "below",
            Self::BelowLeft => "below_left",
            Self::BelowRight => "below_right",
            Self::BelowAuto => "below_auto",
            Self::Left => "left",
            Self::AutoLeft => "auto_left",
            Self::Right => "right",
            Self::AutoRight => "auto_right",
        }
    }
}

#[derive(Debug)]
pub struct GraphLabelInfo {
    pub display: bool,
    pub text: String,
    pub opacity: GraphExpression,
    pub size: GraphExpression,
    pub angle: GraphExpression,
    pub orientation: GraphLabelOrientation,
    pub outline: bool,
}

impl GraphLabelInfo {
    pub fn add_fields(&self, object: &mut JsonValue) {
        object["showLabel"] = self.display.into();
        if !self.text.is_empty() {
            object["label"] = self.text.as_str().into();
        }
        if !self.opacity.is_empty() {
            // Wish you could control the label opacity separately...
            object["pointOpacity"] = self.opacity.to_latex().to_string().into();
        }
        if !self.size.is_empty() {
            object["labelSize"] = self.size.to_latex().to_string().into();
        }
        if !self.angle.is_empty() {
            object["labelAngle"] = self.angle.to_latex().to_string().into();
        }
        if self.orientation != Default::default() {
            object["labelOrientation"] = self.orientation.name().into();
        }
        if !self.outline {
            object["suppressTextOutline"] = true.into();
        }
    }
}

impl Default for GraphLabelInfo {
    fn default() -> Self {
        Self {
            display: false,
            text: String::new(),
            opacity: Default::default(),
            size: Default::default(),
            angle: Default::default(),
            orientation: Default::default(),
            outline: true,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub enum GraphLineStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
}

impl GraphLineStyle {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "solid" | "SOLID" => Some(Self::Solid),
            "dashed" | "DASHED" => Some(Self::Dashed),
            "dotted" | "DOTTED" => Some(Self::Dotted),
            _ => None
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Solid => "SOLID",
            Self::Dashed => "DASHED",
            Self::Dotted => "DOTTED",
        }
    }
}

#[derive(Debug)]
pub struct GraphLineInfo {
    pub display: bool,
    pub opacity: GraphExpression,
    pub width: GraphExpression,
    pub style: GraphLineStyle,
}

impl GraphLineInfo {
    pub fn add_fields(&self, object: &mut JsonValue) {
        object["lines"] = self.display.into();
        if !self.opacity.is_empty() {
            object["lineOpacity"] = self.opacity.to_latex().to_string().into();
        }
        if !self.width.is_empty() {
            object["lineWidth"] = self.width.to_latex().to_string().into();
        }
        if self.style != Default::default() {
            object["lineStyle"] = self.style.name().into();
        }
    }
}

impl Default for GraphLineInfo {
    fn default() -> Self {
        Self {
            display: false,
            opacity: Default::default(),
            width: Default::default(),
            style: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct GraphFillInfo {
    pub display: bool,
    pub opacity: GraphExpression,
}

impl GraphFillInfo {
    pub fn add_fields(&self, object: &mut JsonValue) {
        object["fill"] = self.display.into();
        if !self.opacity.is_empty() {
            object["fillOpacity"] = self.opacity.to_latex().to_string().into();
        }
    }
}

impl Default for GraphFillInfo {
    fn default() -> Self {
        Self {
            display: false,
            opacity: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct GraphClickableInfo {
    pub enabled: bool,
    pub description: String,
    pub expression: GraphExpression,
}

impl GraphClickableInfo {
    pub fn add_fields(&self, object: &mut JsonValue) {
        if !self.description.is_empty() {
            object["description"] = self.description.as_str().into();
        }
        if self.enabled || !self.expression.is_empty() {
            let mut clickable_info = json::object!{};
            if self.enabled {
                clickable_info["enabled"] = true.into();
            }
            if !self.expression.is_empty() {
                clickable_info["latex"] = self.expression.to_latex().to_string().into();
            }
            object["clickableInfo"] = clickable_info;
        }
    }
}

impl Default for GraphClickableInfo {
    fn default() -> Self {
        Self {
            enabled: false,
            description: String::new(),
            expression: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct GraphExpressionEntry {
    pub id: String,
    pub folder_id: Option<String>,
    pub expression: GraphExpression,
    pub display: bool,
    pub color: Option<GraphColor>,
    pub color_expression: GraphExpression,
    pub slider: Option<GraphSlider>,
    pub point: GraphPointInfo,
    pub label: GraphLabelInfo,
    pub line: GraphLineInfo,
    pub fill: GraphFillInfo,
    pub clickable: GraphClickableInfo,
}

impl Default for GraphExpressionEntry {
    fn default() -> Self {
        Self {
            id: String::new(),
            folder_id: None,
            expression: Default::default(),
            display: false,
            color: None,
            color_expression: Default::default(),
            slider: None,
            point: Default::default(),
            label: Default::default(),
            line: Default::default(),
            fill: Default::default(),
            clickable: Default::default(),
        }
    }
}

impl ToJson for GraphExpressionEntry {
    fn to_json(&self) -> JsonValue {
        let mut object = json::object!{
            "type": self.type_name(),
            "id": self.id(),
            "hidden": !self.display,
        };
        if let Some(folder_id) = &self.folder_id {
            object["folderId"] = folder_id.as_str().into();
        }
        if !self.expression.is_empty() {
            object["latex"] = self.expression.to_latex().to_string().into();
        }
        if let Some(color) = &self.color {
            object["color"] = color.to_string().into();
        }
        if !self.color_expression.is_empty() {
            object["colorLatex"] = self.color_expression.to_latex().to_string().into();
        }
        if let Some(slider) = &self.slider {
            object["slider"] = slider.to_json();
        }
        self.point.add_fields(&mut object);
        self.label.add_fields(&mut object);
        self.line.add_fields(&mut object);
        self.fill.add_fields(&mut object);
        self.clickable.add_fields(&mut object);
        object
    }
}

impl GraphEntry for GraphExpressionEntry {
    fn type_name(&self) -> &str {
        "expression"
    }

    fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Debug)]
pub struct GraphImageClickableInfo {
    pub enabled: bool,
    pub description: String,
    pub expression: GraphExpression,
    pub hovered_image_url: String,
    pub depressed_image_url: String,
}

impl GraphImageClickableInfo {
    pub fn add_fields(&self, object: &mut JsonValue) {
        if !self.description.is_empty() {
            object["description"] = self.description.as_str().into();
        }
        if self.enabled || !self.expression.is_empty() || !self.hovered_image_url.is_empty() || !self.depressed_image_url.is_empty() {
            let mut clickable_info = json::object!{};
            if self.enabled {
                clickable_info["enabled"] = true.into();
            }
            if !self.expression.is_empty() {
                clickable_info["latex"] = self.expression.to_latex().to_string().into();
            }
            if !self.hovered_image_url.is_empty() {
                clickable_info["hoveredImage"] = self.hovered_image_url.as_str().into();
            }
            if !self.depressed_image_url.is_empty() {
                clickable_info["depressedImage"] = self.depressed_image_url.as_str().into();
            }
            object["clickableInfo"] = clickable_info;
        }
    }
}

impl Default for GraphImageClickableInfo {
    fn default() -> Self {
        Self {
            enabled: false,
            description: String::new(),
            expression: Default::default(),
            hovered_image_url: String::new(),
            depressed_image_url: String::new(),
        }
    }
}

#[derive(Debug)]
pub struct GraphImageEntry {
    pub id: String,
    pub folder_id: Option<String>,
    pub image_url: String,
    pub name: String,
    pub foreground: bool,
    pub center: GraphExpression,
    pub width: GraphExpression,
    pub height: GraphExpression,
    pub opacity: GraphExpression,
    pub angle: GraphExpression,
    pub clickable: GraphImageClickableInfo,
}

impl ToJson for GraphImageEntry {
    fn to_json(&self) -> JsonValue {
        let mut object = json::object!{
            "type": self.type_name(),
            "id": self.id(),
            "name": self.name.as_str(),
            "foreground": self.foreground,
        };
        if let Some(folder_id) = &self.folder_id {
            object["folderId"] = folder_id.as_str().into();
        }
        if !self.image_url.is_empty() {
            // Suddenly snake case for some reason?
            object["image_url"] = self.image_url.as_str().into();
        }
        if !self.center.is_empty() {
            object["center"] = self.center.to_latex().to_string().into();
        }
        if !self.width.is_empty() {
            object["width"] = self.width.to_latex().to_string().into();
        }
        if !self.height.is_empty() {
            object["height"] = self.height.to_latex().to_string().into();
        }
        if !self.opacity.is_empty() {
            object["opacity"] = self.opacity.to_latex().to_string().into();
        }
        if !self.angle.is_empty() {
            object["angle"] = self.angle.to_latex().to_string().into();
        }
        self.clickable.add_fields(&mut object);
        object
    }
}

impl GraphEntry for GraphImageEntry {
    fn type_name(&self) -> &str {
        "image"
    }

    fn id(&self) -> &str {
        &self.id
    }
}

impl Default for GraphImageEntry {
    fn default() -> Self {
        Self {
            id: String::new(),
            folder_id: None,
            image_url: String::new(),
            name: String::new(),
            foreground: true,
            center: Default::default(),
            width: Default::default(),
            height: Default::default(),
            opacity: Default::default(),
            angle: Default::default(),
            clickable: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct GraphTextEntry {
    pub id: String,
    pub folder_id: Option<String>,
    pub text: String,
}

impl ToJson for GraphTextEntry {
    fn to_json(&self) -> JsonValue {
        let mut object = json::object!{
            "type": self.type_name(),
            "id": self.id(),
            "text": self.text.as_str(),
        };
        if let Some(folder_id) = &self.folder_id {
            object["folderId"] = folder_id.as_str().into();
        }
        object
    }
}

impl GraphEntry for GraphTextEntry {
    fn type_name(&self) -> &str {
        "text"
    }

    fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Debug)]
pub struct GraphTicker {
    pub playing: bool,
    pub handler: GraphExpression,
    pub min_step: GraphExpression,
}

impl ToJson for GraphTicker {
    fn to_json(&self) -> JsonValue {
        let mut object = json::object!{
            "open": true,
            "playing": self.playing,
        };
        if !self.handler.is_empty() {
            object["handlerLatex"] = self.handler.to_latex().to_string().into();
        }
        if !self.min_step.is_empty() {
            object["minStepLatex"] = self.min_step.to_latex().to_string().into();
        }
        object
    }
}

#[derive(Debug)]
pub struct GraphExpressionList {
    pub entries: Vec<Box<dyn GraphEntry>>,
    pub ticker: Option<GraphTicker>,
    pub next_local_id: u64,
}

impl ToJson for GraphExpressionList {
    fn to_json(&self) -> JsonValue {
        let entries: Vec<_> = self.entries
            .iter()
            .map(|entry| entry.to_json())
            .collect();
        let mut object = json::object!{
            "list": entries,
        };
        if let Some(ticker) = &self.ticker {
            object["ticker"] = ticker.to_json();
        }
        object
    }
}

#[derive(Debug)]
pub struct GraphSettings {
    pub product_name: String,
    pub show_grid: bool,
    pub show_x_axis: bool,
    pub show_y_axis: bool,
    pub viewport_x_min: f64,
    pub viewport_y_min: f64,
    pub viewport_x_max: f64,
    pub viewport_y_max: f64,
}

impl ToJson for GraphSettings {
    fn to_json(&self) -> JsonValue {
        json::object!{
            "product": self.product_name.as_str(),
            "showGrid": self.show_grid,
            "showXAxis": self.show_x_axis,
            "showYAxis": self.show_y_axis,
            "viewport": {
                "xmin": self.viewport_x_min,
                "ymin": self.viewport_y_min,
                "xmax": self.viewport_x_max,
                "ymax": self.viewport_y_max,
            },
        }
    }
}

#[derive(Debug)]
pub struct GraphState {
    pub version: i32,
    pub graph: GraphSettings,
    pub expressions: GraphExpressionList,
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
pub enum GraphInequalityKind {
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,
}

impl GraphInequalityKind {
    pub fn to_latex_node(&self) -> LatexNode {
        LatexNode::Escape {
            value: String::from(match self {
                Self::LessThan => "lt",
                Self::GreaterThan => "gt",
                Self::LessEqual => "le",
                Self::GreaterEqual => "ge",
            }),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum GraphUnaryKind {
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
pub enum GraphBinaryKind {
    Equal,
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
    Range,
    Dot,
    PercentOf,
    RightArrow,
}

#[derive(Clone, Default, Debug)]
pub enum GraphExpression {
    #[default]
    Empty,
    Letter(char),
    Integer(i64),
    Decimal(f64),
    OperatorName(String),
    Escape(String),
    Alphanumeric(String),
    Unary {
        kind: GraphUnaryKind,
        inner: Box<Self>,
    },
    Binary {
        kind: GraphBinaryKind,
        lhs: Box<Self>,
        rhs: Box<Self>,
    },
    InequalityChain {
        lhs: Box<Self>,
        first_kind: GraphInequalityKind,
        rhs: Box<Self>,
        chain: Vec<(GraphInequalityKind, Self)>,
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
        body: Box<Self>,
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
    MixedNumber {
        whole: Box<Self>,
        numerator: Box<Self>,
        denominator: Box<Self>,
    },
}

impl GraphExpression {
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    pub fn to_latex(&self) -> Latex {
        match self {
            Self::Empty => {
                Latex::new()
            }
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
            Self::OperatorName(name) => {
                Latex::new().add_operator_name(name.clone())
            }
            Self::Escape(name) => {
                Latex::new().add_escape(name.clone())
            }
            Self::Alphanumeric(value) => {
                Latex::new().add_symbols(value.clone())
            }
            Self::Unary { kind, inner } => match kind {
                GraphUnaryKind::Positive => {
                    Latex::new().add_symbol('+').add(inner.to_latex())
                }
                GraphUnaryKind::Negative => {
                    Latex::new().add_symbol('-').add(inner.to_latex())
                }
                GraphUnaryKind::Factorial => {
                    inner.to_latex().add_symbol('!')
                }
                GraphUnaryKind::Prime => {
                    inner.to_latex().add_symbol('\'')
                }
                GraphUnaryKind::Parentheses => {
                    Latex::new()
                        .add_left(BracketType::Parenthesis)
                        .add(inner.to_latex())
                        .add_right(BracketType::Parenthesis)
                }
                GraphUnaryKind::List => {
                    Latex::new()
                        .add_left(BracketType::Square)
                        .add(inner.to_latex())
                        .add_right(BracketType::Square)
                }
                GraphUnaryKind::Piecewise => {
                    Latex::new()
                        .add_left(BracketType::Curly)
                        .add(inner.to_latex())
                        .add_right(BracketType::Curly)
                }
                GraphUnaryKind::Pipes => {
                    Latex::new()
                        .add_left(BracketType::Pipe)
                        .add(inner.to_latex())
                        .add_right(BracketType::Pipe)
                }
            }
            Self::Binary { kind, lhs, rhs } => match kind {
                GraphBinaryKind::Equal => {
                    lhs.to_latex().add_symbol('=').add(rhs.to_latex())
                }
                GraphBinaryKind::Regression => {
                    lhs.to_latex().add_symbol('~').add(rhs.to_latex())
                }
                GraphBinaryKind::Add => {
                    lhs.to_latex().add_symbol('+').add(rhs.to_latex())
                }
                GraphBinaryKind::Subtract => {
                    lhs.to_latex().add_symbol('-').add(rhs.to_latex())
                }
                GraphBinaryKind::Multiply => {
                    lhs.to_latex().add_symbol('*').add(rhs.to_latex())
                }
                GraphBinaryKind::DotMultiply => {
                    lhs.to_latex().add_escape("cdot".into()).add(rhs.to_latex())
                }
                GraphBinaryKind::CrossMultiply => {
                    lhs.to_latex().add_escape("cross".into()).add(rhs.to_latex())
                }
                GraphBinaryKind::ImplicitMultiply => {
                    lhs.to_latex().add(rhs.to_latex())
                }
                GraphBinaryKind::Divide => {
                    lhs.to_latex().add_symbol('/').add(rhs.to_latex())
                }
                GraphBinaryKind::Fraction => {
                    Latex::new().add_frac(lhs.to_latex(), rhs.to_latex())
                }
                GraphBinaryKind::Call => {
                    lhs.to_latex()
                        .add_left(BracketType::Parenthesis)
                        .add(rhs.to_latex())
                        .add_right(BracketType::Parenthesis)
                }
                GraphBinaryKind::ImplicitCall => {
                    lhs.to_latex().add(rhs.to_latex())
                }
                GraphBinaryKind::Index => {
                    lhs.to_latex()
                        .add_left(BracketType::Square)
                        .add(rhs.to_latex())
                        .add_right(BracketType::Square)
                }
                GraphBinaryKind::Subscript => {
                    lhs.to_latex().add_subscript(rhs.to_latex())
                }
                GraphBinaryKind::Superscript => {
                    lhs.to_latex().add_superscript(rhs.to_latex())
                }
                GraphBinaryKind::Colon => {
                    lhs.to_latex().add_symbol(':').add(rhs.to_latex())
                }
                GraphBinaryKind::For => {
                    lhs.to_latex().add_operator_name("for".into()).add(rhs.to_latex())
                }
                GraphBinaryKind::With => {
                    lhs.to_latex().add_operator_name("with".into()).add(rhs.to_latex())
                }
                GraphBinaryKind::Range => {
                    lhs.to_latex().add_symbols("...".into()).add(rhs.to_latex())
                }
                GraphBinaryKind::Dot => {
                    lhs.to_latex().add_symbol('.').add(rhs.to_latex())
                }
                GraphBinaryKind::PercentOf => {
                    lhs.to_latex().add_symbol('%').add_operator_name("of".into()).add(rhs.to_latex())
                }
                GraphBinaryKind::RightArrow => {
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
                Latex::new()
                    .add_escape("int".into())
                    .add_subscript(lower_bound.to_latex())
                    .add_superscript(upper_bound.to_latex())
                    .add(body.to_latex())
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
            Self::MixedNumber { whole, numerator, denominator } => {
                whole.to_latex().add_frac(numerator.to_latex(), denominator.to_latex())
            }
        }
    }
}
