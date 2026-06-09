use std::path::PathBuf;
use crate::ast::{DefinitionKind, RangeKind, TypeDefinition};
use crate::sema::context::{GlobalContext, LocalContext};
use crate::sema::display::ImageValue;
use crate::sema::types::{ListState, Type};
use crate::sema::values::{BinaryKind, ColorKind, InequalityKind, MathematicalConstant, ReducerKind, UnaryKind, Value, ValueKind};
use crate::target::Target;

pub fn get_core_intrinsics(target: &dyn Target) -> impl Iterator<Item = (&'static str, ValueKind)> {
    CORE_INTRINSIC_FUNCTIONS
        .iter()
        .map(|&function| {
            (function.identifier, ValueKind::IntrinsicFunction(function))
        })
        .chain([
            ("pi", ValueKind::Mathematical(MathematicalConstant::Pi)),
            ("tau", ValueKind::Mathematical(MathematicalConstant::Tau)),
            ("e", ValueKind::Mathematical(MathematicalConstant::E)),
            ("width_pixels", ValueKind::ViewportWidth),
            ("height_pixels", ValueKind::ViewportHeight),
            ("target", ValueKind::Str(target.name().into())),
        ])
}

#[derive(Clone)]
pub struct IntrinsicFunction {
    pub identifier: &'static str,
    pub min_arity: usize,
    pub max_arity: Option<usize>,
    pub interpret_call: fn(
        target: &mut dyn Target,
        context: &GlobalContext,
        local_context: &LocalContext,
        arguments: Box<[Value]>,
    ) -> crate::Result<ValueKind>,
}

impl IntrinsicFunction {
    pub fn interpret_call(
        &self,
        target: &mut dyn Target,
        context: &GlobalContext,
        local_context: &LocalContext,
        span: Option<crate::Span>,
        arguments: Box<[Value]>,
    ) -> crate::Result<ValueKind> {
        self.check_arity(arguments.len(), span)?;

        (self.interpret_call)(target, context, local_context, arguments)
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
    // Statistics
    &MEAN,
    // &MEDIAN,
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
    // &RANDOM,
    // Statistical Tests
    // &TTEST,
    // &TSCORE,
    // &ITTEST,
    // Geometry
    &MIDPOINT,
    // &INTERSECTION,
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
    // &ARC,
    // &ANGLE,
    // &DIRECTED_ANGLE,
    &POLYGON,
    // &TRIANGLE3D,
    // &GLIDER,
    // Properties & Measurements
    // &DOT,
    // &CROSS,
    // &DISTANCE,
    // &LENGTH,
    // &AREA,
    // &PERIMETER,
    // &VERTICES,
    // &ANGLES,
    // &DIRECTED_ANGLES,
    // &SEGMENTS,
    // &RADIUS,
    // &CENTER,
    // &COTERMINAL,
    // &SUPPLEMENT,
    &START,
    &END,
    // Transformations
    &DILATE,
    &ROTATE,
    &REFLECT,
    &TRANSLATE,
    // Color
    &RGB,
    &HSV,
    &OKHSV,
    &OKLAB,
    &OKLCH,
    // Sound
    // &TONE,
    // Desmosify
    &ENUM_VALUES,
    &ENUM_VALUE,
    &INCLUDE_TEXT,
    &INCLUDE_DATA,
    &IMAGE,
    &CONCAT,
    &TARGET_SYMBOL,
];

pub fn interpret_unary_call(
    kind: UnaryKind,
    argument_type: &Type,
    result_type: Type,
    arguments: Box<[Value]>,
) -> crate::Result<ValueKind> {
    let argument = arguments.into_iter().next().unwrap()
        .coerce_to(argument_type, true)?;

    let list_state = argument.get_type().list_state();

    Ok(ValueKind::Unary {
        kind,
        operand: Box::new(argument),
        result_type: result_type.unflatten_list(list_state),
    })
}

macro_rules! unary_intrinsic {
    ($identifier:literal, ($argument_type:expr) => $kind:ident, $result_type:expr) => {
        IntrinsicFunction {
            identifier: $identifier,
            min_arity: 1,
            max_arity: Some(1),
            interpret_call: |_, _, _, arguments| interpret_unary_call(
                UnaryKind::$kind,
                &($argument_type),
                $result_type,
                arguments,
            ),
        }
    };
}

pub fn interpret_binary_call(
    kind: BinaryKind,
    lhs_type: &Type,
    rhs_type: &Type,
    result_type: Type,
    arguments: Box<[Value]>,
) -> crate::Result<ValueKind> {
    let mut arguments = arguments.into_iter();

    let lhs = arguments.next().unwrap()
        .coerce_to(lhs_type, true)?;
    let rhs = arguments.next().unwrap()
        .coerce_to(rhs_type, true)?;

    let list_state = ListState::merge(
        lhs.get_type().list_state(),
        rhs.get_type().list_state(),
    );

    Ok(ValueKind::Binary {
        kind,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
        result_type: result_type.unflatten_list(list_state),
    })
}

macro_rules! binary_intrinsic {
    ($identifier:literal, ($lhs_type:expr, $rhs_type:expr) => $kind:ident, $result_type:expr) => {
        IntrinsicFunction {
            identifier: $identifier,
            min_arity: 2,
            max_arity: Some(2),
            interpret_call: |_, _, _, arguments| interpret_binary_call(
                BinaryKind::$kind,
                &($lhs_type),
                &($rhs_type),
                $result_type,
                arguments,
            ),
        }
    };
}

pub fn interpret_reducer_call(
    kind: ReducerKind,
    argument_check: Option<fn(&Type) -> crate::Result<()>>,
    result_override: Option<Type>,
    arguments: Box<[Value]>,
) -> crate::Result<ValueKind> {
    if let Some(argument_check) = argument_check {
        for argument in &arguments {
            argument_check(argument.get_type().flatten_list().1)
                .map_err(|error| error.with_span(argument.span))?;
        }
    }

    if let [argument] = arguments.as_ref() {
        let (list_state, argument_type) = argument.get_type().into_flatten_list();
        let result_type = result_override.unwrap_or(argument_type);

        if list_state.is_some() {
            // This should also work for any MaybeList
            Ok(ValueKind::Reducer {
                kind,
                list: Box::new(arguments.into_iter().next().unwrap()),
                result_type,
            })
        }
        else {
            Ok(ValueKind::ArgumentsReducer {
                kind,
                arguments,
                result_type,
            })
        }
    }
    else {
        let result_type = Type::broadcast(result_override, arguments
            .iter()
            .map(|value| (value.get_type(), value.span)))?;

        Ok(ValueKind::ArgumentsReducer {
            kind,
            arguments,
            result_type,
        })
    }
}

macro_rules! reducer_intrinsic {
    ($identifier:literal, $argument_check:expr => $kind:ident, $result_override:expr) => {
        IntrinsicFunction {
            identifier: $identifier,
            min_arity: 1,
            max_arity: None,
            interpret_call: |_, _, _, arguments| interpret_reducer_call(
                ReducerKind::$kind,
                $argument_check,
                $result_override,
                arguments,
            ),
        }
    };
}

pub fn interpret_color_call(
    kind: ColorKind,
    arguments: Box<[Value]>,
) -> crate::Result<ValueKind> {
    let mut arguments = arguments.into_iter();
    let value_1 = arguments.next().unwrap()
        .coerce_to(&Type::Real, true)?;
    let value_2 = arguments.next().unwrap()
        .coerce_to(&Type::Real, true)?;
    let value_3 = arguments.next().unwrap()
        .coerce_to(&Type::Real, true)?;
    let list_state = ListState::merge(
        ListState::merge(
            value_1.get_type().list_state(),
            value_2.get_type().list_state(),
        ),
        value_3.get_type().list_state(),
    );

    Ok(ValueKind::Color {
        kind,
        value_1: Box::new(value_1),
        value_2: Box::new(value_2),
        value_3: Box::new(value_3),
        list_state,
    })
}

macro_rules! color_intrinsic {
    ($id:expr => $kind:ident) => {
        IntrinsicFunction {
            identifier: $id,
            min_arity: 3,
            max_arity: Some(3),
            interpret_call: |_, _, _, arguments| {
                interpret_color_call(ColorKind::$kind, arguments)
            },
        }
    };
}

pub fn read_file_bytes(local_context: &LocalContext, path_value: &Value) -> crate::Result<(PathBuf, Vec<u8>)> {
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

pub static SIN: IntrinsicFunction = unary_intrinsic!(
    "sin", (Type::Real) => Sin, Type::Real
);

pub static COS: IntrinsicFunction = unary_intrinsic!(
    "cos", (Type::Real) => Cos, Type::Real
);

pub static TAN: IntrinsicFunction = unary_intrinsic!(
    "tan", (Type::Real) => Tan, Type::Real
);

pub static CSC: IntrinsicFunction = unary_intrinsic!(
    "csc", (Type::Real) => Csc, Type::Real
);

pub static SEC: IntrinsicFunction = unary_intrinsic!(
    "sec", (Type::Real) => Sec, Type::Real
);

pub static COT: IntrinsicFunction = unary_intrinsic!(
    "cot", (Type::Real) => Cot, Type::Real
);

pub static ARCSIN: IntrinsicFunction = unary_intrinsic!(
    "arcsin", (Type::Real) => Arcsin, Type::Real
);

pub static ARCCOS: IntrinsicFunction = unary_intrinsic!(
    "arccos", (Type::Real) => Arccos, Type::Real
);

pub static ARCTAN: IntrinsicFunction = unary_intrinsic!(
    "arctan", (Type::Real) => Arctan, Type::Real
);

pub static ARCCSC: IntrinsicFunction = unary_intrinsic!(
    "arccsc", (Type::Real) => Arccsc, Type::Real
);

pub static ARCSEC: IntrinsicFunction = unary_intrinsic!(
    "arcsec", (Type::Real) => Arcsec, Type::Real
);

pub static ARCCOT: IntrinsicFunction = unary_intrinsic!(
    "arccot", (Type::Real) => Arccot, Type::Real
);

pub static SINH: IntrinsicFunction = unary_intrinsic!(
    "sinh", (Type::Real) => Sinh, Type::Real
);

pub static COSH: IntrinsicFunction = unary_intrinsic!(
    "cosh", (Type::Real) => Cosh, Type::Real
);

pub static TANH: IntrinsicFunction = unary_intrinsic!(
    "tanh", (Type::Real) => Tanh, Type::Real
);

pub static CSCH: IntrinsicFunction = unary_intrinsic!(
    "csch", (Type::Real) => Csch, Type::Real
);

pub static SECH: IntrinsicFunction = unary_intrinsic!(
    "sech", (Type::Real) => Sech, Type::Real
);

pub static COTH: IntrinsicFunction = unary_intrinsic!(
    "coth", (Type::Real) => Coth, Type::Real
);

// ------ Calculus ------

pub static EXP: IntrinsicFunction = unary_intrinsic!(
    "exp", (Type::Real) => Exp, Type::Real
);

pub static LN: IntrinsicFunction = unary_intrinsic!(
    "ln", (Type::Real) => Ln, Type::Real
);

pub static LOG: IntrinsicFunction = binary_intrinsic!(
    "log", (Type::Real, Type::Real) => Log, Type::Real
);

// DERIVATIVE

// INTEGRAL

// SUM

// PRODUCT

// ------ Number Theory ------

pub static LCM: IntrinsicFunction = binary_intrinsic!(
    "lcm", (Type::Int, Type::Int) => Lcm, Type::Int
);

pub static GCD: IntrinsicFunction = binary_intrinsic!(
    "gcd", (Type::Int, Type::Int) => Gcd, Type::Int
);

pub static CEIL: IntrinsicFunction = unary_intrinsic!(
    "ceil", (Type::Real) => Ceil, Type::Int
);

pub static FLOOR: IntrinsicFunction = unary_intrinsic!(
    "floor", (Type::Real) => Floor, Type::Int
);

// FIXME: there is a version of this function with 2 arguments which is not handled
pub static ROUND: IntrinsicFunction = unary_intrinsic!(
    "round", (Type::Real) => Round, Type::Int
);

pub static ABS: IntrinsicFunction = IntrinsicFunction {
    identifier: "abs",
    min_arity: 1,
    max_arity: Some(1),
    interpret_call: |_, _, _, arguments| {
        let argument = arguments.into_iter().next().unwrap();
        let (argument_list, argument_type) = argument.get_type().into_flatten_list();

        argument_type.require_numeric()?;

        Ok(ValueKind::Unary {
            kind: UnaryKind::Abs,
            operand: Box::new(argument),
            result_type: argument_type.unflatten_list(argument_list),
        })
    },
};

pub static SIGN: IntrinsicFunction = unary_intrinsic!(
    "sign", (Type::Real) => Sign, Type::Int
);

pub static SQRT: IntrinsicFunction = unary_intrinsic!(
    "sqrt", (Type::Real) => Sqrt, Type::Real
);

pub static CBRT: IntrinsicFunction = unary_intrinsic!(
    "cbrt", (Type::Real) => Cbrt, Type::Real
);

pub static NTH_ROOT: IntrinsicFunction = binary_intrinsic!(
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

        Ok(ValueKind::Join {
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

        let mut list = arguments.next().unwrap();
        list.get_type()
            .require_flatten_list()
            .map_err(|error| error.with_span(list.span))?;

        let key_list = arguments.next()
            .map(|key_list| {
                key_list.get_type()
                    .require_flatten_list()
                    .map_err(|error| error.with_span(list.span))?;
                key_list.coerce_to(&Type::Real, true)
            })
            .transpose()?;

        // FIXME: the types here are busted
        if key_list.is_none() {
            // The values in the original list are used as keys
            list = list.coerce_to(&Type::Real, true)?;
        }

        Ok(ValueKind::Sort {
            list: Box::new(list),
            key_list: key_list.map(Box::new),
        })
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

        let seed = arguments.next()
            .map(|seed| seed.coerce_to(&Type::Real, false))
            .transpose()?;

        Ok(ValueKind::Shuffle {
            list: Box::new(list),
            seed: seed.map(Box::new),
        })
    },
};

pub static UNIQUE: IntrinsicFunction = IntrinsicFunction {
    identifier: "unique",
    min_arity: 1,
    max_arity: Some(1),
    interpret_call: |_, _, _, arguments| {
        // It seems like unique() accepts basically any type, so don't bother checking the item type
        // TODO: check if any types can go in a list but cannot be uniqued
        let list = arguments.into_iter().next().unwrap();
        list.get_type()
            .require_flatten_list()
            .map_err(|error| error.with_span(list.span))?;

        Ok(ValueKind::Unique {
            list: Box::new(list),
        })
    },
};

// ------ Statistics ------

pub static MEAN: IntrinsicFunction = reducer_intrinsic!(
    "mean", Some(Type::require_numeric_or_point) => Mean, None
);

// MEDIAN

pub static MIN: IntrinsicFunction = reducer_intrinsic!(
    "min", Some(Type::require_numeric) => Min, None
);

pub static MAX: IntrinsicFunction = reducer_intrinsic!(
    "max", Some(Type::require_numeric) => Max, None
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

pub static COUNT: IntrinsicFunction = reducer_intrinsic!(
    "count", None => Count, Some(Type::Int)
);

pub static TOTAL: IntrinsicFunction = reducer_intrinsic!(
    "total", Some(Type::require_numeric_or_point) => Total, None
);

fn get_any_all_total(arguments: Box<[Value]>, transformer: fn(Value) -> Value) -> crate::Result<ValueKind> {
    if arguments.len() == 1 {
        let argument = arguments.into_iter().next().unwrap()
            .coerce_to(&Type::Bool, true)?;
        let argument = transformer(argument);

        if argument.get_type().list_state().is_some() {
            // This should also work for any MaybeList
            Ok(ValueKind::Reducer {
                kind: ReducerKind::Total,
                list: Box::new(argument),
                result_type: Type::Int,
            })
        }
        else {
            Ok(ValueKind::ArgumentsReducer {
                kind: ReducerKind::Total,
                arguments: Box::new([argument]),
                result_type: Type::Int,
            })
        }
    }
    else {
        let arguments: Box<[_]> = arguments.into_iter()
            .map(|argument| {
                let argument = argument.coerce_to(&Type::Bool, true)?;
                let argument = transformer(argument);
                Ok(argument)
            })
            .collect::<crate::Result<_>>()?;
        let list_state = arguments
            .iter()
            .fold(None, |current_state, argument| ListState::merge(
                current_state,
                argument.get_type().list_state(),
            ));

        Ok(ValueKind::ArgumentsReducer {
            kind: ReducerKind::Total,
            arguments,
            result_type: Type::Int.unflatten_list(list_state),
        })
    }
}

// @total(arguments) > 0
pub static ANY: IntrinsicFunction = IntrinsicFunction {
    identifier: "any",
    min_arity: 1,
    max_arity: None,
    interpret_call: |_, _, _, arguments| {
        let total = get_any_all_total(arguments, std::convert::identity)?;
        let list_state = total.get_type().list_state();

        Ok(ValueKind::InequalityChain {
            lhs: Box::new(total.into()),
            chain: Box::new([(
                InequalityKind::GreaterThan,
                ValueKind::Int(0).into(),
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
    interpret_call: |_, _, _, arguments| {
        let total = get_any_all_total(arguments, |argument| {
            let list_state = argument.get_type().list_state();

            ValueKind::Unary {
                kind: UnaryKind::LogicalNot,
                operand: Box::new(argument),
                result_type: Type::Bool.unflatten_list(list_state),
            }.into()
        })?;

        let list_state = total.get_type().list_state();

        Ok(ValueKind::Binary {
            kind: BinaryKind::Equal,
            lhs: Box::new(total.into()),
            rhs: Box::new(ValueKind::Int(0).into()),
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

// RANDOM

// ------ Statistical Tests ------

// TTEST

// TSCORE

// ITTEST

// ------ Geometry ------

pub static MIDPOINT: IntrinsicFunction = IntrinsicFunction {
    identifier: "midpoint",
    min_arity: 1,
    max_arity: Some(2),
    interpret_call: |_, _, _, arguments| {
        let mut arguments = arguments.into_iter();

        let point_1_or_segment = arguments.next().unwrap();

        if let Some(point_2) = arguments.next() {
            let point_type = match point_1_or_segment.get_type().into_flatten_list().1 {
                Type::Point2 { .. } => Type::real_point2(),
                Type::Point3 { .. } => Type::real_point3(),
                got_type => return Err(Box::new(crate::Error {
                    kind: crate::ErrorKind::ExpectedNumericPointType {
                        got_type: got_type.to_string(),
                    },
                    span: point_1_or_segment.span,
                }))
            };

            let point_1 = point_1_or_segment.coerce_to(&point_type, true)?;
            let point_2 = point_2.coerce_to(&point_type, true)?;

            let list_state = ListState::merge(
                point_1.get_type().list_state(),
                point_2.get_type().list_state(),
            );

            Ok(ValueKind::Binary {
                kind: BinaryKind::MidpointOfPoints,
                lhs: Box::new(point_1),
                rhs: Box::new(point_2),
                result_type: point_type.unflatten_list(list_state),
            })
        }
        else {
            let (list_state, segment_type) = point_1_or_segment.get_type().into_flatten_list();
            let point_type = match segment_type {
                Type::Segment => Type::real_point2(),
                Type::Segment3D => Type::real_point3(),
                got_type => return Err(Box::new(crate::Error {
                    kind: crate::ErrorKind::ExpectedSegmentType {
                        got_type: got_type.to_string(),
                    },
                    span: point_1_or_segment.span,
                }))
            };

            Ok(ValueKind::Unary {
                kind: UnaryKind::MidpointOfSegment,
                operand: Box::new(point_1_or_segment),
                result_type: point_type.unflatten_list(list_state),
            })
        }
    },
};

// INTERSECTION

pub static SEGMENT: IntrinsicFunction = binary_intrinsic!(
    "segment", (Type::real_point2(), Type::real_point2()) => Segment, Type::Segment
);

pub static SEGMENT3D: IntrinsicFunction = binary_intrinsic!(
    "segment3d", (Type::real_point3(), Type::real_point3()) => Segment3D, Type::Segment3D
);

pub static LINE: IntrinsicFunction = binary_intrinsic!(
    "line", (Type::real_point2(), Type::real_point2()) => Line, Type::Line
);

pub static RAY: IntrinsicFunction = binary_intrinsic!(
    "ray", (Type::real_point2(), Type::real_point2()) => Ray, Type::Ray
);

pub static VECTOR: IntrinsicFunction = binary_intrinsic!(
    "vector", (Type::real_point2(), Type::real_point2()) => Vector, Type::Vector
);

pub static VECTOR3D: IntrinsicFunction = binary_intrinsic!(
    "vector3d", (Type::real_point3(), Type::real_point3()) => Vector3D, Type::Vector3D
);

// PARALLEL

// PERPENDICULAR

pub static CIRCLE: IntrinsicFunction = binary_intrinsic!(
    "circle", (Type::real_point2(), Type::Real) => Circle, Type::Circle
);

pub static SPHERE3D: IntrinsicFunction = binary_intrinsic!(
    "sphere3d", (Type::real_point3(), Type::Real) => Sphere3D, Type::Sphere3D
);

// ARC

// ANGLE

// DIRECTED_ANGLE

pub static POLYGON: IntrinsicFunction = reducer_intrinsic!(
    "polygon", Some(Type::require_numeric_point_2d) => Polygon, Some(Type::Polygon)
);

// TRIANGLE3D

// GLIDER

// ------ Properties & Measurements ------

// DOT

// CROSS

// DISTANCE

// LENGTH

// AREA

// PERIMETER

// VERTICES

// ANGLES

// DIRECTED_ANGLES

// SEGMENTS

// RADIUS

// CENTER

// COTERMINAL

// SUPPLEMENT

fn interpret_start_end_call(kind: UnaryKind, arguments: Box<[Value]>) -> crate::Result<ValueKind> {
    let vector = arguments.into_iter().next().unwrap();
    let (list_state, vector_type) = vector.get_type().into_flatten_list();

    let point_type = match vector_type {
        Type::Vector => Type::real_point2(),
        Type::Vector3D => Type::real_point3(),
        got_type => return Err(Box::new(crate::Error {
            kind: crate::ErrorKind::ExpectedVectorType {
                got_type: got_type.to_string(),
            },
            span: vector.span,
        }))
    };

    Ok(ValueKind::Unary {
        kind,
        operand: Box::new(vector),
        result_type: point_type.unflatten_list(list_state),
    })
}

pub static START: IntrinsicFunction = IntrinsicFunction {
    identifier: "start",
    min_arity: 1,
    max_arity: Some(1),
    interpret_call: |_, _, _, arguments| {
        interpret_start_end_call(UnaryKind::VectorStart, arguments)
    },
};

pub static END: IntrinsicFunction = IntrinsicFunction {
    identifier: "end",
    min_arity: 1,
    max_arity: Some(1),
    interpret_call: |_, _, _, arguments| {
        interpret_start_end_call(UnaryKind::VectorEnd, arguments)
    },
};

// ------ Transformations ------

pub static DILATE: IntrinsicFunction = IntrinsicFunction {
    identifier: "dilate",
    min_arity: 3,
    max_arity: Some(3),
    interpret_call: |_, _, _, arguments| {
        let mut arguments = arguments.into_iter();

        let object = arguments.next().unwrap();
        let (object_list, object_type) = object.get_type().into_flatten_list();
        object_type.require_transformable()?;

        let point = arguments.next().unwrap()
            .coerce_to(&Type::real_point2(), true)?;

        let factor = arguments.next().unwrap()
            .coerce_to(&Type::Real, true)?;

        let list_state = ListState::merge(
            ListState::merge(
                object_list,
                point.get_type().list_state(),
            ),
            factor.get_type().list_state(),
        );

        Ok(ValueKind::Dilation {
            object: Box::new(object),
            point: Box::new(point),
            factor: Box::new(factor),
            result_type: object_type.unflatten_list(list_state),
        })
    },
};

pub static ROTATE: IntrinsicFunction = IntrinsicFunction {
    identifier: "rotate",
    min_arity: 3,
    max_arity: Some(3),
    interpret_call: |_, _, _, arguments| {
        let mut arguments = arguments.into_iter();

        let object = arguments.next().unwrap();
        let (object_list, object_type) = object.get_type().into_flatten_list();
        object_type.require_transformable()?;

        let point = arguments.next().unwrap()
            .coerce_to(&Type::real_point2(), true)?;

        let angle = arguments.next().unwrap()
            .coerce_to(&Type::Real, true)?;

        let list_state = ListState::merge(
            ListState::merge(
                object_list,
                point.get_type().list_state(),
            ),
            angle.get_type().list_state(),
        );

        Ok(ValueKind::Rotation {
            object: Box::new(object),
            point: Box::new(point),
            angle: Box::new(angle),
            result_type: object_type.unflatten_list(list_state),
        })
    },
};

pub static REFLECT: IntrinsicFunction = IntrinsicFunction {
    identifier: "reflect",
    min_arity: 2,
    max_arity: Some(2),
    interpret_call: |_, _, _, arguments| {
        let mut arguments = arguments.into_iter();

        let object = arguments.next().unwrap();
        let (object_list, object_type) = object.get_type().into_flatten_list();
        object_type.require_transformable()?;

        let line = arguments.next().unwrap();
        line.get_type().into_flatten_list().1.require_line_like()?;

        let list_state = ListState::merge(
            object_list,
            line.get_type().list_state(),
        );

        Ok(ValueKind::Reflection {
            object: Box::new(object),
            line: Box::new(line),
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

        let object = arguments.next().unwrap();
        let (object_list, object_type) = object.get_type().into_flatten_list();
        object_type.require_transformable()?;

        let point_1_or_vector = arguments.next().unwrap();

        if let Some(point_2) = arguments.next() {
            let real_point2 = Type::real_point2();
            let point_1 = point_1_or_vector.coerce_to(&real_point2, true)?;
            let point_2 = point_2.coerce_to(&real_point2, true)?;

            let list_state = ListState::merge(
                object_list,
                ListState::merge(
                    point_1.get_type().list_state(),
                    point_2.get_type().list_state(),
                ),
            );

            Ok(ValueKind::TranslationByPoints {
                object: Box::new(object),
                point_1: Box::new(point_1),
                point_2: Box::new(point_2),
                result_type: object_type.unflatten_list(list_state),
            })
        }
        else {
            let vector = point_1_or_vector.coerce_to(&Type::Vector, true)?;

            let list_state = ListState::merge(
                object_list,
                vector.get_type().list_state(),
            );

            Ok(ValueKind::TranslationByVector {
                object: Box::new(object),
                vector: Box::new(vector),
                result_type: object_type.unflatten_list(list_state),
            })
        }
    },
};

// ------ Color ------

pub static RGB: IntrinsicFunction = color_intrinsic!("rgb" => Rgb);

pub static HSV: IntrinsicFunction = color_intrinsic!("hsv" => Hsv);

pub static OKHSV: IntrinsicFunction = color_intrinsic!("okhsv" => Okhsv);

pub static OKLAB: IntrinsicFunction = color_intrinsic!("oklab" => Oklab);

pub static OKLCH: IntrinsicFunction = color_intrinsic!("oklch" => Oklch);

// ------ Sound ------

// TONE

// ------ Desmosify ------

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

        let definition = context.find_definition(&identifier).unwrap();
        let DefinitionKind::Type(TypeDefinition::Enumeration { variants }) = &definition.definition.kind else {
            return Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ExpectedEnumTypeValue,
                span: enum_type.span,
            }));
        };

        Ok(ValueKind::ListRange {
            kind: RangeKind::Exclusive,
            start: Box::new(ValueKind::EnumVariant {
                type_identifier: identifier.clone(),
                variant_ordinal: 0,
            }.into()),
            end: Box::new(ValueKind::EnumVariant {
                type_identifier: identifier.clone(),
                variant_ordinal: variants.len() as i64,
            }.into()),
            step: Box::new(ValueKind::Int(1).into()),
            item_type: Type::UserValue {
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

        let definition = context.find_definition(&identifier).unwrap();
        let DefinitionKind::Type(TypeDefinition::Enumeration { .. }) = &definition.definition.kind else {
            return Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ExpectedEnumTypeValue,
                span: enum_type.span,
            }));
        };

        let variant_ordinal = arguments.next().unwrap()
            .coerce_to(&Type::Int, true)?;
        let list_state = variant_ordinal.get_type().list_state();

        let result_type = Type::UserValue {
            type_identifier: identifier,
        };

        Ok(ValueKind::Unary {
            kind: UnaryKind::AssumeType,
            operand: Box::new(variant_ordinal),
            result_type: result_type.unflatten_list(list_state),
        })
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

        Ok(ValueKind::Str(text.into()))
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

        Ok(ValueKind::Str(data_url.to_string().into()))
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
            .coerce_to(&Type::Point2 {
                x_type: Box::new(Type::Real),
                y_type: Box::new(Type::Real),
            }, true)?;
        let width = arguments.next().unwrap()
            .coerce_to(&Type::Real, true)?;
        let height = arguments.next().unwrap()
            .coerce_to(&Type::Real, true)?;
        let opacity = arguments.next()
            .unwrap_or(ValueKind::Real(1.0).into())
            .coerce_to(&Type::Real, true)?;
        let angle = arguments.next()
            .unwrap_or(ValueKind::Real(0.0).into())
            .coerce_to(&Type::Real, true)?;
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

        Ok(ValueKind::Image(
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

        Ok(ValueKind::Str(result.into()))
    },
};

pub static TARGET_SYMBOL: IntrinsicFunction = IntrinsicFunction {
    identifier: "target_symbol",
    min_arity: 1,
    max_arity: Some(1),
    interpret_call: |target, _, _, arguments| {
        let argument = arguments.into_iter().next().unwrap();

        let symbol_name = match &argument.kind {
            ValueKind::Global(reference) => {
                target.get_global_symbol_name(&reference.identifier)
            }
            ValueKind::Action(reference) => {
                target.get_action_symbol_name(&reference.identifier)
            }
            _ => return Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ExpectedGlobalOrActionReference,
                span: argument.span,
            }))
        };

        Ok(ValueKind::Str(symbol_name.into()))
    },
};
