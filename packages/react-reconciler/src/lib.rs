use std::cell::{Ref, RefCell};
use std::ops::Deref;
use std::rc::Rc;

use wasm_bindgen::prelude::*;

use crate::fiber::{FiberNode, FiberRootNode, StateNode};
use crate::update_queue::{create_update, enqueue_update};
use crate::work_tags::WorkTag;

mod utils;
pub mod fiber;
mod work_tags;
mod update_queue;
mod begin_work;
mod child_fiber;
mod work_loop;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}


pub fn create_container(container: &JsValue) -> Rc<RefCell<FiberRootNode>> {
    let mut host_root_fiber = Rc::new(RefCell::new(FiberNode::new(WorkTag::HostRoot, &JsValue::null(), None)));
    let root = Rc::new(RefCell::new(FiberRootNode::new(Box::new(container.clone()), host_root_fiber.clone())));
    let r1 = root.clone();
    host_root_fiber.borrow_mut().state_node = Some(StateNode::FiberRootNode(r1));
    root.clone()
}

pub fn update_container(element: Rc<JsValue>, root: Ref<FiberRootNode>) {
    let update = create_update(element);
    enqueue_update(root.current.upgrade().unwrap().borrow(), update);
}

