use crate::desmos::{GraphBinaryKind, GraphExpression};
use crate::desmos::symbol::SymbolTable;

pub mod geometry;
pub mod graphing;
pub mod graphing3d;

pub use geometry::DesmosGeometryTarget;
pub use graphing::DesmosGraphingTarget;
pub use graphing3d::DesmosGraphing3DTarget;

pub fn new_target_by_name(name: &str) -> crate::Result<Box<dyn crate::target::Target>> {
    match name {
        geometry::TARGET_NAME => Ok(Box::new(DesmosGeometryTarget::default())),
        graphing::TARGET_NAME => Ok(Box::new(DesmosGraphingTarget::default())),
        graphing3d::TARGET_NAME => Ok(Box::new(DesmosGraphing3DTarget::default())),
        _ => Err(Box::new(crate::Error {
            kind: crate::ErrorKind::UnsupportedTarget {
                name: name.into(),
            },
            span: None,
        }))
    }
}

#[derive(Debug)]
pub struct DesmosTargetInfo {
    next_local_id: u64,
    next_entry_id: u64,
    global_symbols: SymbolTable,
    action_symbols: SymbolTable,
}

impl DesmosTargetInfo {
    pub fn new() -> Self {
        Self {
            next_local_id: 0,
            next_entry_id: 0,
            global_symbols: SymbolTable::new(GraphExpression::Letter('G')),
            action_symbols: SymbolTable::new(GraphExpression::Letter('A')),
        }
    }

    pub fn create_local_id(&mut self) -> u64 {
        let id = self.next_local_id;
        self.next_local_id += 1;
        id
    }

    pub fn create_entry_id(&mut self) -> String {
        let id = self.next_entry_id;
        self.next_entry_id += 1;
        id.to_string()
    }

    pub fn get_global_symbol(&mut self, identifier: &str) -> GraphExpression {
        self.global_symbols.get_symbol(identifier)
    }

    pub fn get_action_symbol(&mut self, identifier: &str) -> GraphExpression {
        self.action_symbols.get_symbol(identifier)
    }

    pub fn get_local_symbol(&mut self, id: u64) -> GraphExpression {
        GraphExpression::Binary {
            kind: GraphBinaryKind::Subscript,
            lhs: Box::new(GraphExpression::Letter('l')),
            rhs: Box::new(GraphExpression::Alphanumeric(id.to_string())),
        }
    }

    pub fn create_local_symbol(&mut self) -> GraphExpression {
        let id = self.create_local_id();
        self.get_local_symbol(id)
    }
}
