use wasm_bindgen::JsValue;
use web_sys::js_sys::wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub static SuspenseException: JsValue;
}

pub fn track_used_thenable(usable: &JsValue) -> Result<JsValue, JsValue> {
    Err(SuspenseException.__inner.with(JsValue::clone))
}
