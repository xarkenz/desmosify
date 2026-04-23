use std::rc::Rc;
use crate::ast::{BinaryOperation, RangeKind, UnaryOperation};
use crate::sema::intrinsic::Intrinsic;
use crate::sema::types::DataType;

#[derive(Clone, Debug)]
pub enum Constant {
    Real(f64),
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
        item_type: DataType,
    },
    EnumVariant {
        type_identifier: Rc<str>,
        variant_ordinal: i64,
    },
}

impl Constant {
    pub fn get_type(&self) -> DataType {
        match self {
            Self::Real(..) => DataType::Real,
            Self::Int(..) => DataType::Int,
            Self::Bool(..) => DataType::Bool,
            Self::Point2 { x, y } => DataType::Point2 {
                x_type: Box::new(x.get_type()),
                y_type: Box::new(y.get_type()),
            },
            Self::Point3 { x, y, z } => DataType::Point3 {
                x_type: Box::new(x.get_type()),
                y_type: Box::new(y.get_type()),
                z_type: Box::new(z.get_type()),
            },
            Self::List { item_type, .. } => DataType::List {
                item_type: Box::new(item_type.clone()),
            },
            Self::EnumVariant { type_identifier, .. } => DataType::UserValue {
                type_identifier: type_identifier.clone(),
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct GlobalReference {
    pub identifier: Rc<str>,
    pub value_type: DataType,
}

#[derive(Clone, Debug)]
pub struct LocalReference {
    pub id: usize,
    pub value_type: DataType,
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
    Intrinsic(&'static Intrinsic),
    Global(GlobalReference),
    Local(LocalReference),
    Unary {
        operation: UnaryOperation,
        operand: Box<Value>,
        result_type: DataType,
    },
    Binary {
        operation: BinaryOperation,
        lhs: Box<Value>,
        rhs: Box<Value>,
        result_type: DataType,
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
        x_type: DataType,
    },
    GetY {
        point: Box<Value>,
        y_type: DataType,
    },
    GetZ {
        point: Box<Value>,
        z_type: DataType,
    },
    List {
        items: Box<[Value]>,
        item_type: DataType,
    },
    ListRange {
        kind: RangeKind,
        start: Box<Value>,
        end: Box<Value>,
        step: Box<Value>,
        item_type: DataType,
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
        item_type: DataType,
    },
    Index {
        list: Box<Value>,
        operation: ValueIndexOperation,
        item_type: DataType,
    },
    Conditional {
        condition_consequents: Box<[(Value, Value)]>,
        alternative: Box<Value>,
        result_type: DataType,
    },
    FunctionCall {
        function: Box<Value>,
        arguments: Box<[Value]>,
        return_type: DataType,
    },
    Let {
        identifier: Rc<str>,
        value: Box<Value>,
        inner: Box<Value>,
    },
}

impl Value {
    pub fn get_type(&self) -> DataType {
        match self {
            Self::Constant(constant) => constant.get_type(),
            Self::Intrinsic(intrinsic) => intrinsic.get_type(),
            Self::Global(reference) => reference.value_type.clone(),
            Self::Local(reference) => reference.value_type.clone(),
            Self::Unary { result_type, .. } => result_type.clone(),
            Self::Binary { result_type, .. } => result_type.clone(),
            Self::Point2 { x, y } => DataType::Point2 {
                x_type: Box::new(x.get_type()),
                y_type: Box::new(y.get_type()),
            },
            Self::Point3 { x, y, z } => DataType::Point3 {
                x_type: Box::new(x.get_type()),
                y_type: Box::new(y.get_type()),
                z_type: Box::new(z.get_type()),
            },
            Self::GetX { x_type, .. } => x_type.clone(),
            Self::GetY { y_type, .. } => y_type.clone(),
            Self::GetZ { z_type, .. } => z_type.clone(),
            Self::List { item_type, .. } => DataType::List {
                item_type: Box::new(item_type.clone()),
            },
            Self::ListRange { item_type, .. } => DataType::List {
                item_type: Box::new(item_type.clone()),
            },
            Self::ListFill { value, .. } => DataType::List {
                item_type: Box::new(value.get_type()),
            },
            Self::ListMap { value, .. } => DataType::List {
                item_type: Box::new(value.get_type()),
            },
            Self::ListFilter { item_type, .. } => DataType::List {
                item_type: Box::new(item_type.clone()),
            },
            Self::Index { operation, item_type, .. } => match operation.is_range() {
                false => item_type.clone(),
                true => DataType::List {
                    item_type: Box::new(item_type.clone()),
                },
            },
            Self::Conditional { result_type, .. } => result_type.clone(),
            Self::FunctionCall { return_type, .. } => return_type.clone(),
            Self::Let { inner, .. } => inner.get_type(),
        }
    }
}

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
