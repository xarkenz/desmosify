use std::rc::Rc;
use crate::sema::intrinsic::IntrinsicFunction;

#[derive(Clone, PartialEq, Debug)]
pub struct FunctionSignature {
    pub parameter_types: Box<[Type]>,
    pub return_type: Type,
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
    Real,
    Int,
    Bool,
    Color,
    Polygon,
    Segment,
    Str,
    Point2 {
        x_type: Box<Type>,
        y_type: Box<Type>,
    },
    Point3 {
        x_type: Box<Type>,
        y_type: Box<Type>,
        z_type: Box<Type>,
    },
    UserValue {
        type_identifier: Rc<str>,
    },
    UserFunction {
        signature: Box<FunctionSignature>,
    },
    IntrinsicFunction(&'static IntrinsicFunction),
    Action {
        parameter_types: Box<[Type]>,
    },
    List {
        state: ListState,
        item_type: Box<Type>,
    },
}

impl Type {
    pub fn find_primitive(identifier: &str) -> Option<Self> {
        match identifier {
            "real" => Some(Type::Real),
            "int" => Some(Type::Int),
            "bool" => Some(Type::Bool),
            "color" => Some(Type::Color),
            "polygon" => Some(Type::Polygon),
            "segment" => Some(Type::Segment),
            "str" => Some(Type::Str),
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
            (_, Any) => !matches!(self, Meta { .. } | Str | UserFunction { .. } | IntrinsicFunction { .. }),
            (Any, _) => !matches!(target, Meta { .. } | Str | UserFunction { .. } | IntrinsicFunction { .. }),
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
            (UserValue { .. }, Int | Real) => true,
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
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Meta { .. } => write!(f, "<type>"),
            Self::Any => write!(f, "?"),
            Self::Real => write!(f, "real"),
            Self::Int => write!(f, "int"),
            Self::Bool => write!(f, "bool"),
            Self::Color => write!(f, "color"),
            Self::Polygon => write!(f, "polygon"),
            Self::Segment => write!(f, "segment"),
            Self::Str => write!(f, "str"),
            Self::List { state, item_type } => match state {
                ListState::IsList => write!(f, "[{item_type}]"),
                ListState::MaybeList => write!(f, "{item_type}+"),
            }
            Self::Point2 { x_type, y_type } => {
                write!(f, "({x_type}, {y_type})")
            }
            Self::Point3 { x_type, y_type, z_type } => {
                write!(f, "({x_type}, {y_type}, {z_type})")
            }
            Self::UserValue { type_identifier } => {
                write!(f, "{type_identifier}")
            }
            Self::UserFunction { .. } => write!(f, "<function>"),
            Self::IntrinsicFunction { .. } => write!(f, "<intrinsic_function>"),
            Self::Action { .. } => write!(f, "<action>"),
        }
    }
}
