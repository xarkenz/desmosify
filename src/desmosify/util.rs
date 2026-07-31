pub enum LazyConst<T: Clone> {
    Immediate(T),
    Deferred(fn() -> T),
}

impl<T: Clone> LazyConst<T> {
    pub fn get(&self) -> T {
        match self {
            Self::Immediate(value) => value.clone(),
            Self::Deferred(getter) => getter(),
        }
    }
}
