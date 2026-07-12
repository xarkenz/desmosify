use crate::desmos::GraphSettings;
use crate::desmos::target::DesmosTargetInfo;

pub const TARGET_NAME: &str = "desmos-graphing";

pub fn create_target() -> DesmosTargetInfo {
    DesmosTargetInfo {
        name: TARGET_NAME,
        version: 11,
        graph_settings: GraphSettings {
            product_name: "graphing".into(),
            ..Default::default()
        },
        use_dcg_geo_folder: true,
        ..Default::default()
    }
}
