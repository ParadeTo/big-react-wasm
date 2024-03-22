use std::cell::{Ref, RefCell};
use std::ops::Deref;
use std::rc::Rc;

use wasm_bindgen::prelude::*;

use react::ReactElement;
use shared::log;

use crate::fiber::{FiberNode, FiberRootNode, StateNode};
use crate::update_queue::{create_update, enqueue_update};
use crate::work_tags::WorkTag;

mod utils;
pub mod fiber;
mod work_tags;
mod update_queue;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}


pub fn create_container(container: &JsValue) -> Rc<RefCell<FiberRootNode>> {
    let mut host_root_fiber = Rc::new(RefCell::new(FiberNode::new(WorkTag::HostRoot)));
    let root = Rc::new(RefCell::new(FiberRootNode::new(Box::new(container.clone()), host_root_fiber.clone())));
    let r1 = root.clone();
    host_root_fiber.borrow_mut().state_node = Some(StateNode::FiberRootNode(Rc::downgrade(&r1)));
    root.clone()
}

pub fn update_container(element: &ReactElement, root: Ref<FiberRootNode>) {
    log!("{:?}, {:?}", element, root);
    let update = create_update(JsValue::);
    enqueue_update(root.current.borrow(), update);
    // let host_root_fiber = root();

    // let update = create_update
}

