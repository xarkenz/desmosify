pub mod desmos;

pub trait Target {
    type Output;

    fn name(&self) -> &'static str;
    fn compile(&self, definitions: &crate::Definitions, signatures: &crate::Signatures) -> Self::Output;
}