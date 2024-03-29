use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsValue;

use shared::derive_from_js_value;

use crate::child_fiber::reconcile_child_fibers;
use crate::fiber::FiberNode;
use crate::update_queue::process_update_queue;
use crate::work_tags::WorkTag;

pub fn begin_work(work_in_progress: Rc<RefCell<FiberNode>>) -> Option<Rc<RefCell<FiberNode>>> {
    let work_in_progress = Rc::clone(&work_in_progress);
    let borrowed = work_in_progress.borrow();
    return match borrowed.tag {
        WorkTag::FunctionComponent => None,
        WorkTag::HostRoot => update_host_root(work_in_progress),
        WorkTag::HostComponent => update_host_component(work_in_progress),
    };
}

pub fn update_host_root(
    work_in_progress: Rc<RefCell<FiberNode>>,
) -> Option<Rc<RefCell<FiberNode>>> {
    process_update_queue(work_in_progress.clone());
    let next_children = work_in_progress.clone().borrow().memoized_state.clone();
    reconcile_children(work_in_progress.clone(), next_children);
    work_in_progress.clone().borrow().child.clone()
}

pub fn update_host_component(
    work_in_progress: Rc<RefCell<FiberNode>>,
) -> Option<Rc<RefCell<FiberNode>>> {
    let work_in_progress = Rc::clone(&work_in_progress);
    let ref_fiber_node = work_in_progress.borrow();
    let next_children =
        derive_from_js_value(ref_fiber_node.pending_props.clone().unwrap(), "children");
    reconcile_children(work_in_progress, next_children);
    work_in_progress.clone().borrow().child.clone()
}

pub fn reconcile_children(work_in_progress: Rc<RefCell<FiberNode>>, children: Option<Rc<JsValue>>) {
    let work_in_progress = Rc::clone(&work_in_progress);
    let current = work_in_progress.borrow().alternate.clone();
    if current.is_some() {
        // update
        work_in_progress.borrow_mut().child = reconcile_child_fibers(
            work_in_progress,
            current.clone().unwrap().upgrade(),
            children,
        )
    } else {
        // mount
        work_in_progress.borrow_mut().child = reconcile_child_fibers(
            work_in_progress,
            None,
            children,
        )
    }
}
