use std::{cell::RefCell, rc::Rc};

use shared::{derive_from_js_value, type_of};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::js_sys::Function;

use crate::{
    fiber::FiberRootNode, fiber_flags::Flags, fiber_lanes::Lane,
    suspense_context::get_suspense_handler, work_loop::ensure_root_is_scheduled,
};

fn attach_ping_listener(root: Rc<RefCell<FiberRootNode>>, wakeable: JsValue, lane: Lane) {
    let then_value = derive_from_js_value(&wakeable, "then");
    let then = then_value.dyn_ref::<Function>().unwrap();
    let closure = Closure::wrap(Box::new(move || {
        root.clone().borrow_mut().mark_root_updated(lane.clone());
        ensure_root_is_scheduled(root.clone());
    }) as Box<dyn Fn()>);
    let ping = closure.as_ref().unchecked_ref::<Function>().clone();
    closure.forget();
    then.call2(&wakeable, &ping, &ping)
        .expect("failed to call then function");
}

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

        attach_ping_listener(root, value, lane)
    }
}
