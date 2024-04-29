use web_sys::js_sys::Reflect;
use web_sys::wasm_bindgen::JsValue;

pub static REACT_ELEMENT_TYPE: &str = "react.element";

#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

pub fn derive_from_js_value(js_value: &JsValue, str: &str) -> JsValue {
    match Reflect::get(&js_value, &JsValue::from_str(str)) {
        Ok(v) => v,
        Err(_) => {
            log!("derive {} from {:?} error", str, js_value);
            JsValue::undefined()
        }
    }
}

pub fn is_dev() -> bool {
    env!("ENV") == "dev"
}
