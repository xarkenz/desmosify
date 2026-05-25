use std::path::Path;
use crate::sema::Program;

pub struct DesmosGraphing3DTarget;

impl crate::target::Target for DesmosGraphing3DTarget {
    fn name(&self) -> &'static str {
        "desmos-graphing3d"
    }

    fn compile_to(&self, program: &Program, output_path: &Path) -> crate::Result<()> {
        todo!()
    }
}
