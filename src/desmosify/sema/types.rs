use std::rc::Rc;
use crate::sema::intrinsic::IntrinsicFunction;

#[derive(Clone, PartialEq, Debug)]
pub struct FunctionSignature {
    pub parameter_types: Box<[Type]>,
    pub return_type: Type,
}

impl FunctionSignature {
    pub fn get_return_type(&self, arguments: &[Type]) -> Option<&Type> {
        let accepts_arguments = arguments.len() == self.parameter_types.len() &&
            std::iter::zip(arguments, &self.parameter_types)
                .all(|(argument, parameter)| argument.can_coerce_to(parameter));

        accepts_arguments.then_some(&self.return_type)
    }
}

// #[derive(Clone, Debug)]
// pub struct MaybeList<T: Clone + std::fmt::Debug> {
//     pub inner: T,
//     pub is_list: bool,
// }
//
// impl<T: Clone + std::fmt::Debug> MaybeList<T> {
//     pub fn new(inner: T, is_list: bool) -> Self {
//         Self {
//             inner,
//             is_list,
//         }
//     }
// }
//
// impl<T: Clone + std::fmt::Debug> From<T> for MaybeList<T> {
//     fn from(inner: T) -> Self {
//         Self {
//             inner,
//             is_list: false,
//         }
//     }
// }

#[derive(Clone, PartialEq, Debug)]
pub enum Type {
    Meta,
    Any,
    Real,
    Int,
    Bool,
    Color,
    Polygon,
    Segment,
    Str,
    List {
        item_type: Box<Type>,
    },
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

    pub fn into_list(self) -> Self {
        Self::List {
            item_type: Box::new(self),
        }
    }

    pub fn is_list(&self) -> bool {
        matches!(self, Self::List { .. })
    }

    pub fn list_item_type(&self) -> Option<&Self> {
        match self {
            Self::List { item_type } => Some(item_type),
            _ => None
        }
    }

    pub fn flatten_list(&self) -> (bool, &Self) {
        match self {
            Self::List { item_type } => (true, item_type),
            _ => (false, self)
        }
    }

    pub fn into_flatten_list(self) -> (bool, Self) {
        match self {
            Self::List { item_type } => (true, *item_type),
            _ => (false, self)
        }
    }

    pub fn unflatten_list(self, is_list: bool) -> Self {
        if is_list {
            self.into_list()
        }
        else {
            self
        }
    }

    pub fn can_coerce_to(&self, target: &Self) -> bool {
        use self::Type::*;
        self == target || match (self, target) {
            (_, Any) => !matches!(self, Meta | Str | UserFunction { .. } | IntrinsicFunction { .. }),
            (Any, _) => !matches!(target, Meta | Str | UserFunction { .. } | IntrinsicFunction { .. }),
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
                kind: crate::ErrorKind::ExpectedNumericPoint2DType {
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
                kind: crate::ErrorKind::ExpectedNumericPoint3DType {
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
        if let Self::Any = self {
            Ok(Self::Any)
        }
        else if let Self::Any = other {
            Ok(Self::Any)
        }
        else if self.can_coerce_to(&Self::Int) && other.can_coerce_to(&Self::Int) {
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

    pub fn broadcast(
        result_override: Option<Self>,
        arguments: impl IntoIterator<Item = Self>,
    ) -> crate::Result<Self> {
        let mut arguments = arguments.into_iter();
        let first_argument = arguments.next().unwrap().into_flatten_list();

        let (is_list, mut result_type) = arguments.try_fold(
            first_argument,
            |(current_is_list, current), next| {
                let (next_is_list, next) = next.into_flatten_list();
                crate::Result::Ok((current_is_list || next_is_list, current.merge(&next)?))
            },
        )?;

        if let Some(result_override) = result_override {
            result_type = result_override;
        }

        Ok(result_type.unflatten_list(is_list))
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Meta => write!(f, "<type>"),
            Self::Any => write!(f, "?"),
            Self::Real => write!(f, "real"),
            Self::Int => write!(f, "int"),
            Self::Bool => write!(f, "bool"),
            Self::Color => write!(f, "color"),
            Self::Polygon => write!(f, "polygon"),
            Self::Segment => write!(f, "segment"),
            Self::Str => write!(f, "str"),
            Self::List { item_type } => {
                write!(f, "[{item_type}]")
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

// impl std::fmt::Display for MaybeList<DataType> {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         if self.is_list {
//             write!(f, "[{}]", self.inner)
//         }
//         else {
//             write!(f, "{}", self.inner)
//         }
//     }
// }
