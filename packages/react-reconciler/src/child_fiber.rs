use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsValue;

use crate::fiber::FiberNode;

pub fn reconcile_child_fibers(
    return_fiber: Rc<RefCell<FiberNode>>,
    current_first_child: Option<Rc<RefCell<FiberNode>>>,
    new_child: Option<Rc<JsValue>>,
) -> Option<Rc<RefCell<FiberNode>>> {
    if (new_child.is_some()) {
        let new_child = Rc::clone(&new_child.unwrap());
        let borrowed = new_child;
    }
    return None;
}

pub fn mount_child_fibers(
    return_fiber: Rc<RefCell<FiberNode>>,
    current_first_child: Option<Rc<RefCell<FiberNode>>>,
    new_child: Option<Rc<JsValue>>,
) {}
