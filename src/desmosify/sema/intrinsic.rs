use std::path::PathBuf;
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
        span: Option<crate::Span>,
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

        (self.interpret_call)(context, local_context, span, arguments)
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
    span: Option<crate::Span>,
    kind: UnaryKind,
    argument_type: TypeHandle,
    result_type: Option<TypeHandle>,
    arguments: Box<[ValueHandle]>,
) -> crate::Result<ValueEntry> {
    let argument = arguments[0]
        .coerce(context, argument_type, false)?;

    Ok(ValueEntry {
        value: Value::Unary {
            kind,
            operand: argument,
        },
        type_handle: result_type.unwrap_or(argument_type),
        span,
    })
}

pub fn interpret_broadcastable_unary_call(
    context: &mut GlobalContext,
    span: Option<crate::Span>,
    kind: UnaryKind,
    argument_type: TypeHandle,
    result_type: Option<TypeHandle>,
    arguments: Box<[ValueHandle]>,
) -> crate::Result<ValueEntry> {
    let argument = arguments[0]
        .coerce(context, argument_type, true)?;
    let argument_type = argument.get_type(&context.values);
    let list_state = argument_type.flatten_list(&context.types).0;

    Ok(ValueEntry {
        value: Value::Unary {
            kind,
            operand: argument,
        },
        type_handle: match result_type {
            Some(result_type) => result_type.unflatten_list(&mut context.types, list_state, span)?,
            None => argument_type,
        },
        span,
    })
}

pub fn interpret_broadcastable_binary_call(
    context: &mut GlobalContext,
    span: Option<crate::Span>,
    kind: BinaryKind,
    lhs_type: TypeHandle,
    rhs_type: TypeHandle,
    result_type: TypeHandle,
    arguments: Box<[ValueHandle]>,
) -> crate::Result<ValueEntry> {
    let lhs = arguments[0]
        .coerce(context, lhs_type, true)?;
    let rhs = arguments[1]
        .coerce(context, rhs_type, true)?;

    let list_state = ListState::merge(
        lhs.get_type(&context.values).flatten_list(&context.types).0,
        rhs.get_type(&context.values).flatten_list(&context.types).0,
    );

    Ok(ValueEntry {
        value: Value::Binary {
            kind,
            lhs,
            rhs,
        },
        type_handle: result_type.unflatten_list(&mut context.types, list_state, span)?,
        span,
    })
}

pub fn interpret_broadcastable_ternary_call(
    context: &mut GlobalContext,
    span: Option<crate::Span>,
    kind: TernaryKind,
    first_type: TypeHandle,
    second_type: TypeHandle,
    third_type: TypeHandle,
    result_type: TypeHandle,
    arguments: Box<[ValueHandle]>,
) -> crate::Result<ValueEntry> {
    let first = arguments[0]
        .coerce(context, first_type, true)?;
    let second = arguments[1]
        .coerce(context, second_type, true)?;
    let third = arguments[2]
        .coerce(context, third_type, true)?;

    let list_state = ListState::merge_all([
        first.get_type(&context.values).flatten_list(&context.types).0,
        second.get_type(&context.values).flatten_list(&context.types).0,
        third.get_type(&context.values).flatten_list(&context.types).0,
    ]);

    Ok(ValueEntry {
        value: Value::Ternary {
            kind,
            first,
            second,
            third,
        },
        type_handle: result_type.unflatten_list(&mut context.types, list_state, span)?,
        span,
    })
}

pub fn interpret_broadcastable_reducer_call(
    context: &mut GlobalContext,
    span: Option<crate::Span>,
    kind: ReducerKind,
    element_type: Option<TypeHandle>,
    result_type: Option<TypeHandle>,
    arguments: Box<[ValueHandle]>,
) -> crate::Result<ValueEntry> {
    if let &[mut list] = arguments.as_ref() {
        if let &Type::List { item_type, .. } = list.get_type(&context.values).get(&context.types) {
            // This should also work for any MaybeList.
            if let Some(element_type) = element_type {
                list = list.coerce(context, element_type, true)?;
            }

            return Ok(ValueEntry {
                value: Value::Reducer {
                    kind,
                    list,
                },
                type_handle: result_type.unwrap_or(item_type),
                span,
            })
        }
    }

    // Determine the most restrictive type that fits all arguments.
    let mut merged_type = arguments[1..].iter().try_fold(
        arguments[0].get_type(&context.values),
        |current_type, &argument| context.types.merge(
            current_type,
            argument.get_type(&context.values),
            argument.get_span(&context.values),
        ),
    )?;

    if let Some(element_type) = element_type {
        let merged_inner = merged_type.flatten_list(&context.types).1;
        merged_type = context.types.coerce(merged_inner, element_type)
            .ok_or_else(|| Box::new(crate::Error {
                kind: crate::ErrorKind::MismatchedTypes {
                    expected_type: context.types.repr(element_type),
                    got_type: context.types.repr(merged_inner),
                },
                span,
            }))?;
    }

    let mut list_state = None;
    Ok(ValueEntry {
        value: Value::ArgumentsReducer {
            kind,
            arguments: arguments
                .into_iter()
                .map(|argument| {
                    list_state = ListState::merge(
                        list_state,
                        argument.get_type(&context.values).flatten_list(&context.types).0,
                    );
                    argument.coerce(context, merged_type, true)
                })
                .collect::<crate::Result<_>>()?,
        },
        type_handle: result_type.unwrap_or(merged_type).unflatten_list(&mut context.types, list_state, span)?,
        span,
    })
}

macro_rules! strict_intrinsic {
    // Unary
    ($identifier:expr, ($argument_type:expr) => $kind:ident) => {
        IntrinsicFunction {
            identifier: $identifier,
            min_arity: 1,
            max_arity: Some(1),
            interpret_call: |context, _, span, arguments| interpret_strict_unary_call(
                context,
                span,
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
            interpret_call: |context, _, span, arguments| interpret_strict_unary_call(
                context,
                span,
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
            interpret_call: |context, _, span, arguments| interpret_broadcastable_unary_call(
                context,
                span,
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
            interpret_call: |context, _, span, arguments| interpret_broadcastable_unary_call(
                context,
                span,
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
            interpret_call: |context, _, span, arguments| interpret_broadcastable_binary_call(
                context,
                span,
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
            interpret_call: |context, _, span, arguments| interpret_broadcastable_ternary_call(
                context,
                span,
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
            interpret_call: |context, _, span, arguments| interpret_broadcastable_reducer_call(
                context,
                span,
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
            interpret_call: |context, _, span, arguments| interpret_broadcastable_reducer_call(
                context,
                span,
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
            interpret_call: |context, _, span, arguments| interpret_broadcastable_reducer_call(
                context,
                span,
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
            interpret_call: |context, _, span, arguments| interpret_broadcastable_reducer_call(
                context,
                span,
                ReducerKind::$kind,
                Some($element_type),
                Some($result_type),
                arguments,
            ),
        }
    };
}

pub fn read_file_bytes(
    context: &GlobalContext,
    local_context: &LocalContext,
    span: Option<crate::Span>,
    path_value: ValueHandle,
) -> crate::Result<(PathBuf, Vec<u8>)> {
    let relative_path = path_value.expect_const_str(context)?;

    let full_path = local_context.source_directory().join(relative_path.as_ref());

    std::fs::read(&full_path)
        .map_err(|cause| Box::new(crate::Error {
            kind: crate::ErrorKind::FileOpen {
                path: Some(full_path.as_path().into()),
                cause,
            },
            span,
        }))
        .map(|contents| (full_path, contents))
}

// ------ Trigonometric ------

pub static SIN: IntrinsicFunction = broadcastable_intrinsic!(
    "sin", (TypeHandle::REAL) => Sin
);

pub static COS: IntrinsicFunction = broadcastable_intrinsic!(
    "cos", (TypeHandle::REAL) => Cos
);

pub static TAN: IntrinsicFunction = broadcastable_intrinsic!(
    "tan", (TypeHandle::REAL) => Tan
);

pub static CSC: IntrinsicFunction = broadcastable_intrinsic!(
    "csc", (TypeHandle::REAL) => Csc
);

pub static SEC: IntrinsicFunction = broadcastable_intrinsic!(
    "sec", (TypeHandle::REAL) => Sec
);

pub static COT: IntrinsicFunction = broadcastable_intrinsic!(
    "cot", (TypeHandle::REAL) => Cot
);

pub static ARCSIN: IntrinsicFunction = broadcastable_intrinsic!(
    "arcsin", (TypeHandle::REAL) => Arcsin
);

pub static ARCCOS: IntrinsicFunction = broadcastable_intrinsic!(
    "arccos", (TypeHandle::REAL) => Arccos
);

pub static ARCTAN: IntrinsicFunction = broadcastable_intrinsic!(
    "arctan", (TypeHandle::REAL) => Arctan
);

pub static ARCTAN2: IntrinsicFunction = broadcastable_intrinsic!(
    "arctan2", (TypeHandle::REAL, TypeHandle::REAL) => Arctan2, TypeHandle::REAL
);

pub static ARCCSC: IntrinsicFunction = broadcastable_intrinsic!(
    "arccsc", (TypeHandle::REAL) => Arccsc
);

pub static ARCSEC: IntrinsicFunction = broadcastable_intrinsic!(
    "arcsec", (TypeHandle::REAL) => Arcsec
);

pub static ARCCOT: IntrinsicFunction = broadcastable_intrinsic!(
    "arccot", (TypeHandle::REAL) => Arccot
);

pub static SINH: IntrinsicFunction = broadcastable_intrinsic!(
    "sinh", (TypeHandle::REAL) => Sinh
);

pub static COSH: IntrinsicFunction = broadcastable_intrinsic!(
    "cosh", (TypeHandle::REAL) => Cosh
);

pub static TANH: IntrinsicFunction = broadcastable_intrinsic!(
    "tanh", (TypeHandle::REAL) => Tanh
);

pub static CSCH: IntrinsicFunction = broadcastable_intrinsic!(
    "csch", (TypeHandle::REAL) => Csch
);

pub static SECH: IntrinsicFunction = broadcastable_intrinsic!(
    "sech", (TypeHandle::REAL) => Sech
);

pub static COTH: IntrinsicFunction = broadcastable_intrinsic!(
    "coth", (TypeHandle::REAL) => Coth
);

// ------ Calculus ------

pub static EXP: IntrinsicFunction = broadcastable_intrinsic!(
    "exp", (TypeHandle::REAL) => Exp
);

pub static LN: IntrinsicFunction = broadcastable_intrinsic!(
    "ln", (TypeHandle::REAL) => Ln
);

pub static LOG: IntrinsicFunction = broadcastable_intrinsic!(
    "log", (TypeHandle::REAL, TypeHandle::REAL) => Log, TypeHandle::REAL
);

// DERIVATIVE

// INTEGRAL

// SUM

// PRODUCT

// ------ Number Theory ------

pub static LCM: IntrinsicFunction = broadcastable_intrinsic!(
    "lcm", [TypeHandle::INT] => Lcm
);

pub static GCD: IntrinsicFunction = broadcastable_intrinsic!(
    "gcd", [TypeHandle::INT] => Gcd
);

pub static CEIL: IntrinsicFunction = broadcastable_intrinsic!(
    "ceil", (TypeHandle::REAL) => Ceil, TypeHandle::INT
);

pub static FLOOR: IntrinsicFunction = broadcastable_intrinsic!(
    "floor", (TypeHandle::REAL) => Floor, TypeHandle::INT
);

pub static ROUND: IntrinsicFunction = broadcastable_intrinsic!(
    "round", (TypeHandle::REAL) => Round, TypeHandle::INT
);

pub static ROUND_DIGITS: IntrinsicFunction = broadcastable_intrinsic!(
    "round_digits", (TypeHandle::REAL, TypeHandle::INT) => RoundDigits, TypeHandle::REAL
);

pub static ABS: IntrinsicFunction = broadcastable_intrinsic!(
    "abs", (TypeHandle::ARITHMETIC_SCALAR) => Abs
);

pub static SIGN: IntrinsicFunction = broadcastable_intrinsic!(
    "sign", (TypeHandle::REAL) => Sign, TypeHandle::INT
);

pub static SQRT: IntrinsicFunction = broadcastable_intrinsic!(
    "sqrt", (TypeHandle::REAL) => Sqrt, TypeHandle::REAL
);

pub static CBRT: IntrinsicFunction = broadcastable_intrinsic!(
    "cbrt", (TypeHandle::REAL) => Cbrt, TypeHandle::REAL
);

pub static NTH_ROOT: IntrinsicFunction = broadcastable_intrinsic!(
    "nth_root", (TypeHandle::REAL, TypeHandle::REAL) => NthRoot, TypeHandle::REAL
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
    interpret_call: |context, _, span, arguments| {
        let item_type = arguments[1..].iter().try_fold(
            arguments[0].get_type(&context.values).flatten_list(&context.types).1,
            |current_type, &argument| context.types.merge_inner(
                current_type,
                argument.get_type(&context.values).flatten_list(&context.types).1,
                span,
            ),
        )?;

        Ok(ValueEntry {
            value: Value::Join {
                values: arguments,
            },
            type_handle: item_type.into_list(&mut context.types, ListState::IsList, span)?,
            span,
        })
    },
};

pub static SORT: IntrinsicFunction = IntrinsicFunction {
    identifier: "sort",
    min_arity: 1,
    max_arity: Some(2),
    interpret_call: |context, _, span, arguments| {
        let list = arguments[0];
        context.types.expect_list_type(list.get_type(&context.values), list.get_span(&context.values))?;

        if let Some(&key_list) = arguments.get(1) {
            context.types.expect_list_type(key_list.get_type(&context.values), key_list.get_span(&context.values))?;

            let key_list = key_list.coerce(context, TypeHandle::ANY_SORTABLE, true)?;

            Ok(ValueEntry {
                value: Value::Binary {
                    kind: BinaryKind::SortKeyed,
                    lhs: list,
                    rhs: key_list,
                },
                type_handle: list.get_type(&context.values),
                span,
            })
        }
        else {
            let list = list.coerce(context, TypeHandle::ANY_SORTABLE, true)?;

            Ok(ValueEntry {
                value: Value::Unary {
                    kind: UnaryKind::Sort,
                    operand: list,
                },
                type_handle: list.get_type(&context.values),
                span,
            })
        }
    },
};

pub static SHUFFLE: IntrinsicFunction = IntrinsicFunction {
    identifier: "shuffle",
    min_arity: 1,
    max_arity: Some(2),
    interpret_call: |context, _, span, arguments| {
        let list = arguments[0];
        let list_type = list.get_type(&context.values);
        context.types.expect_list_type(list_type, list.get_span(&context.values))?;

        if let Some(&seed) = arguments.get(1) {
            let seed = seed.coerce(context, TypeHandle::REAL, false)?;

            Ok(ValueEntry {
                value: Value::Binary {
                    kind: BinaryKind::ShuffleSeeded,
                    lhs: list,
                    rhs: seed,
                },
                type_handle: list_type,
                span,
            })
        }
        else {
            Ok(ValueEntry {
                value: Value::Unary {
                    kind: UnaryKind::Shuffle,
                    operand: list,
                },
                type_handle: list_type,
                span,
            })
        }
    },
};

pub static UNIQUE: IntrinsicFunction = IntrinsicFunction {
    identifier: "unique",
    min_arity: 1,
    max_arity: Some(1),
    interpret_call: |context, _, span, arguments| {
        // It seems like unique() accepts basically any type, so don't bother checking the item type
        // FIXME: per desmos code, only distributions cannot be uniqued
        let list = arguments[0];
        let list_type = list.get_type(&context.values);
        context.types.expect_list_type(list_type, list.get_span(&context.values))?;

        Ok(ValueEntry {
            value: Value::Unary {
                kind: UnaryKind::Unique,
                operand: list,
            },
            type_handle: list_type,
            span,
        })
    },
};

pub static PREFIX_SUM: IntrinsicFunction = IntrinsicFunction {
    identifier: "prefix_sum",
    min_arity: 1,
    max_arity: Some(1),
    interpret_call: |context, _, span, arguments| {
        let list = arguments[0];
        context.types.expect_list_type(list.get_type(&context.values), list.get_span(&context.values))?;

        let list = list.coerce(context, TypeHandle::ARITHMETIC_SCALAR_OR_POINT, true)?;

        Ok(ValueEntry {
            value: Value::Unary {
                kind: UnaryKind::PrefixSum,
                operand: list,
            },
            type_handle: list.get_type(&context.values),
            span,
        })
    },
};

// ------ Statistics ------

pub static MEAN: IntrinsicFunction = broadcastable_intrinsic!(
    "mean", [TypeHandle::REAL_SCALAR_OR_POINT] => Mean
);

pub static MEDIAN: IntrinsicFunction = broadcastable_intrinsic!(
    "median", [TypeHandle::REAL] => Median
);

pub static MIN: IntrinsicFunction = broadcastable_intrinsic!(
    "min", [TypeHandle::NUMERIC_SCALAR] => Min
);

pub static MAX: IntrinsicFunction = broadcastable_intrinsic!(
    "max", [TypeHandle::NUMERIC_SCALAR] => Max
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
    "count", [?] => Count, TypeHandle::INT
);

pub static TOTAL: IntrinsicFunction = broadcastable_intrinsic!(
    "total", [TypeHandle::ARITHMETIC_SCALAR_OR_POINT] => Total
);

// @total(arguments) > 0
pub static ANY: IntrinsicFunction = IntrinsicFunction {
    identifier: "any",
    min_arity: 1,
    max_arity: None,
    interpret_call: |context, local_context, span, mut arguments| {
        for argument in &mut arguments {
            *argument = argument.coerce(context, TypeHandle::BOOL, true)?;
        }

        let total = (TOTAL.interpret_call)(context, local_context, span, arguments)?;
        let list_state = total.type_handle.flatten_list(&context.types).0;

        Ok(ValueEntry {
            value: Value::InequalityChain {
                lhs: total.register(&mut context.values),
                chain: Box::new([(
                    InequalityKind::GreaterThan,
                    ValueHandle::ZERO_INT,
                )]),
            },
            type_handle: TypeHandle::BOOL.unflatten_list(&mut context.types, list_state, span)?,
            span,
        })
    },
};

// @total(!arguments) == 0
pub static ALL: IntrinsicFunction = IntrinsicFunction {
    identifier: "all",
    min_arity: 1,
    max_arity: None,
    interpret_call: |context, local_context, span, mut arguments| {
        for argument in &mut arguments {
            let coerced = argument.coerce(context, TypeHandle::BOOL, true)?;
            let coerced_span = coerced.get_span(&context.values);
            let list_state = coerced.get_type(&context.values).flatten_list(&context.types).0;

            *argument = context.values.register(ValueEntry {
                value: Value::Unary {
                    kind: UnaryKind::LogicalNot,
                    operand: coerced,
                },
                type_handle: TypeHandle::BOOL.unflatten_list(&mut context.types, list_state, coerced_span)?,
                span: coerced_span,
            });
        }

        let total = (TOTAL.interpret_call)(context, local_context, span, arguments)?;
        let list_state = total.type_handle.flatten_list(&context.types).0;

        Ok(ValueEntry {
            value: Value::Binary {
                kind: BinaryKind::Equal,
                lhs: total.register(&mut context.values),
                rhs: ValueHandle::ZERO_INT,
            },
            type_handle: TypeHandle::BOOL.unflatten_list(&mut context.types, list_state, span)?,
            span,
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
    context: &mut GlobalContext,
    span: Option<crate::Span>,
    arguments: &[ValueHandle],
    source: Option<ValueHandle>,
    result_type: TypeHandle,
) -> crate::Result<ValueEntry> {
    if let Some(&sample_count) = arguments.get(0) {
        let sample_count = sample_count.coerce(context, TypeHandle::INT, false)?;

        if let Some(&seed) = arguments.get(1) {
            let seed = seed.coerce(context, TypeHandle::REAL, false)?;

            Ok(ValueEntry {
                value: Value::RandomSeeded {
                    source,
                    sample_count,
                    seed,
                },
                type_handle: result_type.into_list(&mut context.types, ListState::IsList, span)?,
                span,
            })
        }
        else {
            Ok(ValueEntry {
                value: Value::Random {
                    source,
                    sample_count: Some(sample_count),
                },
                type_handle: result_type.into_list(&mut context.types, ListState::IsList, span)?,
                span,
            })
        }
    }
    else {
        Ok(ValueEntry {
            value: Value::Random {
                source,
                sample_count: None,
            },
            type_handle: result_type,
            span,
        })
    }
}

pub static RANDOM: IntrinsicFunction = IntrinsicFunction {
    identifier: "random",
    min_arity: 0,
    max_arity: Some(2),
    interpret_call: |context, _, span, arguments| {
        interpret_random_call_end(context, span, &arguments, None, TypeHandle::REAL)
    },
};

pub static CHOOSE_RANDOM: IntrinsicFunction = IntrinsicFunction {
    identifier: "choose_random",
    min_arity: 1,
    max_arity: Some(3),
    interpret_call: |context, _, span, arguments| {
        let source = arguments[0];
        let source_type = source.get_type(&context.values);

        let result_type = match source_type.flatten_list(&context.types) {
            (Some(ListState::IsList), item_type) if item_type != TypeHandle::DISTRIBUTION => item_type,
            (None, TypeHandle::DISTRIBUTION) => TypeHandle::REAL,
            _ => return Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ExpectedListOrDistributionType {
                    got_type: source_type.repr(&context.types),
                },
                span: source.get_span(&context.values),
            }))
        };

        interpret_random_call_end(context, span, &arguments[1..], Some(source), result_type)
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
    "segment", (TypeHandle::REAL_POINT_2D, TypeHandle::REAL_POINT_2D) => SegmentFromPoints2D, TypeHandle::SEGMENT
);

pub static SEGMENT3D: IntrinsicFunction = broadcastable_intrinsic!(
    "segment3d", (TypeHandle::REAL_POINT_3D, TypeHandle::REAL_POINT_3D) => SegmentFromPoints3D, TypeHandle::SEGMENT_3D
);

pub static LINE: IntrinsicFunction = IntrinsicFunction {
    identifier: "line",
    min_arity: 1,
    max_arity: Some(2),
    interpret_call: |context, _, span, arguments| {
        if let Some(end) = arguments.get(1) {
            let start = arguments[0].coerce(context, TypeHandle::REAL_POINT_2D, true)?;
            let end = end.coerce(context, TypeHandle::REAL_POINT_2D, true)?;

            let list_state = ListState::merge(
                start.get_type(&context.values).flatten_list(&context.types).0,
                end.get_type(&context.values).flatten_list(&context.types).0,
            );

            Ok(ValueEntry {
                value: Value::Binary {
                    kind: BinaryKind::LineFromPoints2D,
                    lhs: start,
                    rhs: end,
                },
                type_handle: TypeHandle::LINE.unflatten_list(&mut context.types, list_state, span)?,
                span,
            })
        }
        else {
            let segment_or_ray = arguments[0].coerce(context, TypeHandle::SEGMENT_OR_RAY, true)?;

            let (list_state, argument_type) = segment_or_ray.get_type(&context.values).flatten_list(&context.types);

            Ok(ValueEntry {
                value: Value::Unary {
                    kind: match argument_type {
                        TypeHandle::SEGMENT => UnaryKind::LineFromSegment2D,
                        TypeHandle::RAY => UnaryKind::LineFromRay2D,
                        _ => unreachable!()
                    },
                    operand: segment_or_ray,
                },
                type_handle: TypeHandle::LINE.unflatten_list(&mut context.types, list_state, span)?,
                span,
            })
        }
    },
};

pub static RAY: IntrinsicFunction = broadcastable_intrinsic!(
    "ray", (TypeHandle::REAL_POINT_2D, TypeHandle::REAL_POINT_2D) => RayFromPoints2D, TypeHandle::RAY
);

pub static VECTOR: IntrinsicFunction = broadcastable_intrinsic!(
    "vector", (TypeHandle::REAL_POINT_2D, TypeHandle::REAL_POINT_2D) => VectorFromPoints2D, TypeHandle::VECTOR
);

pub static VECTOR3D: IntrinsicFunction = broadcastable_intrinsic!(
    "vector3d", (TypeHandle::REAL_POINT_3D, TypeHandle::REAL_POINT_3D) => VectorFromPoints3D, TypeHandle::VECTOR_3D
);

// PARALLEL

// PERPENDICULAR

pub static CIRCLE: IntrinsicFunction = IntrinsicFunction {
    identifier: "circle",
    min_arity: 2,
    max_arity: Some(2),
    interpret_call: |context, _, span, arguments| {
        let lhs = arguments[0].coerce(context, TypeHandle::REAL_POINT_2D, true)?;
        let (lhs_list, _) = lhs.get_type(&context.values).flatten_list(&context.types);

        let rhs = arguments[1].coerce(context, TypeHandle::REAL_SCALAR_OR_POINT_2D, true)?;
        let (rhs_list, rhs_inner) = rhs.get_type(&context.values).flatten_list(&context.types);

        let list_state = ListState::merge(lhs_list, rhs_list);

        Ok(ValueEntry {
            value: Value::Binary {
                kind: match rhs_inner {
                    TypeHandle::REAL => BinaryKind::CircleFromRadius2D,
                    TypeHandle::REAL_POINT_2D => BinaryKind::CircleFromEdge2D,
                    _ => unreachable!()
                },
                lhs,
                rhs,
            },
            type_handle: TypeHandle::CIRCLE.unflatten_list(&mut context.types, list_state, span)?,
            span,
        })
    },
};

pub static SPHERE3D: IntrinsicFunction = broadcastable_intrinsic!(
    "sphere3d", (TypeHandle::REAL_POINT_3D, TypeHandle::REAL) => SphereFromRadius3D, TypeHandle::SPHERE_3D
);

pub static ARC: IntrinsicFunction = broadcastable_intrinsic!(
    "arc", (TypeHandle::REAL_POINT_2D, TypeHandle::REAL_POINT_2D, TypeHandle::REAL_POINT_2D) => Arc2D, TypeHandle::ARC
);

pub static ANGLE: IntrinsicFunction = broadcastable_intrinsic!(
    "angle", (TypeHandle::REAL_POINT_2D, TypeHandle::REAL_POINT_2D, TypeHandle::REAL_POINT_2D) => UndirectedAngle2D, TypeHandle::ANGLE
);

pub static DIRECTED_ANGLE: IntrinsicFunction = broadcastable_intrinsic!(
    "directed_angle", (TypeHandle::REAL_POINT_2D, TypeHandle::REAL_POINT_2D, TypeHandle::REAL_POINT_2D) => DirectedAngle2D, TypeHandle::DIRECTED_ANGLE
);

pub static POLYGON: IntrinsicFunction = broadcastable_intrinsic!(
    "polygon", [TypeHandle::REAL_POINT_2D] => PolygonFromVertices2D, TypeHandle::POLYGON
);

pub static RECT: IntrinsicFunction = broadcastable_intrinsic!(
    "rect", (TypeHandle::REAL_POINT_2D, TypeHandle::REAL_POINT_2D) => RectangleFromPoints2D, TypeHandle::POLYGON
);

pub static TRIANGLE3D: IntrinsicFunction = broadcastable_intrinsic!(
    "triangle", (TypeHandle::REAL_POINT_3D, TypeHandle::REAL_POINT_3D, TypeHandle::REAL_POINT_3D) => TriangleFromVertices3D, TypeHandle::TRIANGLE_3D
);

pub static GLIDER: IntrinsicFunction = broadcastable_intrinsic!(
    "glider", (TypeHandle::ANY_GLIDER_COMPATIBLE, TypeHandle::REAL) => Glider2D, TypeHandle::REAL_POINT_2D
);

// ------ Properties & Measurements ------

// DOT

// CROSS

// DISTANCE

// LENGTH

pub static AREA: IntrinsicFunction = broadcastable_intrinsic!(
    "area", (TypeHandle::POLYGON) => AreaOfPolygon, TypeHandle::REAL
);

pub static PERIMETER: IntrinsicFunction = broadcastable_intrinsic!(
    "perimeter", (TypeHandle::POLYGON) => PerimeterOfPolygon, TypeHandle::REAL
);

pub static VERTICES: IntrinsicFunction = strict_intrinsic!(
    "vertices", (TypeHandle::POLYGON) => VerticesOfPolygon, TypeHandle::LIST_OF_REAL_POINT_2D
);

pub static ANGLES: IntrinsicFunction = strict_intrinsic!(
    "angles", (TypeHandle::POLYGON) => UndirectedAnglesOfPolygon, TypeHandle::LIST_OF_ANGLE
);

pub static DIRECTED_ANGLES: IntrinsicFunction = strict_intrinsic!(
    "directed_angles", (TypeHandle::POLYGON) => DirectedAnglesOfPolygon, TypeHandle::LIST_OF_DIRECTED_ANGLE
);

pub static SEGMENTS: IntrinsicFunction = strict_intrinsic!(
    "segments", (TypeHandle::POLYGON) => SegmentsOfPolygon, TypeHandle::LIST_OF_SEGMENT
);

pub static RADIUS: IntrinsicFunction = broadcastable_intrinsic!(
    "radius", (TypeHandle::CIRCLE) => RadiusOfCircle, TypeHandle::REAL
);

pub static CENTER: IntrinsicFunction = broadcastable_intrinsic!(
    "center", (TypeHandle::CIRCLE) => CenterOfCircle, TypeHandle::REAL
);

// COTERMINAL

// SUPPLEMENT

pub static MIDPOINT: IntrinsicFunction = IntrinsicFunction {
    identifier: "midpoint",
    min_arity: 1,
    max_arity: Some(2),
    interpret_call: |context, _, span, arguments| {
        if let Some(&end) = arguments.get(1) {
            let start = arguments[0].coerce(context, TypeHandle::ANY_REAL_POINT, true)?;
            let (start_list, point_type) = start.get_type(&context.values).flatten_list(&context.types);

            let kind = match point_type {
                TypeHandle::REAL_POINT_2D => BinaryKind::MidpointOfPoints2D,
                TypeHandle::REAL_POINT_3D => BinaryKind::MidpointOfPoints3D,
                _ => unreachable!()
            };

            let end = end.coerce(context, point_type, true)?;

            let list_state = ListState::merge(
                start_list,
                end.get_type(&context.values).flatten_list(&context.types).0,
            );

            Ok(ValueEntry {
                value: Value::Binary {
                    kind,
                    lhs: start,
                    rhs: end,
                },
                type_handle: point_type.unflatten_list(&mut context.types, list_state, span)?,
                span,
            })
        }
        else {
            let segment = arguments[0].coerce(context, TypeHandle::ANY_SEGMENT, true)?;
            let (list_state, segment_type) = segment.get_type(&context.values).flatten_list(&context.types);

            let (kind, point_type) = match segment_type {
                TypeHandle::SEGMENT => (UnaryKind::MidpointOfSegment2D, TypeHandle::REAL_POINT_2D),
                TypeHandle::SEGMENT_3D => (UnaryKind::MidpointOfSegment3D, TypeHandle::REAL_POINT_3D),
                _ => unreachable!()
            };

            Ok(ValueEntry {
                value: Value::Unary {
                    kind,
                    operand: segment,
                },
                type_handle: point_type.unflatten_list(&mut context.types, list_state, span)?,
                span,
            })
        }
    },
};

fn interpret_start_end_call(
    context: &mut GlobalContext,
    span: Option<crate::Span>,
    arguments: Box<[ValueHandle]>,
    kind_2d: UnaryKind,
    kind_3d: UnaryKind,
) -> crate::Result<ValueEntry> {
    let vector = arguments[0].coerce(context, TypeHandle::ANY_VECTOR, true)?;
    let (list_state, vector_type) = vector.get_type(&context.values).flatten_list(&context.types);

    let (kind, point_type) = match vector_type {
        TypeHandle::VECTOR => (kind_2d, TypeHandle::REAL_POINT_2D),
        TypeHandle::VECTOR_3D => (kind_3d, TypeHandle::REAL_POINT_3D),
        _ => unreachable!()
    };

    Ok(ValueEntry {
        value: Value::Unary {
            kind,
            operand: vector,
        },
        type_handle: point_type.unflatten_list(&mut context.types, list_state, span)?,
        span,
    })
}

pub static START: IntrinsicFunction = IntrinsicFunction {
    identifier: "start",
    min_arity: 1,
    max_arity: Some(1),
    interpret_call: |context, _, span, arguments| interpret_start_end_call(
        context,
        span,
        arguments,
        UnaryKind::StartOfVector2D,
        UnaryKind::StartOfVector3D,
    ),
};

pub static END: IntrinsicFunction = IntrinsicFunction {
    identifier: "end",
    min_arity: 1,
    max_arity: Some(1),
    interpret_call: |context, _, span, arguments| interpret_start_end_call(
        context,
        span,
        arguments,
        UnaryKind::EndOfVector2D,
        UnaryKind::EndOfVector3D,
    ),
};

// ------ Transformations ------

fn interpret_rotate_dilate_call(
    context: &mut GlobalContext,
    span: Option<crate::Span>,
    arguments: Box<[ValueHandle]>,
    kind: TernaryKind,
) -> crate::Result<ValueEntry> {
    let object = arguments[0].coerce(context, TypeHandle::ANY_TRANSFORMABLE, true)?;
    let (object_list, object_type) = object.get_type(&context.values).flatten_list(&context.types);

    let point = arguments[1].coerce(context, TypeHandle::REAL_POINT_2D, true)?;

    let factor_or_angle = arguments[2].coerce(context, TypeHandle::REAL, true)?;

    let list_state = ListState::merge_all([
        object_list,
        point.get_type(&context.values).flatten_list(&context.types).0,
        factor_or_angle.get_type(&context.values).flatten_list(&context.types).0,
    ]);

    Ok(ValueEntry {
        value: Value::Ternary {
            kind,
            first: object,
            second: point,
            third: factor_or_angle,
        },
        type_handle: object_type.unflatten_list(&mut context.types, list_state, span)?,
        span,
    })
}

pub static DILATE: IntrinsicFunction = IntrinsicFunction {
    identifier: "dilate",
    min_arity: 3,
    max_arity: Some(3),
    interpret_call: |context, _, span, arguments| interpret_rotate_dilate_call(
        context,
        span,
        arguments,
        TernaryKind::Dilate2D,
    ),
};

pub static ROTATE: IntrinsicFunction = IntrinsicFunction {
    identifier: "rotate",
    min_arity: 3,
    max_arity: Some(3),
    interpret_call: |context, _, span, arguments| interpret_rotate_dilate_call(
        context,
        span,
        arguments,
        TernaryKind::Rotate2D,
    ),
};

pub static REFLECT: IntrinsicFunction = IntrinsicFunction {
    identifier: "reflect",
    min_arity: 2,
    max_arity: Some(2),
    interpret_call: |context, _, span, arguments| {
        let object = arguments[0].coerce(context, TypeHandle::ANY_TRANSFORMABLE, true)?;
        let (object_list, object_type) = object.get_type(&context.values).flatten_list(&context.types);

        let line = arguments[1].coerce(context, TypeHandle::ANY_LINE_LIKE, true)?;

        let list_state = ListState::merge(
            object_list,
            line.get_type(&context.values).flatten_list(&context.types).0,
        );

        Ok(ValueEntry {
            value: Value::Binary {
                kind: BinaryKind::Reflect2D,
                lhs: object,
                rhs: line,
            },
            type_handle: object_type.unflatten_list(&mut context.types, list_state, span)?,
            span,
        })
    },
};

pub static TRANSLATE: IntrinsicFunction = IntrinsicFunction {
    identifier: "translate",
    min_arity: 2,
    max_arity: Some(3),
    interpret_call: |context, _, span, arguments| {
        let object = arguments[0].coerce(context, TypeHandle::ANY_TRANSFORMABLE, true)?;
        let (object_list, object_type) = object.get_type(&context.values).flatten_list(&context.types);

        if let Some(end) = arguments.get(2) {
            let start = arguments[1].coerce(context, TypeHandle::REAL_POINT_2D, true)?;
            let end = end.coerce(context, TypeHandle::REAL_POINT_2D, true)?;

            let list_state = ListState::merge_all([
                object_list,
                start.get_type(&context.values).flatten_list(&context.types).0,
                end.get_type(&context.values).flatten_list(&context.types).0,
            ]);

            Ok(ValueEntry {
                value: Value::Ternary {
                    kind: TernaryKind::TranslateByPoints2D,
                    first: object,
                    second: start,
                    third: end,
                },
                type_handle: object_type.unflatten_list(&mut context.types, list_state, span)?,
                span,
            })
        }
        else {
            let vector = arguments[1].coerce(context, TypeHandle::VECTOR, true)?;

            let list_state = ListState::merge(
                object_list,
                vector.get_type(&context.values).flatten_list(&context.types).0,
            );

            Ok(ValueEntry {
                value: Value::Binary {
                    kind: BinaryKind::TranslateByVector2D,
                    lhs: object,
                    rhs: vector,
                },
                type_handle: object_type.unflatten_list(&mut context.types, list_state, span)?,
                span,
            })
        }
    },
};

fn interpret_rotation_dilation_call(
    context: &mut GlobalContext,
    span: Option<crate::Span>,
    arguments: Box<[ValueHandle]>,
    kind: BinaryKind,
) -> crate::Result<ValueEntry> {
    let point = arguments[0].coerce(context, TypeHandle::REAL_POINT_2D, true)?;

    let factor_or_angle = arguments[1].coerce(context, TypeHandle::REAL, true)?;

    let list_state = ListState::merge(
        point.get_type(&context.values).flatten_list(&context.types).0,
        factor_or_angle.get_type(&context.values).flatten_list(&context.types).0,
    );

    Ok(ValueEntry {
        value: Value::Binary {
            kind,
            lhs: point,
            rhs: factor_or_angle,
        },
        type_handle: TypeHandle::TRANSFORMATION.unflatten_list(&mut context.types, list_state, span)?,
        span,
    })
}

pub static DILATION: IntrinsicFunction = IntrinsicFunction {
    identifier: "dilation",
    min_arity: 2,
    max_arity: Some(2),
    interpret_call: |context, _, span, arguments| interpret_rotation_dilation_call(
        context,
        span,
        arguments,
        BinaryKind::Dilation2D,
    ),
};

pub static ROTATION: IntrinsicFunction = IntrinsicFunction {
    identifier: "rotation",
    min_arity: 2,
    max_arity: Some(2),
    interpret_call: |context, _, span, arguments| interpret_rotation_dilation_call(
        context,
        span,
        arguments,
        BinaryKind::Rotation2D,
    ),
};

pub static REFLECTION: IntrinsicFunction = IntrinsicFunction {
    identifier: "reflection",
    min_arity: 1,
    max_arity: Some(1),
    interpret_call: |context, _, span, arguments| {
        let line = arguments[0].coerce(context, TypeHandle::LINE, true)?;
        let list_state = line.get_type(&context.values).flatten_list(&context.types).0;

        Ok(ValueEntry {
            value: Value::Unary {
                kind: UnaryKind::ReflectionByLine2D,
                operand: line,
            },
            type_handle: TypeHandle::TRANSFORMATION.unflatten_list(&mut context.types, list_state, span)?,
            span,
        })
    },
};

// TODO: allow 2-point and vector versions, and add this version to @translate
pub static TRANSLATION: IntrinsicFunction = IntrinsicFunction {
    identifier: "translation",
    min_arity: 1,
    max_arity: Some(1),
    interpret_call: |context, _, span, arguments| {
        let displacement = arguments[0].coerce(context, TypeHandle::REAL_POINT_2D, true)?;
        let list_state = displacement.get_type(&context.values).flatten_list(&context.types).0;

        Ok(ValueEntry {
            value: Value::Unary {
                kind: UnaryKind::TranslationByPoint2D,
                operand: displacement,
            },
            type_handle: TypeHandle::TRANSFORMATION.unflatten_list(&mut context.types, list_state, span)?,
            span,
        })
    },
};

pub static APPLY: IntrinsicFunction = IntrinsicFunction {
    identifier: "apply",
    min_arity: 2,
    max_arity: Some(2),
    interpret_call: |context, _, span, arguments| {
        let transformation = arguments[0].coerce(context, TypeHandle::TRANSFORMATION, true)?;

        let object = arguments[1].coerce(context, TypeHandle::ANY_TRANSFORMABLE, true)?;
        let (object_list, object_type) = object.get_type(&context.values).flatten_list(&context.types);

        let list_state = ListState::merge(
            transformation.get_type(&context.values).flatten_list(&context.types).0,
            object_list,
        );

        Ok(ValueEntry {
            value: Value::Binary {
                kind: BinaryKind::ApplyTransform2D,
                lhs: transformation,
                rhs: object,
            },
            type_handle: object_type.unflatten_list(&mut context.types, list_state, span)?,
            span,
        })
    },
};

pub static COMPOSE: IntrinsicFunction = broadcastable_intrinsic!(
    "compose", [TypeHandle::TRANSFORMATION] => ComposeTransforms2D
);

pub static INVERSE: IntrinsicFunction = broadcastable_intrinsic!(
    "inverse", (TypeHandle::TRANSFORMATION) => InverseOfTransform2D
);

// ------ Color ------

pub static RGB: IntrinsicFunction = broadcastable_intrinsic!(
    "rgb", (TypeHandle::REAL, TypeHandle::REAL, TypeHandle::REAL) => Rgb, TypeHandle::COLOR
);

pub static HSV: IntrinsicFunction = broadcastable_intrinsic!(
    "hsv", (TypeHandle::REAL, TypeHandle::REAL, TypeHandle::REAL) => Hsv, TypeHandle::COLOR
);

pub static OKHSV: IntrinsicFunction = broadcastable_intrinsic!(
    "okhsv", (TypeHandle::REAL, TypeHandle::REAL, TypeHandle::REAL) => Okhsv, TypeHandle::COLOR
);

pub static OKLAB: IntrinsicFunction = broadcastable_intrinsic!(
    "oklab", (TypeHandle::REAL, TypeHandle::REAL, TypeHandle::REAL) => Oklab, TypeHandle::COLOR
);

pub static OKLCH: IntrinsicFunction = broadcastable_intrinsic!(
    "oklch", (TypeHandle::REAL, TypeHandle::REAL, TypeHandle::REAL) => Oklch, TypeHandle::COLOR
);

// ------ Sound ------

// TONE

// ------ Desmosify ------

pub static BOOL_TO_INTERNAL: IntrinsicFunction = broadcastable_intrinsic!(
    "bool_to_internal", (TypeHandle::BOOL) => BoolToInternal, TypeHandle::INTERNAL_BOOL
);

pub static BOOL_FROM_INTERNAL: IntrinsicFunction = broadcastable_intrinsic!(
    "bool_from_internal", (TypeHandle::INTERNAL_BOOL) => BoolFromInternal, TypeHandle::BOOL
);

pub static ENUM_VALUES: IntrinsicFunction = IntrinsicFunction {
    identifier: "enum_values",
    min_arity: 1,
    max_arity: Some(1),
    interpret_call: |context, _, span, arguments| {
        let enum_type_value = arguments[0];
        let &Value::Type(enum_type) = enum_type_value.get(&context.values) else {
            return Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ExpectedEnumTypeValue,
                span: enum_type_value.get_span(&context.values),
            }));
        };
        let Type::Enum { values, .. } = enum_type.get(&context.types) else {
            return Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ExpectedEnumTypeValue,
                span: enum_type_value.get_span(&context.values),
            }));
        };

        Ok(ValueEntry {
            value: Value::List {
                // FIXME: these should probably be wrapped with GlobalSymbol
                items: values
                    .iter()
                    .map(|&(_, value)| value)
                    .collect(),
            },
            type_handle: enum_type.into_list(&mut context.types, ListState::IsList, span)?,
            span,
        })
    },
};

pub static ENUM_VALUE: IntrinsicFunction = IntrinsicFunction {
    identifier: "enum_value",
    min_arity: 2,
    max_arity: Some(2),
    interpret_call: |context, _, span, arguments| {
        let enum_type_value = arguments[0];
        let &Value::Type(enum_type) = enum_type_value.get(&context.values) else {
            return Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ExpectedEnumTypeValue,
                span: enum_type_value.get_span(&context.values),
            }));
        };
        let Type::Enum { .. } = enum_type.get(&context.types) else {
            return Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ExpectedEnumTypeValue,
                span: enum_type_value.get_span(&context.values),
            }));
        };

        let ordinal = arguments[1].coerce(context, TypeHandle::INT, true)?;
        let list_state = ordinal.get_type(&context.values).flatten_list(&context.types).0;

        Ok(ValueEntry {
            value: Value::Alias(ordinal),
            type_handle: enum_type.unflatten_list(&mut context.types, list_state, span)?,
            span,
        })
    },
};

pub static INCLUDE_TEXT: IntrinsicFunction = IntrinsicFunction {
    identifier: "include_text",
    min_arity: 1,
    max_arity: Some(1),
    interpret_call: |context, local_context, span, arguments| {
        // TODO: allow user to specify encoding; better error handling
        let path_value = arguments.into_iter().next().unwrap();
        let (path, bytes) = read_file_bytes(context, local_context, span, path_value)?;

        let text = String::from_utf8(bytes)
            .map_err(|_| Box::new(crate::Error {
                kind: crate::ErrorKind::FileRead {
                    path: Some(path.into_boxed_path()),
                    cause: std::io::ErrorKind::InvalidData.into(),
                },
                span: path_value.get_span(&context.values),
            }))?;

        Ok(ValueEntry {
            value: Value::Str(text.into()),
            type_handle: TypeHandle::STR,
            span,
        })
    },
};

pub static INCLUDE_DATA: IntrinsicFunction = IntrinsicFunction {
    identifier: "include_data",
    min_arity: 1,
    max_arity: Some(2),
    interpret_call: |context, local_context, span, arguments| {
        let mut arguments = arguments.into_iter();

        let path_value = arguments.next().unwrap();
        let media_type = arguments.next()
            .map(|media_type_value| media_type_value.expect_const_str(context))
            .transpose()?;

        let (path, bytes) = read_file_bytes(context, local_context, span, path_value)?;

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

        Ok(ValueEntry {
            value: Value::Str(data_url.to_string().into()),
            type_handle: TypeHandle::STR,
            span,
        })
    },
};

pub static IMAGE: IntrinsicFunction = IntrinsicFunction {
    identifier: "image",
    min_arity: 5,
    max_arity: Some(8),
    interpret_call: |context, _, span, arguments| {
        let url = arguments[0].expect_const_str(context)?;
        let name = arguments[1].expect_const_str(context)?;
        let center = arguments[2].coerce(context, TypeHandle::REAL_POINT_2D, true)?;
        let width = arguments[3].coerce(context, TypeHandle::REAL, true)?;
        let height = arguments[4].coerce(context, TypeHandle::REAL, true)?;
        let opacity = arguments.get(5)
            .copied()
            .unwrap_or(ValueHandle::ONE_REAL)
            .coerce(context, TypeHandle::REAL, true)?;
        let angle = arguments.get(6)
            .copied()
            .unwrap_or(ValueHandle::ZERO_REAL)
            .coerce(context, TypeHandle::REAL, true)?;
        let background = arguments.get(7)
            .map_or(Ok(false), |&background| background.expect_const_bool(context))?;

        let list_state = ListState::merge_all([
            center.get_type(&context.values).flatten_list(&context.types).0,
            width.get_type(&context.values).flatten_list(&context.types).0,
            height.get_type(&context.values).flatten_list(&context.types).0,
            opacity.get_type(&context.values).flatten_list(&context.types).0,
            angle.get_type(&context.values).flatten_list(&context.types).0,
        ]);

        Ok(ValueEntry {
            value: Value::Image(Box::new(ImageValue {
                url,
                name,
                center,
                width,
                height,
                opacity,
                angle,
                background,
            })),
            type_handle: TypeHandle::IMAGE.unflatten_list(&mut context.types, list_state, span)?,
            span,
        })
    },
};

pub static CONCAT: IntrinsicFunction = IntrinsicFunction {
    identifier: "concat",
    min_arity: 0,
    max_arity: None,
    interpret_call: |context, _, span, arguments| {
        let mut result = String::new();

        for argument in arguments {
            result.push_str(argument.expect_const_str(context)?.as_ref());
        }

        Ok(ValueEntry {
            value: Value::Str(result.into()),
            type_handle: TypeHandle::STR,
            span,
        })
    },
};

pub static TARGET_SYMBOL: IntrinsicFunction = IntrinsicFunction {
    identifier: "target_symbol",
    min_arity: 1,
    max_arity: Some(1),
    interpret_call: |context, _, span, arguments| {
        let reference = arguments[0];

        // FIXME: Change how constant strings are handled, defer to actual target methods later on.
        //        This is a hack to get around not having access to target here. Instead, this
        //        intrinsic should produce something like a Value::GlobalTargetSymbol, and then it
        //        would be converted into a Value::Str later by a constant folder which has access
        //        to the target.
        let symbol = match reference.get(&context.values) {
            Value::GlobalReference(reference) if reference.kind.is_user_value() => {
                format!("G_{{{}}}", crate::desmos::symbol::to_subscript(&reference.identifier))
            }
            Value::ActionReference(reference) => {
                format!("A_{{{}}}", crate::desmos::symbol::to_subscript(&reference.identifier))
            }
            _ => return Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ExpectedGlobalOrActionReference,
                span: reference.get_span(&context.values),
            }))
        };

        Ok(ValueEntry {
            value: Value::Str(symbol.into()),
            type_handle: TypeHandle::STR,
            span,
        })
    },
};
