use std::rc::Rc;
use crate::sema::intrinsic::IntrinsicFunction;

#[derive(Clone, Debug)]
pub struct FunctionSignature {
    pub parameter_types: Box<[DataType]>,
    pub return_type: DataType,
}

impl FunctionSignature {
    pub fn get_return_type(&self, arguments: &[DataType]) -> Option<&DataType> {
        let accepts_arguments = arguments.len() == self.parameter_types.len() &&
            std::iter::zip(arguments, &self.parameter_types)
                .all(|(argument, parameter)| argument.can_coerce_to(parameter));

        accepts_arguments.then_some(&self.return_type)
    }
}

#[derive(Clone, Debug)]
pub enum DataType {
    Any,
    Real,
    Int,
    Bool,
    Color,
    Polygon,
    Segment,
    Str,
    List {
        item_type: Box<DataType>,
    },
    Point2 {
        x_type: Box<DataType>,
        y_type: Box<DataType>,
    },
    Point3 {
        x_type: Box<DataType>,
        y_type: Box<DataType>,
        z_type: Box<DataType>,
    },
    UserValue {
        type_identifier: Rc<str>,
    },
    UserFunction {
        signature: Box<FunctionSignature>,
    },
    IntrinsicFunction {
        function: &'static IntrinsicFunction,
    },
    Action {
        parameter_types: Box<[DataType]>,
    },
    UserType {
        identifier: Rc<str>,
    },
}

impl DataType {
    pub fn find_primitive(identifier: &str) -> Option<Self> {
        match identifier {
            "real" => Some(DataType::Real),
            "int" => Some(DataType::Int),
            "bool" => Some(DataType::Bool),
            "color" => Some(DataType::Color),
            "polygon" => Some(DataType::Polygon),
            "segment" => Some(DataType::Segment),
            "str" => Some(DataType::Str),
            _ => None,
        }
    }

    pub fn can_coerce_to(&self, target: &Self) -> bool {
        use DataType::*;
        match self {
            Any => match target {
                Str | List { .. } | UserFunction { .. } | IntrinsicFunction { .. } | UserType { .. } => false,
                _ => true
            },
            Real => match target {
                Any | Real => true,
                _ => false
            },
            Int => match target {
                Any | Real | Int => true,
                _ => false
            },
            Bool => match target {
                Any | Real | Int | Bool => true,
                _ => false
            },
            Point2 { x_type, y_type } => match target {
                Any => true,
                Point2 { x_type: target_x, y_type: target_y } => {
                    x_type.can_coerce_to(target_x) && y_type.can_coerce_to(target_y)
                }
                _ => false
            },
            Point3 { x_type, y_type, z_type } => match target {
                Any => true,
                Point3 { x_type: target_x, y_type: target_y, z_type: target_z } => {
                    x_type.can_coerce_to(target_x) && y_type.can_coerce_to(target_y) && z_type.can_coerce_to(target_z)
                }
                _ => false
            },
            Color => match target {
                Any | Color => true,
                _ => false
            },
            Polygon => match target {
                Any | Polygon => true,
                _ => false
            },
            Segment => match target {
                Any | Segment => true,
                _ => false
            },
            List { item_type } => match target {
                List { item_type: target_item } => {
                    item_type.can_coerce_to(target_item)
                },
                _ => false
            },
            UserValue { type_identifier } => match target {
                Any | Real | Int => true,
                UserValue { type_identifier: target_identifier } => target_identifier == type_identifier,
                _ => false
            },
            _ => false
        }
    }

    pub fn merge(&self, other: &Self) -> Option<Self> {
        if self.can_coerce_to(other) {
            Some(other.clone())
        }
        else if other.can_coerce_to(self) {
            Some(self.clone())
        }
        else {
            None
        }
    }

    pub fn merge_numeric(&self, other: &Self) -> Option<Self> {
        if let Self::Any = self {
            Some(Self::Any)
        }
        else if let Self::Any = other {
            Some(Self::Any)
        }
        else if self.can_coerce_to(&Self::Int) && other.can_coerce_to(&Self::Int) {
            Some(Self::Int)
        }
        else if self.can_coerce_to(&Self::Real) && other.can_coerce_to(&Self::Real) {
            Some(Self::Real)
        }
        else {
            None
        }
    }
}
