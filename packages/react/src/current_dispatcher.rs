use js_sys::{Function, Reflect};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

use crate::use_callback;

#[derive(Debug)]
pub struct Dispatcher {
    pub use_state: Function,
    pub use_effect: Function,
    pub use_ref: Function,
    pub use_memo: Function,
    pub use_callback: Function,
}

unsafe impl Send for Dispatcher {}

impl Dispatcher {
    pub fn new(
        use_state: Function,
        use_effect: Function,
        use_ref: Function,
        use_memo: Function,
        use_callback: Function,
    ) -> Self {
        Dispatcher {
            use_state,
            use_effect,
            use_ref,
            use_memo,
            use_callback,
        }
    }
}

pub struct CurrentDispatcher {
    pub current: Option<Box<Dispatcher>>,
}

pub static mut CURRENT_DISPATCHER: CurrentDispatcher = CurrentDispatcher { current: None };

fn derive_function_from_js_value(js_value: &JsValue, name: &str) -> Function {
    Reflect::get(js_value, &name.into())
        .unwrap()
        .dyn_into::<Function>()
        .unwrap()
}

#[wasm_bindgen(js_name = updateDispatcher)]
pub unsafe fn update_dispatcher(args: &JsValue) {
    let use_state = derive_function_from_js_value(args, "use_state");
    let use_effect = derive_function_from_js_value(args, "use_effect");
    let use_ref = derive_function_from_js_value(args, "use_ref");
    let use_memo = derive_function_from_js_value(args, "use_memo");
    let use_callback = derive_function_from_js_value(args, "use_callback");
    CURRENT_DISPATCHER.current = Some(Box::new(Dispatcher::new(
        use_state,
        use_effect,
        use_ref,
        use_memo,
        use_callback,
    )))
}
