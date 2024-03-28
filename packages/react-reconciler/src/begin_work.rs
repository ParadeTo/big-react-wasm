use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsValue;

use crate::fiber::FiberNode;
use crate::work_tags::WorkTag;

pub fn begin_work(work_in_progress: Rc<RefCell<FiberNode>>) {
    let work_in_progress = Rc::clone(&work_in_progress);
    let borrowed = work_in_progress.borrow();
    match borrowed.tag {
        WorkTag::FunctionComponent => {}
        WorkTag::HostRoot => {}
        WorkTag::HostComponent => {}
    }
}

pub fn update_host_component(work_in_progress: Rc<RefCell<FiberNode>>) {
    // let work_in_progress = Rc::clone(&work_in_progress);
    // let ref_fiber_node = work_in_progress.borrow();
    // let next_children = derive_from_js_value(ref_fiber_node.pending_props.clone(), "children");
    // reconcile_children(work_in_progress, next_children);
}


pub fn reconcile_children(work_in_progress: Rc<RefCell<FiberNode>>, children: Option<Rc<JsValue>>) {
    let work_in_progress = Rc::clone(&work_in_progress);
    let current = work_in_progress.borrow().alternate.clone();
    if current.is_some() {
        let child = current.unwrap().upgrade().unwrap().borrow().child.clone();
    } else {}
}
