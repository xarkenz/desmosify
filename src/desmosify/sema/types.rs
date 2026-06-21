use std::rc::Rc;
use crate::sema::intrinsic::IntrinsicFunction;
use crate::sema::values::{Value, ValueKind};

#[derive(Clone, PartialEq, Debug)]
pub struct FunctionSignature {
    pub parameter_types: Box<[Type]>,
    pub return_type: Type,
}

impl std::fmt::Display for FunctionSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.parameter_types.as_ref() {
            [] => write!(f, "function(")?,
            [first, rest @ ..] => {
                write!(f, "function({first}")?;
                for parameter_type in rest {
                    write!(f, ", {parameter_type}")?;
                }
            }
        }
        write!(f, "): {}", self.return_type)
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ListState {
    IsList,
    MaybeList,
}

impl ListState {
    pub fn can_coerce(from_state: Option<Self>, to_state: Option<Self>, allow_broadcast: bool) -> bool {
        from_state == to_state || match (from_state, to_state) {
            (Some(Self::IsList) | None, Some(Self::MaybeList)) => true,
            (Some(Self::IsList | Self::MaybeList), None) => allow_broadcast,
            _ => false
        }
    }

    pub fn merge(state_1: Option<Self>, state_2: Option<Self>) -> Option<Self> {
        match (state_1, state_2) {
            (Some(Self::IsList), _) => Some(Self::IsList),
            (_, Some(Self::IsList)) => Some(Self::IsList),
            (Some(Self::MaybeList), _) => Some(Self::MaybeList),
            (_, Some(Self::MaybeList)) => Some(Self::MaybeList),
            (None, None) => None,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Type {
    Meta {
        identifier: Rc<str>,
    },
    Any,
    Complex,
    Real,
    Int,
    Bool,
    Color,
    Tone,
    Distribution,
    Polygon,
    Segment,
    Circle,
    Arc,
    Line,
    Ray,
    Vector,
    Angle,
    DirectedAngle,
    Segment3D,
    Triangle3D,
    Sphere3D,
    Vector3D,
    Str,
    Image,
    Point2 {
        x_type: Box<Self>,
        y_type: Box<Self>,
    },
    Point3 {
        x_type: Box<Self>,
        y_type: Box<Self>,
        z_type: Box<Self>,
    },
    Enum {
        type_identifier: Rc<str>,
    },
    UserFunction {
        signature: Box<FunctionSignature>,
    },
    IntrinsicFunction(&'static IntrinsicFunction),
    Action {
        parameter_types: Box<[Self]>,
    },
    List {
        state: ListState,
        item_type: Box<Self>,
    },
}

impl Type {
    pub fn real_point2() -> Self {
        Self::Point2 {
            x_type: Box::new(Self::Real),
            y_type: Box::new(Self::Real),
        }
    }

    pub fn real_point3() -> Self {
        Self::Point3 {
            x_type: Box::new(Self::Real),
            y_type: Box::new(Self::Real),
            z_type: Box::new(Self::Real),
        }
    }

    pub fn find_primitive(identifier: &str) -> Option<Self> {
        match identifier {
            "any" => Some(Self::Any),
            "complex" => Some(Self::Complex),
            "real" => Some(Self::Real),
            "int" => Some(Self::Int),
            "bool" => Some(Self::Bool),
            "color" => Some(Self::Color),
            "tone" => Some(Self::Tone),
            "distribution" => Some(Self::Distribution),
            "polygon" => Some(Self::Polygon),
            "segment" => Some(Self::Segment),
            "circle" => Some(Self::Circle),
            "arc" => Some(Self::Arc),
            "line" => Some(Self::Line),
            "ray" => Some(Self::Ray),
            "vector" => Some(Self::Vector),
            "angle" => Some(Self::Angle),
            "directed_angle" => Some(Self::DirectedAngle),
            "segment3d" => Some(Self::Segment3D),
            "triangle3d" => Some(Self::Triangle3D),
            "sphere3d" => Some(Self::Sphere3D),
            "vector3d" => Some(Self::Vector3D),
            "str" => Some(Self::Str),
            "image" => Some(Self::Image),
            _ => None,
        }
    }

    pub fn into_list(self, state: ListState) -> Self {
        Self::List {
            state,
            item_type: Box::new(self),
        }
    }

    pub fn list_state(&self) -> Option<ListState> {
        match self {
            Self::List { state, .. } => Some(*state),
            _ => None
        }
    }

    pub fn flatten_list(&self) -> (Option<ListState>, &Self) {
        match self {
            Self::List { state, item_type } => (Some(*state), item_type),
            _ => (None, self)
        }
    }

    pub fn into_flatten_list(self) -> (Option<ListState>, Self) {
        match self {
            Self::List { state, item_type } => (Some(state), *item_type),
            _ => (None, self)
        }
    }

    pub fn require_flatten_list(self) -> crate::Result<Self> {
        match self {
            // TODO: require state to be IsList?
            Self::List { item_type, .. } => Ok(*item_type),
            other_type => Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ExpectedListType {
                    got_type: other_type.to_string(),
                },
                span: None,
            }))
        }
    }

    pub fn unflatten_list(self, state: Option<ListState>) -> Self {
        match state {
            Some(state) => self.into_list(state),
            None => self,
        }
    }

    pub fn can_coerce_to(&self, target: &Self) -> bool {
        use self::Type::*;
        self == target || match (self, target) {
            (_, Any) => self.is_first_class(),
            (Any, _) => target.is_first_class(),
            (Int, Real) => true,
            (Bool, Int | Real) => true,
            (
                Point2 { x_type: self_x, y_type: self_y },
                Point2 { x_type: target_x, y_type: target_y },
            ) => self_x.can_coerce_to(target_x) && self_y.can_coerce_to(target_y),
            (
                Point3 { x_type: self_x, y_type: self_y, z_type: self_z },
                Point3 { x_type: target_x, y_type: target_y, z_type: target_z },
            ) => self_x.can_coerce_to(target_x) && self_y.can_coerce_to(target_y) && self_z.can_coerce_to(target_z),
            (Enum { .. }, Int | Real) => true,
            _ => false
        }
    }

    pub fn is_first_class(&self) -> bool {
        match self {
            Self::Meta { .. } => false,
            Self::Any => true,
            Self::Complex => true,
            Self::Real => true,
            Self::Int => true,
            Self::Bool => true,
            Self::Color => true,
            Self::Tone => true,
            Self::Distribution => true,
            Self::Polygon => true,
            Self::Segment => true,
            Self::Circle => true,
            Self::Arc => true,
            Self::Line => true,
            Self::Ray => true,
            Self::Vector => true,
            Self::Angle => true,
            Self::DirectedAngle => true,
            Self::Segment3D => true,
            Self::Triangle3D => true,
            Self::Sphere3D => true,
            Self::Vector3D => true,
            Self::Str => false,
            Self::Image => false,
            Self::Point2 { .. } => true,
            Self::Point3 { .. } => true,
            Self::Enum { .. } => true,
            Self::UserFunction { .. } => false,
            Self::IntrinsicFunction(..) => false,
            Self::Action { .. } => false,
            Self::List { .. } => true,
        }
    }

    pub fn is_numeric(&self) -> bool {
        self.can_coerce_to(&Type::Real)
    }

    pub fn require_numeric(&self) -> crate::Result<()> {
        if self.is_numeric() {
            Ok(())
        }
        else {
            Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ExpectedNumericType {
                    got_type: self.to_string(),
                },
                span: None,
            }))
        }
    }

    pub fn is_numeric_point_2d(&self) -> bool {
        self.can_coerce_to(&Type::Point2 {
            x_type: Box::new(Type::Real),
            y_type: Box::new(Type::Real),
        })
    }

    pub fn require_numeric_point_2d(&self) -> crate::Result<()> {
        if self.is_numeric_point_2d() {
            Ok(())
        }
        else {
            Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ExpectedNumericPoint2Type {
                    got_type: self.to_string(),
                },
                span: None,
            }))
        }
    }

    pub fn is_numeric_point_3d(&self) -> bool {
        self.can_coerce_to(&Type::Point3 {
            x_type: Box::new(Type::Real),
            y_type: Box::new(Type::Real),
            z_type: Box::new(Type::Real),
        })
    }

    pub fn require_numeric_point_3d(&self) -> crate::Result<()> {
        if self.is_numeric_point_3d() {
            Ok(())
        }
        else {
            Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ExpectedNumericPoint3Type {
                    got_type: self.to_string(),
                },
                span: None,
            }))
        }
    }

    pub fn is_numeric_or_point(&self) -> bool {
        self.is_numeric() || self.is_numeric_point_2d() || self.is_numeric_point_3d()
    }

    pub fn require_numeric_or_point(&self) -> crate::Result<()> {
        if self.is_numeric_or_point() {
            Ok(())
        }
        else {
            Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ExpectedNumericOrPointType {
                    got_type: self.to_string(),
                },
                span: None,
            }))
        }
    }

    pub fn is_transformable(&self) -> bool {
        match self {
            Self::Any => true,
            Self::Polygon => true,
            Self::Segment => true,
            Self::Circle => true,
            Self::Arc => true,
            Self::Line => true,
            Self::Ray => true,
            Self::Vector => true,
            Self::Angle => true,
            Self::DirectedAngle => true,
            Self::Point2 { .. } => true,
            _ => false
        }
    }

    pub fn require_transformable(&self) -> crate::Result<()> {
        if self.is_transformable() {
            Ok(())
        }
        else {
            Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ExpectedTransformableType {
                    got_type: self.to_string(),
                },
                span: None,
            }))
        }
    }

    pub fn is_line_like(&self) -> bool {
        match self {
            Self::Any => true,
            Self::Segment => true,
            Self::Line => true,
            Self::Ray => true,
            Self::Vector => true,
            _ => false
        }
    }

    pub fn require_line_like(&self) -> crate::Result<()> {
        if self.is_line_like() {
            Ok(())
        }
        else {
            Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ExpectedLineLikeType {
                    got_type: self.to_string(),
                },
                span: None,
            }))
        }
    }

    pub fn require_action(&self, min_arity: usize, argument_types: &[Self]) -> crate::Result<&[Self]> {
        if let Self::Action { parameter_types } = self {
            if (min_arity ..= argument_types.len()).contains(&parameter_types.len())
                && std::iter::zip(argument_types, parameter_types)
                .all(|(argument_type, parameter_type)| argument_type.can_coerce_to(parameter_type))
            {
                return Ok(parameter_types)
            }
        }
        Err(Box::new(crate::Error {
            kind: crate::ErrorKind::ExpectedActionType {
                expected_parameter_lists: (min_arity ..= argument_types.len())
                    .map(|arity| argument_types[..arity]
                        .iter()
                        .map(ToString::to_string)
                        .collect())
                    .collect(),
                got_type: self.to_string(),
            },
            span: None,
        }))
    }

    pub fn merge(&self, other: &Self) -> crate::Result<Self> {
        let (self_list, self_inner) = self.flatten_list();
        let (other_list, other_inner) = other.flatten_list();

        let merged_inner = Self::merge_inner(self_inner, other_inner)?;

        Ok(merged_inner.unflatten_list(ListState::merge(self_list, other_list)))
    }

    pub fn merge_inner(&self, other: &Self) -> crate::Result<Self> {
        if self == other {
            return Ok(self.clone());
        }

        match (self, other) {
            (Self::Any, _) => Ok(other.clone()),
            (_, Self::Any) => Ok(self.clone()),
            (
                Self::Point2 { x_type: self_x, y_type: self_y },
                Self::Point2 { x_type: other_x, y_type: other_y },
            ) => Ok(Self::Point2 {
                x_type: Box::new(Self::merge(self_x, other_x)?),
                y_type: Box::new(Self::merge(self_y, other_y)?),
            }),
            (
                Self::Point3 { x_type: self_x, y_type: self_y, z_type: self_z },
                Self::Point3 { x_type: other_x, y_type: other_y, z_type: other_z },
            ) => Ok(Self::Point3 {
                x_type: Box::new(Self::merge(self_x, other_x)?),
                y_type: Box::new(Self::merge(self_y, other_y)?),
                z_type: Box::new(Self::merge(self_z, other_z)?),
            }),
            _ => {
                if self.can_coerce_to(&Self::Int) && other.can_coerce_to(&Self::Int) {
                    Ok(Self::Int)
                }
                else if self.can_coerce_to(&Self::Real) && other.can_coerce_to(&Self::Real) {
                    Ok(Self::Real)
                }
                else if self.can_coerce_to(other) {
                    Ok(other.clone())
                }
                else if other.can_coerce_to(self) {
                    Ok(self.clone())
                }
                else {
                    Err(Box::new(crate::Error {
                        kind: crate::ErrorKind::CannotMergeTypes {
                            type_1: self.to_string(),
                            type_2: other.to_string(),
                        },
                        span: None,
                    }))
                }
            }
        }
    }

    pub fn broadcast(
        result_override: Option<Self>,
        arguments: impl IntoIterator<Item = (Self, Option<crate::Span>)>,
    ) -> crate::Result<Self> {
        let mut arguments = arguments.into_iter();
        let first_argument = arguments.next().unwrap().0;

        let mut result_type = arguments.try_fold(
            first_argument,
            |current_type, (next_type, span)| {
                current_type.merge(&next_type)
                    .map_err(|error| error.with_span(span))
            },
        )?;

        if let Some(result_override) = result_override {
            result_type = result_override.unflatten_list(result_type.list_state());
        }

        Ok(result_type)
    }

    pub fn value_range(&self) -> Option<(Option<Value>, Option<Value>, Option<Value>)> {
        match self {
            Self::Real => Some((None, None, None)),
            Self::Int | Self::Enum { .. } => Some((
                None,
                None,
                Some(ValueKind::Int(1).into()),
            )),
            Self::Bool => Some((
                Some(ValueKind::Bool(false).into()),
                Some(ValueKind::Bool(true).into()),
                Some(ValueKind::Bool(true).into()),
            )),
            _ => None
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Meta { .. } => write!(f, "type"),
            Self::Any => write!(f, "any"),
            Self::Complex => write!(f, "complex"),
            Self::Real => write!(f, "real"),
            Self::Int => write!(f, "int"),
            Self::Bool => write!(f, "bool"),
            Self::Color => write!(f, "color"),
            Self::Tone => write!(f, "tone"),
            Self::Distribution => write!(f, "distribution"),
            Self::Polygon => write!(f, "polygon"),
            Self::Segment => write!(f, "segment"),
            Self::Circle => write!(f, "circle"),
            Self::Arc => write!(f, "arc"),
            Self::Line => write!(f, "line"),
            Self::Ray => write!(f, "ray"),
            Self::Vector => write!(f, "vector"),
            Self::Angle => write!(f, "angle"),
            Self::DirectedAngle => write!(f, "directed_angle"),
            Self::Segment3D => write!(f, "segment3d"),
            Self::Triangle3D => write!(f, "triangle3d"),
            Self::Sphere3D => write!(f, "sphere3d"),
            Self::Vector3D => write!(f, "vector3d"),
            Self::Str => write!(f, "str"),
            Self::Image => write!(f, "image"),
            Self::Point2 { x_type, y_type } => {
                write!(f, "({x_type}, {y_type})")
            }
            Self::Point3 { x_type, y_type, z_type } => {
                write!(f, "({x_type}, {y_type}, {z_type})")
            }
            Self::Enum { type_identifier } => {
                write!(f, "{type_identifier}")
            }
            Self::UserFunction { signature } => signature.fmt(f),
            Self::IntrinsicFunction { .. } => write!(f, "intrinsic_function"),
            Self::Action { parameter_types } => match parameter_types.as_ref() {
                [] => write!(f, "action()"),
                [first, rest @ ..] => {
                    write!(f, "action({first}")?;
                    for parameter_type in rest {
                        write!(f, ", {parameter_type}")?;
                    }
                    Ok(())
                }
            }
            Self::List { state, item_type } => match state {
                ListState::IsList => write!(f, "[{item_type}]"),
                ListState::MaybeList => write!(f, "{item_type}+"),
            }
        }
    }
}
