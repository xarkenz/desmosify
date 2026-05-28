use std::path::PathBuf;
use crate::ast::{DefinitionKind, RangeKind, TypeDefinition};
use crate::sema::context::{GlobalContext, LocalContext};
use crate::sema::display::ImageValue;
use crate::sema::types::{ListState, Type};
use crate::sema::values::{BinaryKind, ColorKind, MathematicalConstant, ReducerKind, UnaryKind, Value, ValueKind};

#[derive(Clone)]
pub struct IntrinsicFunction {
    pub identifier: &'static str,
    pub min_arity: usize,
    pub max_arity: Option<usize>,
    pub interpret_call: fn(
        context: &GlobalContext,
        local_context: &LocalContext,
        arguments: Box<[Value]>,
    ) -> crate::Result<ValueKind>,
}

impl IntrinsicFunction {
    pub fn interpret_call(
        &self,
        context: &GlobalContext,
        local_context: &LocalContext,
        span: Option<crate::Span>,
        arguments: Box<[Value]>,
    ) -> crate::Result<ValueKind> {
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

// TODO: per target
pub fn get_core_intrinsics() -> impl Iterator<Item = (&'static str, ValueKind)> {
    CORE_INTRINSIC_FUNCTIONS
        .iter()
        .map(|&function| {
            (function.identifier, ValueKind::IntrinsicFunction(function))
        })
        .chain([
            ("PI", ValueKind::Mathematical(MathematicalConstant::Pi)),
            ("TAU", ValueKind::Mathematical(MathematicalConstant::Tau)),
            ("E", ValueKind::Mathematical(MathematicalConstant::E)),
            ("width_pixels", ValueKind::ViewportWidth),
            ("height_pixels", ValueKind::ViewportHeight),
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
    &SORT,
    &SHUFFLE,
    &UNIQUE,
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
    &ENUM_VALUES,
    &ENUM_VALUE,
    &INCLUDE_TEXT,
    &INCLUDE_DATA,
    &IMAGE,
];

pub fn interpret_trig_call(
    kind: UnaryKind,
    arguments: Box<[Value]>,
) -> crate::Result<ValueKind> {
    let argument = arguments.into_iter().next().unwrap()
        .coerce_to(&Type::Real, true)?;
    let is_list = argument.get_type().list_state();

    Ok(ValueKind::Unary {
        kind,
        operand: Box::new(argument),
        result_type: Type::Real.unflatten_list(is_list),
    })
}

macro_rules! trig_intrinsic {
    ($id:expr => $kind:ident) => {
        IntrinsicFunction {
            identifier: $id,
            min_arity: 1,
            max_arity: Some(1),
            interpret_call: |_, _, arguments| {
                interpret_trig_call(UnaryKind::$kind, arguments)
            },
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
    ($id:expr, $chk:expr => $kind:ident, $res:expr) => {
        IntrinsicFunction {
            identifier: $id,
            min_arity: 1,
            max_arity: None,
            interpret_call: |_, _, arguments| {
                interpret_reducer_call(ReducerKind::$kind, $chk, $res, arguments)
            },
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
            interpret_call: |_, _, arguments| {
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
                type_name: Type::Str.to_string(),
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
    interpret_call: |_, _, arguments| {
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
    interpret_call: |_, _, arguments| {
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
    interpret_call: |_, _, arguments| {
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
    interpret_call: |_, _, arguments| {
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
    interpret_call: |_, _, arguments| {
        // TODO: check types
        let mut arguments = arguments.into_iter();
        let point_1 = arguments.next().unwrap();
        let point_2 = arguments.next().unwrap();
        let list_state = ListState::merge(
            point_1.get_type().list_state(),
            point_2.get_type().list_state(),
        );

        Ok(ValueKind::Binary {
            kind: BinaryKind::Segment,
            lhs: Box::new(point_1),
            rhs: Box::new(point_2),
            result_type: Type::Segment.unflatten_list(list_state),
        })
    },
};
// SEGMENT3D
// LINE
// RAY
// VECTOR
// VECTOR3D
// PARALLEL
// PERPENDICULAR
// CIRCLE
// SPHERE3D
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
// START
// END

// ------ Transformations ------

// DILATE
pub static ROTATE: IntrinsicFunction = IntrinsicFunction {
    identifier: "rotate",
    min_arity: 3,
    max_arity: Some(3),
    interpret_call: |_, _, arguments| {
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
        let result_type = object.get_type().into_flatten_list().1;
        let list_state = ListState::merge(
            ListState::merge(
                object.get_type().list_state(),
                point.get_type().list_state(),
            ),
            angle.get_type().list_state(),
        );

        Ok(ValueKind::Rotate {
            object: Box::new(object),
            point: Box::new(point),
            angle: Box::new(angle),
            result_type: result_type.unflatten_list(list_state),
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

pub static ENUM_VALUES: IntrinsicFunction = IntrinsicFunction {
    identifier: "enum_values",
    min_arity: 1,
    max_arity: Some(1),
    interpret_call: |context, _, arguments| {
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
    interpret_call: |context, _, arguments| {
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
    interpret_call: |_, local_context, arguments| {
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
    interpret_call: |_, local_context, arguments| {
        let mut arguments = arguments.into_iter();

        let path_value = arguments.next().unwrap();
        let media_type = arguments.next()
            .map(|media_type_value| media_type_value.kind
                .as_const_str()
                .ok_or_else(|| Box::new(crate::Error {
                    kind: crate::ErrorKind::ExpectedConstant {
                        type_name: Type::Str.to_string(),
                    },
                    span: media_type_value.span,
                })))
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
    interpret_call: |_, _, arguments| {
        let mut arguments = arguments.into_iter();

        let url_value = arguments.next().unwrap();
        let url = url_value.kind
            .as_const_str()
            .ok_or_else(|| Box::new(crate::Error {
                kind: crate::ErrorKind::ExpectedConstant {
                    type_name: Type::Str.to_string(),
                },
                span: url_value.span,
            }))?;
        let name_value = arguments.next().unwrap();
        let name = name_value.kind
            .as_const_str()
            .ok_or_else(|| Box::new(crate::Error {
                kind: crate::ErrorKind::ExpectedConstant {
                    type_name: Type::Str.to_string(),
                },
                span: name_value.span,
            }))?;
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
                            type_name: Type::Bool.to_string(),
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
