use super::*;

use crate::{Definitions, Signatures, ConstantValue};
use crate::syntax::{Expression, ExpressionValue};

use json::JsonValue;

pub struct Graphing3DTarget;

impl crate::target::Target for Graphing3DTarget {
    type Output = JsonValue;

    fn name(&self) -> &'static str {
        "desmos-graphing-3d"
    }

    fn compile(&self, definitions: &Definitions, signatures: &Signatures) -> Self::Output {
        todo!()
    }
}