use super::*;

pub const GLOBAL_INTRINSICS: &[&Intrinsic] = &[
];

// Trigonometric

pub static SIN: Intrinsic = Intrinsic {
    identifier: "sin",
    kind: IntrinsicKind::Function(IntrinsicFunction {
        signature_test: |arguments| simple_signature(arguments, FunctionSignature {
            parameter_types: Box::new([DataType::Real]),
            return_type: DataType::Real,
        }),
    }),
};
pub static COS: Intrinsic = Intrinsic {
    identifier: "cos",
    kind: IntrinsicKind::Function(IntrinsicFunction {
        signature_test: |arguments| simple_signature(arguments, FunctionSignature {
            parameter_types: Box::new([DataType::Real]),
            return_type: DataType::Real,
        }),
    }),
};
pub static TAN: Intrinsic = Intrinsic {
    identifier: "tan",
    kind: IntrinsicKind::Function(IntrinsicFunction {
        signature_test: |arguments| simple_signature(arguments, FunctionSignature {
            parameter_types: Box::new([DataType::Real]),
            return_type: DataType::Real,
        }),
    }),
};
pub static CSC: Intrinsic = Intrinsic {
    identifier: "csc",
    kind: IntrinsicKind::Function(IntrinsicFunction {
        signature_test: |arguments| simple_signature(arguments, FunctionSignature {
            parameter_types: Box::new([DataType::Real]),
            return_type: DataType::Real,
        }),
    }),
};
pub static SEC: Intrinsic = Intrinsic {
    identifier: "sec",
    kind: IntrinsicKind::Function(IntrinsicFunction {
        signature_test: |arguments| simple_signature(arguments, FunctionSignature {
            parameter_types: Box::new([DataType::Real]),
            return_type: DataType::Real,
        }),
    }),
};
pub static COT: Intrinsic = Intrinsic {
    identifier: "cot",
    kind: IntrinsicKind::Function(IntrinsicFunction {
        signature_test: |arguments| simple_signature(arguments, FunctionSignature {
            parameter_types: Box::new([DataType::Real]),
            return_type: DataType::Real,
        }),
    }),
};

// Inverse Trigonometric

// ARCSIN
// ARCCOS
// ARCTAN
// ARCCSC
// ARCSEC
// ARCCOT

// Statistics

// MEAN
// MEDIAN
pub static MIN: Intrinsic = Intrinsic {
    identifier: "min",
    kind: IntrinsicKind::Function(IntrinsicFunction {
        signature_test: |arguments| simple_signature(arguments, FunctionSignature {
            parameter_types: Box::new([DataType::Real]),
            return_type: DataType::Real,
        }),
    }),
};
pub static MAX: Intrinsic = Intrinsic {
    identifier: "max",
    kind: IntrinsicKind::Function(IntrinsicFunction {
        signature_test: |arguments| simple_signature(arguments, FunctionSignature {
            parameter_types: Box::new([DataType::Real]),
            return_type: DataType::Real,
        }),
    }),
};
// QUARTILE
// QUANTILE
// STDEV
// STDEVP
// VAR
// MAD
// COV
// COVP
// CORR
// SPEARMAN
// STATS
pub static COUNT: Intrinsic = Intrinsic {
    identifier: "count",
    kind: IntrinsicKind::Function(IntrinsicFunction {
        signature_test: |arguments| simple_signature(arguments, FunctionSignature {
            parameter_types: Box::new([DataType::Real]),
            return_type: DataType::Real,
        }),
    }),
};
pub static TOTAL: Intrinsic = Intrinsic {
    identifier: "total",
    kind: IntrinsicKind::Function(IntrinsicFunction {
        signature_test: |arguments| simple_signature(arguments, FunctionSignature {
            parameter_types: Box::new([DataType::Real]),
            return_type: DataType::Real,
        }),
    }),
};

// List Operations

pub static JOIN: Intrinsic = Intrinsic {
    identifier: "join",
    kind: IntrinsicKind::Function(IntrinsicFunction {
        signature_test: |arguments| simple_signature(arguments, FunctionSignature {
            parameter_types: Box::new([DataType::Real]),
            return_type: DataType::Real,
        }),
    }),
};
pub static SORT: Intrinsic = Intrinsic {
    identifier: "sort",
    kind: IntrinsicKind::Function(IntrinsicFunction {
        signature_test: |arguments| simple_signature(arguments, FunctionSignature {
            parameter_types: Box::new([DataType::Real]),
            return_type: DataType::Real,
        }),
    }),
};
pub static SHUFFLE: Intrinsic = Intrinsic {
    identifier: "shuffle",
    kind: IntrinsicKind::Function(IntrinsicFunction {
        signature_test: |arguments| simple_signature(arguments, FunctionSignature {
            parameter_types: Box::new([DataType::Real]),
            return_type: DataType::Real,
        }),
    }),
};
pub static UNIQUE: Intrinsic = Intrinsic {
    identifier: "unique",
    kind: IntrinsicKind::Function(IntrinsicFunction {
        signature_test: |arguments| simple_signature(arguments, FunctionSignature {
            parameter_types: Box::new([DataType::Real]),
            return_type: DataType::Real,
        }),
    }),
};

// Visualizations

// HISTOGRAM
// DOT_PLOT
// BOX_PLOT

// Distributions

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

// Statistical Tests

// TTEST
// TSCORE
// ITTEST

// Calculus

// EXP
// LN
// LOG
// DERIVATIVE
// INTEGRAL
// SUM
// PRODUCT

// Hyperbolic Trigonometric

// SINH
// COSH
// TANH
// CSCH
// SECH
// COTH

// Geometry

// MIDPOINT
// INTERSECTION
// SEGMENT
// LINE
// RAY
// VECTOR
// PARALLEL
// PERPENDICULAR
// CIRCLE
// ARC
// ANGLE
// DIRECTED_ANGLE
// POLYGON
// GLIDER

// Properties & Measurements

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

// Transformations

// DILATE
// ROTATE
// REFLECT
// TRANSLATE

// Color

// RGB
// HSV
// OKHSV
// OKLAB
// OKLCH

// Sound

// TONE

// Number Theory

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

// Complex

// REAL
// IMAG
// CONJ
// ARG

// Constant

pub static PI: Intrinsic = Intrinsic {
    identifier: "PI",
    kind: IntrinsicKind::Value(IntrinsicValue {
        value_type: || DataType::Real,
    }),
};
pub static TAU: Intrinsic = Intrinsic {
    identifier: "TAU",
    kind: IntrinsicKind::Value(IntrinsicValue {
        value_type: || DataType::Real,
    }),
};
pub static E: Intrinsic = Intrinsic {
    identifier: "E",
    kind: IntrinsicKind::Value(IntrinsicValue {
        value_type: || DataType::Real,
    }),
};

// Advanced

pub static WIDTH_PIXELS: Intrinsic = Intrinsic {
    identifier: "width_pixels",
    kind: IntrinsicKind::Value(IntrinsicValue {
        value_type: || DataType::Real,
    }),
};
pub static HEIGHT_PIXELS: Intrinsic = Intrinsic {
    identifier: "height_pixels",
    kind: IntrinsicKind::Value(IntrinsicValue {
        value_type: || DataType::Real,
    }),
};
pub static DT: Intrinsic = Intrinsic {
    identifier: "dt",
    kind: IntrinsicKind::Value(IntrinsicValue {
        value_type: || DataType::Real,
    }),
};
pub static INDEX: Intrinsic = Intrinsic {
    identifier: "index",
    kind: IntrinsicKind::Value(IntrinsicValue {
        value_type: || DataType::Int,
    }),
};

// Desmosify

pub static ENUM_VARIANTS: Intrinsic = Intrinsic {
    identifier: "enum_variants",
    kind: IntrinsicKind::Function(IntrinsicFunction {
        signature_test: |arguments| {
            if arguments.len() != 1 {
                return None;
            }
            let DataType::UserType { identifier } = &arguments[0] else {
                return None;
            };
            Some(DataType::List {
                item_type: Box::new(DataType::UserValue {
                    type_identifier: identifier.clone(),
                }),
            })
        },
    }),
};
