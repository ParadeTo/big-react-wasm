use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsValue;

use shared::derive_from_js_value;

use crate::child_fiber::{mount_child_fibers, reconcile_child_fibers};
use crate::fiber::FiberNode;
use crate::fiber_hooks::FiberHooks;
use crate::update_queue::process_update_queue;
use crate::work_tags::WorkTag;

pub fn begin_work(work_in_progress: Rc<RefCell<FiberNode>>) -> Result<Option<Rc<RefCell<FiberNode>>>, JsValue> {
    let tag = work_in_progress.clone().borrow().tag.clone();
    return match tag {
        WorkTag::FunctionComponent => update_function_component(work_in_progress.clone()),
        WorkTag::HostRoot => Ok(update_host_root(work_in_progress.clone())),
        WorkTag::HostComponent => Ok(update_host_component(work_in_progress.clone())),
        WorkTag::HostText => Ok(None),
    };
}

fn update_function_component(
    work_in_progress: Rc<RefCell<FiberNode>>,
) -> Result<Option<Rc<RefCell<FiberNode>>>, JsValue> {
    let fiber_hooks = &mut FiberHooks::new();
    let next_children = Rc::new(fiber_hooks.render_with_hooks(work_in_progress.clone())?);
    reconcile_children(work_in_progress.clone(), Some(next_children));
    Ok(work_in_progress.clone().borrow().child.clone())
}

fn update_host_root(work_in_progress: Rc<RefCell<FiberNode>>) -> Option<Rc<RefCell<FiberNode>>> {
    process_update_queue(work_in_progress.clone());
    let next_children = work_in_progress.clone().borrow().memoized_state.clone();
    reconcile_children(work_in_progress.clone(), next_children);
    work_in_progress.clone().borrow().child.clone()
}

fn update_host_component(
    work_in_progress: Rc<RefCell<FiberNode>>,
) -> Option<Rc<RefCell<FiberNode>>> {
    let work_in_progress = Rc::clone(&work_in_progress);

    let next_children = {
        let ref_fiber_node = work_in_progress.borrow();
        derive_from_js_value(ref_fiber_node.pending_props.clone().unwrap(), "children")
    };

    {
        reconcile_children(work_in_progress.clone(), next_children);
    }
    work_in_progress.clone().borrow().child.clone()
}

fn reconcile_children(work_in_progress: Rc<RefCell<FiberNode>>, children: Option<Rc<JsValue>>) {
    let work_in_progress = Rc::clone(&work_in_progress);
    let current = { work_in_progress.borrow().alternate.clone() };
    if current.is_some() {
        // update
        work_in_progress.borrow_mut().child =
            reconcile_child_fibers(work_in_progress.clone(), current.clone(), children)
    } else {
        // mount
        work_in_progress.borrow_mut().child =
            mount_child_fibers(work_in_progress.clone(), None, children)
    }
}
