#[macro_export]
macro_rules! desmos_expression {
    (($($inner:tt)*)) => {
        desmos_expression!($($inner)*)
    };
    ({&$raw:expr}) => {
        $crate::desmos::GraphExpression::clone(&$raw)
    };
    ({$raw:expr}) => {
        $crate::desmos::GraphExpression::from($raw)
    };
    () => {
        $crate::desmos::GraphExpression::Empty
    };
    (@letter $value:expr) => {
        $crate::desmos::GraphExpression::Letter($value)
    };
    (@int $value:expr) => {
        $crate::desmos::GraphExpression::Integer($value)
    };
    (@real $value:expr) => {
        $crate::desmos::GraphExpression::Decimal($value)
    };
    (@operatorname $value:expr) => {
        $crate::desmos::GraphExpression::OperatorName(Into::into($value))
    };
    (@escape $value:expr) => {
        $crate::desmos::GraphExpression::Escape(Into::into($value))
    };
    (@alnum $value:expr) => {
        $crate::desmos::GraphExpression::Alphanumeric(Into::into($value))
    };
    ($unary_kind:ident $inner:tt) => {
        $crate::desmos::GraphExpression::Unary {
            kind: $crate::desmos::GraphUnaryKind::$unary_kind,
            inner: Box::new(desmos_expression!($inner)),
        }
    };
    ($lhs:tt $binary_kind:ident $rhs:tt) => {
        $crate::desmos::GraphExpression::Binary {
            kind: $crate::desmos::GraphBinaryKind::$binary_kind,
            lhs: Box::new(desmos_expression!($lhs)),
            rhs: Box::new(desmos_expression!($rhs)),
        }
    };
    ([$($element:tt),* $(,)?]) => {
        $crate::desmos::GraphExpression::Sequence {
            elements: Vec::from([$(desmos_expression!($element)),*]),
        }
    };
    ([@ $elements:expr]) => {
        $crate::desmos::GraphExpression::Sequence {
            elements: Vec::from_iter($elements),
        }
    };
    ([@? $elements:expr]) => {
        $crate::desmos::GraphExpression::Sequence {
            elements: crate::Result::from_iter($elements)?,
        }
    };
    (@ineq $lhs:tt $($kind:ident $rhs:tt)+) => {
        $crate::desmos::GraphExpression::InequalityChain {
            lhs: Box::new(desmos_expression!($lhs)),
            chain: Vec::from([$((
                $crate::desmos::GraphInequalityKind::$kind,
                desmos_expression!($rhs),
            )),+]),
        }
    };
}
