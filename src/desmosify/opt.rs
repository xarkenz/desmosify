use std::collections::HashMap;
use std::rc::Rc;
use crate::sema::values::{BinaryKind, UnaryKind, Value, ValueKind};

fn replace_all_global_references(value: &mut Value, identifier: Rc<str>, replacement: ValueKind) {
    value.visit_postorder_mut(|value| match &mut value.kind {
        ValueKind::Global(global) if global.identifier == identifier => {
            value.kind = replacement.clone();
        }
        _ => {}
    });
}

#[derive(Debug)]
struct SymbolInfo {
    dependencies: Vec<Rc<str>>,
    constant_value: Option<ValueKind>,
}

#[derive(Debug)]
pub struct ConstantPropagate {
    symbols: HashMap<Rc<str>, SymbolInfo>,
    fold_order: Vec<Rc<str>>,
}

impl ConstantPropagate {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            fold_order: Vec::new(),
        }
    }

    pub fn determine_fold_order(&mut self) -> crate::Result<()> {
        self.fold_order.clear();

        let mut visited = HashMap::new();
        while let Some(next_identifier) = self.symbols
            .keys()
            .find(|&identifier| !visited.contains_key(identifier))
            .cloned()
        {
            visit(next_identifier, &mut visited, &self.symbols, &mut self.fold_order)?;
        }

        fn visit(
            identifier: Rc<str>,
            visited: &mut HashMap<Rc<str>, bool>,
            symbols: &HashMap<Rc<str>, SymbolInfo>,
            fold_order: &mut Vec<Rc<str>>,
        ) -> crate::Result<()> {
            match visited.get(&identifier) {
                Some(true) => {
                    Ok(())
                }
                Some(false) => {
                    Err(Box::new(crate::Error {
                        kind: crate::ErrorKind::CyclicDefinitions {
                            identifier: identifier.into(),
                        },
                    }))
                }
                None => {
                    package.visit_marker.set(PackageVisitMarker::InProgress);

                    for dependency in package.dependencies() {
                        visit(&package_registry[dependency.name()], package_registry, compile_stack)?;
                    }

                    package.visit_marker.set(PackageVisitMarker::Visited);
                    compile_stack.push_back(package.name().into());

                    Ok(())
                }
            }
        }

        Ok(())
    }

    fn constant_propagate(&self, )
}

pub fn constant_fold(value: &mut ValueKind) {
    match value {
        //
        _ => {}
    }
}

fn constant_fold_unary(kind: UnaryKind, operand: &mut ValueKind) {}

fn constant_fold_binary(kind: BinaryKind, lhs: &mut ValueKind, rhs: &mut ValueKind) {}

fn constant_fold_index()
