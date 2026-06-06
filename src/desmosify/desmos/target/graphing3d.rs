use std::path::Path;
use crate::desmos::target::DesmosTargetInfo;
use crate::sema::Program;

pub const TARGET_NAME: &str = "desmos-graphing3d";

#[derive(Debug)]
pub struct DesmosGraphing3DTarget {
    info: DesmosTargetInfo,
}

impl Default for DesmosGraphing3DTarget {
    fn default() -> Self {
        Self {
            info: DesmosTargetInfo::new(),
        }
    }
}

impl crate::target::Target for DesmosGraphing3DTarget {
    fn name(&self) -> &'static str {
        TARGET_NAME
    }

    fn create_local_id(&mut self) -> u64 {
        self.info.create_local_id()
    }

    fn get_global_symbol_name(&mut self, identifier: &str) -> String {
        self.info.get_global_symbol(identifier).to_latex().to_string()
    }

    fn get_action_symbol_name(&mut self, identifier: &str) -> String {
        self.info.get_action_symbol(identifier).to_latex().to_string()
    }

    fn compile_to(&mut self, program: &Program, output_path: &Path) -> crate::Result<()> {
        todo!()
    }
}
