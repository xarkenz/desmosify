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

    pub fn merge_all(states: impl IntoIterator<Item = Option<Self>>) -> Option<Self> {
        states.into_iter().fold(None, Self::merge)
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
    Transformation,
    InternalBool,
    Str,
    Image,
    Point2D {
        x_type: Box<Self>,
        y_type: Box<Self>,
    },
    Point3D {
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
    Union {
        variants: Box<[Self]>,
    },
}

impl Type {
    pub fn point_2d(x_type: Self, y_type: Self) -> Self {
        Self::Point2D {
            x_type: Box::new(x_type),
            y_type: Box::new(y_type),
        }
    }

    pub fn real_point_2d() -> Self {
        Self::point_2d(Self::Real, Self::Real)
    }

    pub fn point_3d(x_type: Self, y_type: Self, z_type: Self) -> Self {
        Self::Point3D {
            x_type: Box::new(x_type),
            y_type: Box::new(y_type),
            z_type: Box::new(z_type),
        }
    }

    pub fn real_point_3d() -> Self {
        Self::point_3d(Self::Real, Self::Real, Self::Real)
    }

    pub fn union(variants: impl IntoIterator<Item = Self>) -> Self {
        Self::Union {
            variants: variants.into_iter().collect(),
        }
    }

    pub fn real_or_real_point() -> Self {
        Self::union([
            Self::Real,
            Self::real_point_2d(),
            Self::real_point_3d(),
        ])
    }

    pub fn transformable() -> Self {
        Self::union([
            Self::Polygon,
            Self::Segment,
            Self::Circle,
            Self::Arc,
            Self::Line,
            Self::Ray,
            Self::Vector,
            Self::Angle,
            Self::DirectedAngle,
            Self::real_point_2d(),
        ])
    }

    pub fn line_like() -> Self {
        Self::union([
            Self::Segment,
            Self::Line,
            Self::Ray,
            Self::Vector,
        ])
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
            "transformation" => Some(Self::Transformation),
            "internal_bool" => Some(Self::InternalBool),
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

    pub fn coerce_to(self, target: &Self) -> Option<Self> {
        if &self == target {
            return Some(self)
        }
        else if let Self::Union { variants } = target {
            return variants
                .iter()
                .find_map(|variant| self.clone().coerce_to(variant))
        }

        match (self, target) {
            (self_, Self::Any) => self_.is_first_class().then_some(self_),
            (Self::Any, _) => target.is_first_class().then_some(target.clone()),
            (Self::Real, Self::Complex) => Some(target.clone()),
            (Self::Int, Self::Real | Self::Complex) => Some(target.clone()),
            (Self::Bool, Self::Int | Self::Real | Self::Complex) => Some(target.clone()),
            (
                Self::Point2D { x_type: self_x, y_type: self_y },
                Self::Point2D { x_type: target_x, y_type: target_y },
            ) => Some(Self::Point2D {
                x_type: Box::new(self_x.coerce_to(target_x)?),
                y_type: Box::new(self_y.coerce_to(target_y)?),
            }),
            (
                Self::Point3D { x_type: self_x, y_type: self_y, z_type: self_z },
                Self::Point3D { x_type: target_x, y_type: target_y, z_type: target_z },
            ) => Some(Self::Point3D {
                x_type: Box::new(self_x.coerce_to(target_x)?),
                y_type: Box::new(self_y.coerce_to(target_y)?),
                z_type: Box::new(self_z.coerce_to(target_z)?),
            }),
            (Self::Angle, Self::Real | Self::Complex) => Some(target.clone()),
            (Self::DirectedAngle, Self::Real | Self::Complex) => Some(target.clone()),
            (Self::Enum { .. }, Self::Int | Self::Real | Self::Complex) => Some(target.clone()),
            _ => None
        }
    }

    pub fn can_coerce_to(&self, target: &Self) -> bool {
        self.clone().coerce_to(target).is_some()
    }

    pub fn is_first_class(&self) -> bool {
        // TODO: use this more
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
            Self::Transformation => true,
            Self::InternalBool => true,
            Self::Str => false,
            Self::Image => false,
            Self::Point2D { .. } => true,
            Self::Point3D { .. } => true,
            Self::Enum { .. } => true,
            Self::UserFunction { .. } => false,
            Self::IntrinsicFunction(..) => false,
            Self::Action { .. } => false,
            Self::List { .. } => true,
            Self::Union { variants } => variants.iter().all(Self::is_first_class),
        }
    }

    pub fn is_valid_var_type(&self) -> bool {
        // TODO: actually use this
        match self {
            Self::Any => true,
            Self::Complex => true,
            Self::Real => true,
            Self::Int => true,
            Self::Bool => true,
            Self::Color => true,
            Self::Tone => true,
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
            Self::Point2D { .. } => true,
            Self::Point3D { .. } => true,
            Self::Enum { .. } => true,
            Self::List { item_type, .. } => item_type.is_valid_var_type(),
            _ => false
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
        self.can_coerce_to(&Type::Point2D {
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
        self.can_coerce_to(&Type::Point3D {
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

    pub fn require_action(&self, min_arity: usize, argument_types: &[Self]) -> crate::Result<&[Self]> {
        if let Self::Action { parameter_types } = self {
            if (min_arity ..= argument_types.len()).contains(&parameter_types.len())
                && std::iter::zip(argument_types, parameter_types)
                .all(|(argument_type, parameter_type)| {
                    argument_type.can_coerce_to(parameter_type)
                })
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
                Self::Point2D { x_type: self_x, y_type: self_y },
                Self::Point2D { x_type: other_x, y_type: other_y },
            ) => Ok(Self::Point2D {
                x_type: Box::new(Self::merge(self_x, other_x)?),
                y_type: Box::new(Self::merge(self_y, other_y)?),
            }),
            (
                Self::Point3D { x_type: self_x, y_type: self_y, z_type: self_z },
                Self::Point3D { x_type: other_x, y_type: other_y, z_type: other_z },
            ) => Ok(Self::Point3D {
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
            Self::Transformation => write!(f, "transformation"),
            Self::InternalBool => write!(f, "internal_bool"),
            Self::Str => write!(f, "str"),
            Self::Image => write!(f, "image"),
            Self::Point2D { x_type, y_type } => {
                write!(f, "({x_type}, {y_type})")
            }
            Self::Point3D { x_type, y_type, z_type } => {
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
            Self::Union { variants } => match variants.as_ref() {
                [] => write!(f, "empty_union"),
                [first, rest @ ..] => {
                    write!(f, "({first}")?;
                    for variant in rest {
                        write!(f, " | {variant}")?;
                    }
                    write!(f, ")")
                }
            }
        }
    }
}
