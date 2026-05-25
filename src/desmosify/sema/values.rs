use std::rc::Rc;
use crate::ast::{BinaryOperation, RangeKind, UnaryOperation};
use crate::sema::display::ImageValue;
use crate::sema::intrinsic::{IntrinsicColorKind, IntrinsicFunction, IntrinsicValue};
use crate::sema::types::{ListState, Type};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MathematicalConstant {
    Pi,
    Tau,
    E,
}

#[derive(Clone, PartialEq)]
pub struct GlobalReference {
    pub identifier: Rc<str>,
    pub value_type: Type,
}

impl std::fmt::Debug for GlobalReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GlobalReference<{}>({})", self.value_type, self.identifier)
    }
}

#[derive(Clone, PartialEq)]
pub struct LocalReference {
    pub id: u64,
    pub value_type: Type,
}

impl std::fmt::Debug for LocalReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LocalReference<{}>({})", self.value_type, self.id)
    }
}

#[derive(Clone, PartialEq)]
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
    pub fn list_state(&self) -> Option<ListState> {
        match self {
            Self::Single { index } => index.get_type().list_state(),
            Self::Range { .. } |
            Self::RangeFrom { .. } |
            Self::RangeTo { .. } => Some(ListState::IsList),
        }
    }
}

impl std::fmt::Debug for ValueIndexOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ValueIndexOperation::Single { index } => {
                f.debug_tuple("Single").field(index).finish()
            }
            ValueIndexOperation::Range { kind, from_index, to_index, step } => {
                write!(f, "Range{kind:?}")?;
                f.debug_tuple("").field(from_index).field(to_index).field(step).finish()
            }
            ValueIndexOperation::RangeFrom { from_index, step } => {
                f.debug_tuple("RangeFrom").field(from_index).field(step).finish()
            }
            ValueIndexOperation::RangeTo { kind, to_index } => {
                write!(f, "RangeTo{kind:?}")?;
                f.debug_tuple("").field(to_index).finish()
            }
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct ValueListMapLoop {
    pub local: LocalReference,
    pub local_span: Option<crate::Span>,
    pub list: Value,
}

impl std::fmt::Debug for ValueListMapLoop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("MapLoop").field(&self.local).field(&self.list).finish()
    }
}

#[derive(Clone, PartialEq)]
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
    Image(Box<ImageValue>, Option<ListState>),
    Intrinsic(Box<IntrinsicValue>),
    IntrinsicFunction(&'static IntrinsicFunction),
    Global(GlobalReference),
    Local(LocalReference),
    AssumeType(Box<Value>, Type),
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
    pub fn with_span(self, span: Option<crate::Span>) -> Value {
        Value {
            kind: self,
            span,
        }
    }

    pub fn as_const_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None
        }
    }

    pub fn as_const_str(&self) -> Option<Rc<str>> {
        match self {
            Self::Str(value) => Some(value.clone()),
            _ => None
        }
    }

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
            Self::Image(_, list_state) => {
                Type::Image.unflatten_list(*list_state)
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
            Self::AssumeType(_, result_type) => {
                result_type.clone()
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
                item_type.clone().into_list(ListState::IsList)
            }
            Self::ListRange { item_type, .. } => {
                item_type.clone().into_list(ListState::IsList)
            }
            Self::ListFill { value, .. } => {
                value.get_type().into_list(ListState::IsList)
            }
            Self::ListMap { value, .. } => {
                value.get_type().into_list(ListState::IsList)
            }
            Self::ListFilter { item_type, .. } => {
                item_type.clone().into_list(ListState::IsList)
            }
            Self::Index { operation, item_type, .. } => {
                item_type.clone().unflatten_list(operation.list_state())
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
        let (self_list, self_type) = self_type.flatten_list();
        let (target_list, target_type) = target_type.flatten_list();

        let mismatched_types_error = || Box::new(crate::Error {
            kind: crate::ErrorKind::MismatchedTypes {
                expected: target_type.clone().unflatten_list(target_list).to_string(),
                got: self_type.clone().unflatten_list(self_list).to_string(),
            },
            span,
        });

        if !ListState::can_coerce(self_list, target_list, allow_broadcast) {
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
                (Self::Int(value), Type::Color) => {
                    Ok(Self::Intrinsic(Box::new(IntrinsicValue::Color {
                        kind: IntrinsicColorKind::Rgb,
                        value_1: Self::Int(value >> 16 & 0xFF).with_span(span),
                        value_2: Self::Int(value >> 8 & 0xFF).with_span(span),
                        value_3: Self::Int(value & 0xFF).with_span(span),
                        list_state: None,
                    })))
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
        let (self_list, mut self_type) = self.get_type().into_flatten_list();

        constraint(&self_type).map_err(|error| error.with_span(span))?;

        match &self_type {
            Type::Bool => {
                self = self.coerce_to(&Type::Int, false, span)?;
                self_type = Type::Int;
            }
            _ => {}
        }

        Ok((self, self_type.unflatten_list(self_list)))
    }
}

impl std::fmt::Debug for ValueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let self_type = self.get_type();
        match self {
            ValueKind::Type { identifier } => {
                write!(f, "Type({identifier})")
            }
            ValueKind::Undefined(..) => {
                write!(f, "Undefined<{self_type}>")
            }
            ValueKind::Real(value) => {
                f.debug_tuple("Real").field(value).finish()
            }
            ValueKind::Mathematical { kind, coefficient } => {
                f.debug_tuple("Mathematical").field(kind).field(coefficient).finish()
            }
            ValueKind::Int(value) => {
                f.debug_tuple("Int").field(value).finish()
            }
            ValueKind::Bool(value) => {
                f.debug_tuple("Bool").field(value).finish()
            }
            ValueKind::EnumVariant { variant_ordinal, .. } => {
                write!(f, "EnumVariant<{self_type}>")?;
                f.debug_tuple("").field(variant_ordinal).finish()
            }
            ValueKind::Str(value) => {
                f.debug_tuple("Str").field(value).finish()
            }
            ValueKind::Image(image, _) => {
                image.fmt(f)
            }
            ValueKind::Intrinsic(value) => {
                value.fmt(f)
            }
            ValueKind::IntrinsicFunction(function) => {
                function.fmt(f)
            }
            ValueKind::Global(global) => {
                global.fmt(f)
            }
            ValueKind::Local(local) => {
                local.fmt(f)
            }
            ValueKind::AssumeType(value, _) => {
                write!(f, "AssumeType<{self_type}>")?;
                f.debug_tuple("").field(value).finish()
            }
            ValueKind::Unary { operation, operand, .. } => {
                write!(f, "{operation:?}<{self_type}>")?;
                f.debug_tuple("").field(operand).finish()
            }
            ValueKind::Binary { operation, lhs, rhs, .. } => {
                write!(f, "{operation:?}<{self_type}>")?;
                f.debug_tuple("").field(lhs).field(rhs).finish()
            }
            ValueKind::Point2 { x, y, .. } => {
                write!(f, "Point2<{self_type}>")?;
                f.debug_tuple("").field(x).field(y).finish()
            }
            ValueKind::Point3 { x, y, z, .. } => {
                write!(f, "Point3<{self_type}>")?;
                f.debug_tuple("").field(x).field(y).field(z).finish()
            }
            ValueKind::GetX { point, .. } => {
                write!(f, "GetX<{self_type}>")?;
                f.debug_tuple("").field(point).finish()
            }
            ValueKind::GetY { point, .. } => {
                write!(f, "GetY<{self_type}>")?;
                f.debug_tuple("").field(point).finish()
            }
            ValueKind::GetZ { point, .. } => {
                write!(f, "GetZ<{self_type}>")?;
                f.debug_tuple("").field(point).finish()
            }
            ValueKind::List { items, .. } => {
                write!(f, "List<{self_type}>")?;
                items
                    .iter()
                    .fold(
                        &mut f.debug_tuple(""),
                        |tuple, item| tuple.field(item),
                    )
                    .finish()
            }
            ValueKind::ListRange { kind, start, end, step, .. } => {
                write!(f, "ListRange{kind:?}<{self_type}>")?;
                f.debug_tuple("").field(start).field(end).field(step).finish()
            }
            ValueKind::ListFill { value, count } => {
                write!(f, "ListFill<{self_type}>")?;
                f.debug_tuple("").field(value).field(count).finish()
            }
            ValueKind::ListMap { loops, value } => {
                write!(f, "ListMap<{self_type}>")?;
                f.debug_tuple("").field(loops).field(value).finish()
            }
            ValueKind::ListFilter { list, condition, .. } => {
                write!(f, "ListFilter<{self_type}>")?;
                f.debug_tuple("").field(list).field(condition).finish()
            }
            ValueKind::Index { list, operation, .. } => {
                write!(f, "Index<{self_type}>")?;
                f.debug_tuple("").field(list).field(operation).finish()
            }
            ValueKind::Conditional { condition_consequents, alternative, .. } => {
                write!(f, "Conditional<{self_type}>")?;
                condition_consequents
                    .iter()
                    .fold(
                        &mut f.debug_tuple(""),
                        |tuple, pair| tuple.field(pair),
                    )
                    .field(alternative)
                    .finish()
            }
            ValueKind::UserFunctionCall { function, arguments, .. } => {
                write!(f, "UserFunctionCall<{self_type}>")?;
                f.debug_tuple("").field(function).field(arguments).finish()
            }
            ValueKind::Let { local, value, inner, .. } => {
                write!(f, "Let<{self_type}>")?;
                f.debug_tuple("").field(local).field(value).field(inner).finish()
            }
        }
    }
}

#[derive(Clone)]
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

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(f)
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

#[derive(Clone)]
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

impl std::fmt::Debug for ActionValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}
