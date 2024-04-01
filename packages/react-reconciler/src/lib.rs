use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use wasm_bindgen::prelude::*;

use shared::log;

use crate::fiber::{FiberNode, FiberRootNode, StateNode};
use crate::update_queue::{create_update, enqueue_update};
use crate::work_loop::WorkLoop;
use crate::work_tags::WorkTag;

mod utils;
pub mod fiber;
mod work_tags;
mod update_queue;
mod begin_work;
mod child_fiber;
mod work_loop;
mod complete_work;
mod commit_work;
pub mod fiber_flags;
pub mod host_config;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}


pub fn create_container(container: &JsValue) -> Rc<RefCell<FiberRootNode>> {
    let host_root_fiber = Rc::new(RefCell::new(FiberNode::new(WorkTag::HostRoot, None, None)));
    host_root_fiber.clone().borrow_mut().initialize_update_queue();
    let root = Rc::new(RefCell::new(FiberRootNode::new(Box::new(container.clone()), host_root_fiber.clone())));
    let r1 = root.clone();
    host_root_fiber.borrow_mut().state_node = Some(Rc::new(StateNode::FiberRootNode(r1)));
    log!("create_container, {:?}", root.clone().borrow().current.clone().borrow().tag);
    root.clone()
}

pub fn update_container(element: Rc<JsValue>, root: Rc<RefCell<FiberRootNode>>) {
    let host_root_fiber = Rc::clone(&root).borrow().current.clone();
    let update = create_update(element);
    enqueue_update(host_root_fiber.borrow(), update);
    log!("update_queue, {:?}", host_root_fiber.borrow().update_queue);

    let mut work_loop = WorkLoop::new();
    work_loop.schedule_update_on_fiber(host_root_fiber);
}

