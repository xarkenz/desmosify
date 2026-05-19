use crate::ast::{DefinitionKind, RangeKind, TypeDefinition};
use crate::sema::context::GlobalContext;
use crate::sema::types::Type;
use crate::sema::values::{MathematicalConstant, Value, ValueKind};

#[derive(Clone, Debug)]
pub struct IntrinsicFunction {
    identifier: &'static str,
    min_arity: usize,
    max_arity: Option<usize>,
    interpret_call: fn(context: &GlobalContext, arguments: Box<[Value]>) -> crate::Result<ValueKind>,
}

impl IntrinsicFunction {
    pub fn interpret_call(&self, context: &GlobalContext, arguments: Box<[Value]>) -> crate::Result<ValueKind> {
        self.check_arity(arguments.len())?;

        (self.interpret_call)(context, arguments)
    }

    pub fn check_arity(&self, argument_count: usize) -> crate::Result<()> {
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
                    span: None,
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
                span: None,
            }))
        }
    }
}

impl PartialEq for IntrinsicFunction {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum IntrinsicUnaryKind {
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
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum IntrinsicBinaryKind {
    Dot,
    Cross,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum IntrinsicReducerKind {
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
pub enum IntrinsicDoubleReducerKind {
    Cov,
    Covp,
    Corr,
    Spearman,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum IntrinsicParameterizedReducerKind {
    Quartile,
    Quantile,
    Tscore,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum IntrinsicColorKind {
    Rgb,
    Hsv,
    Okhsv,
    Oklab,
    Oklch,
}

#[derive(Clone, PartialEq, Debug)]
pub enum IntrinsicValue {
    Unary {
        kind: IntrinsicUnaryKind,
        argument: Value,
        result_type: Type,
    },
    Binary {
        kind: IntrinsicBinaryKind,
        lhs: Value,
        rhs: Value,
        result_type: Type,
    },
    Reducer {
        kind: IntrinsicReducerKind,
        list: Value,
        result_type: Type,
    },
    ArgumentsReducer {
        kind: IntrinsicReducerKind,
        arguments: Box<[Value]>,
        result_type: Type,
    },
    DoubleReducer {
        kind: IntrinsicDoubleReducerKind,
        list_1: Value,
        list_2: Value,
        result_type: Type,
    },
    ParameterizedReducer {
        kind: IntrinsicParameterizedReducerKind,
        list: Value,
        parameter: Value,
        result_type: Type,
    },
    Color {
        kind: IntrinsicColorKind,
        value_1: Value,
        value_2: Value,
        value_3: Value,
        is_list: bool,
    },
    Segment {
        point_1: Value,
        point_2: Value,
        is_list: bool,
    },
    Rotate {
        object: Value,
        point: Value,
        angle: Value,
        result_type: Type,
    },
    Join {
        arguments: Box<[Value]>,
        result_type: Type,
    },
    Width,
    Height,
    Dt,
    Index,
}

impl IntrinsicValue {
    pub fn get_type(&self) -> Type {
        match self {
            Self::Unary { result_type, .. } => result_type.clone(),
            Self::Binary { result_type, .. } => result_type.clone(),
            Self::Reducer { result_type, .. } => result_type.clone(),
            Self::ArgumentsReducer { result_type, .. } => result_type.clone(),
            Self::DoubleReducer { result_type, .. } => result_type.clone(),
            Self::ParameterizedReducer { result_type, .. } => result_type.clone(),
            Self::Color { is_list, .. } => Type::Color.unflatten_list(*is_list),
            Self::Segment { is_list, .. } => Type::Segment.unflatten_list(*is_list),
            Self::Rotate { result_type, .. } => result_type.clone(),
            Self::Join { result_type, .. } => result_type.clone(),
            Self::Width |
            Self::Height |
            Self::Dt => Type::Real,
            Self::Index => Type::Int,
        }
    }
}

impl From<IntrinsicValue> for ValueKind {
    fn from(value: IntrinsicValue) -> Self {
        Self::Intrinsic(Box::new(value))
    }
}

// TODO: per target
pub fn get_core_intrinsics() -> impl Iterator<Item = (&'static str, ValueKind)> {
    CORE_INTRINSIC_FUNCTIONS
        .iter()
        .map(|&function| {
            (function.identifier, ValueKind::IntrinsicFunction(function))
        })
        .chain([
            ("PI", ValueKind::Mathematical {
                kind: MathematicalConstant::Pi,
                coefficient: 1.0,
            }),
            ("TAU", ValueKind::Mathematical {
                kind: MathematicalConstant::Tau,
                coefficient: 1.0,
            }),
            ("E", ValueKind::Mathematical {
                kind: MathematicalConstant::E,
                coefficient: 1.0,
            }),
            ("width_pixels", IntrinsicValue::Width.into()),
            ("height_pixels", IntrinsicValue::Height.into()),
        ])
}

pub const CORE_INTRINSIC_FUNCTIONS: &[&IntrinsicFunction] = &[
    // Trigonometric
    &SIN,
    &COS,
    &TAN,
    // &CSC,
    // &SEC,
    // &COT,
    // &ARCSIN,
    // &ARCCOS,
    // &ARCTAN,
    // &ARCCSC,
    // &ARCSEC,
    // &ARCCOT,
    // &SINH,
    // &COSH,
    // &TANH,
    // &CSCH,
    // &SECH,
    // &COTH,
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
    // List Operations
    &JOIN,
    // &SORT,
    // &SHUFFLE,
    // &UNIQUE,
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
    // Calculus
    // &EXP,
    // &LN,
    // &LOG,
    // &DERIVATIVE,
    // &INTEGRAL,
    // &SUM,
    // &PRODUCT,
    // Geometry
    // &MIDPOINT,
    // &INTERSECTION,
    &SEGMENT,
    // &LINE,
    // &RAY,
    // &VECTOR,
    // &PARALLEL,
    // &PERPENDICULAR,
    // &CIRCLE,
    // &ARC,
    // &ANGLE,
    // &DIRECTED_ANGLE,
    &POLYGON,
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
    // &START,
    // &END,
    // Transformations
    // &DILATE,
    &ROTATE,
    // &REFLECT,
    // &TRANSLATE,
    // Color
    &RGB,
    &HSV,
    &OKHSV,
    &OKLAB,
    &OKLCH,
    // Sound
    // &TONE,
    // Number Theory
    // &LCM,
    // &GCD,
    // &CEIL,
    // &FLOOR,
    // &ROUND,
    // &SIGN,
    // &SQRT,
    // &CBRT,
    // &NTHROOT,
    // &NPR,
    // &NCR,
    // &FACTORIAL,
    // Complex
    // &REAL,
    // &IMAG,
    // &CONJ,
    // &ARG,
    // Desmosify
    &ENUM_VARIANTS,
];

pub fn interpret_trig_call(
    kind: IntrinsicUnaryKind,
    arguments: Box<[Value]>,
) -> crate::Result<ValueKind> {
    let argument = arguments.into_iter().next().unwrap();
    let is_list = argument.get_type().is_list();

    Ok(IntrinsicValue::Unary {
        kind,
        argument: argument.coerce_to(&Type::Real, true)?,
        result_type: Type::Real.unflatten_list(is_list),
    }.into())
}

macro_rules! trig_intrinsic {
    ($id:expr => $kind:ident) => {
        IntrinsicFunction {
            identifier: $id,
            min_arity: 1,
            max_arity: Some(1),
            interpret_call: |_, arguments| {
                interpret_trig_call(IntrinsicUnaryKind::$kind, arguments)
            },
        }
    };
}

pub fn interpret_reducer_call(
    kind: IntrinsicReducerKind,
    argument_check: Option<fn(&Type) -> crate::Result<()>>,
    result_override: Option<Type>,
    arguments: Box<[Value]>,
) -> crate::Result<ValueKind> {
    if let Some(argument_check) = argument_check {
        for argument in &arguments {
            argument_check(argument.get_type().flatten_list().1)?;
        }
    }

    if let [argument] = arguments.as_ref() {
        let (is_list, argument_type) = argument.get_type().into_flatten_list();
        let result_type = result_override.unwrap_or(argument_type);

        if is_list {
            Ok(IntrinsicValue::Reducer {
                kind,
                list: arguments.into_iter().next().unwrap(),
                result_type,
            }.into())
        }
        else {
            Ok(IntrinsicValue::ArgumentsReducer {
                kind,
                arguments,
                result_type,
            }.into())
        }
    }
    else {
        let result_type = Type::broadcast(result_override, arguments
            .iter()
            .map(|value| (value.get_type(), value.span)))?;

        Ok(IntrinsicValue::ArgumentsReducer {
            kind,
            arguments,
            result_type,
        }.into())
    }
}

macro_rules! reducer_intrinsic {
    ($id:expr, $chk:expr => $kind:ident, $res:expr) => {
        IntrinsicFunction {
            identifier: $id,
            min_arity: 1,
            max_arity: None,
            interpret_call: |_, arguments| {
                interpret_reducer_call(IntrinsicReducerKind::$kind, $chk, $res, arguments)
            },
        }
    };
}

pub fn interpret_color_call(
    kind: IntrinsicColorKind,
    arguments: Box<[Value]>,
) -> crate::Result<ValueKind> {
    let mut arguments = arguments.into_iter();
    let value_1 = arguments.next().unwrap()
        .coerce_to(&Type::Real, true)?;
    let value_2 = arguments.next().unwrap()
        .coerce_to(&Type::Real, true)?;
    let value_3 = arguments.next().unwrap()
        .coerce_to(&Type::Real, true)?;
    let is_list = value_1.get_type().is_list() ||
        value_2.get_type().is_list() ||
        value_3.get_type().is_list();

    Ok(IntrinsicValue::Color {
        kind,
        value_1,
        value_2,
        value_3,
        is_list,
    }.into())
}

macro_rules! color_intrinsic {
    ($id:expr => $kind:ident) => {
        IntrinsicFunction {
            identifier: $id,
            min_arity: 3,
            max_arity: Some(3),
            interpret_call: |_, arguments| {
                interpret_color_call(IntrinsicColorKind::$kind, arguments)
            },
        }
    };
}

// ------ Trigonometric ------

pub static SIN: IntrinsicFunction = trig_intrinsic!("sin" => Sin);
pub static COS: IntrinsicFunction = trig_intrinsic!("cos" => Cos);
pub static TAN: IntrinsicFunction = trig_intrinsic!("tan" => Tan);
// CSC
// SEC
// COT
// ARCSIN
// ARCCOS
// ARCTAN
// ARCCSC
// ARCSEC
// ARCCOT
// SINH
// COSH
// TANH
// CSCH
// SECH
// COTH

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

// ------ List Operations ------

pub static JOIN: IntrinsicFunction = IntrinsicFunction {
    identifier: "join",
    min_arity: 2,
    max_arity: None,
    interpret_call: |_, arguments| {
        let item_type = arguments[1..].iter().try_fold(
            arguments[0].get_type().into_flatten_list().1,
            |current_type, argument| {
                current_type.merge(argument.get_type().flatten_list().1, argument.span)
            },
        )?;

        Ok(IntrinsicValue::Join {
            arguments,
            result_type: item_type.into_list(),
        }.into())
    },
};
// SORT
// SHUFFLE
// UNIQUE

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

// ------ Calculus ------

// EXP
// LN
// LOG
// DERIVATIVE
// INTEGRAL
// SUM
// PRODUCT

// ------ Geometry ------

// MIDPOINT
// INTERSECTION
pub static SEGMENT: IntrinsicFunction = IntrinsicFunction {
    identifier: "segment",
    min_arity: 2,
    max_arity: Some(2),
    interpret_call: |_, arguments| {
        // TODO: check types
        let mut arguments = arguments.into_iter();
        let point_1 = arguments.next().unwrap();
        let point_2 = arguments.next().unwrap();
        let is_list = point_1.get_type().is_list() ||
            point_2.get_type().is_list();

        Ok(IntrinsicValue::Segment {
            point_1,
            point_2,
            is_list,
        }.into())
    },
};
// LINE
// RAY
// VECTOR
// PARALLEL
// PERPENDICULAR
// CIRCLE
// ARC
// ANGLE
// DIRECTED_ANGLE
pub static POLYGON: IntrinsicFunction = reducer_intrinsic!(
    "polygon", Some(Type::require_numeric_point_2d) => Polygon, Some(Type::Polygon)
);
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
// START
// END

// ------ Transformations ------

// DILATE
pub static ROTATE: IntrinsicFunction = IntrinsicFunction {
    identifier: "rotate",
    min_arity: 3,
    max_arity: Some(3),
    interpret_call: |_, arguments| {
        // TODO: check types better
        let mut arguments = arguments.into_iter();
        let object = arguments.next().unwrap();
        let point = arguments.next().unwrap()
            .coerce_to(&Type::Point2 {
                x_type: Box::new(Type::Real),
                y_type: Box::new(Type::Real),
            }, true)?;
        let angle = arguments.next().unwrap()
            .coerce_to(&Type::Real, true)?;
        let result_type = object.get_type();
        let is_list = object.get_type().is_list() ||
            point.get_type().is_list() ||
            angle.get_type().is_list();

        Ok(IntrinsicValue::Rotate {
            object,
            point,
            angle,
            result_type: result_type.unflatten_list(is_list),
        }.into())
    },
};
// REFLECT
// TRANSLATE

// ------ Color ------

pub static RGB: IntrinsicFunction = color_intrinsic!("rgb" => Rgb);
pub static HSV: IntrinsicFunction = color_intrinsic!("hsv" => Hsv);
pub static OKHSV: IntrinsicFunction = color_intrinsic!("okhsv" => Okhsv);
pub static OKLAB: IntrinsicFunction = color_intrinsic!("oklab" => Oklab);
pub static OKLCH: IntrinsicFunction = color_intrinsic!("oklch" => Oklch);

// ------ Sound ------

// TONE

// ------ Number Theory ------

// LCM
// GCD
// CEIL
// FLOOR
// ROUND
// SIGN
// SQRT
// CBRT
// NTHROOT
// NPR
// NCR
// FACTORIAL

// ------ Complex ------

// REAL
// IMAG
// CONJ
// ARG

// ------ Desmosify ------

pub static ENUM_VARIANTS: IntrinsicFunction = IntrinsicFunction {
    identifier: "enum_variants",
    min_arity: 1,
    max_arity: Some(1),
    interpret_call: |context, arguments| {
        let Type::Meta { identifier } = arguments.into_iter().next().unwrap().get_type() else {
            return Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ExpectedTypeArgument,
                span: None,
            }));
        };

        let definition = context.find_definition(&identifier).unwrap();
        let DefinitionKind::Type(TypeDefinition::Enumeration { variants }) = &definition.definition.kind else {
            return Err(Box::new(crate::Error {
                kind: crate::ErrorKind::ExpectedEnumType,
                span: None,
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
