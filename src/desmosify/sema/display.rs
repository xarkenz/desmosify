use std::rc::Rc;
use crate::sema::values::{ActionValue, ValueRegistryEntry};

#[derive(Clone, PartialEq, Debug)]
pub struct ImageValue {
    pub url: Rc<str>,
    pub name: Rc<str>,
    pub center: ValueRegistryEntry,
    pub width: ValueRegistryEntry,
    pub height: ValueRegistryEntry,
    pub opacity: ValueRegistryEntry,
    pub angle: ValueRegistryEntry,
    pub background: bool,
}

#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub enum PointStyle {
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

impl PointStyle {
    pub const NAMES: &'static [&'static str] = &[
        "point",
        "open",
        "cross",
        "square",
        "plus",
        "triangle",
        "diamond",
        "star",
    ];

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

#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub enum DragMode {
    #[default]
    None,
    X,
    Y,
    XY,
    Auto,
}

impl DragMode {
    pub const NAMES: &'static [&'static str] = &[
        "none",
        "x",
        "y",
        "xy",
        "auto",
    ];

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "none" | "NONE" => Some(Self::None),
            "x" | "X" => Some(Self::X),
            "y" | "Y" => Some(Self::Y),
            "xy" | "XY" => Some(Self::XY),
            "auto" | "AUTO" => Some(Self::Auto),
            _ => None
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::None => "NONE",
            Self::X => "X",
            Self::Y => "Y",
            Self::XY => "XY",
            Self::Auto => "AUTO",
        }
    }
}

#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub enum LabelOrientation {
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

impl LabelOrientation {
    pub const NAMES: &'static [&'static str] = &[
        "default",
        "center",
        "center_auto",
        "auto_center",
        "above",
        "above_left",
        "above_right",
        "above_auto",
        "below",
        "below_left",
        "below_right",
        "below_auto",
        "left",
        "auto_left",
        "right",
        "auto_right",
    ];

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

#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub enum LineStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
}

impl LineStyle {
    pub const NAMES: &'static [&'static str] = &[
        "solid",
        "dashed",
        "dotted",
    ];

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

#[derive(Clone, Debug)]
pub enum ProgramDisplayAttributeKind {
    Color {
        value: ValueRegistryEntry,
    },
    Point {
        opacity: Option<ValueRegistryEntry>,
        size: Option<ValueRegistryEntry>,
        style: PointStyle,
        outline: bool,
    },
    Drag {
        mode: DragMode,
    },
    Label {
        text: Rc<str>,
        opacity: Option<ValueRegistryEntry>,
        size: Option<ValueRegistryEntry>,
        angle: Option<ValueRegistryEntry>,
        orientation: LabelOrientation,
        outline: bool,
    },
    Line {
        opacity: Option<ValueRegistryEntry>,
        width: Option<ValueRegistryEntry>,
        style: LineStyle,
    },
    Fill {
        opacity: Option<ValueRegistryEntry>,
    },
    Click {
        action: ActionValue,
    },
    Hovered {
        url: Rc<str>,
    },
    Pressed {
        url: Rc<str>,
    },
    Description {
        text: Rc<str>,
    },
}

#[derive(Clone, Debug)]
pub struct ProgramDisplayAttribute {
    pub kind: ProgramDisplayAttributeKind,
    pub key_span: Option<crate::Span>,
}

#[derive(Clone, Debug)]
pub struct ProgramDisplayElement {
    pub value: ValueRegistryEntry,
    pub span: Option<crate::Span>,
    pub attributes: Box<[ProgramDisplayAttribute]>,
}

#[derive(Clone, Debug)]
pub struct ProgramDisplayList {
    pub elements: Box<[ProgramDisplayElement]>,
}
