use web_sys::wasm_bindgen::JsValue;

pub fn get_react_element_type() -> JsValue {
    JsValue::symbol(Some("react.element"))
}

#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

// pub enum ElementType {
//
// }
