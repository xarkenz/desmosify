use std::rc::Rc;
use crate::ast::{BinaryOperation, RangeKind, UnaryOperation};
use crate::sema::intrinsic::{IntrinsicFunction, IntrinsicValue};
use crate::sema::types::Type;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MathematicalConstant {
    Pi,
    Tau,
    E,
}

#[derive(Clone, PartialEq, Debug)]
pub struct GlobalReference {
    pub identifier: Rc<str>,
    pub value_type: Type,
}

#[derive(Clone, PartialEq, Debug)]
pub struct LocalReference {
    pub id: u64,
    pub value_type: Type,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ValueIndexOperation {
    Single {
        index: Box<Value>,
    },
    Range {
        kind: RangeKind,
        from_index: Box<Value>,
        to_index: Box<Value>,
        step: Box<Value>,
    },
    RangeFrom {
        from_index: Box<Value>,
        step: Box<Value>,
    },
    RangeTo {
        kind: RangeKind,
        to_index: Box<Value>,
    },
}

impl ValueIndexOperation {
    pub fn generates_list(&self) -> bool {
        match self {
            Self::Single { index } => index.get_type().is_list(),
            Self::Range { .. } | Self::RangeFrom { .. } | Self::RangeTo { .. } => true,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct ValueListMapLoop {
    pub local: LocalReference,
    pub local_span: Option<crate::Span>,
    pub list: Value,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ValueKind {
    Type {
        identifier: Rc<str>,
    },
    Undefined(Type),
    Real(f64),
    Mathematical {
        kind: MathematicalConstant,
        coefficient: f64,
    },
    Int(i64),
    Bool(bool),
    EnumVariant {
        type_identifier: Rc<str>,
        variant_ordinal: i64,
    },
    Str(Rc<str>),
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
        point_type: Type,
    },
    Point3 {
        x: Box<Value>,
        y: Box<Value>,
        z: Box<Value>,
        point_type: Type,
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
    UserFunctionCall {
        function: Box<Value>,
        arguments: Box<[Value]>,
        return_type: Type,
    },
    Let {
        local: LocalReference,
        local_span: Option<crate::Span>,
        value: Box<Value>,
        inner: Box<Value>,
    },
}

impl ValueKind {
    pub fn get_type(&self) -> Type {
        match self {
            Self::Type { identifier } => {
                Type::Meta {
                    identifier: identifier.clone(),
                }
            }
            Self::Undefined(value_type) => {
                value_type.clone()
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
            Self::EnumVariant { type_identifier, .. } => {
                Type::UserValue {
                    type_identifier: type_identifier.clone(),
                }
            }
            Self::Str(..) => {
                Type::Str
            }
            Self::Intrinsic(intrinsic) => {
                intrinsic.get_type()
            }
            Self::IntrinsicFunction(function) => {
                Type::IntrinsicFunction(*function)
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
            Self::Point2 { point_type, .. } => {
                point_type.clone()
            }
            Self::Point3 { point_type, .. } => {
                point_type.clone()
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
                item_type.clone().unflatten_list(operation.generates_list())
            }
            Self::Conditional { result_type, .. } => {
                result_type.clone()
            }
            Self::UserFunctionCall { return_type, .. } => {
                return_type.clone()
            }
            Self::Let { inner, .. } => {
                inner.get_type()
            }
        }
    }

    pub fn is_zero(&self) -> bool {
        match *self {
            Self::Real(value) => value == 0.0,
            Self::Mathematical { coefficient, .. } => coefficient == 0.0,
            Self::Int(value) => value == 0,
            Self::Bool(value) => !value,
            _ => false
        }
    }

    pub fn is_one(&self) -> bool {
        match *self {
            Self::Real(value) => value == 1.0,
            Self::Int(value) => value == 1,
            Self::Bool(value) => value,
            _ => false
        }
    }

    pub fn is_undefined(&self) -> bool {
        match *self {
            Self::Undefined(..) => true,
            Self::Real(value) if value.is_nan() => true,
            _ => false
        }
    }

    pub fn coerce_to(self, target_type: &Type, allow_broadcast: bool, span: Option<crate::Span>) -> crate::Result<Self> {
        let self_type = self.get_type();
        let (self_is_list, self_type) = self_type.flatten_list();
        let (target_is_list, target_type) = target_type.flatten_list();

        let mismatched_types_error = || Box::new(crate::Error {
            kind: crate::ErrorKind::MismatchedTypes {
                expected: target_type.clone().unflatten_list(target_is_list).to_string(),
                got: self_type.clone().unflatten_list(self_is_list).to_string(),
            },
            span,
        });

        if self_is_list != target_is_list && !(allow_broadcast && self_is_list) {
            Err(mismatched_types_error())
        }
        else if self_type == target_type || matches!(self_type, Type::Any) || matches!(target_type, Type::Any) {
            // The value shouldn't need to be transformed in any way
            Ok(self)
        }
        else {
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
                (Self::EnumVariant { variant_ordinal, .. }, Type::Int) => {
                    Ok(Self::Int(variant_ordinal))
                }
                (Self::EnumVariant { variant_ordinal, .. }, Type::Real) => {
                    Ok(Self::Real(variant_ordinal as f64))
                }
                (self_, _) => {
                    if self_type.can_coerce_to(target_type) {
                        Ok(self_)
                    }
                    else {
                        Err(mismatched_types_error())
                    }
                }
            }
        }
    }

    pub fn coerce_to_arithmetic(mut self, constraint: fn(&Type) -> crate::Result<()>, span: Option<crate::Span>) -> crate::Result<(Self, Type)> {
        let (self_is_list, mut self_type) = self.get_type().into_flatten_list();

        constraint(&self_type)?;

        match &self_type {
            Type::Bool => {
                self = self.coerce_to(&Type::Int, false, span)?;
                self_type = Type::Int;
            }
            _ => {}
        }

        Ok((self, self_type.unflatten_list(self_is_list)))
    }
}

#[derive(Clone, Debug)]
pub struct Value {
    pub kind: ValueKind,
    pub span: Option<crate::Span>,
}

impl Value {
    pub fn get_type(&self) -> Type {
        self.kind.get_type()
    }

    pub fn is_zero(&self) -> bool {
        self.kind.is_zero()
    }

    pub fn is_one(&self) -> bool {
        self.kind.is_one()
    }

    pub fn is_undefined(&self) -> bool {
        self.kind.is_undefined()
    }

    pub fn coerce_to(mut self, target_type: &Type, allow_broadcast: bool) -> crate::Result<Self> {
        self.kind = self.kind.coerce_to(target_type, allow_broadcast, self.span)?;
        Ok(self)
    }

    pub fn coerce_to_arithmetic(mut self, constraint: fn(&Type) -> crate::Result<()>) -> crate::Result<(Self, Type)> {
        let result_type;
        (self.kind, result_type) = self.kind.coerce_to_arithmetic(constraint, self.span)?;
        Ok((self, result_type))
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl From<ValueKind> for Value {
    fn from(kind: ValueKind) -> Self {
        Self {
            kind,
            span: None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum ActionValueKind {
    Disable,
    Compound {
        actions: Box<[ActionValue]>,
    },
    Update {
        variable: GlobalReference,
        variable_span: Option<crate::Span>,
        value: Box<Value>,
    },
    ActionCall {
        identifier: Rc<str>,
        identifier_span: Option<crate::Span>,
        arguments: Box<[Value]>,
    },
    Conditional {
        condition_consequents: Box<[(Value, ActionValue)]>,
        alternative: Box<ActionValue>,
    },
}

impl ActionValueKind {
    pub fn empty() -> Self {
        Self::Compound {
            actions: Box::new([]),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Compound { actions } => actions.iter().all(ActionValue::is_empty),
            _ => false
        }
    }

    pub fn with_span(self, span: Option<crate::Span>) -> ActionValue {
        ActionValue {
            kind: self,
            span,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ActionValue {
    pub kind: ActionValueKind,
    pub span: Option<crate::Span>,
}

impl ActionValue {
    pub fn is_empty(&self) -> bool {
        self.kind.is_empty()
    }

    pub fn merge(self, other: Self) -> Self {
        Self {
            kind: ActionValueKind::Compound {
                actions: Box::new([self, other]),
            },
            span: None,
        }
    }
}

impl From<ActionValueKind> for ActionValue {
    fn from(kind: ActionValueKind) -> Self {
        Self {
            kind,
            span: None,
        }
    }
}
