use std::path::Path;
use crate::desmos::{GraphSettings, GraphState, ToJson};
use crate::desmos::target::GraphExpressionListBuilder;
use crate::sema::Program;

pub struct DesmosGraphingTarget;

impl crate::target::Target for DesmosGraphingTarget {
    fn name(&self) -> &'static str {
        "desmos-graphing"
    }

    fn compile_to(&self, program: &Program, output_path: &Path) -> crate::Result<()> {
        let state = GraphState {
            version: 11,
            graph: GraphSettings {
                product_name: "graphing".into(),
                show_grid: false,
                show_x_axis: false,
                show_y_axis: false,
                viewport_x_min: -10.0,
                viewport_y_min: -10.0,
                viewport_x_max: 10.0,
                viewport_y_max: 10.0,
            },
            expressions: GraphExpressionListBuilder::build_program(program)?,
        };

        crate::target::write_output_file(output_path, state.to_json())
    }
}
