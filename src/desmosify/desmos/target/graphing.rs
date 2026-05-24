use crate::desmos::{GraphSettings, GraphState};
use crate::desmos::target::GraphExpressionListBuilder;
use crate::sema::Program;

pub struct DesmosGraphingTarget;

impl crate::target::Target for DesmosGraphingTarget {
    type Output = crate::Result<GraphState>;

    fn name(&self) -> &'static str {
        "desmos-graphing"
    }

    fn compile(&self, program: &Program) -> Self::Output {
        Ok(GraphState {
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
        })
    }
}
