use crate::desmos::GraphState;
use crate::sema::Program;

pub struct DesmosGraphingTarget;

impl crate::target::Target for DesmosGraphingTarget {
    type Output = crate::Result<GraphState>;

    fn name(&self) -> &'static str {
        "desmos-graphing"
    }

    fn compile(&self, program: &Program) -> Self::Output {
        todo!()
    }
}
