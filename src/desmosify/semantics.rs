use super::*;

#[derive(Debug)]
pub struct Analyzer<'a> {
    definitions: &'a mut Definitions,
}

impl<'a> Analyzer<'a> {
    pub fn new(definitions: &'a mut Definitions) -> Self {
        Self {
            definitions,
        }
    }

    pub fn analyze_expression(&self, expression: &mut Expression) {
        match &expression.value {
            ExpressionValue::Literal(value) => {},
            ExpressionValue::Name(name) => {},
            ExpressionValue::Operator(operation, operands) => {},
        }
    }
}

pub fn analyze(definitions: &mut Definitions) -> Result<(), DesmosifyError> {
    let analyzer = Analyzer::new(definitions);
}