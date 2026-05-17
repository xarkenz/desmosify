use crate::desmos::{GraphFolderEntry, GraphSettings, GraphState, ToJson};
use crate::desmos::error::DesmosResult;
use crate::desmos::target::GraphExpressionListBuilder;
use crate::sema::Program;

pub struct DesmosGeometryTarget;

impl crate::target::Target for DesmosGeometryTarget {
    type Output = DesmosResult<GraphState>;

    fn name(&self) -> &'static str {
        "desmos-geometry"
    }

    fn compile(&self, program: &Program) -> Self::Output {
        let mut state = GraphState {
            version: 11,
            graph: GraphSettings {
                product_name: "geometry-calculator".into()
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
