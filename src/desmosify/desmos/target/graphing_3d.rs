use crate::desmos::GraphState;
use crate::sema::Program;

pub struct DesmosGraphing3DTarget;

impl crate::target::Target for DesmosGraphing3DTarget {
    type Output = crate::Result<GraphState>;

    fn name(&self) -> &'static str {
        "desmos-graphing3d"
    }

    fn compile(&self, program: &Program) -> Self::Output {
        todo!()
    }
}
