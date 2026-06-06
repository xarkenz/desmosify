use std::path::Path;
use crate::desmos::{GraphFolderEntry, GraphSettings, GraphState, ToJson};
use crate::desmos::builder::GraphExpressionListBuilder;
use crate::desmos::target::DesmosTargetInfo;
use crate::sema::Program;

pub const TARGET_NAME: &str = "desmos-geometry";

#[derive(Debug)]
pub struct DesmosGeometryTarget {
    info: DesmosTargetInfo,
}

impl Default for DesmosGeometryTarget {
    fn default() -> Self {
        Self {
            info: DesmosTargetInfo::new(),
        }
    }
}

impl crate::target::Target for DesmosGeometryTarget {
    fn name(&self) -> &'static str {
        TARGET_NAME
    }

    fn create_local_id(&mut self) -> u64 {
        self.info.create_local_id()
    }

    fn get_global_symbol_name(&mut self, identifier: &str) -> String {
        self.info.get_global_symbol(identifier).to_latex().to_string()
    }

    fn get_action_symbol_name(&mut self, identifier: &str) -> String {
        self.info.get_action_symbol(identifier).to_latex().to_string()
    }

    fn compile_to(&mut self, program: &Program, output_path: &Path) -> crate::Result<()> {
        let mut state = GraphState {
            version: 11,
            graph: GraphSettings {
                product_name: "geometry-calculator".into(),
                show_grid: false,
                show_x_axis: false,
                show_y_axis: false,
                viewport_x_min: -10.0,
                viewport_y_min: -10.0,
                viewport_x_max: 10.0,
                viewport_y_max: 10.0,
                degree_mode: false,
            },
            expressions: GraphExpressionListBuilder::build_program(program, &mut self.info)?,
        };

        state.expressions.entries.insert(0, Box::new(GraphFolderEntry {
            id: "**dcg_geo_folder**".into(),
            title: "geometry".into(),
            collapsed: true,
            secret: true,
        }));

        crate::target::write_output_file(output_path, state.to_json())
    }
}
