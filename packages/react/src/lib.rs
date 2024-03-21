mod utils;

use shared::REACT_ELEMENT_TYPE;
use wasm_bindgen::prelude::*;
use web_sys::console;
use web_sys::js_sys::{Function, JsString};
use crate::utils::set_panic_hook;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, react!");
}

#[wasm_bindgen]
pub struct ReactElement {
    _typeof: REACT_ELEMENT_TYPE,
}

impl ReactElement {
    fn new() -> Self {
        Self { _typeof: REACT_ELEMENT_TYPE }
    }
}

#[wasm_bindgen]
pub fn jsxDEV(_type: &JsValue, config: &JsValue) -> ReactElement {
    set_panic_hook();
    // console::log_2(_type, config);

    // let this = JsValue::null();
    // for &x in &self.xs {
    //     let x = JsValue::from(x);
    //     let _ = f.call1(&this, &x);
    // }
    // let a = _type.dyn_ref();
    let a = _type.dyn_ref::<Function>();
    if a.is_some() {
        let this = JsValue::null();
        console::log_1(&a.unwrap().call0(&this).unwrap());
    } else {
        let a = _type.dyn_ref::<JsString>();
        console::log_1(a.unwrap());
    }

    // (_type as Function).call1();

    ReactElement::new()
    // Some(Box::new())
}