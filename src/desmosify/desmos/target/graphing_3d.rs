use json::JsonValue;
use crate::sema::Program;

pub struct Graphing3DTarget;

impl crate::target::Target for Graphing3DTarget {
    type Output = JsonValue;

    fn name(&self) -> &'static str {
        "desmos-graphing-3d"
    }

    fn compile(&self, program: &Program) -> Self::Output {
        todo!()
    }
}
