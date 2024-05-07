use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsValue;

use shared::derive_from_js_value;

use crate::child_fiber::{mount_child_fibers, reconcile_child_fibers};
use crate::fiber::{FiberNode, MemoizedState};
use crate::fiber_hooks::render_with_hooks;
use crate::update_queue::process_update_queue;
use crate::work_tags::WorkTag;

pub fn begin_work(
    work_in_progress: Rc<RefCell<FiberNode>>,
) -> Result<Option<Rc<RefCell<FiberNode>>>, JsValue> {
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
    let next_children = render_with_hooks(work_in_progress.clone())?;
    reconcile_children(work_in_progress.clone(), Some(next_children));
    Ok(work_in_progress.clone().borrow().child.clone())
}

fn update_host_root(work_in_progress: Rc<RefCell<FiberNode>>) -> Option<Rc<RefCell<FiberNode>>> {
    let work_in_progress_cloned = work_in_progress.clone();

    let base_state;
    let update_queue;
    {
        let work_in_progress_borrowed = work_in_progress_cloned.borrow();
        base_state = work_in_progress_borrowed.memoized_state.clone();
        update_queue = work_in_progress_borrowed.update_queue.clone();
    }

    {
        work_in_progress.clone().borrow_mut().memoized_state =
            process_update_queue(base_state, update_queue, work_in_progress.clone());
    }

    let next_children = work_in_progress_cloned.borrow().memoized_state.clone();
    if next_children.is_none() {
        panic!("update_host_root next_children is none")
    }

    if let MemoizedState::MemoizedJsValue(next_children) = next_children.unwrap() {
        reconcile_children(work_in_progress.clone(), Some(next_children));
    }
    work_in_progress.clone().borrow().child.clone()
}

fn update_host_component(
    work_in_progress: Rc<RefCell<FiberNode>>,
) -> Option<Rc<RefCell<FiberNode>>> {
    let work_in_progress = Rc::clone(&work_in_progress);

    let next_children = {
        let ref_fiber_node = work_in_progress.borrow();
        derive_from_js_value(&ref_fiber_node.pending_props, "children")
    };

    {
        reconcile_children(work_in_progress.clone(), Some(next_children));
    }
    work_in_progress.clone().borrow().child.clone()
}

fn reconcile_children(work_in_progress: Rc<RefCell<FiberNode>>, children: Option<JsValue>) {
    let work_in_progress = Rc::clone(&work_in_progress);
    let current = { work_in_progress.borrow().alternate.clone() };
    if current.is_some() {
        // update
        work_in_progress.borrow_mut().child = reconcile_child_fibers(
            work_in_progress.clone(),
            current.clone().unwrap().clone().borrow().child.clone(),
            children,
        )
    } else {
        // mount
        work_in_progress.borrow_mut().child =
            mount_child_fibers(work_in_progress.clone(), None, children)
    }
}
