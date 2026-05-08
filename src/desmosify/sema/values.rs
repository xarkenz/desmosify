use std::rc::Rc;
use crate::ast::{BinaryOperation, RangeKind, UnaryOperation};
use crate::sema::intrinsic::{IntrinsicFunction, IntrinsicValue};
use crate::sema::types::Type;

// #[derive(Clone, Debug)]
// pub struct MaybeBroadcast<T: Clone + std::fmt::Debug> {
//     pub inner: T,
//     pub is_list: bool,
// }

// impl<T: Clone + std::fmt::Debug> MaybeBroadcast<T> {
//     pub fn new(inner: T, is_list: bool) -> Self {
//         Self {
//             inner,
//             is_list,
//         }
//     }
// }

#[derive(Copy, Clone, Debug)]
pub enum MathematicalConstant {
    Pi,
    Tau,
    E,
}

#[derive(Clone, Debug)]
pub enum Constant {
    Type {
        identifier: Rc<str>,
    },
    Real(f64),
    Mathematical {
        kind: MathematicalConstant,
        coefficient: f64,
    },
    Int(i64),
    Bool(bool),
    Point2 {
        x: Box<Constant>,
        y: Box<Constant>,
    },
    Point3 {
        x: Box<Constant>,
        y: Box<Constant>,
        z: Box<Constant>,
    },
    List {
        items: Vec<Constant>,
        item_type: Type,
    },
    EnumVariant {
        type_identifier: Rc<str>,
        variant_ordinal: i64,
    },
}

impl Constant {
    pub fn get_type(&self) -> Type {
        match self {
            Self::Type { .. } => {
                Type::Meta
            }
            Self::Real(..) | Self::Mathematical { .. } => {
                Type::Real
            }
            Self::Int(..) => {
                Type::Int
            }
            Self::Bool(..) => {
                Type::Bool
            }
            Self::Point2 { x, y } => {
                Type::Point2 {
                    x_type: Box::new(x.get_type()),
                    y_type: Box::new(y.get_type()),
                }
            }
            Self::Point3 { x, y, z } => {
                Type::Point3 {
                    x_type: Box::new(x.get_type()),
                    y_type: Box::new(y.get_type()),
                    z_type: Box::new(z.get_type()),
                }
            }
            Self::List { item_type, .. } => {
                item_type.clone().into_list()
            }
            Self::EnumVariant { type_identifier, .. } => {
                Type::UserValue {
                    type_identifier: type_identifier.clone(),
                }
            }
        }
    }

    pub fn coerce_to(self, target_type: &Type) -> crate::Result<Self> {
        let self_type = self.get_type();
        let (self_is_list, self_type) = self_type.flatten_list();
        let (target_is_list, target_type) = target_type.flatten_list();

        let mismatched_types_error = || Box::new(crate::Error {
            kind: crate::ErrorKind::MismatchedTypes {
                expected: target_type.to_string(),
                got: self_type.to_string(),
            },
            span: None,
        });

        if target_is_list != self_is_list {
            return Err(mismatched_types_error());
        }
        else if self_type == target_type || matches!(self_type, Type::Any) || matches!(target_type, Type::Any) {
            // The value shouldn't need to be transformed in any way
            return Ok(self);
        }

        match (self, target_type) {
            (Self::Int(value), Type::Real) => {
                Ok(Self::Real(value as f64))
            }
            (Self::Bool(value), Type::Int) => {
                Ok(Self::Int(value as i64))
            }
            (Self::Bool(value), Type::Real) => {
                Ok(Self::Real(value as i32 as f64))
            }
            (Self::Point2 { x, y }, Type::Point2 { x_type, y_type }) => {
                Ok(Self::Point2 {
                    x: Box::new(x.coerce_to(x_type)?),
                    y: Box::new(y.coerce_to(y_type)?),
                })
            }
            (Self::Point3 { x, y, z }, Type::Point3 { x_type, y_type, z_type }) => {
                Ok(Self::Point3 {
                    x: Box::new(x.coerce_to(x_type)?),
                    y: Box::new(y.coerce_to(y_type)?),
                    z: Box::new(z.coerce_to(z_type)?)
                })
            }
            (Self::EnumVariant { variant_ordinal, .. }, Type::Int) => {
                Ok(Self::Int(variant_ordinal))
            }
            (Self::EnumVariant { variant_ordinal, .. }, Type::Real) => {
                Ok(Self::Real(variant_ordinal as f64))
            }
            _ => Err(mismatched_types_error())
        }
    }
}

#[derive(Clone, Debug)]
pub struct GlobalReference {
    pub identifier: Rc<str>,
    pub value_type: Type,
}

#[derive(Clone, Debug)]
pub struct LocalReference {
    pub id: usize,
    pub value_type: Type,
}

#[derive(Clone, Debug)]
pub enum ValueIndexOperation {
    Single {
        index: Box<Value>,
    },
    Range {
        kind: RangeKind,
        from_index: Box<Value>,
        to_index: Box<Value>,
        step: Option<Box<Value>>,
    },
    RangeFrom {
        from_index: Box<Value>,
        step: Option<Box<Value>>,
    },
    RangeTo {
        kind: RangeKind,
        to_index: Box<Value>,
    },
}

impl ValueIndexOperation {
    pub fn is_range(&self) -> bool {
        matches!(self, Self::Range { .. } | Self::RangeFrom { .. } | Self::RangeTo { .. })
    }
}

#[derive(Clone, Debug)]
pub struct ValueListMapLoop {
    pub identifier: Rc<str>,
    pub list: Value,
}

#[derive(Clone, Debug)]
pub enum Value {
    Constant(Constant),
    Intrinsic(Box<IntrinsicValue>),
    IntrinsicFunction(&'static IntrinsicFunction),
    Global(GlobalReference),
    Local(LocalReference),
    Unary {
        operation: UnaryOperation,
        operand: Box<Value>,
        result_type: Type,
    },
    Binary {
        operation: BinaryOperation,
        lhs: Box<Value>,
        rhs: Box<Value>,
        result_type: Type,
    },
    Point2 {
        x: Box<Value>,
        y: Box<Value>,
    },
    Point3 {
        x: Box<Value>,
        y: Box<Value>,
        z: Box<Value>,
    },
    GetX {
        point: Box<Value>,
        x_type: Type,
    },
    GetY {
        point: Box<Value>,
        y_type: Type,
    },
    GetZ {
        point: Box<Value>,
        z_type: Type,
    },
    List {
        items: Box<[Value]>,
        item_type: Type,
    },
    ListRange {
        kind: RangeKind,
        start: Box<Value>,
        end: Box<Value>,
        step: Box<Value>,
        item_type: Type,
    },
    ListFill {
        value: Box<Value>,
        count: Box<Value>,
    },
    ListMap {
        loops: Box<[ValueListMapLoop]>,
        value: Box<Value>,
    },
    ListFilter {
        list: Box<Value>,
        condition: Box<Value>,
        item_type: Type,
    },
    Index {
        list: Box<Value>,
        operation: ValueIndexOperation,
        item_type: Type,
    },
    Conditional {
        condition_consequents: Box<[(Value, Value)]>,
        alternative: Box<Value>,
        result_type: Type,
    },
    FunctionCall {
        function: Box<Value>,
        arguments: Box<[Value]>,
        return_type: Type,
    },
    Let {
        identifier: Rc<str>,
        value: Box<Value>,
        inner: Box<Value>,
    },
}

impl Value {
    pub fn get_type(&self) -> Type {
        match self {
            Self::Constant(constant) => {
                constant.get_type()
            }
            Self::Intrinsic(intrinsic) => {
                intrinsic.get_type()
            }
            Self::IntrinsicFunction(function) => {
                Type::IntrinsicFunction(function)
            }
            Self::Global(reference) => {
                reference.value_type.clone()
            }
            Self::Local(reference) => {
                reference.value_type.clone()
            }
            Self::Unary { result_type, .. } => {
                result_type.clone()
            }
            Self::Binary { result_type, .. } => {
                result_type.clone()
            }
            Self::Point2 { x, y } => {
                Type::Point2 {
                    x_type: Box::new(x.get_type()),
                    y_type: Box::new(y.get_type()),
                }
            }
            Self::Point3 { x, y, z } => {
                Type::Point3 {
                    x_type: Box::new(x.get_type()),
                    y_type: Box::new(y.get_type()),
                    z_type: Box::new(z.get_type()),
                }
            }
            Self::GetX { x_type, .. } => {
                x_type.clone()
            }
            Self::GetY { y_type, .. } => {
                y_type.clone()
            }
            Self::GetZ { z_type, .. } => {
                z_type.clone()
            }
            Self::List { item_type, .. } => {
                item_type.clone().into_list()
            }
            Self::ListRange { item_type, .. } => {
                item_type.clone().into_list()
            }
            Self::ListFill { value, .. } => {
                value.get_type().into_list()
            }
            Self::ListMap { value, .. } => {
                value.get_type().into_list()
            }
            Self::ListFilter { item_type, .. } => {
                item_type.clone().into_list()
            }
            Self::Index { operation, item_type, .. } => {
                item_type.clone().unflatten_list(operation.is_range())
            }
            Self::Conditional { result_type, .. } => {
                result_type.clone()
            }
            Self::FunctionCall { return_type, .. } => {
                return_type.clone()
            }
            Self::Let { inner, .. } => {
                inner.get_type()
            }
        }
    }

    pub fn coerce_to(self, target_type: &Type) -> crate::Result<Self> {
        if let Self::Constant(constant) = self {
            return constant.coerce_to(target_type).map(Self::Constant);
        }

        let self_type = self.get_type();
        let (self_is_list, self_type) = self_type.flatten_list();
        let (target_is_list, target_type) = target_type.flatten_list();

        if target_is_list == self_is_list && self_type.can_coerce_to(target_type) {
            // The value shouldn't need to be transformed in any way
            Ok(self)
        }
        else {
            Err(Box::new(crate::Error {
                kind: crate::ErrorKind::MismatchedTypes {
                    expected: target_type.to_string(),
                    got: self_type.to_string(),
                },
                span: None,
            }))
        }
    }
}

// impl MaybeBroadcast<Value> {
//     pub fn get_type(&self) -> DataType {
//         let inner_type = self.inner.get_type();
//         let wrap_list = self.is_list && !matches!(inner_type, DataType::List { .. });
//         inner_type.unflatten_list(wrap_list)
//     }
//
//     pub fn get_item_type(&self) -> DataType {
//         let inner_type = self.inner.get_type();
//     }
//
//     pub fn coerce_to(self, target_type: &DataType) -> crate::Result<Self> {
//         //
//     }
// }

#[derive(Clone, Debug)]
pub enum ActionValue {
    Disable,
    Compound {
        actions: Box<[ActionValue]>,
    },
    Update {
        variable: GlobalReference,
        value: Box<Value>,
    },
    ActionCall {
        identifier: Rc<str>,
        arguments: Box<[Value]>,
    },
    Conditional {
        condition_consequents: Box<[(Value, ActionValue)]>,
        alternative: Box<ActionValue>,
    },
}
