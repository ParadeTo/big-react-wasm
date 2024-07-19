use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsValue;

use shared::{derive_from_js_value, is_dev, log};
use web_sys::js_sys::Object;

use crate::child_fiber::{clone_child_fiblers, mount_child_fibers, reconcile_child_fibers};
use crate::fiber::{FiberNode, MemoizedState};
use crate::fiber_flags::Flags;
use crate::fiber_hooks::render_with_hooks;
use crate::fiber_lanes::{include_some_lanes, Lane};
use crate::update_queue::{process_update_queue, ReturnOfProcessUpdateQueue};
use crate::work_tags::WorkTag;

static mut DID_RECEIVE_UPDATE: bool = false;

pub fn mark_wip_received_update() {
    unsafe { DID_RECEIVE_UPDATE = true };
}

fn bailout_on_already_finished_work(
    wip: Rc<RefCell<FiberNode>>,
    render_lane: Lane,
) -> Option<Rc<RefCell<FiberNode>>> {
    // log!(
    //     "tag:{:?} child_lanes:{:?} render_lanes:{:?} result:{:?}",
    //     wip.borrow().tag,
    //     wip.borrow().child_lanes.clone(),
    //     render_lane,
    //     wip.borrow().child_lanes.clone() & render_lane.clone()
    // );
    if !include_some_lanes(wip.borrow().child_lanes.clone(), render_lane) {
        if is_dev() {
            log!("bailout the whole subtree {:?}", wip);
        }
        return None;
    }
    if is_dev() {
        log!("bailout current fiber {:?}", wip);
    }
    clone_child_fiblers(wip.clone());
    wip.borrow().child.clone()
}

fn check_scheduled_update_or_context(current: Rc<RefCell<FiberNode>>, render_lane: Lane) -> bool {
    let update_lanes = current.borrow().lanes.clone();
    if include_some_lanes(update_lanes, render_lane) {
        return true;
    }
    false
}

pub fn begin_work(
    work_in_progress: Rc<RefCell<FiberNode>>,
    render_lane: Lane,
) -> Result<Option<Rc<RefCell<FiberNode>>>, JsValue> {
    unsafe {
        DID_RECEIVE_UPDATE = false;
    };
    let current = work_in_progress.borrow().alternate.clone();

    if current.is_some() {
        let current = current.unwrap();
        let old_props = current.borrow().memoized_props.clone();
        let old_type = current.borrow()._type.clone();
        let new_props = work_in_progress.borrow().pending_props.clone();
        let new_type = work_in_progress.borrow()._type.clone();
        if !Object::is(&old_props, &new_props) || !Object::is(&old_type, &new_type) {
            unsafe { DID_RECEIVE_UPDATE = true }
        } else {
            let has_scheduled_update_or_context =
                check_scheduled_update_or_context(current.clone(), render_lane.clone());
            // The current fiber lane is not included in render_lane
            // TODO context
            if !has_scheduled_update_or_context {
                unsafe { DID_RECEIVE_UPDATE = false }
                // // if current.is_some() {
                // let c = current.clone();
                // log!(
                //     "current tag:{:?} lanes:{:?} child_lanes:{:?} render_lane:{:?}",
                //     c.borrow().tag,
                //     c.borrow().lanes,
                //     c.borrow().child_lanes,
                //     render_lane
                // );
                // // }
                return Ok(bailout_on_already_finished_work(
                    work_in_progress,
                    render_lane,
                ));
            }
        }
    }

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

    // let current = work_in_progress.borrow().alternate.clone();
    // if current.is_some()&& !d

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
