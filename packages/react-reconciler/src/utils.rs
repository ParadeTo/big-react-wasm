use wasm_bindgen::JsValue;
use web_sys::js_sys::Reflect;

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}


pub fn derive_from_js_value(js_value: Option<JsValue>, str: &str) -> Option<JsValue> {
    if js_value.is_none() {
        return None;
    }
    match Reflect::get(&js_value.unwrap(), &JsValue::from_str(str)) {
        Ok(v) => Some(v),
        Err(_) => { None }
    }
}