use std::path::PathBuf;
use crate::ast::{DefinitionKind, RangeKind, TypeDefinition};
use crate::sema::context::{GlobalContext, LocalContext};
use crate::sema::display::ImageValue;
use crate::sema::types::{ListState, Type, TypeHandle};
use crate::sema::values::{BinaryKind, InequalityKind, ReducerKind, TernaryKind, UnaryKind, Value, ValueEntry, ValueHandle};
use crate::target::Target;

#[derive(Debug)]
pub enum Intrinsic {
    Entry(ValueEntry),
    Handle(ValueHandle),
}

impl From<ValueEntry> for Intrinsic {
    fn from(entry: ValueEntry) -> Self {
        Self::Entry(entry)
    }
}

impl From<ValueHandle> for Intrinsic {
    fn from(handle: ValueHandle) -> Self {
        Self::Handle(handle)
    }
}

pub fn get_core_intrinsics(target: &dyn Target) -> impl Iterator<Item = (&'static str, Intrinsic)> {
    CORE_INTRINSIC_FUNCTIONS
        .iter()
        .map(|&function| {
            (function.identifier, Intrinsic::Entry(ValueEntry {
                value: Value::IntrinsicFunction(function),
                type_handle: TypeHandle::INTRINSIC_FUNCTION,
                span: None,
            }))
        })
        .chain([
            ("pi", ValueHandle::PI.into()),
            ("tau", ValueHandle::TAU.into()),
            ("e", ValueHandle::E.into()),
            ("width_pixels", ValueHandle::WIDTH_PIXELS.into()),
            ("height_pixels", ValueHandle::WIDTH_PIXELS.into()),
            ("black", ValueHandle::BLACK.into()),
            ("white", ValueHandle::WHITE.into()),
            ("transparent_image_data", ValueHandle::TRANSPARENT_IMAGE_DATA.into()),
            ("target", ValueEntry {
                value: Value::Str(target.name().into()),
                type_handle: TypeHandle::STR,
                span: None,
            }.into()),
        ])
}

#[derive(Clone)]
pub struct IntrinsicFunction {
    pub identifier: &'static str,
    pub min_arity: usize,
    pub max_arity: Option<usize>,
    pub interpret_call: fn(
        context: &mut GlobalContext,
        local_context: &LocalContext,
        arguments: Box<[ValueHandle]>,
    ) -> crate::Result<ValueEntry>,
}

impl IntrinsicFunction {
    pub fn interpret_call(
        &self,
        context: &mut GlobalContext,
        local_context: &LocalContext,
        span: Option<crate::Span>,
        arguments: Box<[ValueHandle]>,
    ) -> crate::Result<ValueEntry> {
        self.check_arity(arguments.len(), span)?;

        (self.interpret_call)(context, local_context, arguments)
    }

    pub fn check_arity(&self, argument_count: usize, span: Option<crate::Span>) -> crate::Result<()> {
        if let Some(max_arity) = self.max_arity {
            if (self.min_arity ..= max_arity).contains(&argument_count) {
                Ok(())
            }
            else {
                Err(Box::new(crate::Error {
                    kind: crate::ErrorKind::InvalidIntrinsicArity {
                        identifier: self.identifier.into(),
                        min: self.min_arity,
                        max: max_arity,
                        got: argument_count,
                    },
                    span,
                }))
            }
        }
        else if argument_count >= self.min_arity {
            Ok(())
        }
        else {
            Err(Box::new(crate::Error {
                kind: crate::ErrorKind::InvalidVariadicIntrinsicArity {
                    identifier: self.identifier.into(),
                    min: self.min_arity,
                    got: argument_count,
                },
                span,
            }))
        }
    }
}

impl PartialEq for IntrinsicFunction {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}

impl std::fmt::Debug for IntrinsicFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IntrinsicFunction(@{})", self.identifier)
    }
}

pub const CORE_INTRINSIC_FUNCTIONS: &[&IntrinsicFunction] = &[
    // Trigonometric
    &SIN,
    &COS,
    &TAN,
    &CSC,
    &SEC,
    &COT,
    &ARCSIN,
    &ARCCOS,
    &ARCTAN,
    &ARCTAN2,
    &ARCCSC,
    &ARCSEC,
    &ARCCOT,
    &SINH,
    &COSH,
    &TANH,
    &CSCH,
    &SECH,
    &COTH,
    // Calculus
    &EXP,
    &LN,
    &LOG,
    // &DERIVATIVE,
    // &INTEGRAL,
    // &SUM,
    // &PRODUCT,
    // Number Theory
    &LCM,
    &GCD,
    &CEIL,
    &FLOOR,
    &ROUND,
    &ROUND_DIGITS,
    &ABS,
    &SIGN,
    &SQRT,
    &CBRT,
    &NTH_ROOT,
    // &NPR,
    // &NCR,
    // &FACTORIAL,
    // Complex
    // &REAL,
    // &IMAG,
    // &CONJ,
    // &ARG,
    // List Operations
    &JOIN,
    &SORT,
    &SHUFFLE,
    &UNIQUE,
    &PREFIX_SUM,
    // Statistics
    &MEAN,
    &MEDIAN,
    &MIN,
    &MAX,
    // &QUARTILE,
    // &QUANTILE,
    // &STDEV,
    // &STDEVP,
    // &VAR,
    // &VARP,
    // &COV,
    // &COVP,
    // &MAD,
    // &CORR,
    // &SPEARMAN,
    // &STATS,
    &COUNT,
    &TOTAL,
    &ANY,
    &ALL,
    // Visualizations
    // &HISTOGRAM,
    // &DOT_PLOT,
    // &BOX_PLOT,
    // Distributions
    // &NORMAL_DIST,
    // &T_DIST,
    // &CHISQ_DIST,
    // &UNIFORM_DIST,
    // &BINOMIAL_DIST,
    // &POISSON_DIST,
    // &GEO_DIST,
    // &PDF,
    // &CDF,
    // &INVERSE_CDF,
    &RANDOM,
    &CHOOSE_RANDOM,
    // Statistical Tests
    // &TTEST,
    // &TSCORE,
    // &ITTEST,
    // Geometry
    &MIDPOINT,
    // &INTERSECTION,
    // &STRICT_INTERSECTION,
    &SEGMENT,
    &SEGMENT3D,
    &LINE,
    &RAY,
    &VECTOR,
    &VECTOR3D,
    // &PARALLEL,
    // &PERPENDICULAR,
    &CIRCLE,
    &SPHERE3D,
    &ARC,
    &ANGLE,
    &DIRECTED_ANGLE,
    &POLYGON,
    &RECT,
    &TRIANGLE3D,
    &GLIDER,
    // Properties & Measurements
    // &DOT,
    // &CROSS,
    // &DISTANCE,
    // &LENGTH,
    &AREA,
    &PERIMETER,
    &VERTICES,
    &ANGLES,
    &DIRECTED_ANGLES,
    &SEGMENTS,
    &RADIUS,
    &CENTER,
    // &COTERMINAL,
    // &SUPPLEMENT,
    &START,
    &END,
    // Transformations
    &DILATE,
    &ROTATE,
    &REFLECT,
    &TRANSLATE,
    &DILATION,
    &ROTATION,
    &REFLECTION,
    &TRANSLATION,
    &APPLY,
    &COMPOSE,
    &INVERSE,
    // Color
    &RGB,
    &HSV,
    &OKHSV,
    &OKLAB,
    &OKLCH,
    // Sound
    // &TONE,
    // Desmosify
    &BOOL_TO_INTERNAL,
    &BOOL_FROM_INTERNAL,
    &ENUM_VALUES,
    &ENUM_VALUE,
    &INCLUDE_TEXT,
    &INCLUDE_DATA,
    &IMAGE,
    &CONCAT,
    &TARGET_SYMBOL,
];

pub fn interpret_strict_unary_call(
    context: &mut GlobalContext,
    kind: UnaryKind,
    argument_type: TypeHandle,
    result_type: Option<TypeHandle>,
    arguments: Box<[ValueHandle]>,
) -> crate::Result<Value> {
    let argument = arguments.into_iter().next().unwrap()
        .coerce(context, argument_type, false)?;

    Ok(Value::Unary {
        kind,
        result_type: result_type.unwrap_or(argument_type),
        operand: Box::new(argument),
    })
}

pub fn interpret_broadcastable_unary_call(
    context: &mut GlobalContext,
    kind: UnaryKind,
    argument_type: TypeHandle,
    result_type: Option<TypeHandle>,
    arguments: Box<[ValueHandle]>,
) -> crate::Result<Value> {
    let argument = arguments.into_iter().next().unwrap()
        .coerce(context, &argument_type, true)?;

    Ok(Value::Unary {
        kind,
        result_type: match result_type {
            Some(result_type) => result_type.unflatten_list(argument.get_type().list_state()),
            None => argument.get_type(),
        },
        operand: Box::new(argument),
    })
}

pub fn interpret_broadcastable_binary_call(
    context: &mut GlobalContext,
    kind: BinaryKind,
    lhs_type: TypeHandle,
    rhs_type: TypeHandle,
    result_type: Type,
    arguments: Box<[ValueHandle]>,
) -> crate::Result<Value> {
    let mut arguments = arguments.into_iter();

    let lhs = arguments.next().unwrap()
        .coerce(context, lhs_type, true)?;
    let rhs = arguments.next().unwrap()
        .coerce(context, rhs_type, true)?;

    let list_state = ListState::merge(
        lhs.get_type().list_state(),
        rhs.get_type().list_state(),
    );

    Ok(Value::Binary {
        kind,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
        result_type: result_type.unflatten_list(list_state),
    })
}

pub fn interpret_broadcastable_ternary_call(
    context: &mut GlobalContext,
    kind: TernaryKind,
    first_type: TypeHandle,
    second_type: TypeHandle,
    third_type: TypeHandle,
    result_type: TypeHandle,
    arguments: Box<[ValueHandle]>,
) -> crate::Result<Value> {
    let mut arguments = arguments.into_iter();

    let first = arguments.next().unwrap()
        .coerce(context, &first_type, true)?;
    let second = arguments.next().unwrap()
        .coerce(context, &second_type, true)?;
    let third = arguments.next().unwrap()
        .coerce(context, &third_type, true)?;

    let list_state = ListState::merge_all([
        first.get_type().list_state(),
        second.get_type().list_state(),
        third.get_type().list_state(),
    ]);

    Ok(Value::Ternary {
        kind,
        first: Box::new(first),
        second: Box::new(second),
        third: Box::new(third),
        result_type: result_type.unflatten_list(list_state),
    })
}

pub fn interpret_broadcastable_reducer_call(
    context: &mut GlobalContext,
    kind: ReducerKind,
    element_type: Option<Type>,
    result_type: Option<Type>,
    arguments: Box<[ValueHandle]>,
) -> crate::Result<Value> {
    if let [argument] = arguments.as_ref() {
        let (list_state, argument_type) = argument.get_type().into_flatten_list();
        if list_state.is_some() {
            // This should also work for any MaybeList.
            let list = arguments.into_iter().next().unwrap();
            let list = match &element_type {
                Some(element_type) => list.coerce(context, element_type, true)?,
                None => list,
            };

            return Ok(Value::Reducer {
                kind,
                list: Box::new(list),
                result_type: result_type.unwrap_or(argument_type),
            })
        }
    }

    // Determine the most restrictive type that fits all arguments.
    let merged_type = arguments[1..].iter().try_fold(
        arguments[0].get_type(),
        |current_type, argument| {
            current_type.merge(&argument.get_type())
                .map_err(|error| error.with_span(argument.span))
        },
    )?;

    let element_type = match element_type {
        Some(element_type) => merged_type.flatten_list().1.clone().coerce(context, &element_type)
            .ok_or_else(|| Box::new(crate::Error {
                kind: crate::ErrorKind::MismatchedTypes {
                    expected_type: element_type.to_string(),
                    got_type: merged_type.to_string(),
                },
                span: match (arguments[0].span, arguments.last().unwrap().span) {
                    (Some(start), Some(end)) => Some(start.expand_to(end)),
                    (span, _) => span
                },
            }))?,
        None => merged_type,
    };

    let mut list_state = None;
    Ok(Value::ArgumentsReducer {
        kind,
        arguments: arguments
            .into_iter()
            .map(|argument| {
                list_state = ListState::merge(list_state, argument.get_type().list_state());
                argument.coerce(context, &element_type, true)
            })
            .collect::<crate::Result<_>>()?,
        result_type: result_type.unwrap_or(element_type).unflatten_list(list_state),
    })
}

macro_rules! strict_intrinsic {
    // Unary
    ($identifier:expr, ($argument_type:expr) => $kind:ident) => {
        IntrinsicFunction {
            identifier: $identifier,
            min_arity: 1,
            max_arity: Some(1),
            interpret_call: |_, _, _, arguments| interpret_strict_unary_call(
                UnaryKind::$kind,
                $argument_type,
                None,
                arguments,
            ),
        }
    };
    ($identifier:expr, ($argument_type:expr) => $kind:ident, $result_type:expr) => {
        IntrinsicFunction {
            identifier: $identifier,
            min_arity: 1,
            max_arity: Some(1),
            interpret_call: |_, _, _, arguments| interpret_strict_unary_call(
                UnaryKind::$kind,
                $argument_type,
                Some($result_type),
                arguments,
            ),
        }
    };
}

macro_rules! broadcastable_intrinsic {
    // Unary
    ($identifier:expr, ($argument_type:expr) => $kind:ident) => {
        IntrinsicFunction {
            identifier: $identifier,
            min_arity: 1,
            max_arity: Some(1),
            interpret_call: |_, _, _, arguments| interpret_broadcastable_unary_call(
                UnaryKind::$kind,
                $argument_type,
                None,
                arguments,
            ),
        }
    };
    ($identifier:expr, ($argument_type:expr) => $kind:ident, $result_type:expr) => {
        IntrinsicFunction {
            identifier: $identifier,
            min_arity: 1,
            max_arity: Some(1),
            interpret_call: |_, _, _, arguments| interpret_broadcastable_unary_call(
                UnaryKind::$kind,
                $argument_type,
                Some($result_type),
                arguments,
            ),
        }
    };
    // Binary
    ($identifier:expr, ($lhs_type:expr, $rhs_type:expr) => $kind:ident, $result_type:expr) => {
        IntrinsicFunction {
            identifier: $identifier,
            min_arity: 2,
            max_arity: Some(2),
            interpret_call: |_, _, _, arguments| interpret_broadcastable_binary_call(
                BinaryKind::$kind,
                $lhs_type,
                $rhs_type,
                $result_type,
                arguments,
            ),
        }
    };
    // Ternary
    ($identifier:expr, ($first_type:expr, $second_type:expr, $third_type: expr) => $kind:ident, $result_type:expr) => {
        IntrinsicFunction {
            identifier: $identifier,
            min_arity: 3,
            max_arity: Some(3),
            interpret_call: |_, _, _, arguments| interpret_broadcastable_ternary_call(
                TernaryKind::$kind,
                $first_type,
                $second_type,
                $third_type,
                $result_type,
                arguments,
            ),
        }
    };
    // Reducer
    ($identifier:expr, [?] => $kind:ident) => {
        IntrinsicFunction {
            identifier: $identifier,
            min_arity: 1,
            max_arity: None,
            interpret_call: |_, _, _, arguments| interpret_broadcastable_reducer_call(
                ReducerKind::$kind,
                None,
                None,
                arguments,
            ),
        }
    };
    ($identifier:expr, [?] => $kind:ident, $result_type:expr) => {
        IntrinsicFunction {
            identifier: $identifier,
            min_arity: 1,
            max_arity: None,
            interpret_call: |_, _, _, arguments| interpret_broadcastable_reducer_call(
                ReducerKind::$kind,
                None,
                Some($result_type),
                arguments,
            ),
        }
    };
    ($identifier:expr, [$element_type:expr] => $kind:ident) => {
        IntrinsicFunction {
            identifier: $identifier,
            min_arity: 1,
            max_arity: None,
            interpret_call: |_, _, _, arguments| interpret_broadcastable_reducer_call(
                ReducerKind::$kind,
                Some($element_type),
                None,
                arguments,
            ),
        }
    };
    ($identifier:expr, [$element_type:expr] => $kind:ident, $result_type:expr) => {
        IntrinsicFunction {
            identifier: $identifier,
            min_arity: 1,
            max_arity: None,
            interpret_call: |_, _, _, arguments| interpret_broadcastable_reducer_call(
                ReducerKind::$kind,
                Some($element_type),
                Some($result_type),
                arguments,
            ),
        }
    };
}

pub fn read_file_bytes(local_context: &LocalContext, path_value: &ValueHandle) -> crate::Result<(PathBuf, Vec<u8>)> {
    let relative_path = path_value.kind
        .as_const_str()
        .ok_or_else(|| Box::new(crate::Error {
            kind: crate::ErrorKind::ExpectedConstant {
                type_identifier: Type::Str.to_string(),
            },
            span: path_value.span,
        }))?;

    let full_path = local_context.source_directory().join(relative_path.as_ref());

    std::fs::read(&full_path)
        .map_err(|cause| Box::new(crate::Error {
            kind: crate::ErrorKind::FileOpen {
                path: Some(full_path.as_path().into()),
                cause,
            },
            span: path_value.span,
        }))
        .map(|contents| (full_path, contents))
}

// ------ Trigonometric ------

pub static SIN: IntrinsicFunction = broadcastable_intrinsic!(
    "sin", (Type::Real) => Sin
);

pub static COS: IntrinsicFunction = broadcastable_intrinsic!(
    "cos", (Type::Real) => Cos
);

pub static TAN: IntrinsicFunction = broadcastable_intrinsic!(
    "tan", (Type::Real) => Tan
);

pub static CSC: IntrinsicFunction = broadcastable_intrinsic!(
    "csc", (Type::Real) => Csc
);

pub static SEC: IntrinsicFunction = broadcastable_intrinsic!(
    "sec", (Type::Real) => Sec
);

pub static COT: IntrinsicFunction = broadcastable_intrinsic!(
    "cot", (Type::Real) => Cot
);

pub static ARCSIN: IntrinsicFunction = broadcastable_intrinsic!(
    "arcsin", (Type::Real) => Arcsin
);

pub static ARCCOS: IntrinsicFunction = broadcastable_intrinsic!(
    "arccos", (Type::Real) => Arccos
);

pub static ARCTAN: IntrinsicFunction = broadcastable_intrinsic!(
    "arctan", (Type::Real) => Arctan
);

pub static ARCTAN2: IntrinsicFunction = broadcastable_intrinsic!(
    "arctan2", (Type::Real, Type::Real) => Arctan2, Type::Real
);

pub static ARCCSC: IntrinsicFunction = broadcastable_intrinsic!(
    "arccsc", (Type::Real) => Arccsc
);

pub static ARCSEC: IntrinsicFunction = broadcastable_intrinsic!(
    "arcsec", (Type::Real) => Arcsec
);

pub static ARCCOT: IntrinsicFunction = broadcastable_intrinsic!(
    "arccot", (Type::Real) => Arccot
);

pub static SINH: IntrinsicFunction = broadcastable_intrinsic!(
    "sinh", (Type::Real) => Sinh
);

pub static COSH: IntrinsicFunction = broadcastable_intrinsic!(
    "cosh", (Type::Real) => Cosh
);

pub static TANH: IntrinsicFunction = broadcastable_intrinsic!(
    "tanh", (Type::Real) => Tanh
);

pub static CSCH: IntrinsicFunction = broadcastable_intrinsic!(
    "csch", (Type::Real) => Csch
);

pub static SECH: IntrinsicFunction = broadcastable_intrinsic!(
    "sech", (Type::Real) => Sech
);

pub static COTH: IntrinsicFunction = broadcastable_intrinsic!(
    "coth", (Type::Real) => Coth
);

// ------ Calculus ------

pub static EXP: IntrinsicFunction = broadcastable_intrinsic!(
    "exp", (Type::Real) => Exp
);

pub static LN: IntrinsicFunction = broadcastable_intrinsic!(
    "ln", (Type::Real) => Ln
);

pub static LOG: IntrinsicFunction = broadcastable_intrinsic!(
    "log", (Type::Real, Type::Real) => Log, Type::Real
);

// DERIVATIVE

// INTEGRAL

// SUM

// PRODUCT

// ------ Number Theory ------

pub static LCM: IntrinsicFunction = broadcastable_intrinsic!(
    "lcm", [Type::Int] => Lcm
);

pub static GCD: IntrinsicFunction = broadcastable_intrinsic!(
    "gcd", [Type::Int] => Gcd
);

pub static CEIL: IntrinsicFunction = broadcastable_intrinsic!(
    "ceil", (Type::Real) => Ceil, Type::Int
);

pub static FLOOR: IntrinsicFunction = broadcastable_intrinsic!(
    "floor", (Type::Real) => Floor, Type::Int
);

pub static ROUND: IntrinsicFunction = broadcastable_intrinsic!(
    "round", (Type::Real) => Round, Type::Int
);

pub static ROUND_DIGITS: IntrinsicFunction = broadcastable_intrinsic!(
    "round_digits", (Type::Real, Type::Int) => RoundDigits, Type::Real
);

pub static ABS: IntrinsicFunction = broadcastable_intrinsic!(
    "abs", (Type::union([Type::Int, Type::Real])) => Abs
);

pub static SIGN: IntrinsicFunction = broadcastable_intrinsic!(
    "sign", (Type::Real) => Sign, Type::Int
);

pub static SQRT: IntrinsicFunction = broadcastable_intrinsic!(
    "sqrt", (Type::Real) => Sqrt, Type::Real
);

pub static CBRT: IntrinsicFunction = broadcastable_intrinsic!(
    "cbrt", (Type::Real) => Cbrt, Type::Real
);

pub static NTH_ROOT: IntrinsicFunction = broadcastable_intrinsic!(
    "nth_root", (Type::Real, Type::Real) => NthRoot, Type::Real
);

// NPR

// NCR

// FACTORIAL

// ------ Complex ------

// REAL

// IMAG

// CONJ

// ARG

// ------ List Operations ------

pub static JOIN: IntrinsicFunction = IntrinsicFunction {
    identifier: "join",
    min_arity: 2,
    max_arity: None,
    interpret_call: |_, _, _, arguments| {
        let item_type = arguments[1..].iter().try_fold(
            arguments[0].get_type().into_flatten_list().1,
            |current_type, argument| {
                current_type.merge(argument.get_type().flatten_list().1)
                    .map_err(|error| error.with_span(argument.span))
            },
        )?;

        Ok(Value::Join {
            values: arguments,
            result_type: item_type.into_list(ListState::IsList),
        })
    },
};

pub static SORT: IntrinsicFunction = IntrinsicFunction {
    identifier: "sort",
    min_arity: 1,
    max_arity: Some(2),
    interpret_call: |_, _, _, arguments| {
        let mut arguments = arguments.into_iter();

        let list = arguments.next().unwrap();
        list.get_type()
            .require_flatten_list()
            .map_err(|error| error.with_span(list.span))?;

        let key_type = Type::union([
            Type::Bool,
            Type::Int,
            Type::Real,
            Type::Complex,
        ]);

        if let Some(key_list) = arguments.next() {
            key_list.get_type()
                .require_flatten_list()
                .map_err(|error| error.with_span(list.span))?;

            let key_list = key_list.coerce(context, &key_type, true)?;

            Ok(Value::Binary {
                kind: BinaryKind::SortKeyed,
                result_type: list.get_type(),
                lhs: Box::new(list),
                rhs: Box::new(key_list),
            })
        }
        else {
            let list = list.coerce(context, &key_type, true)?;

            Ok(Value::Unary {
                kind: UnaryKind::Sort,
                result_type: list.get_type(),
                operand: Box::new(list),
            })
        }
    },
};

pub static SHUFFLE: IntrinsicFunction = IntrinsicFunction {
    identifier: "shuffle",
    min_arity: 1,
    max_arity: Some(2),
    interpret_call: |_, _, _, arguments| {
        let mut arguments = arguments.into_iter();

        let list = arguments.next().unwrap();
        list.get_type()
            .require_flatten_list()
            .map_err(|error| error.with_span(list.span))?;

        if let Some(seed) = arguments.next() {
            let seed = seed.coerce(context, &Type::Real, false)?;

            Ok(Value::Binary {
                kind: BinaryKind::ShuffleSeeded,
                result_type: list.get_type(),
                lhs: Box::new(list),
                rhs: Box::new(seed),
            })
        }
        else {
            Ok(Value::Unary {
                kind: UnaryKind::Shuffle,
                result_type: list.get_type(),
                operand: Box::new(list),
            })
        }
    },
};

pub static UNIQUE: IntrinsicFunction = IntrinsicFunction {
    identifier: "unique",
    min_arity: 1,
    max_arity: Some(1),
    interpret_call: |_, _, _, arguments| {
        // It seems like unique() accepts basically any type, so don't bother checking the item type
        // FIXME: per desmos code, only distributions cannot be uniqued
        let list = arguments.into_iter().next().unwrap();
        list.get_type()
            .require_flatten_list()
            .map_err(|error| error.with_span(list.span))?;

        Ok(Value::Unary {
            kind: UnaryKind::Unique,
            result_type: list.get_type(),
            operand: Box::new(list),
        })
    },
};

pub static PREFIX_SUM: IntrinsicFunction = IntrinsicFunction {
    identifier: "prefix_sum",
    min_arity: 1,
    max_arity: Some(1),
    interpret_call: |_, _, _, arguments| {
        let list = arguments.into_iter().next().unwrap();
        list.get_type()
            .require_flatten_list()
            .and_then(|item_type| item_type.require_numeric_or_point().map(|()| item_type))
            .map_err(|error| error.with_span(list.span))?;

        Ok(Value::Unary {
            kind: UnaryKind::PrefixSum,
            result_type: list.get_type(),
            operand: Box::new(list),
        })
    },
};

// ------ Statistics ------

pub static MEAN: IntrinsicFunction = broadcastable_intrinsic!(
    "mean", [Type::union([Type::Real, Type::real_point_2d(), Type::real_point_3d()])] => Mean
);

pub static MEDIAN: IntrinsicFunction = broadcastable_intrinsic!(
    "median", [Type::Real] => Median
);

pub static MIN: IntrinsicFunction = broadcastable_intrinsic!(
    "min", [Type::union([Type::Bool, Type::Int, Type::Real])] => Min
);

pub static MAX: IntrinsicFunction = broadcastable_intrinsic!(
    "max", [Type::union([Type::Bool, Type::Int, Type::Real])] => Max
);

// QUARTILE

// QUANTILE

// STDEV

// STDEVP

// VAR

// VARP

// COV

// COVP

// MAD

// CORR

// SPEARMAN

// STATS

pub static COUNT: IntrinsicFunction = broadcastable_intrinsic!(
    "count", [?] => Count, Type::Int
);

pub static TOTAL: IntrinsicFunction = broadcastable_intrinsic!(
    "total", [Type::union([
        Type::Int,
        Type::Real,
        Type::point_2d(
            Type::union([Type::Int, Type::Real]),
            Type::union([Type::Int, Type::Real]),
        ),
        Type::point_3d(
            Type::union([Type::Int, Type::Real]),
            Type::union([Type::Int, Type::Real]),
            Type::union([Type::Int, Type::Real]),
        ),
    ])] => Total
);

// @total(arguments) > 0
pub static ANY: IntrinsicFunction = IntrinsicFunction {
    identifier: "any",
    min_arity: 1,
    max_arity: None,
    interpret_call: |target, context, local_context, arguments| {
        let arguments: Box<[_]> = arguments
            .into_iter()
            .map(|argument| argument.coerce(context, &Type::Bool, true))
            .collect::<crate::Result<_>>()?;
        let total = (TOTAL.interpret_call)(target, context, local_context, arguments)?;
        let list_state = total.get_type().list_state();

        Ok(Value::InequalityChain {
            lhs: Box::new(total.into()),
            chain: Box::new([(
                InequalityKind::GreaterThan,
                Value::Int(0).into(),
            )]),
            result_type: Type::Bool.unflatten_list(list_state),
        })
    },
};

// @total(!arguments) == 0
pub static ALL: IntrinsicFunction = IntrinsicFunction {
    identifier: "all",
    min_arity: 1,
    max_arity: None,
    interpret_call: |target, context, local_context, arguments| {
        let arguments: Box<[_]> = arguments
            .into_iter()
            .map(|argument| {
                let list_state = argument.get_type().list_state();

                Ok(Value::Unary {
                    kind: UnaryKind::LogicalNot,
                    operand: Box::new(argument.coerce(context, &Type::Bool, true)?),
                    result_type: Type::Bool.unflatten_list(list_state),
                }.into())
            })
            .collect::<crate::Result<_>>()?;
        let total = (TOTAL.interpret_call)(target, context, local_context, arguments)?;

        let list_state = total.get_type().list_state();

        Ok(Value::Binary {
            kind: BinaryKind::Equal,
            lhs: Box::new(total.into()),
            rhs: Box::new(Value::Int(0).into()),
            result_type: Type::Bool.unflatten_list(list_state),
        })
    },
};

// ------ Visualizations ------

// HISTOGRAM

// DOT_PLOT

// BOX_PLOT

// ------ Distributions ------

// NORMAL_DIST

// T_DIST

// CHISQ_DIST

// UNIFORM_DIST

// BINOMIAL_DIST

// POISSON_DIST

// GEO_DIST

// PDF

// CDF

// INVERSE_CDF

fn interpret_random_call_end(
    arguments: impl IntoIterator<Item = ValueHandle>,
    source: Option<Box<ValueHandle>>,
    source_type: Type,
) -> crate::Result<Value> {
    let mut arguments = arguments.into_iter();

    if let Some(sample_count) = arguments.next() {
        let sample_count = sample_count.coerce(context, &Type::Int, false)?;

        if let Some(seed) = arguments.next() {
            let seed = seed.coerce(context, &Type::Real, false)?;

            Ok(Value::RandomSeeded {
                source,
                sample_count: Box::new(sample_count),
                seed: Box::new(seed),
                result_type: source_type.into_list(ListState::IsList),
            })
        }
        else {
            Ok(Value::Random {
                source,
                sample_count: Some(Box::new(sample_count)),
                result_type: source_type.into_list(ListState::IsList),
            })
        }
    }
    else {
        Ok(Value::Random {
            source,
            sample_count: None,
            result_type: source_type,
        })
    }
}

pub static RANDOM: IntrinsicFunction = IntrinsicFunction {
    identifier: "random",
    min_arity: 0,
    max_arity: Some(2),
    interpret_call: |_, _, _, arguments| {
        interpret_random_call_end(arguments, None, Type::Real)
    },
};

pub static CHOOSE_RANDOM: IntrinsicFunction = IntrinsicFunction {
    identifier: "choose_random",
    min_arity: 1,
    max_arity: Some(3),
    interpret_call: |_, _, _, arguments| {
        let mut arguments = arguments.into_iter();

        let source = arguments.next().unwrap();
        let source_type = match source.get_type().into_flatten_list() {
            (Some(ListState::IsList), item_type) if item_type != Type::Distribution => item_type,
            (None, Type::Distribution) => Type::Real,
            _ => return Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ExpectedListOrDistributionType {
                    got_type: source.get_type().to_string(),
                },
                span: source.span,
            }))
        };

        interpret_random_call_end(arguments, Some(Box::new(source)), source_type)
    },
};

// ------ Statistical Tests ------

// TTEST

// TSCORE

// ITTEST

// ------ Geometry ------

// INTERSECTION

// STRICT_INTERSECTION

pub static SEGMENT: IntrinsicFunction = broadcastable_intrinsic!(
    "segment", (Type::real_point_2d(), Type::real_point_2d()) => SegmentFromPoints2D, Type::Segment
);

pub static SEGMENT3D: IntrinsicFunction = broadcastable_intrinsic!(
    "segment3d", (Type::real_point_3d(), Type::real_point_3d()) => SegmentFromPoints3D, Type::Segment3D
);

pub static LINE: IntrinsicFunction = IntrinsicFunction {
    identifier: "line",
    min_arity: 1,
    max_arity: Some(2),
    interpret_call: |_, _, _, arguments| {
        let mut arguments = arguments.into_iter();

        let first_argument = arguments.next().unwrap();

        if let Some(end) = arguments.next() {
            let real_point_2d = Type::real_point_2d();
            let start = first_argument.coerce(context, &real_point_2d, true)?;
            let end = end.coerce(context, &real_point_2d, true)?;

            let list_state = ListState::merge(
                start.get_type().list_state(),
                end.get_type().list_state(),
            );

            Ok(Value::Binary {
                kind: BinaryKind::LineFromPoints2D,
                lhs: Box::new(start),
                rhs: Box::new(end),
                result_type: Type::Line.unflatten_list(list_state),
            })
        }
        else {
            let segment_or_ray = first_argument.coerce(context, &Type::union([
                Type::Segment,
                Type::Ray,
            ]), true)?;

            let (list_state, argument_type) = segment_or_ray.get_type().into_flatten_list();

            Ok(Value::Unary {
                kind: match argument_type {
                    Type::Segment => UnaryKind::LineFromSegment2D,
                    Type::Ray => UnaryKind::LineFromRay2D,
                    _ => unreachable!()
                },
                operand: Box::new(segment_or_ray),
                result_type: Type::Line.unflatten_list(list_state),
            })
        }
    },
};

pub static RAY: IntrinsicFunction = broadcastable_intrinsic!(
    "ray", (Type::real_point_2d(), Type::real_point_2d()) => RayFromPoints2D, Type::Ray
);

pub static VECTOR: IntrinsicFunction = broadcastable_intrinsic!(
    "vector", (Type::real_point_2d(), Type::real_point_2d()) => VectorFromPoints2D, Type::Vector
);

pub static VECTOR3D: IntrinsicFunction = broadcastable_intrinsic!(
    "vector3d", (Type::real_point_3d(), Type::real_point_3d()) => VectorFromPoints3D, Type::Vector3D
);

// PARALLEL

// PERPENDICULAR

pub static CIRCLE: IntrinsicFunction = IntrinsicFunction {
    identifier: "circle",
    min_arity: 2,
    max_arity: Some(2),
    interpret_call: |_, _, _, arguments| {
        let mut arguments = arguments.into_iter();

        let lhs = arguments.next().unwrap()
            .coerce(context, &Type::real_point_2d(), true)?;
        let rhs = arguments.next().unwrap()
            .coerce(context, &Type::union([
                Type::Real,
                Type::real_point_2d(),
            ]), true)?;

        let list_state = ListState::merge(
            lhs.get_type().list_state(),
            rhs.get_type().list_state(),
        );

        Ok(Value::Binary {
            kind: match rhs.get_type() {
                Type::Point2D { .. } => BinaryKind::CircleFromEdge2D,
                _ => BinaryKind::CircleFromRadius2D,
            },
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            result_type: Type::Circle.unflatten_list(list_state),
        })
    },
};

pub static SPHERE3D: IntrinsicFunction = broadcastable_intrinsic!(
    "sphere3d", (Type::real_point_3d(), Type::Real) => SphereFromRadius3D, Type::Sphere3D
);

pub static ARC: IntrinsicFunction = broadcastable_intrinsic!(
    "arc", (Type::real_point_2d(), Type::real_point_2d(), Type::real_point_2d()) => Arc2D, Type::Arc
);

pub static ANGLE: IntrinsicFunction = broadcastable_intrinsic!(
    "angle", (Type::real_point_2d(), Type::real_point_2d(), Type::real_point_2d()) => UndirectedAngle2D, Type::Angle
);

pub static DIRECTED_ANGLE: IntrinsicFunction = broadcastable_intrinsic!(
    "directed_angle", (Type::real_point_2d(), Type::real_point_2d(), Type::real_point_2d()) => DirectedAngle2D, Type::DirectedAngle
);

pub static POLYGON: IntrinsicFunction = broadcastable_intrinsic!(
    "polygon", [Type::real_point_2d()] => Polygon2D, Type::Polygon
);

pub static RECT: IntrinsicFunction = broadcastable_intrinsic!(
    "rect", (Type::real_point_2d(), Type::real_point_2d()) => RectangleFromPoints2D, Type::Polygon
);

pub static TRIANGLE3D: IntrinsicFunction = broadcastable_intrinsic!(
    "triangle", (Type::real_point_3d(), Type::real_point_3d(), Type::real_point_3d()) => Triangle3D, Type::Triangle3D
);

pub static GLIDER: IntrinsicFunction = broadcastable_intrinsic!(
    "glider", (Type::union([
        Type::Segment,
        Type::Circle,
        Type::Line,
        Type::Ray,
        Type::Arc,
        Type::Polygon,
    ]), Type::Real) => Glider2D, Type::real_point_2d()
);

// ------ Properties & Measurements ------

// DOT

// CROSS

// DISTANCE

// LENGTH

pub static AREA: IntrinsicFunction = broadcastable_intrinsic!(
    "area", (Type::Polygon) => AreaOfPolygon, Type::Real
);

pub static PERIMETER: IntrinsicFunction = broadcastable_intrinsic!(
    "perimeter", (Type::Polygon) => PerimeterOfPolygon, Type::Real
);

pub static VERTICES: IntrinsicFunction = strict_intrinsic!(
    "vertices", (Type::Polygon) => VerticesOfPolygon, Type::real_point_2d().into_list(ListState::IsList)
);

pub static ANGLES: IntrinsicFunction = strict_intrinsic!(
    "angles", (Type::Polygon) => UndirectedAnglesOfPolygon, Type::Angle.into_list(ListState::IsList)
);

pub static DIRECTED_ANGLES: IntrinsicFunction = strict_intrinsic!(
    "directed_angles", (Type::Polygon) => DirectedAnglesOfPolygon, Type::DirectedAngle.into_list(ListState::IsList)
);

pub static SEGMENTS: IntrinsicFunction = strict_intrinsic!(
    "segments", (Type::Polygon) => SegmentsOfPolygon, Type::Segment.into_list(ListState::IsList)
);

pub static RADIUS: IntrinsicFunction = broadcastable_intrinsic!(
    "radius", (Type::Circle) => RadiusOfCircle, Type::Real
);

pub static CENTER: IntrinsicFunction = broadcastable_intrinsic!(
    "center", (Type::Circle) => CenterOfCircle, Type::Real
);

// COTERMINAL

// SUPPLEMENT

pub static MIDPOINT: IntrinsicFunction = IntrinsicFunction {
    identifier: "midpoint",
    min_arity: 1,
    max_arity: Some(2),
    interpret_call: |_, _, _, arguments| {
        let mut arguments = arguments.into_iter();

        let point_1_or_segment = arguments.next().unwrap();

        if let Some(point_2) = arguments.next() {
            let point_1 = point_1_or_segment.coerce(context, &Type::union([
                Type::real_point_2d(),
                Type::real_point_3d(),
            ]), true)?;
            let (point_1_list, point_type) = point_1.get_type().into_flatten_list();

            let kind = match &point_type {
                Type::Point2D { .. } => BinaryKind::MidpointOfPoints2D,
                Type::Point3D { .. } => BinaryKind::MidpointOfPoints3D,
                _ => unreachable!()
            };

            let point_2 = point_2.coerce(context, &point_type, true)?;

            let list_state = ListState::merge(
                point_1_list,
                point_2.get_type().list_state(),
            );

            Ok(Value::Binary {
                kind,
                lhs: Box::new(point_1),
                rhs: Box::new(point_2),
                result_type: point_type.unflatten_list(list_state),
            })
        }
        else {
            let segment = point_1_or_segment.coerce(context, &Type::union([
                Type::Segment,
                Type::Segment3D,
            ]), true)?;
            let (list_state, segment_type) = segment.get_type().into_flatten_list();

            let (kind, point_type) = match segment_type {
                Type::Segment => (UnaryKind::MidpointOfSegment2D, Type::real_point_2d()),
                Type::Segment3D => (UnaryKind::MidpointOfSegment3D, Type::real_point_3d()),
                _ => unreachable!()
            };

            Ok(Value::Unary {
                kind,
                operand: Box::new(segment),
                result_type: point_type.unflatten_list(list_state),
            })
        }
    },
};

fn interpret_start_end_call(
    kind_2d: UnaryKind,
    kind_3d: UnaryKind,
    arguments: Box<[ValueHandle]>,
) -> crate::Result<Value> {
    let vector = arguments.into_iter().next().unwrap()
        .coerce(context, &Type::union([Type::Vector, Type::Vector3D]), true)?;
    let (list_state, vector_type) = vector.get_type().into_flatten_list();

    let (kind, point_type) = match vector_type {
        Type::Vector => (kind_2d, Type::real_point_2d()),
        Type::Vector3D => (kind_3d, Type::real_point_3d()),
        _ => unreachable!()
    };

    Ok(Value::Unary {
        kind,
        operand: Box::new(vector),
        result_type: point_type.unflatten_list(list_state),
    })
}

pub static START: IntrinsicFunction = IntrinsicFunction {
    identifier: "start",
    min_arity: 1,
    max_arity: Some(1),
    interpret_call: |_, _, _, arguments| interpret_start_end_call(
        UnaryKind::StartOfVector2D,
        UnaryKind::StartOfVector3D,
        arguments,
    ),
};

pub static END: IntrinsicFunction = IntrinsicFunction {
    identifier: "end",
    min_arity: 1,
    max_arity: Some(1),
    interpret_call: |_, _, _, arguments| interpret_start_end_call(
        UnaryKind::EndOfVector2D,
        UnaryKind::EndOfVector3D,
        arguments,
    ),
};

// ------ Transformations ------

fn interpret_rotate_dilate_call(
    kind: TernaryKind,
    arguments: Box<[ValueHandle]>,
) -> crate::Result<Value> {
    let mut arguments = arguments.into_iter();

    let object = arguments.next().unwrap()
        .coerce(context, &Type::transformable(), true)?;
    let (object_list, object_type) = object.get_type().into_flatten_list();

    let point = arguments.next().unwrap()
        .coerce(context, &Type::real_point_2d(), true)?;

    let factor_or_angle = arguments.next().unwrap()
        .coerce(context, &Type::Real, true)?;

    let list_state = ListState::merge_all([
        object_list,
        point.get_type().list_state(),
        factor_or_angle.get_type().list_state(),
    ]);

    Ok(Value::Ternary {
        kind,
        first: Box::new(object),
        second: Box::new(point),
        third: Box::new(factor_or_angle),
        result_type: object_type.unflatten_list(list_state),
    })
}

pub static DILATE: IntrinsicFunction = IntrinsicFunction {
    identifier: "dilate",
    min_arity: 3,
    max_arity: Some(3),
    interpret_call: |_, _, _, arguments| interpret_rotate_dilate_call(
        TernaryKind::Dilate2D,
        arguments,
    ),
};

pub static ROTATE: IntrinsicFunction = IntrinsicFunction {
    identifier: "rotate",
    min_arity: 3,
    max_arity: Some(3),
    interpret_call: |_, _, _, arguments| interpret_rotate_dilate_call(
        TernaryKind::Rotate2D,
        arguments,
    ),
};

pub static REFLECT: IntrinsicFunction = IntrinsicFunction {
    identifier: "reflect",
    min_arity: 2,
    max_arity: Some(2),
    interpret_call: |_, _, _, arguments| {
        let mut arguments = arguments.into_iter();

        let object = arguments.next().unwrap()
            .coerce(context, &Type::transformable(), true)?;
        let (object_list, object_type) = object.get_type().into_flatten_list();

        let line = arguments.next().unwrap()
            .coerce(context, &Type::line_like(), true)?;

        let list_state = ListState::merge(
            object_list,
            line.get_type().list_state(),
        );

        Ok(Value::Binary {
            kind: BinaryKind::Reflect2D,
            lhs: Box::new(object),
            rhs: Box::new(line),
            result_type: object_type.unflatten_list(list_state),
        })
    },
};

pub static TRANSLATE: IntrinsicFunction = IntrinsicFunction {
    identifier: "translate",
    min_arity: 2,
    max_arity: Some(3),
    interpret_call: |_, _, _, arguments| {
        let mut arguments = arguments.into_iter();

        let object = arguments.next().unwrap()
            .coerce(context, &Type::transformable(), true)?;
        let (object_list, object_type) = object.get_type().into_flatten_list();

        let first_argument = arguments.next().unwrap();

        if let Some(end) = arguments.next() {
            let real_point2 = Type::real_point_2d();
            let start = first_argument.coerce(context, &real_point2, true)?;
            let end = end.coerce(context, &real_point2, true)?;

            let list_state = ListState::merge_all([
                object_list,
                start.get_type().list_state(),
                end.get_type().list_state(),
            ]);

            Ok(Value::Ternary {
                kind: TernaryKind::TranslateByPoints2D,
                first: Box::new(object),
                second: Box::new(start),
                third: Box::new(end),
                result_type: object_type.unflatten_list(list_state),
            })
        }
        else {
            let vector = first_argument.coerce(context, &Type::Vector, true)?;

            let list_state = ListState::merge(
                object_list,
                vector.get_type().list_state(),
            );

            Ok(Value::Binary {
                kind: BinaryKind::TranslateByVector2D,
                lhs: Box::new(object),
                rhs: Box::new(vector),
                result_type: object_type.unflatten_list(list_state),
            })
        }
    },
};

fn interpret_rotation_dilation_call(
    kind: BinaryKind,
    arguments: Box<[ValueHandle]>,
) -> crate::Result<Value> {
    let mut arguments = arguments.into_iter();

    let point = arguments.next().unwrap()
        .coerce(context, &Type::real_point_2d(), true)?;

    let factor_or_angle = arguments.next().unwrap()
        .coerce(context, &Type::Real, true)?;

    let list_state = ListState::merge(
        point.get_type().list_state(),
        factor_or_angle.get_type().list_state(),
    );

    Ok(Value::Binary {
        kind,
        lhs: Box::new(point),
        rhs: Box::new(factor_or_angle),
        result_type: Type::Transformation.unflatten_list(list_state),
    })
}

pub static DILATION: IntrinsicFunction = IntrinsicFunction {
    identifier: "dilation",
    min_arity: 2,
    max_arity: Some(2),
    interpret_call: |_, _, _, arguments| interpret_rotation_dilation_call(
        BinaryKind::Dilation2D,
        arguments,
    ),
};

pub static ROTATION: IntrinsicFunction = IntrinsicFunction {
    identifier: "rotation",
    min_arity: 2,
    max_arity: Some(2),
    interpret_call: |_, _, _, arguments| interpret_rotation_dilation_call(
        BinaryKind::Rotation2D,
        arguments,
    ),
};

pub static REFLECTION: IntrinsicFunction = IntrinsicFunction {
    identifier: "reflection",
    min_arity: 1,
    max_arity: Some(1),
    interpret_call: |_, _, _, arguments| {
        let mut arguments = arguments.into_iter();

        let line = arguments.next().unwrap()
            .coerce(context, &Type::Line, true)?;

        Ok(Value::Unary {
            kind: UnaryKind::ReflectionByLine2D,
            result_type: Type::Transformation.unflatten_list(line.get_type().list_state()),
            operand: Box::new(line),
        })
    },
};

// TODO: allow 2-point and vector versions, and add this version to @translate
pub static TRANSLATION: IntrinsicFunction = IntrinsicFunction {
    identifier: "translation",
    min_arity: 1,
    max_arity: Some(1),
    interpret_call: |_, _, _, arguments| {
        let mut arguments = arguments.into_iter();

        let displacement = arguments.next().unwrap()
            .coerce(context, &Type::real_point_2d(), true)?;

        Ok(Value::Unary {
            kind: UnaryKind::TranslationByPoint2D,
            result_type: Type::Transformation.unflatten_list(displacement.get_type().list_state()),
            operand: Box::new(displacement),
        })
    },
};

pub static APPLY: IntrinsicFunction = IntrinsicFunction {
    identifier: "apply",
    min_arity: 2,
    max_arity: Some(2),
    interpret_call: |_, _, _, arguments| {
        let mut arguments = arguments.into_iter();

        let transformation = arguments.next().unwrap()
            .coerce(context, &Type::Transformation, true)?;

        let object = arguments.next().unwrap()
            .coerce(context, &Type::transformable(), true)?;
        let (object_list, object_type) = object.get_type().into_flatten_list();

        let list_state = ListState::merge(
            transformation.get_type().list_state(),
            object_list,
        );

        Ok(Value::Binary {
            kind: BinaryKind::ApplyTransform2D,
            lhs: Box::new(transformation),
            rhs: Box::new(object),
            result_type: object_type.unflatten_list(list_state),
        })
    },
};

pub static COMPOSE: IntrinsicFunction = broadcastable_intrinsic!(
    "compose", [Type::Transformation] => ComposeTransforms2D
);

pub static INVERSE: IntrinsicFunction = broadcastable_intrinsic!(
    "inverse", (Type::Transformation) => InverseOfTransform2D
);

// ------ Color ------

pub static RGB: IntrinsicFunction = broadcastable_intrinsic!(
    "rgb", (Type::Real, Type::Real, Type::Real) => Rgb, Type::Color
);

pub static HSV: IntrinsicFunction = broadcastable_intrinsic!(
    "hsv", (Type::Real, Type::Real, Type::Real) => Hsv, Type::Color
);

pub static OKHSV: IntrinsicFunction = broadcastable_intrinsic!(
    "okhsv", (Type::Real, Type::Real, Type::Real) => Okhsv, Type::Color
);

pub static OKLAB: IntrinsicFunction = broadcastable_intrinsic!(
    "oklab", (Type::Real, Type::Real, Type::Real) => Oklab, Type::Color
);

pub static OKLCH: IntrinsicFunction = broadcastable_intrinsic!(
    "oklch", (Type::Real, Type::Real, Type::Real) => Oklch, Type::Color
);

// ------ Sound ------

// TONE

// ------ Desmosify ------

pub static BOOL_TO_INTERNAL: IntrinsicFunction = broadcastable_intrinsic!(
    "bool_to_internal", (Type::Bool) => BoolToInternal, Type::InternalBool
);

pub static BOOL_FROM_INTERNAL: IntrinsicFunction = broadcastable_intrinsic!(
    "bool_from_internal", (Type::InternalBool) => BoolFromInternal, Type::Bool
);

// FIXME: this is broken for any enum that is not the default shape. to fix, generate a global list
//        and reference it here
pub static ENUM_VALUES: IntrinsicFunction = IntrinsicFunction {
    identifier: "enum_values",
    min_arity: 1,
    max_arity: Some(1),
    interpret_call: |_, context, _, arguments| {
        let enum_type = arguments.into_iter().next().unwrap();
        let Type::Meta { identifier } = enum_type.get_type() else {
            return Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ExpectedEnumTypeValue,
                span: enum_type.span,
            }));
        };

        let definition = context.find_global(&identifier).unwrap();
        let DefinitionKind::Type(TypeDefinition::Enumeration { variants }) = &definition.definition.kind else {
            return Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ExpectedEnumTypeValue,
                span: enum_type.span,
            }));
        };

        Ok(Value::ListRange {
            kind: RangeKind::Exclusive,
            start: Box::new(Value::EnumVariant {
                type_identifier: identifier.clone(),
                ordinal: 0,
            }.into()),
            end: Box::new(Value::EnumVariant {
                type_identifier: identifier.clone(),
                ordinal: variants.len() as i64,
            }.into()),
            step: Box::new(Value::Int(1).into()),
            item_type: Type::Enum {
                type_identifier: identifier,
            },
        })
    },
};

pub static ENUM_VALUE: IntrinsicFunction = IntrinsicFunction {
    identifier: "enum_value",
    min_arity: 2,
    max_arity: Some(2),
    interpret_call: |_, context, _, arguments| {
        let mut arguments = arguments.into_iter();

        let enum_type = arguments.next().unwrap();
        let Type::Meta { identifier } = enum_type.get_type() else {
            return Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ExpectedEnumTypeValue,
                span: enum_type.span,
            }));
        };

        let definition = context.find_global(&identifier).unwrap();
        let DefinitionKind::Type(TypeDefinition::Enumeration { .. }) = &definition.definition.kind else {
            return Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ExpectedEnumTypeValue,
                span: enum_type.span,
            }));
        };

        let variant_ordinal = arguments.next().unwrap()
            .coerce(context, &Type::Int, true)?;
        let list_state = variant_ordinal.get_type().list_state();

        let result_type = Type::Enum {
            type_identifier: identifier,
        };

        Ok(variant_ordinal.assume_type(result_type.unflatten_list(list_state)).kind)
    },
};

pub static INCLUDE_TEXT: IntrinsicFunction = IntrinsicFunction {
    identifier: "include_text",
    min_arity: 1,
    max_arity: Some(1),
    interpret_call: |_, _, local_context, arguments| {
        // TODO: allow user to specify encoding; better error handling
        let path_value = arguments.into_iter().next().unwrap();
        let (path, bytes) = read_file_bytes(local_context, &path_value)?;

        let text = String::from_utf8(bytes)
            .map_err(|_| Box::new(crate::Error {
                kind: crate::ErrorKind::FileRead {
                    path: Some(path.into_boxed_path()),
                    cause: std::io::ErrorKind::InvalidData.into(),
                },
                span: path_value.span,
            }))?;

        Ok(Value::Str(text.into()))
    },
};

pub static INCLUDE_DATA: IntrinsicFunction = IntrinsicFunction {
    identifier: "include_data",
    min_arity: 1,
    max_arity: Some(2),
    interpret_call: |_, _, local_context, arguments| {
        let mut arguments = arguments.into_iter();

        let path_value = arguments.next().unwrap();
        let media_type = arguments.next()
            .map(|media_type_value| media_type_value.get_const_str())
            .transpose()?;

        let (path, bytes) = read_file_bytes(local_context, &path_value)?;

        let mut data_url = dataurl::DataUrl::new();
        // Annoyingly, this immediately calls to_vec() on the argument. What's even the point of
        // making us pass in a &[u8], then??
        data_url.set_data(&bytes);
        data_url.set_is_base64_encoded(true);

        // Guess the media type for the file based on the path or user override
        if let Some(media_type) = media_type {
            data_url.set_media_type(Some(media_type.to_string()));
        }
        else if let Some(mime) = mime_guess::from_path(&path).first() {
            data_url.set_media_type(Some(mime.essence_str().to_string()));
        }

        Ok(Value::Str(data_url.to_string().into()))
    },
};

pub static IMAGE: IntrinsicFunction = IntrinsicFunction {
    identifier: "image",
    min_arity: 5,
    max_arity: Some(8),
    interpret_call: |_, _, _, arguments| {
        let mut arguments = arguments.into_iter();

        let url_value = arguments.next().unwrap();
        let url = url_value.get_const_str()?;
        let name_value = arguments.next().unwrap();
        let name = name_value.get_const_str()?;
        let center = arguments.next().unwrap()
            .coerce(context, &Type::Point2D {
                x_type: Box::new(Type::Real),
                y_type: Box::new(Type::Real),
            }, true)?;
        let width = arguments.next().unwrap()
            .coerce(context, &Type::Real, true)?;
        let height = arguments.next().unwrap()
            .coerce(context, &Type::Real, true)?;
        let opacity = arguments.next()
            .unwrap_or(Value::Real(1.0).into())
            .coerce(context, &Type::Real, true)?;
        let angle = arguments.next()
            .unwrap_or(Value::Real(0.0).into())
            .coerce(context, &Type::Real, true)?;
        let background = arguments.next()
            .map_or(Ok(false), |background_value| {
                background_value.kind
                    .as_const_bool()
                    .ok_or_else(|| Box::new(crate::Error {
                        kind: crate::ErrorKind::ExpectedConstant {
                            type_identifier: Type::Bool.to_string(),
                        },
                        span: background_value.span,
                    }))
            })?;

        let list_state = center.get_type().list_state();
        let list_state = ListState::merge(list_state, width.get_type().list_state());
        let list_state = ListState::merge(list_state, height.get_type().list_state());
        let list_state = ListState::merge(list_state, opacity.get_type().list_state());
        let list_state = ListState::merge(list_state, angle.get_type().list_state());

        Ok(Value::Image(
            Box::new(ImageValue {
                url,
                name,
                center,
                width,
                height,
                opacity,
                angle,
                background,
            }),
            list_state,
        ))
    },
};

pub static CONCAT: IntrinsicFunction = IntrinsicFunction {
    identifier: "concat",
    min_arity: 0,
    max_arity: None,
    interpret_call: |_, _, _, arguments| {
        let mut result = String::new();

        for argument in arguments {
            result.push_str(argument.get_const_str()?.as_ref());
        }

        Ok(Value::Str(result.into()))
    },
};

pub static TARGET_SYMBOL: IntrinsicFunction = IntrinsicFunction {
    identifier: "target_symbol",
    min_arity: 1,
    max_arity: Some(1),
    interpret_call: |target, _, _, arguments| {
        let argument = arguments.into_iter().next().unwrap();

        let symbol_name = match &argument.kind {
            Value::GlobalReference(reference) => {
                target.get_global_symbol_name(&reference.identifier)
            }
            Value::ActionReference(reference) => {
                target.get_action_symbol_name(&reference.identifier)
            }
            _ => return Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ExpectedGlobalOrActionReference,
                span: argument.span,
            }))
        };

        Ok(Value::Str(symbol_name.into()))
    },
};
