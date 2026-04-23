use crate::sema::types::{DataType, FunctionSignature};

pub mod core;

pub fn simple_signature(arguments: &[DataType], signature: FunctionSignature) -> Option<DataType> {
    signature.get_return_type(arguments).cloned()
}

pub fn overloaded_signature(arguments: &[DataType], signatures: Box<[FunctionSignature]>) -> Option<DataType> {
    signatures
        .into_iter()
        .find_map(|signature| simple_signature(arguments, signature))
}

#[derive(Debug)]
pub struct IntrinsicValue {
    value_type: fn() -> DataType,
}

impl IntrinsicValue {
    pub fn get_type(&'static self) -> DataType {
        (self.value_type)()
    }
}

#[derive(Debug)]
pub struct IntrinsicFunction {
    signature_test: fn(arguments: &[DataType]) -> Option<DataType>,
}

#[derive(Debug)]
pub enum IntrinsicKind {
    Value(IntrinsicValue),
    Function(IntrinsicFunction),
}

impl IntrinsicKind {
    pub fn get_type(&'static self) -> DataType {
        match self {
            Self::Value(value) => value.get_type(),
            Self::Function(function) => DataType::IntrinsicFunction {
                function,
            },
        }
    }
}

#[derive(Debug)]
pub struct Intrinsic {
    pub identifier: &'static str,
    pub kind: IntrinsicKind,
}

impl Intrinsic {
    pub fn get_type(&'static self) -> DataType {
        self.kind.get_type()
    }
}
