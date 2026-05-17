use crate::desmos::GraphState;
use crate::desmos::error::DesmosResult;
use crate::sema::Program;

pub struct DesmosGraphingTarget;

impl crate::target::Target for DesmosGraphingTarget {
    type Output = DesmosResult<GraphState>;

    fn name(&self) -> &'static str {
        "desmos-graphing"
    }

    fn compile(&self, program: &Program) -> Self::Output {
        todo!()
    }
}
