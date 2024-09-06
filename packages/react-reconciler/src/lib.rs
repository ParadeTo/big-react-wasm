use std::any::Any;
use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use wasm_bindgen::JsValue;
use web_sys::js_sys::Object;

use crate::complete_work::CompleteWork;
use crate::fiber::{FiberNode, FiberRootNode, StateNode};
// use crate::fiber_hooks::{WORK_LOOP as Fiber_HOOKS};
use crate::fiber_lanes::Lane;
use crate::update_queue::{create_update, create_update_queue, enqueue_update};
use crate::work_loop::schedule_update_on_fiber;
use crate::work_tags::WorkTag;

mod begin_work;
mod child_fiber;
mod commit_work;
mod complete_work;
pub mod fiber;
mod fiber_context;
mod fiber_flags;
mod fiber_hooks;
pub mod fiber_lanes;
mod fiber_throw;
mod fiber_unwind_work;
mod suspense_context;
mod sync_task_queue;
mod thenable;
mod update_queue;
mod work_loop;
pub mod work_tags;

pub static mut HOST_CONFIG: Option<Rc<dyn HostConfig>> = None;
static mut COMPLETE_WORK: Option<CompleteWork> = None;

pub trait HostConfig {
    fn create_text_instance(&self, content: &JsValue) -> Rc<dyn Any>;
    fn create_instance(&self, _type: String, props: Rc<dyn Any>) -> Rc<dyn Any>;
    fn append_initial_child(&self, parent: Rc<dyn Any>, child: Rc<dyn Any>);
    fn append_child_to_container(&self, child: Rc<dyn Any>, parent: Rc<dyn Any>);
    fn remove_child(&self, child: Rc<dyn Any>, container: Rc<dyn Any>);
    // fn commit_text_update(&self, text_instance: Rc<dyn Any>, content: &JsValue);
    fn commit_update(&self, fiber: Rc<RefCell<FiberNode>>);
    fn insert_child_to_container(
        &self,
        child: Rc<dyn Any>,
        container: Rc<dyn Any>,
        before: Rc<dyn Any>,
    );
    fn schedule_microtask(&self, callback: Box<dyn FnMut()>);
}

pub struct Reconciler {
    pub host_config: Rc<dyn HostConfig>,
}

impl Reconciler {
    pub fn new(host_config: Rc<dyn HostConfig>) -> Self {
        Reconciler { host_config }
    }
    pub fn create_container(&self, container: Rc<dyn Any>) -> Rc<RefCell<FiberRootNode>> {
        let host_root_fiber = Rc::new(RefCell::new(FiberNode::new(
            WorkTag::HostRoot,
            JsValue::null(),
            JsValue::null(),
            JsValue::null(),
        )));
        host_root_fiber.clone().borrow_mut().update_queue = Some(create_update_queue());
        let root = Rc::new(RefCell::new(FiberRootNode::new(
            container.clone(),
            host_root_fiber.clone(),
        )));
        let r1 = root.clone();
        host_root_fiber.borrow_mut().state_node = Some(Rc::new(StateNode::FiberRootNode(r1)));
        root.clone()
    }

    pub fn update_container(&self, element: JsValue, root: Rc<RefCell<FiberRootNode>>) -> JsValue {
        let host_root_fiber = Rc::clone(&root).borrow().current.clone();
        let root_render_priority = Lane::SyncLane;
        let update = create_update(element.clone(), root_render_priority.clone());
        let update_queue = { host_root_fiber.borrow().update_queue.clone().unwrap() };
        enqueue_update(
            update_queue,
            update,
            host_root_fiber.clone(),
            root_render_priority.clone(),
        );
        unsafe {
            HOST_CONFIG = Some(self.host_config.clone());
            COMPLETE_WORK = Some(CompleteWork::new(self.host_config.clone()));
            schedule_update_on_fiber(host_root_fiber, root_render_priority);
        }
        element.clone()
    }
}

#[derive(Clone, Debug)]
struct JsValueKey(JsValue);

impl PartialEq for JsValueKey {
    fn eq(&self, other: &Self) -> bool {
        Object::is(&self.0, &other.0)
    }
}

impl Eq for JsValueKey {}

impl Hash for JsValueKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if self.0.is_string() {
            self.0.as_string().unwrap().hash(state)
        } else if let Some(n) = self.0.as_f64() {
            n.to_bits().hash(state)
        } else if self.0.is_null() {
            "null".hash(state)
        }
    }
}
