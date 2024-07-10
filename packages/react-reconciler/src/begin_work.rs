use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsValue;

use shared::derive_from_js_value;
use web_sys::js_sys::Object;

use crate::child_fiber::{mount_child_fibers, reconcile_child_fibers};
use crate::fiber::{FiberNode, MemoizedState};
use crate::fiber_flags::Flags;
use crate::fiber_hooks::render_with_hooks;
use crate::fiber_lanes::Lane;
use crate::update_queue::{process_update_queue, ReturnOfProcessUpdateQueue};
use crate::work_tags::WorkTag;

pub fn begin_work(
    work_in_progress: Rc<RefCell<FiberNode>>,
    render_lane: Lane,
) -> Result<Option<Rc<RefCell<FiberNode>>>, JsValue> {
    let tag = work_in_progress.clone().borrow().tag.clone();
    return match tag {
        WorkTag::FunctionComponent => {
            update_function_component(work_in_progress.clone(), render_lane)
        }
        WorkTag::HostRoot => Ok(update_host_root(work_in_progress.clone(), render_lane)),
        WorkTag::HostComponent => Ok(update_host_component(work_in_progress.clone())),
        WorkTag::HostText => Ok(None),
    };
}

fn update_function_component(
    work_in_progress: Rc<RefCell<FiberNode>>,
    render_lane: Lane,
) -> Result<Option<Rc<RefCell<FiberNode>>>, JsValue> {
    let next_children = render_with_hooks(work_in_progress.clone(), render_lane)?;
    reconcile_children(work_in_progress.clone(), Some(next_children));
    Ok(work_in_progress.clone().borrow().child.clone())
}

fn update_host_root(
    work_in_progress: Rc<RefCell<FiberNode>>,
    render_lane: Lane,
) -> Option<Rc<RefCell<FiberNode>>> {
    let work_in_progress_cloned = work_in_progress.clone();

    let base_state;
    let mut pending;
    {
        let work_in_progress_borrowed = work_in_progress_cloned.borrow();
        base_state = work_in_progress_borrowed.memoized_state.clone();
        pending = work_in_progress_borrowed
            .update_queue
            .clone()
            .unwrap()
            .borrow()
            .shared
            .pending
            .clone();
    }

    {
        let ReturnOfProcessUpdateQueue { memoized_state, .. } =
            process_update_queue(base_state, pending, render_lane);
        work_in_progress.clone().borrow_mut().memoized_state = memoized_state;
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

fn mark_ref(current: Option<Rc<RefCell<FiberNode>>>, work_in_progress: Rc<RefCell<FiberNode>>) {
    let _ref = { work_in_progress.borrow()._ref.clone() };
    if (current.is_none() && !_ref.is_null())
        || (current.is_some() && !Object::is(&current.as_ref().unwrap().borrow()._ref, &_ref))
    {
        work_in_progress.borrow_mut().flags |= Flags::Ref;
    }
}

fn update_host_component(
    work_in_progress: Rc<RefCell<FiberNode>>,
) -> Option<Rc<RefCell<FiberNode>>> {
    let work_in_progress = Rc::clone(&work_in_progress);

    let next_children = {
        let ref_fiber_node = work_in_progress.borrow();
        derive_from_js_value(&ref_fiber_node.pending_props, "children")
    };

    let alternate = { work_in_progress.borrow().alternate.clone() };
    mark_ref(alternate, work_in_progress.clone());

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
