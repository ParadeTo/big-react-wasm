use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsValue;

use crate::fiber::{FiberNode, FiberRootNode, StateNode};
use crate::update_queue::{create_update, create_update_queue, enqueue_update};
use crate::work_loop::WorkLoop;
use crate::work_tags::WorkTag;

pub mod fiber;
pub mod fiber_flags;
mod work_tags;
mod update_queue;
mod work_loop;
mod begin_work;
mod child_fiber;
mod complete_work;
mod commit_work;
mod fiber_hooks;

pub trait HostConfig {
    fn create_text_instance(&self, content: String) -> Rc<dyn Any>;
    fn create_instance(&self, _type: String) -> Rc<dyn Any>;
    fn append_initial_child(&self, parent: Rc<dyn Any>, child: Rc<dyn Any>);
    fn append_child_to_container(&self, child: Rc<dyn Any>, parent: Rc<dyn Any>);
}

pub struct Reconciler {
    host_config: Rc<dyn HostConfig>,
}


impl Reconciler {
    pub fn new(host_config: Rc<dyn HostConfig>) -> Self {
        Reconciler { host_config }
    }
    pub fn create_container(&self, container: Rc<dyn Any>) -> Rc<RefCell<FiberRootNode>> {
        let host_root_fiber = Rc::new(RefCell::new(FiberNode::new(WorkTag::HostRoot, None, None)));
        host_root_fiber.clone().borrow_mut().update_queue = Some(create_update_queue());
        let root = Rc::new(RefCell::new(FiberRootNode::new(container.clone(), host_root_fiber.clone())));
        let r1 = root.clone();
        host_root_fiber.borrow_mut().state_node = Some(Rc::new(StateNode::FiberRootNode(r1)));
        root.clone()
    }

    pub fn update_container(&self, element: Rc<JsValue>, root: Rc<RefCell<FiberRootNode>>) {
        let host_root_fiber = Rc::clone(&root).borrow().current.clone();
        let update = create_update(element);
        enqueue_update(host_root_fiber.borrow().update_queue.clone().unwrap(), update);
        let mut work_loop = WorkLoop::new(self.host_config.clone());
        work_loop.schedule_update_on_fiber(host_root_fiber);
    }
}


