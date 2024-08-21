use std::{cell::RefCell, rc::Rc};

use shared::{derive_from_js_value, type_of};
use wasm_bindgen::JsValue;

use crate::{
    fiber::FiberRootNode, fiber_flags::Flags, fiber_lanes::Lane,
    suspense_context::get_suspense_handler,
};

pub fn throw_exception(root: Rc<RefCell<FiberRootNode>>, value: JsValue, lane: Lane) {
    if !value.is_null()
        && type_of(&value, "object")
        && derive_from_js_value(&value, "then").is_function()
    {
        let suspense_boundary = get_suspense_handler();
        if suspense_boundary.is_some() {
            let suspense_boundary = suspense_boundary.unwrap();
            suspense_boundary.borrow_mut().flags |= Flags::ShouldCapture;
        }
    }
}
