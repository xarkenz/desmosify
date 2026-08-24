use super::*;

pub const TARGET_NAME: &str = "desmos-graphing3d";

pub fn create_descriptor(options: &crate::CompileOptions) -> crate::Result<DesmosTargetDescriptor> {
    // TODO
    let _ = options;
    Err(Box::new(crate::Error {
        kind: crate::ErrorKind::UnsupportedTarget {
            name: TARGET_NAME.into(),
        },
        span: None,
    }))
}
