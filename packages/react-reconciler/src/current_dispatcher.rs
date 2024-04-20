use std::any::Any;

use wasm_bindgen::JsValue;

enum UseStateParams {
    Function(Box<dyn Fn() -> Box<dyn Any>>),
    Value(Box<dyn Any>),
}

#[derive(Debug)]
pub struct Dispatcher {
    pub use_state: *const dyn Fn(&JsValue) -> Vec<JsValue>,
    pub use_callback: *const dyn Fn(),
}

unsafe impl Send for Dispatcher {}

impl Dispatcher {
    pub fn new(use_state: *const dyn Fn(&JsValue) -> Vec<JsValue>, use_callback: *const dyn Fn()) -> Self {
        Dispatcher {
            use_state,
            use_callback,
        }
    }
}

pub struct CurrentDispatcher {
    pub current: Option<Box<Dispatcher>>,
}

pub static mut CURRENT_DISPATCHER: CurrentDispatcher = CurrentDispatcher { current: None };



