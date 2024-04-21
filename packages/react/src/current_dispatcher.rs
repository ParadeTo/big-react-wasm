use js_sys::{Function, Reflect};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

#[derive(Debug)]
pub struct Dispatcher {
    pub use_state: Function,
    // pub use_callback: *const dyn Fn(),
}

unsafe impl Send for Dispatcher {}

impl Dispatcher {
    pub fn new(use_state: Function/*, use_callback: *const dyn Fn()*/) -> Self {
        Dispatcher {
            use_state,
            // use_callback,
        }
    }
}

pub struct CurrentDispatcher {
    pub current: Option<Box<Dispatcher>>,
}

pub static mut CURRENT_DISPATCHER: CurrentDispatcher = CurrentDispatcher { current: None };

fn derive_function_from_js_value(js_value: &JsValue, name: &str) -> Function {
    Reflect::get(js_value, &name.into()).unwrap().dyn_into::<Function>().unwrap()
}

#[wasm_bindgen(js_name = updateDispatcher)]
pub unsafe fn update_dispatcher(args: &JsValue) {
    let use_state = derive_function_from_js_value(args, "use_state");
    CURRENT_DISPATCHER.current = Some(Box::new(Dispatcher::new(use_state)))
}



