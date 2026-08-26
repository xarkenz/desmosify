use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn compile(source: &str) -> Result<String, JsValue> {
    console_error_panic_hook::set_once();
    Ok(source.to_string())
}
