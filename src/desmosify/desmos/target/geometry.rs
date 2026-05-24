use crate::desmos::{GraphFolderEntry, GraphSettings, GraphState};
use crate::desmos::target::GraphExpressionListBuilder;
use crate::sema::Program;

pub struct DesmosGeometryTarget;

impl crate::target::Target for DesmosGeometryTarget {
    type Output = crate::Result<GraphState>;

    fn name(&self) -> &'static str {
        "desmos-geometry"
    }

    fn compile(&self, program: &Program) -> Self::Output {
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
            },
            expressions: GraphExpressionListBuilder::build_program(program)?,
        };

        state.expressions.entries.insert(0, Box::new(GraphFolderEntry {
            id: "**dcg_geo_folder**".into(),
            title: "geometry".into(),
            collapsed: true,
            secret: true,
        }));

        Ok(state)
    }
}
