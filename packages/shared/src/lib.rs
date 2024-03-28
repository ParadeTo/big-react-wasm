#[macro_use]
extern crate lazy_static;

use std::rc::Rc;

use wasm_bindgen::prelude::*;
use web_sys::js_sys::Reflect;
use web_sys::wasm_bindgen::JsValue;

pub fn compare_js_value(a: &JsValue, b: &JsValue) -> bool {
    a.eq(b)
}

pub static REACT_ELEMENT: &str = "react.element";


pub fn derive_from_js_value(js_value: Rc<JsValue>, str: &str) -> Option<Rc<JsValue>> {
    match Reflect::get(&js_value, &JsValue::from_str(str)) {
        Ok(v) => Some(Rc::new(v)),
        Err(_) => None,
    }
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
