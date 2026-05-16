#[derive(Debug)]
pub enum DesmosErrorKind {
    UnsupportedValue,
}

impl std::fmt::Display for DesmosErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::UnsupportedValue => {
                write!(f, "the target does not support this value")
            }
        }
    }
}

#[derive(Debug)]
pub struct DesmosError {
    pub kind: DesmosErrorKind,
}

pub type DesmosResult<T> = Result<T, Box<DesmosError>>;

impl std::fmt::Display for DesmosError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl std::error::Error for DesmosError {}
