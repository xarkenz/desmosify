use super::*;

use std::str::FromStr;

pub trait Attribute {
    const NAME: &'static str;

    fn parse(parser: &mut Parser) -> Result<Self, DesmosifyError> where Self: Sized;
}

// TODO: images

#[derive(Copy, Clone, Debug)]
pub enum PointStyle {
    Point,
    Open,
    Cross,
}

impl PointStyle {
    pub fn parse(parser: &Parser, expression: Expression) -> Result<Self, DesmosifyError> {
        let token = parser.token()?;
        Self::from_str(&parser.get_constant_string(expression)?)
            .map_err(|message| DesmosifyError::new(
                message,
                Some(token.start),
                Some(token.end),
            ))
    }
}

impl Default for PointStyle {
    fn default() -> Self {
        Self::Point
    }
}

impl FromStr for PointStyle {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "point" => Ok(Self::Point),
            "open" => Ok(Self::Open),
            "cross" => Ok(Self::Cross),
            _ => Err(String::from("expected string 'point', 'open', or 'cross'"))
        }
    }
}

#[derive(Debug)]
pub struct PointAttribute {
    pub size_pixels: Box<Expression>,
    pub opacity: Box<Expression>,
    pub style: PointStyle,
}

impl Attribute for PointAttribute {
    const NAME: &'static str = "point";

    fn parse(parser: &mut Parser) -> Result<Self, DesmosifyError> {
        let start = parser.token()?.start;
        parser.next();
        let mut arguments = parser.parse_call()?;
        if arguments.len() < 1 || 3 < arguments.len() {
            return Err(DesmosifyError::new(
                String::from("expected 1-3 arguments for 'point' attribute"),
                Some(start),
                Some(parser.token()?.end),
            ));
        }
        let style = if arguments.len() == 3 {
            PointStyle::parse(parser, arguments.pop().unwrap())?
        } else {
            PointStyle::default()
        };
        let opacity = Box::new(if arguments.len() == 2 {
            arguments.pop().unwrap()
        } else {
            Expression::from_constant(ConstantValue::Real(1.0))
        });
        let size_pixels = Box::new(arguments.pop().unwrap());
        Ok(Self {
            opacity,
            size_pixels,
            style,
        })
    }
}

#[derive(Copy, Clone, Debug)]
pub enum StrokeStyle {
    Solid,
    Dashed,
    Dotted,
}

impl StrokeStyle {
    pub fn parse(parser: &Parser, expression: Expression) -> Result<Self, DesmosifyError> {
        let token = parser.token()?;
        Self::from_str(&parser.get_constant_string(expression)?)
            .map_err(|message| DesmosifyError::new(
                message,
                Some(token.start),
                Some(token.end),
            ))
    }
}

impl Default for StrokeStyle {
    fn default() -> Self {
        Self::Solid
    }
}

impl FromStr for StrokeStyle {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "solid" => Ok(Self::Solid),
            "dashed" => Ok(Self::Dashed),
            "dotted" => Ok(Self::Dotted),
            _ => Err(String::from("expected string 'solid', 'dashed', or 'dotted'"))
        }
    }
}

#[derive(Debug)]
pub struct StrokeAttribute {
    pub width_pixels: Box<Expression>,
    pub opacity: Box<Expression>,
    pub style: StrokeStyle,
}

impl Attribute for StrokeAttribute {
    const NAME: &'static str = "stroke";

    fn parse(parser: &mut Parser) -> Result<Self, DesmosifyError> {
        let start = parser.token()?.start;
        parser.next();
        let mut arguments = parser.parse_call()?;
        if arguments.len() < 1 || 3 < arguments.len() {
            return Err(DesmosifyError::new(
                String::from("expected 1-3 arguments for 'stroke' attribute"),
                Some(start),
                Some(parser.token()?.end),
            ));
        }
        let style = if arguments.len() == 3 {
            StrokeStyle::parse(parser, arguments.pop().unwrap())?
        } else {
            StrokeStyle::default()
        };
        let opacity = Box::new(if arguments.len() == 2 {
            arguments.pop().unwrap()
        } else {
            Expression::from_constant(ConstantValue::Real(1.0))
        });
        let width_pixels = Box::new(arguments.pop().unwrap());
        Ok(Self {
            opacity,
            width_pixels,
            style,
        })
    }
}

#[derive(Debug)]
pub struct FillAttribute {
    pub opacity: Box<Expression>,
}

impl Attribute for FillAttribute {
    const NAME: &'static str = "fill";

    fn parse(parser: &mut Parser) -> Result<Self, DesmosifyError> {
        let start = parser.token()?.start;
        parser.next();
        let mut arguments = parser.parse_call()?;
        if arguments.len() > 1 {
            return Err(DesmosifyError::new(
                String::from("expected 0-1 arguments for 'fill' attribute"),
                Some(start),
                Some(parser.token()?.end),
            ));
        }
        let opacity = Box::new(if arguments.len() == 1 {
            arguments.pop().unwrap()
        } else {
            Expression::from_constant(ConstantValue::Real(1.0))
        });
        Ok(Self {
            opacity,
        })
    }
}

#[derive(Copy, Clone, Debug)]
pub enum LabelOrientation {
    Center,
    Left,
    Right,
    Above,
    Below,
    AboveLeft,
    AboveRight,
    BelowLeft,
    BelowRight,
}

impl LabelOrientation {
    pub fn parse(parser: &Parser, expression: Expression) -> Result<Self, DesmosifyError> {
        let token = parser.token()?;
        Self::from_str(&parser.get_constant_string(expression)?)
            .map_err(|message| DesmosifyError::new(
                message,
                Some(token.start),
                Some(token.end),
            ))
    }
}

impl Default for LabelOrientation {
    fn default() -> Self {
        Self::Center
    }
}

impl FromStr for LabelOrientation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "center" => Ok(Self::Center),
            "left" => Ok(Self::Left),
            "right" => Ok(Self::Right),
            "above" => Ok(Self::Above),
            "below" => Ok(Self::Below),
            "above_left" => Ok(Self::AboveLeft),
            "above_right" => Ok(Self::AboveRight),
            "below_left" => Ok(Self::BelowLeft),
            "below_right" => Ok(Self::BelowRight),
            _ => Err(String::from("expected string 'center', 'left', 'right', 'above', 'below', 'above_left', 'above_right', 'below_left', or 'below_right'"))
        }
    }
}

#[derive(Debug)]
pub struct LabelAttribute {
    pub text: String,
    pub opacity: Box<Expression>,
    pub scale_factor: Box<Expression>,
    pub angle_degrees: Box<Expression>,
    pub orientation: LabelOrientation,
}

impl Attribute for LabelAttribute {
    const NAME: &'static str = "label";

    fn parse(parser: &mut Parser) -> Result<Self, DesmosifyError> {
        let start = parser.token()?.start;
        parser.next();
        let mut arguments = parser.parse_call()?;
        if arguments.len() < 1 || 5 < arguments.len() {
            return Err(DesmosifyError::new(
                String::from("expected 1-5 arguments for 'label' attribute"),
                Some(start),
                Some(parser.token()?.end),
            ));
        }
        let orientation = if arguments.len() == 5 {
            LabelOrientation::parse(parser, arguments.pop().unwrap())?
        } else {
            LabelOrientation::default()
        };
        let angle_degrees = Box::new(if arguments.len() == 4 {
            arguments.pop().unwrap()
        } else {
            Expression::from_constant(ConstantValue::Real(0.0))
        });
        let scale_factor = Box::new(if arguments.len() == 3 {
            arguments.pop().unwrap()
        } else {
            Expression::from_constant(ConstantValue::Real(1.0))
        });
        let opacity = Box::new(if arguments.len() == 2 {
            arguments.pop().unwrap()
        } else {
            Expression::from_constant(ConstantValue::Real(1.0))
        });
        let text = parser.get_constant_string(arguments.pop().unwrap())?;
        Ok(Self {
            text,
            opacity,
            scale_factor,
            angle_degrees,
            orientation
        })
    }
}

#[derive(Copy, Clone, Debug)]
pub enum DragMode {
    XY,
    X,
    Y,
}

impl DragMode {
    pub fn parse(parser: &Parser, expression: Expression) -> Result<Self, DesmosifyError> {
        let token = parser.token()?;
        Self::from_str(&parser.get_constant_string(expression)?)
            .map_err(|message| DesmosifyError::new(
                message,
                Some(token.start),
                Some(token.end),
            ))
    }
}

impl Default for DragMode {
    fn default() -> Self {
        Self::XY
    }
}

impl FromStr for DragMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "xy" => Ok(Self::XY),
            "x" => Ok(Self::X),
            "y" => Ok(Self::Y),
            _ => Err(String::from("expected string 'xy', 'x', or 'y'"))
        }
    }
}

#[derive(Debug)]
pub struct DragAttribute {
    pub mode: DragMode,
}

impl Attribute for DragAttribute {
    const NAME: &'static str = "drag";

    fn parse(parser: &mut Parser) -> Result<Self, DesmosifyError> {
        let start = parser.token()?.start;
        parser.next();
        let mut arguments = parser.parse_call()?;
        if arguments.len() > 1 {
            return Err(DesmosifyError::new(
                String::from("expected 0-1 arguments for 'drag' attribute"),
                Some(start),
                Some(parser.token()?.end),
            ));
        }
        let mode = if arguments.len() == 1 {
            DragMode::parse(parser, arguments.pop().unwrap())?
        } else {
            DragMode::default()
        };
        Ok(Self {
            mode,
        })
    }
}

#[derive(Debug)]
pub struct ClickAttribute {
    pub action: Box<Action>,
}

impl Attribute for ClickAttribute {
    const NAME: &'static str = "click";

    fn parse(parser: &mut Parser) -> Result<Self, DesmosifyError> {
        parser.next();
        let action = Box::new(parser.parse_action(false)?);
        Ok(Self {
            action,
        })
    }
}

#[derive(Debug)]
pub struct DescriptionAttribute {
    pub text: String,
}

impl Attribute for DescriptionAttribute {
    const NAME: &'static str = "description";

    fn parse(parser: &mut Parser) -> Result<Self, DesmosifyError> {
        let start = parser.token()?.start;
        parser.next();
        let mut arguments = parser.parse_call()?;
        if arguments.len() != 1 {
            return Err(DesmosifyError::new(
                String::from("expected 1 argument for 'description' attribute"),
                Some(start),
                Some(parser.token()?.end),
            ));
        }
        let text = parser.get_constant_string(arguments.pop().unwrap())?;
        Ok(Self {
            text,
        })
    }
}

#[derive(Debug)]
pub struct Element {
    pub what: Box<Expression>,
    pub color: Box<Expression>,

    pub point: Option<PointAttribute>,
    pub stroke: Option<StrokeAttribute>,
    pub fill: Option<FillAttribute>,
    pub label: Option<LabelAttribute>,
    pub drag: Option<DragAttribute>,
    pub click: Option<ClickAttribute>,
    pub description: Option<DescriptionAttribute>,
}

impl Element {
    pub fn new(what: Box<Expression>, color: Box<Expression>) -> Self {
        Self {
            what,
            color,
            point: None,
            stroke: None,
            fill: None,
            label: None,
            drag: None,
            click: None,
            description: None,
        }
    }

    pub fn parse(parser: &mut Parser) -> Result<Self, DesmosifyError> {
        let what = Box::new(parser.parse_expression(&[Symbol::Colon], &[])?);
        parser.next();
        let color = Box::new(parser.parse_expression(&[Symbol::Comma, Symbol::Semicolon], &[])?);
        let mut element = Self::new(what, color);
        if parser.is_at_symbol(Symbol::Comma)? {
            parser.next();
        }
        while !parser.is_at_symbol(Symbol::Semicolon)? {
            match parser.expect_name()?.as_str() {
                PointAttribute::NAME => if element.point.is_none() {
                    element.point = Some(PointAttribute::parse(parser)?);
                } else {
                    return Err(Self::duplicate_attribute_error(parser, PointAttribute::NAME));
                },
                StrokeAttribute::NAME => if element.stroke.is_none() {
                    element.stroke = Some(StrokeAttribute::parse(parser)?);
                } else {
                    return Err(Self::duplicate_attribute_error(parser, StrokeAttribute::NAME));
                },
                FillAttribute::NAME => if element.fill.is_none() {
                    element.fill = Some(FillAttribute::parse(parser)?);
                } else {
                    return Err(Self::duplicate_attribute_error(parser, FillAttribute::NAME));
                },
                LabelAttribute::NAME => if element.label.is_none() {
                    element.label = Some(LabelAttribute::parse(parser)?);
                } else {
                    return Err(Self::duplicate_attribute_error(parser, LabelAttribute::NAME));
                },
                DragAttribute::NAME => if element.drag.is_none() {
                    element.drag = Some(DragAttribute::parse(parser)?);
                } else {
                    return Err(Self::duplicate_attribute_error(parser, DragAttribute::NAME));
                },
                ClickAttribute::NAME => if element.click.is_none() {
                    element.click = Some(ClickAttribute::parse(parser)?);
                } else {
                    return Err(Self::duplicate_attribute_error(parser, ClickAttribute::NAME));
                },
                DescriptionAttribute::NAME => if element.description.is_none() {
                    element.description = Some(DescriptionAttribute::parse(parser)?);
                } else {
                    return Err(Self::duplicate_attribute_error(parser, DescriptionAttribute::NAME));
                },
                name => {
                    let token = parser.token()?;
                    return Err(DesmosifyError::new(
                        format!("unknown display attribute '{name}'"),
                        Some(token.start),
                        Some(token.end),
                    ));
                }
            }
            parser.next();
            parser.expect_one_of(&[Symbol::Comma, Symbol::Semicolon], &[])?;
            if parser.is_at_symbol(Symbol::Comma)? {
                parser.next();
            }
        }
        Ok(element)
    }

    fn duplicate_attribute_error(parser: &Parser, name: &str) -> DesmosifyError {
        // we can safely unwrap() because this function is called at the attribute name
        let token = parser.token().unwrap();
        DesmosifyError::new(
            format!("only one '{name}' attribute can be defined per element"),
            Some(token.start),
            Some(token.end),
        )
    }
}