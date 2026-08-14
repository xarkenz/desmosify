use super::*;

pub const TARGET_NAME: &str = "desmos-graphing";

pub fn create_descriptor(args: &DesmosifyArgs) -> DesmosTargetDescriptor {
    DesmosTargetDescriptor {
        name: TARGET_NAME,
        version: 11,
        default_graph_settings: GraphSettings {
            product_name: "graphing".into(),
            ..Default::default()
        },
        use_geometry_folder: false,
        enable_transform_fns: false,
        fragile_strategy: args.fragile_strategy,
    }
}
