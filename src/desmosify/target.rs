use std::path::Path;
use crate::sema::Program;

pub trait Target : std::fmt::Debug {
    fn name(&self) -> &str;

    fn create_local_id(&mut self) -> u64;

    fn get_global_symbol_name(&mut self, identifier: &str) -> String;

    fn get_action_symbol_name(&mut self, identifier: &str) -> String;

    fn compile_to(&mut self, program: &Program, output_path: &Path) -> crate::Result<()>;
}

pub fn create_target_by_name(name: &str) -> crate::Result<Box<dyn Target>> {
    if name.starts_with("desmos") {
        crate::desmos::target::create_target_by_name(name)
    }
    else {
        Err(Box::new(crate::Error {
            kind: crate::ErrorKind::UnsupportedTarget {
                name: name.into(),
            },
            span: None,
        }))
    }
}

pub fn write_output_file(output_path: &Path, content: impl std::fmt::Display) -> crate::Result<()> {
    use std::io::Write;

    let mut output_file = output_path
        .parent()
        .map_or(Ok(()), |output_dir| std::fs::create_dir_all(output_dir))
        .and_then(|_| std::fs::File::create(output_path))
        .map_err(|cause| Box::new(crate::Error {
            kind: crate::ErrorKind::FileCreate {
                path: Some(output_path.into()),
                cause,
            },
            span: None,
        }))?;

    write!(output_file, "{content}")
        .map_err(|cause| Box::new(crate::Error {
            kind: crate::ErrorKind::FileWrite {
                path: Some(output_path.into()),
                cause,
            },
            span: None,
        }))
}
