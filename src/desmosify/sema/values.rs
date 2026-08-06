use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::rc::Rc;
use crate::ast::RangeKind;
use crate::sema::display::ImageValue;
use crate::sema::intrinsic::IntrinsicFunction;
use crate::sema::types::{ListState, Type, TypeHandle, TypeRegistry};
use crate::util::LazyConst;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum MathematicalConstant {
    Pi,
    Tau,
    E,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum GlobalSymbolKind {
    PrimitiveType,
    UserDefinedType,
    EnumOrdinal,
    Intrinsic,
    Immutable,
    Variable,
    Action,
}

#[derive(Clone, Debug)]
pub struct GlobalSymbol {
    pub kind: GlobalSymbolKind,
    pub identifier: Rc<str>,
    pub value: ValueHandle,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum UnaryKind {
    Positive,
    Negative,
    LogicalNot,
    XOfPoint2D,
    YOfPoint2D,
    XOfPoint3D,
    YOfPoint3D,
    ZOfPoint3D,
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
    Sort,
    Shuffle,
    Unique,
    PrefixSum,
    LineFromSegment2D,
    LineFromRay2D,
    AreaOfPolygon,
    PerimeterOfPolygon,
    VerticesOfPolygon,
    SegmentsOfPolygon,
    UndirectedAnglesOfPolygon,
    DirectedAnglesOfPolygon,
    RadiusOfCircle,
    CenterOfCircle,
    MidpointOfSegment2D,
    MidpointOfSegment3D,
    StartOfVector2D,
    StartOfVector3D,
    EndOfVector2D,
    EndOfVector3D,
    ReflectionByLine2D,
    TranslationByPoint2D,
    InverseOfTransform2D,
    BoolToInternal,
    BoolFromInternal,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
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
    /// lhs = y, rhs = x
    Arctan2,
    /// lhs = base, rhs = value
    Log,
    /// lhs = value, rhs = digits
    RoundDigits,
    /// lhs = index, rhs = value
    NthRoot,
    /// lhs = list, rhs = key_list
    SortKeyed,
    /// lhs = list, rhs = seed
    ShuffleSeeded,
    /// lhs = x, rhs = y
    Point2D,
    /// lhs = start, rhs = end
    SegmentFromPoints2D,
    /// lhs = start, rhs = end
    SegmentFromPoints3D,
    /// lhs = start, rhs = end
    LineFromPoints2D,
    /// lhs = closed_end, rhs = open_end
    RayFromPoints2D,
    /// lhs = start, rhs = end
    VectorFromPoints2D,
    /// lhs = start, rhs = end
    VectorFromPoints3D,
    /// lhs = center, rhs = radius
    CircleFromRadius2D,
    /// lhs = center, rhs = edge
    CircleFromEdge2D,
    /// lhs = center, rhs = radius
    SphereFromRadius3D,
    /// lhs = corner_1, rhs = corner_2
    RectangleFromPoints2D,
    /// lhs = object, rhs = distance
    Glider2D,
    /// lhs = object, rhs = axis
    Reflect2D,
    /// lhs = object, rhs = vector
    TranslateByVector2D,
    /// lhs = point, rhs = factor
    Dilation2D,
    /// lhs = point, rhs = angle
    Rotation2D,
    /// lhs = transformation, rhs = object
    ApplyTransform2D,
    /// lhs = start, rhs = end
    MidpointOfPoints2D,
    /// lhs = start, rhs = end
    MidpointOfPoints3D,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum TernaryKind {
    /// first = x, second = y, third = z
    Point3D,
    /// first = start, second = thru, third = end
    Arc2D,
    /// first = leg_a, second = center, third = leg_b
    UndirectedAngle2D,
    /// first = start_leg, second = center, third = end_leg
    DirectedAngle2D,
    /// first = a, second = b, third = c
    Triangle3D,
    /// first = object, second = point, third = factor
    Dilate2D,
    /// first = object, second = point, third = angle
    Rotate2D,
    /// first = object, second = start_point, third = end_point
    TranslateByPoints2D,
    Rgb,
    Hsv,
    Okhsv,
    Oklab,
    Oklch,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum InequalityKind {
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum ReducerKind {
    Lcm,
    Gcd,
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
    Polygon2D,
    ComposeTransforms2D,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum DoubleReducerKind {
    Cov,
    Covp,
    Corr,
    Spearman,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum ParameterizedReducerKind {
    Quartile,
    Quantile,
    Tscore,
}

#[derive(Clone, Debug)]
pub enum IndexKind {
    Single {
        index: ValueHandle,
    },
    Range {
        kind: RangeKind,
        from_index: ValueHandle,
        to_index: ValueHandle,
        step: ValueHandle,
    },
    RangeFrom {
        from_index: ValueHandle,
        step: ValueHandle,
    },
    RangeTo {
        kind: RangeKind,
        to_index: ValueHandle,
    },
}

impl IndexKind {
    pub const fn result_is_list(&self) -> bool {
        !matches!(self, Self::Single { .. })
    }
}

#[derive(Clone, Debug)]
pub struct ListMapLoop {
    pub local: ValueHandle,
    pub list: ValueHandle,
}

#[derive(Clone, Default, Debug)]
pub enum Value {
    /// Indicates a value that is currently unknown. This is distinct from `Undefined`, which is a
    /// known value. This is used as a placeholder for the value of a global variable before it is
    /// fully interpreted. It is also used as the value for local variables such as parameters.
    #[default]
    Opaque,
    Alias(ValueHandle),
    Type(TypeHandle),
    Undefined,
    Infinity,
    Real(f64),
    Mathematical(MathematicalConstant),
    Int(i64),
    Bool(bool),
    Str(Rc<str>),
    Image(Box<ImageValue>),
    IntrinsicFunction(&'static IntrinsicFunction),
    IntrinsicReference(GlobalSymbol),
    GlobalReference(GlobalSymbol),
    ActionReference(GlobalSymbol),
    Local {
        id: u64,
    },
    ViewportWidth,
    ViewportHeight,
    TickerDt,
    ClickIndex,
    Unary {
        kind: UnaryKind,
        operand: ValueHandle,
    },
    Binary {
        kind: BinaryKind,
        lhs: ValueHandle,
        rhs: ValueHandle,
    },
    Ternary {
        kind: TernaryKind,
        first: ValueHandle,
        second: ValueHandle,
        third: ValueHandle,
    },
    InequalityChain {
        lhs: ValueHandle,
        chain: Box<[(InequalityKind, ValueHandle)]>,
    },
    Reducer {
        kind: ReducerKind,
        list: ValueHandle,
    },
    ArgumentsReducer {
        kind: ReducerKind,
        arguments: Box<[ValueHandle]>,
    },
    DoubleReducer {
        kind: DoubleReducerKind,
        lhs_list: ValueHandle,
        rhs_list: ValueHandle,
    },
    ParameterizedReducer {
        kind: ParameterizedReducerKind,
        list: ValueHandle,
        parameter: ValueHandle,
    },
    Random {
        source: Option<ValueHandle>,
        sample_count: Option<ValueHandle>,
    },
    RandomSeeded {
        source: Option<ValueHandle>,
        sample_count: ValueHandle,
        seed: ValueHandle,
    },
    Join {
        values: Box<[ValueHandle]>,
    },
    List {
        items: Box<[ValueHandle]>,
    },
    ListRange {
        kind: RangeKind,
        start: ValueHandle,
        end: ValueHandle,
        step: ValueHandle,
    },
    ListFill {
        value: ValueHandle,
        count: ValueHandle,
    },
    ListMap {
        loops: Box<[ListMapLoop]>,
        value: ValueHandle,
    },
    ListFilter {
        list: ValueHandle,
        condition: ValueHandle,
    },
    Index {
        list: ValueHandle,
        kind: IndexKind,
    },
    Conditional {
        condition_consequents: Box<[(ValueHandle, ValueHandle)]>,
        alternative: ValueHandle,
    },
    UserFunctionCall {
        function: ValueHandle,
        arguments: Box<[ValueHandle]>,
    },
    Action {
        parameters: Box<[ValueHandle]>,
        action: Box<ActionValue>,
    },
}

impl Value {
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
            Self::Real(value) => value.signum() == 1.0,
            Self::Int(value) => value == 1,
            Self::Bool(value) => value,
            _ => false
        }
    }

    pub fn is_undefined(&self) -> bool {
        match *self {
            Self::Undefined => true,
            Self::Real(value) => value.is_nan(),
            _ => false
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct ValueEntry {
    pub value: Value,
    pub type_handle: TypeHandle,
    pub span: Option<crate::Span>,
}

impl ValueEntry {
    pub fn with_span(self, span: Option<crate::Span>) -> Self {
        Self {
            span,
            ..self
        }
    }

    pub const fn undefined(type_handle: TypeHandle) -> Self {
        Self {
            value: Value::Undefined,
            type_handle,
            span: None,
        }
    }

    pub const fn infinity(type_handle: TypeHandle) -> Self {
        Self {
            value: Value::Infinity,
            type_handle,
            span: None,
        }
    }

    pub const fn real(value: f64) -> Self {
        Self {
            value: Value::Real(value),
            type_handle: TypeHandle::REAL,
            span: None,
        }
    }

    pub const fn int(value: i64) -> Self {
        Self {
            value: Value::Int(value),
            type_handle: TypeHandle::INT,
            span: None,
        }
    }

    pub const fn bool(value: bool) -> Self {
        Self {
            value: Value::Bool(value),
            type_handle: TypeHandle::BOOL,
            span: None,
        }
    }

    pub fn register(self, registry: &mut ValueRegistry) -> ValueHandle {
        registry.register(self)
    }
}

#[derive(Clone, Debug)]
pub enum ActionValueKind {
    /// Indicates an action value that is not yet known. This is used as a placeholder for the value
    /// of an action definition before it is fully interpreted.
    Opaque,
    Disable,
    Compound {
        actions: Box<[ActionValue]>,
    },
    Update {
        variable_identifier: Rc<str>,
        variable_span: Option<crate::Span>,
        value: ValueHandle,
    },
    ActionCall {
        action: ValueHandle,
        arguments: Box<[ValueHandle]>,
    },
    Conditional {
        condition_consequents: Box<[(ValueHandle, ActionValue)]>,
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

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct ValueHandle(NonZeroUsize);

impl ValueHandle {
    const fn new(index: usize) -> Self {
        // This may overflow if index == usize::MAX, but memory will run out before that happens.
        Self(NonZeroUsize::new(index + 1).unwrap())
    }

    const fn index(self) -> usize {
        // This is trivially guaranteed to never underflow.
        self.0.get() - 1
    }

    pub fn entry(self, registry: &ValueRegistry) -> &ValueEntry {
        registry.entry(self)
    }

    pub fn entry_mut(self, registry: &mut ValueRegistry) -> &mut ValueEntry {
        registry.entry_mut(self)
    }

    pub fn get(self, registry: &ValueRegistry) -> &Value {
        registry.get(self)
    }

    pub fn get_type(self, registry: &ValueRegistry) -> TypeHandle {
        registry.get_type(self)
    }

    pub fn get_span(self, registry: &ValueRegistry) -> Option<crate::Span> {
        registry.get_span(self)
    }
}

#[derive(Debug)]
pub struct ValueRegistry {
    entries: Vec<ValueEntry>,
    type_values: HashMap<TypeHandle, ValueHandle>,
}

impl ValueRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            entries: Vec::new(),
            type_values: HashMap::new(),
        };

        for known_type in &KNOWN_VALUES {
            registry.register(known_type.get());
        }

        registry
    }

    pub fn register(&mut self, entry: ValueEntry) -> ValueHandle {
        let handle = ValueHandle::new(self.entries.len());
        self.entries.push(entry);
        handle
    }

    pub fn entry(&self, handle: ValueHandle) -> &ValueEntry {
        &self.entries[handle.index()]
    }

    pub fn entry_mut(&mut self, handle: ValueHandle) -> &mut ValueEntry {
        &mut self.entries[handle.index()]
    }

    pub fn replace_entry(&mut self, handle: ValueHandle, entry: ValueEntry) -> ValueEntry {
        std::mem::replace(&mut self.entries[handle.index()], entry)
    }

    pub fn get(&self, handle: ValueHandle) -> &Value {
        &self.entries[handle.index()].value
    }

    pub fn replace(&mut self, handle: ValueHandle, value: Value) -> Value {
        std::mem::replace(&mut self.entries[handle.index()].value, value)
    }

    pub fn get_type(&self, handle: ValueHandle) -> TypeHandle {
        self.entries[handle.index()].type_handle
    }

    pub fn set_type(&mut self, handle: ValueHandle, type_handle: TypeHandle) {
        self.entries[handle.index()].type_handle = type_handle;
    }

    pub fn get_span(&self, handle: ValueHandle) -> Option<crate::Span> {
        self.entries[handle.index()].span
    }

    pub fn set_span(&mut self, handle: ValueHandle, span: Option<crate::Span>) {
        self.entries[handle.index()].span = span;
    }

    pub fn ignoring_alias(&self, mut handle: ValueHandle) -> ValueHandle {
        while let Value::Alias(alias_handle) = *self.get(handle) {
            handle = alias_handle;
        }
        handle
    }

    pub fn type_value(&mut self, type_handle: TypeHandle) -> ValueHandle {
        if let Some(&handle) = self.type_values.get(&type_handle) {
            handle
        }
        else {
            self.register(ValueEntry {
                value: Value::Type(type_handle),
                type_handle: TypeHandle::META,
                ..Default::default()
            })
        }
    }

    pub fn assume_type(&mut self, handle: ValueHandle, type_handle: TypeHandle, span: Option<crate::Span>) -> ValueHandle {
        if self.get_type(handle) == type_handle {
            handle
        }
        else {
            self.register(ValueEntry {
                value: Value::Alias(handle),
                type_handle,
                span,
                ..Default::default()
            })
        }
    }

    pub fn coerce(&mut self, types: &mut TypeRegistry, handle: ValueHandle, to_type: TypeHandle, allow_list: bool) -> Option<ValueHandle> {
        let from_type = self.get_type(handle);
        let (from_list, from_inner) = types.flatten_list(from_type);
        let (to_list, to_inner) = types.flatten_list(to_type);

        if !ListState::can_coerce(from_list, to_list, allow_list) {
            return None
        }
        let Some(coerced_type) = types.coerce(from_inner, to_inner) else {
            return None
        };
        let result_type = types.unflatten_list(from_list, coerced_type).ok()?;
        let span = self.get_span(handle);

        let coerced = match (self.get(handle), types.get(coerced_type)) {
            (Value::Undefined, _) => {
                Some(Value::Undefined)
            }
            (Value::Infinity, _) => {
                Some(Value::Infinity)
            }
            (&Value::Int(value), Type::Real) => {
                Some(Value::Real(value as f64))
            }
            (&Value::Bool(value), Type::Int) => {
                Some(Value::Int(value as i64))
            }
            (&Value::Bool(value), Type::Real) => {
                Some(Value::Real(value as i32 as f64))
            }
            _ => None
        };

        Some(match coerced {
            Some(value) => self.register(ValueEntry {
                value,
                type_handle: result_type,
                span,
            }),
            None => self.assume_type(handle, result_type, span),
        })
    }
}

macro_rules! known_value_handles {
    ($($handle:ident => ($($rest:tt)+)),* $(,)?) => {
        known_value_handles!(@handle_consts 0usize, $($handle)*);

        pub const KNOWN_VALUES: [LazyConst<ValueEntry>; KNOWN_VALUE_COUNT] = [
            $(known_value_handles!(@lazy_const $($rest)+),)*
        ];
    };
    // I wish I could use ${index(0)} and ${count(0)} and have it be stable.
    (@handle_consts $index:expr, $handle:ident $($rest:ident)*) => {
        impl ValueHandle {
            pub const $handle: Self = Self::new($index);
        }
        known_value_handles!(@handle_consts $index + 1usize, $($rest)*);
    };
    (@handle_consts $count:expr,) => {
        const KNOWN_VALUE_COUNT: usize = $count;
    };
    // I wish I could use into() in a const context and have it be stable.
    (@lazy_const || $definition:expr) => {
        LazyConst::Deferred(|| $definition)
    };
    (@lazy_const $definition:expr) => {
        LazyConst::Immediate($definition)
    };
}

known_value_handles! {
    UNDEFINED => (ValueEntry::undefined(TypeHandle::ANY)),
    INFINITY => (ValueEntry::infinity(TypeHandle::INT)),
    ZERO_REAL => (ValueEntry::real(0.0)),
    ONE_REAL => (ValueEntry::real(1.0)),
    TWO_REAL => (ValueEntry::real(2.0)),
    ZERO_INT => (ValueEntry::int(0)),
    ONE_INT => (ValueEntry::int(1)),
    TWO_INT => (ValueEntry::int(2)),
    FALSE => (ValueEntry::bool(false)),
    TRUE => (ValueEntry::bool(true)),
    PI => (ValueEntry {
        value: Value::Mathematical(MathematicalConstant::Pi),
        type_handle: TypeHandle::REAL,
        span: None,
    }),
    TAU => (ValueEntry {
        value: Value::Mathematical(MathematicalConstant::Tau),
        type_handle: TypeHandle::REAL,
        span: None,
    }),
    E => (ValueEntry {
        value: Value::Mathematical(MathematicalConstant::E),
        type_handle: TypeHandle::REAL,
        span: None,
    }),
    WIDTH_PIXELS => (ValueEntry {
        value: Value::ViewportWidth,
        type_handle: TypeHandle::REAL,
        span: None,
    }),
    HEIGHT_PIXELS => (ValueEntry {
        value: Value::ViewportHeight,
        type_handle: TypeHandle::REAL,
        span: None,
    }),
    TICKER_DT => (ValueEntry {
        value: Value::TickerDt,
        type_handle: TypeHandle::REAL,
        span: None,
    }),
    CLICK_INDEX => (ValueEntry {
        value: Value::ClickIndex,
        type_handle: TypeHandle::INT,
        span: None,
    }),
    BLACK => (ValueEntry {
        value: Value::Ternary {
            kind: TernaryKind::Hsv,
            first: ValueHandle::ZERO_REAL,
            second: ValueHandle::ZERO_REAL,
            third: ValueHandle::ZERO_REAL,
        },
        type_handle: TypeHandle::COLOR,
        span: None,
    }),
    WHITE => (ValueEntry {
        value: Value::Ternary {
            kind: TernaryKind::Hsv,
            first: ValueHandle::ZERO_REAL,
            second: ValueHandle::ZERO_REAL,
            third: ValueHandle::ONE_REAL,
        },
        type_handle: TypeHandle::COLOR,
        span: None,
    }),
    TRANSPARENT_IMAGE_DATA => (|| ValueEntry {
        value: Value::Str("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAAAXNSR0IArs4c6QAAAAtJREFUGFdjYAACAAAFAAGq1chRAAAAAElFTkSuQmCC".into()),
        type_handle: TypeHandle::STR,
        span: None,
    }),
}
