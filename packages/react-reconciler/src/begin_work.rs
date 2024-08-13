use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::{JsCast, JsValue};

use shared::{derive_from_js_value, is_dev, log, shallow_equal};
use web_sys::js_sys::{Function, Object};

use crate::child_fiber::{clone_child_fiblers, mount_child_fibers, reconcile_child_fibers};
use crate::fiber::{FiberNode, MemoizedState};
use crate::fiber_context::{prepare_to_read_context, propagate_context_change, push_provider};
use crate::fiber_flags::Flags;
use crate::fiber_hooks::{bailout_hook, render_with_hooks};
use crate::fiber_lanes::{include_some_lanes, Lane};
use crate::suspense_context::push_suspense_handler;
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
    log!("begin_work {:?}", work_in_progress);
    unsafe {
        DID_RECEIVE_UPDATE = false;
    };
    let current = { work_in_progress.borrow().alternate.clone() };

    if current.is_some() {
        let current = current.clone().unwrap();
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
                match work_in_progress.borrow().tag {
                    WorkTag::ContextProvider => {
                        let new_value = derive_from_js_value(
                            &work_in_progress.borrow().memoized_props,
                            "value",
                        );
                        let context =
                            derive_from_js_value(&work_in_progress.borrow()._type, "_context");
                        push_provider(&context, new_value);
                    }
                    _ => {}
                }
                return Ok(bailout_on_already_finished_work(
                    work_in_progress,
                    render_lane,
                ));
            }
        }
    }

    work_in_progress.borrow_mut().lanes = Lane::NoLane;
    // if current.is_some() {
    //     let current = current.clone().unwrap();
    //     current.borrow_mut().lanes = Lane::NoLane;
    // }

    let tag = { work_in_progress.clone().borrow().tag.clone() };
    return match tag {
        WorkTag::FunctionComponent => {
            let Component = { work_in_progress.borrow()._type.clone() };
            update_function_component(work_in_progress.clone(), Component, render_lane)
        }
        WorkTag::HostRoot => Ok(update_host_root(work_in_progress.clone(), render_lane)),
        WorkTag::HostComponent => Ok(update_host_component(work_in_progress.clone())),
        WorkTag::HostText => Ok(None),
        WorkTag::ContextProvider => Ok(update_context_provider(
            work_in_progress.clone(),
            render_lane.clone(),
        )),
        WorkTag::MemoComponent => update_memo_component(work_in_progress.clone(), render_lane),
        WorkTag::Fragment => Ok(update_fragment(work_in_progress.clone())),
        WorkTag::SuspenseComponent => todo!(),
    };
}

fn mount_suspense_fallback_children(
    work_in_progress: Rc<RefCell<FiberNode>>,
    primary_children: JsValue,
    fallback_children: JsValue,
) {
    // let primary_child_props
}

fn update_suspense_component(work_in_progress: Rc<RefCell<FiberNode>>) {
    let current = { work_in_progress.borrow().alternate.clone() };
    let next_props = { work_in_progress.borrow().pending_props.clone() };

    let mut show_fallback = false;
    let did_suspend =
        (work_in_progress.borrow().flags.clone() & Flags::DidCapture) != Flags::NoFlags;

    if did_suspend {
        show_fallback = true;
        work_in_progress.borrow_mut().flags -= Flags::DidCapture;
    }

    let next_primary_children = derive_from_js_value(&next_props, "children");
    let next_fallback_children = derive_from_js_value(&next_props, "fallback");
    push_suspense_handler(work_in_progress.clone());

    if current.is_none() {
        if show_fallback {
            return mount_suspense_fallback_children(
                work_in_progress.clone(),
                next_primary_children.clone(),
                next_fallback_children.clone(),
            );
        }
    }
}

fn update_fragment(work_in_progress: Rc<RefCell<FiberNode>>) -> Option<Rc<RefCell<FiberNode>>> {
    let next_children = work_in_progress.borrow().pending_props.clone();
    reconcile_children(work_in_progress.clone(), Some(next_children));
    work_in_progress.borrow().child.clone()
}

fn update_memo_component(
    work_in_progress: Rc<RefCell<FiberNode>>,
    render_lane: Lane,
) -> Result<Option<Rc<RefCell<FiberNode>>>, JsValue> {
    let current = { work_in_progress.borrow().alternate.clone() };
    let next_props = { work_in_progress.borrow().pending_props.clone() };

    if current.is_some() {
        let current = current.unwrap();
        let prev_props = current.borrow().memoized_props.clone();
        if !check_scheduled_update_or_context(current.clone(), render_lane.clone()) {
            let mut props_equal = false;
            let compare = derive_from_js_value(&work_in_progress.borrow()._type, "compare");
            if compare.is_function() {
                let f = compare.dyn_ref::<Function>().unwrap();
                props_equal = f
                    .call2(&JsValue::null(), &prev_props, &next_props)
                    .unwrap()
                    .as_bool()
                    .unwrap();
            } else {
                props_equal = shallow_equal(&prev_props, &next_props);
            }

            if props_equal && Object::is(&current.borrow()._ref, &work_in_progress.borrow()._ref) {
                unsafe { DID_RECEIVE_UPDATE = false };
                work_in_progress.borrow_mut().pending_props = prev_props;
                work_in_progress.borrow_mut().lanes = current.borrow().lanes.clone();
                return Ok(bailout_on_already_finished_work(
                    work_in_progress.clone(),
                    render_lane,
                ));
            }
        }
    }
    let Component = { derive_from_js_value(&work_in_progress.borrow()._type, "type") };
    update_function_component(work_in_progress.clone(), Component, render_lane)
}

fn update_context_provider(
    work_in_progress: Rc<RefCell<FiberNode>>,
    render_lane: Lane,
) -> Option<Rc<RefCell<FiberNode>>> {
    let provider_type = { work_in_progress.borrow()._type.clone() };
    let context = derive_from_js_value(&provider_type, "_context");
    let new_props = { work_in_progress.borrow().pending_props.clone() };
    let old_props = { work_in_progress.borrow().memoized_props.clone() };
    let new_value = derive_from_js_value(&new_props, "value");

    push_provider(&context, derive_from_js_value(&new_props, "value"));

    if !old_props.is_null() {
        let old_value = derive_from_js_value(&old_props, "value");
        if Object::is(&old_value, &new_value)
            && Object::is(
                &derive_from_js_value(&old_props, "children"),
                &derive_from_js_value(&new_props, "children"),
            )
        {
            return bailout_on_already_finished_work(work_in_progress.clone(), render_lane);
        } else {
            propagate_context_change(work_in_progress.clone(), context, render_lane);
        }
    }

    let next_children = derive_from_js_value(&new_props, "children");
    reconcile_children(work_in_progress.clone(), Some(next_children));
    work_in_progress.clone().borrow().child.clone()
}

fn update_function_component(
    work_in_progress: Rc<RefCell<FiberNode>>,
    Component: JsValue,
    render_lane: Lane,
) -> Result<Option<Rc<RefCell<FiberNode>>>, JsValue> {
    prepare_to_read_context(work_in_progress.clone(), render_lane.clone());
    let next_children =
        render_with_hooks(work_in_progress.clone(), Component, render_lane.clone())?;

    let current = work_in_progress.borrow().alternate.clone();
    log!("{:?} {:?}", work_in_progress.clone(), unsafe {
        DID_RECEIVE_UPDATE
    });
    if current.is_some() && unsafe { !DID_RECEIVE_UPDATE } {
        bailout_hook(work_in_progress.clone(), render_lane.clone());
        return Ok(bailout_on_already_finished_work(
            work_in_progress,
            render_lane,
        ));
    }

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

    let prev_children = { work_in_progress_cloned.borrow().memoized_state.clone() };

    {
        work_in_progress
            .clone()
            .borrow_mut()
            .update_queue
            .clone()
            .unwrap()
            .borrow_mut()
            .shared
            .pending = None;
        let ReturnOfProcessUpdateQueue { memoized_state, .. } =
            process_update_queue(base_state, pending, render_lane.clone(), None);
        work_in_progress.clone().borrow_mut().memoized_state = memoized_state.clone();
        let current = { work_in_progress.borrow().alternate.clone() };
        if current.is_some() {
            let current = current.unwrap();
            if current.borrow().memoized_state.is_none() {
                current.borrow_mut().memoized_state = memoized_state;
            }
        }
    }

    let next_children = work_in_progress_cloned.borrow().memoized_state.clone();
    if next_children.is_none() {
        panic!("update_host_root next_children is none")
    }

    // let prev_children = prev_children.unwrap();
    if let Some(MemoizedState::MemoizedJsValue(prev_children)) = prev_children {
        if let Some(MemoizedState::MemoizedJsValue(next_children)) = next_children.clone() {
            if Object::is(&prev_children, &next_children) {
                return bailout_on_already_finished_work(
                    work_in_progress.clone(),
                    render_lane.clone(),
                );
            }
        }
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
