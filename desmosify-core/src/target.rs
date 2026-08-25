use crate::sema::Program;

pub trait Target : std::fmt::Debug {
    fn name(&self) -> &str;

    fn create_local_id(&mut self) -> u64;

    fn get_global_symbol_name(&mut self, identifier: &str) -> String;

    fn get_action_symbol_name(&mut self, identifier: &str) -> String;

    fn generate_output(&mut self, program: &Program) -> crate::Result<String>;
}

pub fn create_target(options: &crate::CompileOptions) -> crate::Result<Box<dyn Target>> {
    if options.target_name.starts_with("desmos") {
        crate::desmos::target::create_target(options)
    }
    else {
        Err(Box::new(crate::Error {
            kind: crate::ErrorKind::UnsupportedTarget {
                name: options.target_name.as_str().into(),
            },
            span: None,
        }))
    }
}
