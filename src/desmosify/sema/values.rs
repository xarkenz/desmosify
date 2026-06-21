use std::rc::Rc;
use crate::ast::RangeKind;
use crate::sema::display::ImageValue;
use crate::sema::intrinsic::IntrinsicFunction;
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
pub struct ActionReference {
    pub identifier: Rc<str>,
    pub action_type: Type,
}

impl std::fmt::Debug for ActionReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ActionReference({})", self.identifier)
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

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum UnaryKind {
    AssumeType,
    Positive,
    Negative,
    LogicalNot,
    GetX,
    GetY,
    GetZ,
    Sin,
    Cos,
    Tan,
    Csc,
    Sec,
    Cot,
    Arcsin,
    Arccos,
    Arctan,
    Arccsc,
    Arcsec,
    Arccot,
    Sinh,
    Cosh,
    Tanh,
    Csch,
    Sech,
    Coth,
    Exp,
    Ln,
    Ceil,
    Floor,
    Round,
    Abs,
    Sign,
    Sqrt,
    Cbrt,
    Factorial,
    MidpointOfSegment,
    VectorStart,
    VectorEnd,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum BinaryKind {
    Exponent,
    Multiply,
    DotProduct,
    CrossProduct,
    Divide,
    Remainder,
    Add,
    Subtract,
    Equal,
    NotEqual,
    LogicalAnd,
    LogicalOr,
    Log,
    Lcm,
    Gcd,
    NthRoot,
    MidpointOfPoints,
    Segment,
    Segment3D,
    Line,
    Ray,
    Vector,
    Vector3D,
    Circle,
    Sphere3D,
    Rectangle,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum InequalityKind {
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ReducerKind {
    Mean,
    Median,
    Min,
    Max,
    Stdev,
    Stdevp,
    Var,
    Varp,
    Mad,
    Count,
    Total,
    Polygon,
    Lcm,
    Gcd,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum DoubleReducerKind {
    Cov,
    Covp,
    Corr,
    Spearman,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ParameterizedReducerKind {
    Quartile,
    Quantile,
    Tscore,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ColorKind {
    Rgb,
    Hsv,
    Okhsv,
    Oklab,
    Oklch,
}

#[derive(Clone, PartialEq)]
pub enum IndexKind {
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

impl IndexKind {
    pub fn list_state(&self) -> Option<ListState> {
        match self {
            Self::Single { index } => index.get_type().list_state(),
            Self::Range { .. } |
            Self::RangeFrom { .. } |
            Self::RangeTo { .. } => Some(ListState::IsList),
        }
    }
}

impl std::fmt::Debug for IndexKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            IndexKind::Single { index } => {
                f.debug_tuple("Single").field(index).finish()
            }
            IndexKind::Range { kind, from_index, to_index, step } => {
                write!(f, "Range{kind:?}")?;
                f.debug_tuple("").field(from_index).field(to_index).field(step).finish()
            }
            IndexKind::RangeFrom { from_index, step } => {
                f.debug_tuple("RangeFrom").field(from_index).field(step).finish()
            }
            IndexKind::RangeTo { kind, to_index } => {
                write!(f, "RangeTo{kind:?}")?;
                f.debug_tuple("").field(to_index).finish()
            }
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct ListMapLoop {
    pub local: LocalReference,
    pub local_span: Option<crate::Span>,
    pub list: Value,
}

impl std::fmt::Debug for ListMapLoop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ListMapLoop").field(&self.local).field(&self.list).finish()
    }
}

#[derive(Clone, PartialEq)]
pub enum ValueKind {
    Type {
        identifier: Rc<str>,
    },
    Undefined(Type),
    Infinity(Type),
    Real(f64),
    Mathematical(MathematicalConstant),
    Int(i64),
    Bool(bool),
    EnumVariant {
        type_identifier: Rc<str>,
        variant_ordinal: i64,
    },
    Str(Rc<str>),
    Image(Box<ImageValue>, Option<ListState>),
    IntrinsicFunction(&'static IntrinsicFunction),
    Global(GlobalReference),
    Action(ActionReference),
    Local(LocalReference),
    ViewportWidth,
    ViewportHeight,
    TickerDt,
    ClickIndex,
    Unary {
        kind: UnaryKind,
        operand: Box<Value>,
        result_type: Type,
    },
    Binary {
        kind: BinaryKind,
        lhs: Box<Value>,
        rhs: Box<Value>,
        result_type: Type,
    },
    InequalityChain {
        lhs: Box<Value>,
        chain: Box<[(InequalityKind, Value)]>,
        result_type: Type,
    },
    Reducer {
        kind: ReducerKind,
        list: Box<Value>,
        result_type: Type,
    },
    ArgumentsReducer {
        kind: ReducerKind,
        arguments: Box<[Value]>,
        result_type: Type,
    },
    DoubleReducer {
        kind: DoubleReducerKind,
        list_1: Box<Value>,
        list_2: Box<Value>,
        result_type: Type,
    },
    ParameterizedReducer {
        kind: ParameterizedReducerKind,
        list: Box<Value>,
        parameter: Box<Value>,
        result_type: Type,
    },
    Color {
        kind: ColorKind,
        value_1: Box<Value>,
        value_2: Box<Value>,
        value_3: Box<Value>,
        list_state: Option<ListState>,
    },
    Random {
        source: Option<Box<Value>>,
        sample_count: Option<Box<Value>>,
        result_type: Type,
    },
    SeededRandom {
        source: Option<Box<Value>>,
        sample_count: Box<Value>,
        seed: Box<Value>,
        result_type: Type,
    },
    Join {
        values: Box<[Value]>,
        result_type: Type,
    },
    Sort {
        list: Box<Value>,
        key_list: Option<Box<Value>>,
    },
    Shuffle {
        list: Box<Value>,
        seed: Option<Box<Value>>,
    },
    Unique {
        list: Box<Value>,
    },
    Dilation {
        object: Box<Value>,
        point: Box<Value>,
        factor: Box<Value>,
        result_type: Type,
    },
    Rotation {
        object: Box<Value>,
        point: Box<Value>,
        angle: Box<Value>,
        result_type: Type,
    },
    Reflection {
        object: Box<Value>,
        line: Box<Value>,
        result_type: Type,
    },
    TranslationByPoints {
        object: Box<Value>,
        point_1: Box<Value>,
        point_2: Box<Value>,
        result_type: Type,
    },
    TranslationByVector {
        object: Box<Value>,
        vector: Box<Value>,
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
        loops: Box<[ListMapLoop]>,
        value: Box<Value>,
    },
    ListFilter {
        list: Box<Value>,
        condition: Box<Value>,
        item_type: Type,
    },
    Index {
        list: Box<Value>,
        kind: IndexKind,
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
    InlineAction {
        parameters: Box<[LocalReference]>,
        action: Box<ActionValue>,
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
            Self::Infinity(value_type) => {
                value_type.clone()
            }
            Self::Real(..) |
            Self::Mathematical { .. } |
            Self::ViewportWidth |
            Self::ViewportHeight |
            Self::TickerDt => {
                Type::Real
            }
            Self::Int(..) |
            Self::ClickIndex => {
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
            Self::IntrinsicFunction(function) => {
                Type::IntrinsicFunction(*function)
            }
            Self::Global(reference) => {
                reference.value_type.clone()
            }
            Self::Action(reference) => {
                reference.action_type.clone()
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
            Self::InequalityChain { result_type, .. } => {
                result_type.clone()
            }
            Self::Reducer { result_type, .. } => {
                result_type.clone()
            }
            Self::ArgumentsReducer { result_type, .. } => {
                result_type.clone()
            }
            Self::DoubleReducer { result_type, .. } => {
                result_type.clone()
            }
            Self::ParameterizedReducer { result_type, .. } => {
                result_type.clone()
            }
            Self::Color { list_state, .. } => {
                Type::Color.unflatten_list(*list_state)
            }
            Self::Join { result_type, .. } => {
                result_type.clone()
            }
            Self::Sort { list, .. } => {
                list.get_type()
            }
            Self::Random { result_type, .. } => {
                result_type.clone()
            }
            Self::SeededRandom { result_type, .. } => {
                result_type.clone()
            }
            Self::Shuffle { list, .. } => {
                list.get_type()
            }
            Self::Unique { list } => {
                list.get_type()
            }
            Self::Dilation { result_type, .. } => {
                result_type.clone()
            }
            Self::Rotation { result_type, .. } => {
                result_type.clone()
            }
            Self::Reflection { result_type, .. } => {
                result_type.clone()
            }
            Self::TranslationByPoints { result_type, .. } => {
                result_type.clone()
            }
            Self::TranslationByVector { result_type, .. } => {
                result_type.clone()
            }
            Self::Point2 { point_type, .. } => {
                point_type.clone()
            }
            Self::Point3 { point_type, .. } => {
                point_type.clone()
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
            Self::Index { kind: operation, item_type, .. } => {
                item_type.clone().unflatten_list(operation.list_state())
            }
            Self::Conditional { result_type, .. } => {
                result_type.clone()
            }
            Self::UserFunctionCall { return_type, .. } => {
                return_type.clone()
            }
            Self::InlineAction { parameters, .. } => {
                Type::Action {
                    parameter_types: parameters
                        .iter()
                        .map(|local| local.value_type.clone())
                        .collect(),
                }
            }
        }
    }

    pub fn is_zero(&self) -> bool {
        match *self {
            Self::Real(value) => value == 0.0,
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
            Self::Real(value) => value.is_nan(),
            _ => false
        }
    }

    pub fn assume_type(self, value_type: Type, span: Option<crate::Span>) -> Self {
        if value_type == self.get_type() {
            self
        }
        else {
            ValueKind::Unary {
                kind: UnaryKind::AssumeType,
                operand: Box::new(Value {
                    kind: self,
                    span,
                }),
                result_type: value_type,
            }
        }
    }

    pub fn coerce_to(self, target_type: &Type, allow_list: bool, span: Option<crate::Span>) -> crate::Result<Self> {
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

        if !ListState::can_coerce(self_list, target_list, allow_list) {
            Err(mismatched_types_error())
        }
        else if self_type == target_type {
            // The value shouldn't need to be transformed in any way
            Ok(self)
        }
        else if !self_type.can_coerce_to(target_type) {
            Err(mismatched_types_error())
        }
        else {
            match (self, target_type) {
                (Self::Undefined(..), _) => {
                    Ok(Self::Undefined(target_type.clone()))
                }
                (Self::Infinity(..), _) => {
                    Ok(Self::Infinity(target_type.clone()))
                }
                (Self::Int(value), Type::Real) => {
                    Ok(Self::Real(value as f64))
                }
                (Self::Int(value), Type::Color) => {
                    Ok(Self::Color {
                        kind: ColorKind::Rgb,
                        value_1: Box::new(Self::Int(value >> 16 & 0xFF).with_span(span)),
                        value_2: Box::new(Self::Int(value >> 8 & 0xFF).with_span(span)),
                        value_3: Box::new(Self::Int(value & 0xFF).with_span(span)),
                        list_state: None,
                    })
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
                (self_, _) => Ok(self_)
            }
        }
    }

    pub fn coerce_to_arithmetic(mut self, constraint: fn(&Type) -> crate::Result<()>, span: Option<crate::Span>) -> crate::Result<Self> {
        let (self_list, self_type) = self.get_type().into_flatten_list();

        constraint(&self_type).map_err(|error| error.with_span(span))?;

        let result_type = match &self_type {
            Type::Bool => {
                self = self.coerce_to(&Type::Int, true, span)?;
                Type::Int
            }
            _ => self_type
        };

        Ok(self.assume_type(result_type.unflatten_list(self_list), span))
    }
}

impl std::fmt::Debug for ValueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let self_type = self.get_type();
        match self {
            Self::Type { identifier } => {
                write!(f, "Type({identifier})")
            }
            Self::Undefined(..) => {
                write!(f, "Undefined<{self_type}>")
            }
            Self::Infinity(..) => {
                write!(f, "Infinity<{self_type}>")
            }
            Self::Real(value) => {
                f.debug_tuple("Real").field(value).finish()
            }
            Self::Mathematical(kind) => {
                kind.fmt(f)
            }
            Self::Int(value) => {
                f.debug_tuple("Int").field(value).finish()
            }
            Self::Bool(value) => {
                f.debug_tuple("Bool").field(value).finish()
            }
            Self::EnumVariant { variant_ordinal, .. } => {
                write!(f, "EnumVariant<{self_type}>")?;
                f.debug_tuple("").field(variant_ordinal).finish()
            }
            Self::Str(value) => {
                f.debug_tuple("Str").field(value).finish()
            }
            Self::Image(image, _) => {
                image.fmt(f)
            }
            Self::IntrinsicFunction(function) => {
                function.fmt(f)
            }
            Self::Global(reference) => {
                reference.fmt(f)
            }
            Self::Action(reference) => {
                reference.fmt(f)
            }
            Self::Local(reference) => {
                reference.fmt(f)
            }
            Self::ViewportWidth => {
                write!(f, "ViewportWidth")
            }
            Self::ViewportHeight => {
                write!(f, "ViewportHeight")
            }
            Self::TickerDt => {
                write!(f, "TickerDt")
            }
            Self::ClickIndex => {
                write!(f, "ClickIndex")
            }
            Self::Unary { kind, operand, .. } => {
                write!(f, "{kind:?}<{self_type}>")?;
                f.debug_tuple("").field(operand).finish()
            }
            Self::Binary { kind, lhs, rhs, .. } => {
                write!(f, "{kind:?}<{self_type}>")?;
                f.debug_tuple("").field(lhs).field(rhs).finish()
            }
            Self::InequalityChain { lhs, chain, .. } => {
                write!(f, "InequalityChain<{self_type}>")?;
                let mut tuple = f.debug_tuple("");
                tuple.field(lhs);
                for (kind, rhs) in chain {
                    tuple.field(kind).field(rhs);
                }
                tuple.finish()
            }
            Self::Reducer { kind, list, .. } => {
                write!(f, "{kind:?}<{self_type}>")?;
                f.debug_tuple("").field(list).finish()
            }
            Self::ArgumentsReducer { kind, arguments, .. } => {
                write!(f, "{kind:?}<{self_type}>")?;
                f.debug_tuple("").field(arguments).finish()
            }
            Self::DoubleReducer { kind, list_1, list_2, .. } => {
                write!(f, "{kind:?}<{self_type}>")?;
                f.debug_tuple("").field(list_1).field(list_2).finish()
            }
            Self::ParameterizedReducer { kind, list, parameter, .. } => {
                write!(f, "{kind:?}<{self_type}>")?;
                f.debug_tuple("").field(list).field(parameter).finish()
            }
            Self::Color { kind, value_1, value_2, value_3, .. } => {
                write!(f, "{kind:?}<{self_type}>")?;
                f.debug_tuple("").field(value_1).field(value_2).field(value_3).finish()
            }
            Self::Join { values, .. } => {
                write!(f, "Join<{self_type}>")?;
                values
                    .iter()
                    .fold(
                        &mut f.debug_tuple(""),
                        |tuple, argument| tuple.field(argument),
                    )
                    .finish()
            }
            Self::Sort { list, key_list } => {
                write!(f, "Sort<{self_type}>")?;
                let mut tuple = f.debug_tuple("");
                tuple.field(list);
                if let Some(key_list) = key_list {
                    tuple.field(key_list);
                }
                tuple.finish()
            }
            Self::Random { source, sample_count, .. } => {
                write!(f, "Random<{self_type}>")?;
                let mut tuple = f.debug_tuple("");
                if let Some(source) = source {
                    tuple.field(source);
                }
                if let Some(sample_count) = sample_count {
                    tuple.field(sample_count);
                }
                tuple.finish()
            }
            Self::SeededRandom { source, sample_count, seed, .. } => {
                write!(f, "Random<{self_type}>")?;
                let mut tuple = f.debug_tuple("");
                if let Some(source) = source {
                    tuple.field(source);
                }
                tuple.field(sample_count).field(seed).finish()
            }
            Self::Shuffle { list, seed } => {
                write!(f, "Shuffle<{self_type}>")?;
                let mut tuple = f.debug_tuple("");
                tuple.field(list);
                if let Some(seed) = seed {
                    tuple.field(seed);
                }
                tuple.finish()
            }
            Self::Unique { list } => {
                write!(f, "Unique<{self_type}>")?;
                f.debug_tuple("").field(list).finish()
            }
            Self::Dilation { object, point, factor, .. } => {
                write!(f, "Dilation<{self_type}>")?;
                f.debug_tuple("").field(object).field(point).field(factor).finish()
            }
            Self::Rotation { object, point, angle, .. } => {
                write!(f, "Rotation<{self_type}>")?;
                f.debug_tuple("").field(object).field(point).field(angle).finish()
            }
            Self::Reflection { object, line, .. } => {
                write!(f, "Reflection<{self_type}>")?;
                f.debug_tuple("").field(object).field(line).finish()
            }
            Self::TranslationByPoints { object, point_1, point_2, .. } => {
                write!(f, "TranslationByPoints<{self_type}>")?;
                f.debug_tuple("").field(object).field(point_1).field(point_2).finish()
            }
            Self::TranslationByVector { object, vector, .. } => {
                write!(f, "TranslationByVector<{self_type}>")?;
                f.debug_tuple("").field(object).field(vector).finish()
            }
            Self::Point2 { x, y, .. } => {
                write!(f, "Point2<{self_type}>")?;
                f.debug_tuple("").field(x).field(y).finish()
            }
            Self::Point3 { x, y, z, .. } => {
                write!(f, "Point3<{self_type}>")?;
                f.debug_tuple("").field(x).field(y).field(z).finish()
            }
            Self::List { items, .. } => {
                write!(f, "List<{self_type}>")?;
                items
                    .iter()
                    .fold(
                        &mut f.debug_tuple(""),
                        |tuple, item| tuple.field(item),
                    )
                    .finish()
            }
            Self::ListRange { kind, start, end, step, .. } => {
                write!(f, "ListRange{kind:?}<{self_type}>")?;
                f.debug_tuple("").field(start).field(end).field(step).finish()
            }
            Self::ListFill { value, count } => {
                write!(f, "ListFill<{self_type}>")?;
                f.debug_tuple("").field(value).field(count).finish()
            }
            Self::ListMap { loops, value } => {
                write!(f, "ListMap<{self_type}>")?;
                f.debug_tuple("").field(loops).field(value).finish()
            }
            Self::ListFilter { list, condition, .. } => {
                write!(f, "ListFilter<{self_type}>")?;
                f.debug_tuple("").field(list).field(condition).finish()
            }
            Self::Index { list, kind: operation, .. } => {
                write!(f, "Index<{self_type}>")?;
                f.debug_tuple("").field(list).field(operation).finish()
            }
            Self::Conditional { condition_consequents, alternative, .. } => {
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
            Self::UserFunctionCall { function, arguments, .. } => {
                write!(f, "UserFunctionCall<{self_type}>")?;
                f.debug_tuple("").field(function).field(arguments).finish()
            }
            Self::InlineAction { parameters, action } => {
                write!(f, "InlineAction<{self_type}>")?;
                f.debug_tuple("").field(parameters).field(action).finish()
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

    pub fn assume_type(mut self, value_type: Type) -> Self {
        self.kind = self.kind.assume_type(value_type, self.span);
        self
    }

    pub fn coerce_to(mut self, target_type: &Type, allow_list: bool) -> crate::Result<Self> {
        self.kind = self.kind.coerce_to(target_type, allow_list, self.span)?;
        Ok(self)
    }

    pub fn coerce_to_arithmetic(mut self, constraint: fn(&Type) -> crate::Result<()>) -> crate::Result<Self> {
        self.kind = self.kind.coerce_to_arithmetic(constraint, self.span)?;
        Ok(self)
    }

    pub fn get_const_str(&self) -> crate::Result<Rc<str>> {
        self.kind
            .as_const_str()
            .ok_or_else(|| Box::new(crate::Error {
                kind: crate::ErrorKind::ExpectedConstant {
                    type_identifier: Type::Str.to_string(),
                },
                span: self.span,
            }))
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

#[derive(Clone, PartialEq)]
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
        action: Box<Value>,
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

impl std::fmt::Debug for ActionValueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disable => {
                write!(f, "Disable")
            }
            Self::Compound { actions } => {
                let mut tuple = f.debug_tuple("Compound");
                for action in actions {
                    tuple.field(action);
                }
                tuple.finish()
            }
            Self::Update { variable, value, .. } => {
                f.debug_tuple("Update")
                    .field(variable)
                    .field(value)
                    .finish()
            }
            Self::ActionCall { action, arguments } => {
                f.debug_tuple("ActionCall")
                    .field(action)
                    .field(arguments)
                    .finish()
            }
            Self::Conditional { condition_consequents, alternative } => {
                condition_consequents
                    .iter()
                    .fold(
                        &mut f.debug_tuple("Conditional"),
                        |tuple, pair| tuple.field(pair),
                    )
                    .field(alternative)
                    .finish()
            }
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

impl PartialEq for ActionValue {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
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
