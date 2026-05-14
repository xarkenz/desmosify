use json::JsonValue;
use crate::sema::Program;

pub struct GraphingTarget;

impl crate::target::Target for GraphingTarget {
    type Output = JsonValue;

    fn name(&self) -> &'static str {
        "desmos-graphing"
    }

    fn compile(&self, program: &Program) -> Self::Output {
        todo!()
    }
}
