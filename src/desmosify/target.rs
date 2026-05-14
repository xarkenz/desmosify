use crate::sema::Program;

pub trait Target {
    type Output;

    fn name(&self) -> &'static str;

    fn compile(&self, program: &Program) -> Self::Output;
}
