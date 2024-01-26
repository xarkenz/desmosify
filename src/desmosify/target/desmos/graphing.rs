use super::*;

use crate::{Definitions, Signatures, ConstantValue};
use crate::syntax::{Expression, ExpressionValue};

use json::JsonValue;

pub struct GraphingTarget;

impl crate::target::Target for GraphingTarget {
    type Output = JsonValue;

    fn name(&self) -> &'static str {
        "desmos-graphing"
    }

    fn compile(&self, definitions: &Definitions, signatures: &Signatures) -> Self::Output {
        todo!()
    }
}