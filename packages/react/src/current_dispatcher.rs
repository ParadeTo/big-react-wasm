use std::any::Any;
use std::rc::Rc;

use wasm_bindgen::JsValue;

use shared::log;

enum UseStateParams {
    Function(Box<dyn Fn() -> Box<dyn Any>>),
    Value(Box<dyn Any>),
}

pub struct Dispatcher {
    pub use_state: *const dyn Fn() -> Vec<JsValue>,
    pub use_callback: *const dyn Fn(),
}

unsafe impl Send for Dispatcher {}

impl Dispatcher {
    pub fn new(use_state: *const dyn Fn() -> Vec<JsValue>, use_callback: *const dyn Fn()) -> Self {
        Dispatcher {
            use_state,
            use_callback,
        }
    }
}

pub struct CurrentDispatcher {
    pub current: Option<Rc<Dispatcher>>,
}

pub static mut CURRENT_DISPATCHER: CurrentDispatcher = CurrentDispatcher { current: None };

pub fn resolve_dispatche() -> Rc<Dispatcher> {
    unsafe {
        let dispatcher = CURRENT_DISPATCHER.current.clone();
        if dispatcher.is_none() {
            log!("dispatcher doesn't exist")
        }
        return dispatcher.unwrap().clone();
    }
}