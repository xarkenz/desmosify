use super::*;

pub const TARGET_NAME: &str = "desmos-geometry";

pub fn create_descriptor(options: &crate::CompileOptions) -> crate::Result<DesmosTargetDescriptor> {
    Ok(DesmosTargetDescriptor {
        name: TARGET_NAME,
        version: 11,
        default_graph_settings: GraphSettings {
            product_name: "geometry-calculator".into(),
            ..Default::default()
        },
        use_geometry_folder: true,
        enable_transform_fns: true,
        fragile_strategy: options.fragile_strategy,
    })
}
