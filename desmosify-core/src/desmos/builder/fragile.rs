use std::collections::HashSet;
use crate::desmos::{BoxedGraphEntry, GraphExpression, GraphExpressionEntry};
use crate::desmos::target::DesmosTargetContext;
use crate::desmos_expression;

#[derive(Debug)]
pub struct FragileEncapsulator {
    folder_id: Option<String>,
    prefix: GraphExpression,
    known_signatures: HashSet<FragileSignature>,
    encapsulated_entries: Vec<BoxedGraphEntry>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct FragileSignature {
    name: Box<str>,
    arity: usize,
}

impl FragileEncapsulator {
    pub fn new(folder_id: Option<String>, prefix: GraphExpression) -> Self {
        Self {
            folder_id,
            prefix,
            known_signatures: HashSet::new(),
            encapsulated_entries: Vec::new(),
        }
    }

    pub fn finish(self) -> Vec<BoxedGraphEntry> {
        self.encapsulated_entries
    }

    pub fn get_symbol(&mut self, name: &str, arity: usize, info: &mut DesmosTargetContext) -> GraphExpression {
        let symbol = desmos_expression!({&self.prefix} Subscript (@alnum name));

        let signature = FragileSignature {
            name: name.into(),
            arity,
        };
        if self.known_signatures.insert(signature) {
            self.encapsulated_entries.push(Box::new(GraphExpressionEntry {
                id: info.create_entry_id(),
                folder_id: self.folder_id.clone(),
                expression: if arity == 0 {
                    desmos_expression!(
                        {&symbol} Equal (@operatorname name)
                    )
                } else {
                    let argument_sequence = GraphExpression::Sequence {
                        elements: std::iter::from_fn(|| Some(info.create_local_symbol()))
                            .take(arity)
                            .collect()
                    };
                    desmos_expression!(
                        ({&symbol} Call {&argument_sequence})
                        Equal
                        ((@operatorname name) Call {argument_sequence})
                    )
                },
                ..Default::default()
            }));
        }

        symbol
    }
}

#[derive(clap::ValueEnum, Default, Copy, Clone, PartialEq, Eq, Debug)]
pub enum FragileStrategy {
    /// Include fragile functions where they are called without any special handling. Beware:
    /// fragile functions can break if the compiled expression is tampered with.
    Inline,
    /// Create proper functions for each fragile function used, and put the definitions in a
    /// separate folder. This minimizes the chance of accidentally breaking the fragile functions
    /// when browsing the compiled graph.
    #[default]
    Encapsulate,
}

#[derive(Debug)]
pub enum FragileHandler {
    Inline,
    Encapsulate(FragileEncapsulator),
}

impl FragileHandler {
    pub fn new(strategy: FragileStrategy, folder_id: Option<&str>, prefix: GraphExpression) -> Self {
        match strategy {
            FragileStrategy::Inline => Self::Inline,
            FragileStrategy::Encapsulate => Self::Encapsulate(FragileEncapsulator::new(
                folder_id.map(Into::into),
                prefix,
            )),
        }
    }

    pub fn finish(self) -> Vec<BoxedGraphEntry> {
        match self {
            Self::Inline => Vec::new(),
            Self::Encapsulate(encapsulator) => encapsulator.finish()
        }
    }

    pub fn get_symbol(&mut self, name: &str, arity: usize, info: &mut DesmosTargetContext) -> GraphExpression {
        match self {
            Self::Inline => {
                GraphExpression::OperatorName(name.into())
            }
            Self::Encapsulate(encapsulator) => {
                encapsulator.get_symbol(name, arity, info)
            }
        }
    }
}
