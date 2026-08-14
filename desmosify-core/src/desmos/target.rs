use std::path::Path;
use crate::cli::DesmosifyArgs;
use crate::desmos::{GraphExpression, GraphSettings, GraphState, ToJson};
use crate::desmos::builder::fragile::FragileStrategy;
use crate::desmos::builder::GraphExpressionListBuilder;
use crate::desmos::symbol::SymbolTable;
use crate::desmos_expression;
use crate::sema::Program;
use crate::target::Target;

macro_rules! import_target_modules {
    ($($mod_name:ident),* $(,)?) => {
        $(pub mod $mod_name;)*

        pub fn create_target(args: &DesmosifyArgs) -> crate::Result<Box<dyn Target>> {
            match args.target_name.as_str() {
                $($mod_name::TARGET_NAME => Ok(Box::new(DesmosTargetContext::new(
                    $mod_name::create_descriptor(args),
                ))),)*
                _ => Err(Box::new(crate::Error {
                    kind: crate::ErrorKind::UnsupportedTarget {
                        name: args.target_name.as_str().into(),
                    },
                    span: None,
                }))
            }
        }
    };
}

import_target_modules! {
    geometry,
    graphing,
    graphing3d,
}

#[derive(Clone, Debug)]
pub struct DesmosTargetDescriptor {
    pub name: &'static str,
    pub version: u32,
    pub default_graph_settings: GraphSettings,
    pub use_geometry_folder: bool,
    pub enable_transform_fns: bool,
    pub fragile_strategy: FragileStrategy,
}

#[derive(Debug)]
pub struct DesmosTargetContext {
    descriptor: DesmosTargetDescriptor,
    graph_settings: GraphSettings,
    global_symbols: SymbolTable,
    action_symbols: SymbolTable,
    next_entry_id: u64,
    next_local_id: u64,
    next_inline_action_id: u64,
}

impl DesmosTargetContext {
    pub fn new(descriptor: DesmosTargetDescriptor) -> Self {
        Self {
            graph_settings: descriptor.default_graph_settings.clone(),
            descriptor,
            global_symbols: SymbolTable::new(GraphExpression::Letter('G')),
            action_symbols: SymbolTable::new(GraphExpression::Letter('A')),
            next_entry_id: 0,
            next_local_id: 0,
            next_inline_action_id: 0,
        }
    }

    pub fn descriptor(&self) -> &DesmosTargetDescriptor {
        &self.descriptor
    }

    pub fn graph_settings(&self) -> &GraphSettings {
        &self.graph_settings
    }

    pub fn graph_settings_mut(&mut self) -> &mut GraphSettings {
        &mut self.graph_settings
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

    pub fn create_local_id(&mut self) -> u64 {
        let id = self.next_local_id;
        self.next_local_id += 1;
        id
    }

    pub fn get_local_symbol(&mut self, id: u64) -> GraphExpression {
        desmos_expression!((@letter 'l') Subscript (@alnum id.to_string()))
    }

    pub fn create_local_symbol(&mut self) -> GraphExpression {
        let id = self.create_local_id();
        self.get_local_symbol(id)
    }

    pub fn create_inline_action_symbol(&mut self) -> GraphExpression {
        let id = self.next_inline_action_id;
        self.next_inline_action_id += 1;
        desmos_expression!((@letter 'a') Subscript (@alnum id.to_string()))
    }
}

impl Target for DesmosTargetContext {
    fn name(&self) -> &str {
        self.descriptor().name
    }

    fn create_local_id(&mut self) -> u64 {
        self.create_local_id()
    }

    fn get_global_symbol_name(&mut self, identifier: &str) -> String {
        self.get_global_symbol(identifier).to_latex().to_string()
    }

    fn get_action_symbol_name(&mut self, identifier: &str) -> String {
        self.get_action_symbol(identifier).to_latex().to_string()
    }

    fn compile_to(&mut self, program: &Program, output_path: &Path) -> crate::Result<()> {
        let state = GraphState {
            version: self.descriptor().version,
            graph: self.graph_settings.clone(),
            expressions: GraphExpressionListBuilder::build_program(program, self)?,
            include_function_parameters_in_random_seed: true,
        };

        crate::target::write_output_file(output_path, state.to_json())
    }
}
