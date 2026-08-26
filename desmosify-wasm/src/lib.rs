use std::path::Path;
use wasm_bindgen::prelude::*;
use desmosify::{CompileOptions, SourceFile, SourceFiles};

#[wasm_bindgen]
pub fn compile(source: &str) -> Result<String, JsValue> {
    console_error_panic_hook::set_once();

    let sources = SourceFiles::from_iter(std::iter::once(SourceFile {
        path: Path::new("<playground>"),
        content: source,
    }));

    desmosify::compile(&sources, &CompileOptions::default_for_target("desmos-graphing"))
        .map_err(|error| {
            JsValue::from_str(&format!("Error: {}", error.display_with_context(&sources)))
        })
}
