use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::js_sys::{Function, Object};

use scheduler::{
    unstable_cancel_callback, unstable_schedule_callback_no_delay, unstable_should_yield_to_host,
    Priority,
};
use shared::{derive_from_js_value, is_dev, log, type_of};

use crate::begin_work::begin_work;
use crate::commit_work::{
    commit_hook_effect_list_destroy, commit_hook_effect_list_mount,
    commit_hook_effect_list_unmount, commit_layout_effects, commit_mutation_effects,
};
use crate::fiber::{FiberNode, FiberRootNode, PendingPassiveEffects, StateNode};
use crate::fiber_flags::{get_host_effect_mask, get_mutation_mask, get_passive_mask, Flags};
use crate::fiber_hooks::reset_hooks_on_unwind;
use crate::fiber_lanes::{
    get_highest_priority, lanes_to_scheduler_priority, mark_root_suspended, merge_lanes, Lane,
};
use crate::fiber_throw::throw_exception;
use crate::fiber_unwind_work::unwind_work;
use crate::sync_task_queue::{flush_sync_callbacks, schedule_sync_callback};
use crate::thenable::{get_suspense_thenable, SUSPENSE_EXCEPTION};
use crate::work_tags::WorkTag;
use crate::{COMPLETE_WORK, HOST_CONFIG};

static mut WORK_IN_PROGRESS: Option<Rc<RefCell<FiberNode>>> = None;
static mut WORK_IN_PROGRESS_ROOT_RENDER_LANE: Lane = Lane::NoLane;
static mut ROOT_DOES_HAVE_PASSIVE_EFFECTS: bool = false;
static mut WORK_IN_PROGRESS_ROOT_EXIT_STATUS: u8 = ROOT_IN_PROGRESS;
static mut WORK_IN_PROGRESS_SUSPENDED_REASON: u8 = NOT_SUSPENDED;
static mut WORK_IN_PROGRESS_THROWN_VALUE: Option<JsValue> = None;

static ROOT_IN_PROGRESS: u8 = 0;
static ROOT_INCOMPLETE: u8 = 1;
static ROOT_COMPLETED: u8 = 2;
static ROOT_DID_NOT_COMPLETE: u8 = 3;

static NOT_SUSPENDED: u8 = 0;
static SUSPENDED_ON_ERROR: u8 = 1;
static SUSPENDED_ON_DATA: u8 = 2;
static SUSPENDED_ON_DEPRECATED_THROW_PROMISE: u8 = 4;

pub fn schedule_update_on_fiber(fiber: Rc<RefCell<FiberNode>>, lane: Lane) {
    if is_dev() {
        log!("schedule_update_on_fiber, {:?} {:?}", fiber, lane);
    }

    let root = mark_update_lane_from_fiber_to_root(fiber, lane.clone());
    if root.is_none() {
        return;
    }
    root.as_ref().unwrap().borrow_mut().mark_root_updated(lane);
    ensure_root_is_scheduled(root.unwrap())
}

pub fn mark_update_lane_from_fiber_to_root(
    fiber: Rc<RefCell<FiberNode>>,
    lane: Lane,
) -> Option<Rc<RefCell<FiberRootNode>>> {
    let mut node = Rc::clone(&fiber);
    let mut parent = Rc::clone(&fiber).borrow()._return.clone();

    while parent.is_some() {
        let p = parent.clone().unwrap();
        let child_lanes = { p.borrow().child_lanes.clone() };
        p.borrow_mut().child_lanes = merge_lanes(child_lanes, lane.clone());
        let alternate = p.borrow().alternate.clone();
        if alternate.is_some() {
            let alternate = alternate.unwrap();
            let child_lanes = { alternate.borrow().child_lanes.clone() };
            alternate.borrow_mut().child_lanes = merge_lanes(child_lanes, lane.clone());
        }

        node = parent.clone().unwrap();
        parent = match parent.clone().unwrap().borrow()._return.as_ref() {
            None => None,
            Some(node) => {
                let a = node.clone();
                Some(a)
            }
        };
    }

    let fiber_node_rc = Rc::clone(&node);
    let fiber_node = fiber_node_rc.borrow();
    if fiber_node.tag == WorkTag::HostRoot {
        if let Some(state_node) = fiber_node.state_node.clone() {
            if let StateNode::FiberRootNode(fiber_root_node) = &*(state_node.clone()) {
                return Some(Rc::clone(fiber_root_node));
            }
        }
    }

    None
}

pub fn ensure_root_is_scheduled(root: Rc<RefCell<FiberRootNode>>) {
    let root_cloned = root.clone();
    let update_lanes = root_cloned.borrow().get_next_lanes();
    let existing_callback = root_cloned.borrow().callback_node.clone();
    if update_lanes == Lane::NoLane {
        if existing_callback.is_some() {
            unstable_cancel_callback(existing_callback.unwrap())
        }
        root.borrow_mut().callback_node = None;
        root.borrow_mut().callback_priority = Lane::NoLane;
        return;
    }

    let cur_priority = get_highest_priority(update_lanes.clone());
    let prev_priority = root.borrow().callback_priority.clone();

    if cur_priority == prev_priority {
        // 有更新在进行，比较该更新与正在进行的更新的优先级
        // 如果优先级相同，则不需要调度新的，退出调度
        return;
    }

    if existing_callback.is_some() {
        unstable_cancel_callback(existing_callback.unwrap())
    }

    let mut new_callback_node = None;
    // 如果使用Scheduler调度，则会存在新的callbackNode，用React微任务调度不会存在
    if cur_priority == Lane::SyncLane {
        if is_dev() {
            log!("Schedule in microtask, priority {:?}", update_lanes);
        }
        schedule_sync_callback(Box::new(move || {
            perform_sync_work_on_root(root_cloned.clone(), update_lanes.clone());
        }));
        unsafe {
            HOST_CONFIG
                .as_ref()
                .unwrap()
                .schedule_microtask(Box::new(|| flush_sync_callbacks()));
        }
    } else {
        if is_dev() {
            log!("Schedule in macrotask, priority {:?}", update_lanes);
        }
        let scheduler_priority = lanes_to_scheduler_priority(cur_priority.clone());
        let closure = Closure::wrap(Box::new(move |did_timeout_js_value: JsValue| {
            log!("did_timeout_js_value1 {:?}", did_timeout_js_value);
            let did_timeout = did_timeout_js_value.as_bool().unwrap();
            perform_concurrent_work_on_root(root_cloned.clone(), did_timeout)
        }) as Box<dyn Fn(JsValue) -> JsValue>);
        let function = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        new_callback_node = Some(unstable_schedule_callback_no_delay(
            scheduler_priority,
            function,
        ))
    }

    root.borrow_mut().callback_node = new_callback_node;
    root.borrow_mut().callback_priority = cur_priority;
}

fn render_root(root: Rc<RefCell<FiberRootNode>>, lane: Lane, should_time_slice: bool) -> u8 {
    if is_dev() {
        log!(
            "Start {:?} render",
            if should_time_slice {
                "concurrent"
            } else {
                "sync"
            }
        );
    }

    if unsafe { WORK_IN_PROGRESS_ROOT_RENDER_LANE != lane } {
        prepare_fresh_stack(root.clone(), lane.clone());
    }

    loop {
        unsafe {
            if WORK_IN_PROGRESS_SUSPENDED_REASON != NOT_SUSPENDED && WORK_IN_PROGRESS.is_some() {
                let thrown_value = WORK_IN_PROGRESS_THROWN_VALUE.clone().unwrap();

                WORK_IN_PROGRESS_SUSPENDED_REASON = NOT_SUSPENDED;
                WORK_IN_PROGRESS_THROWN_VALUE = None;

                // TODO
                mark_update_lane_from_fiber_to_root(
                    WORK_IN_PROGRESS.clone().unwrap(),
                    lane.clone(),
                );

                throw_and_unwind_work_loop(
                    root.clone(),
                    WORK_IN_PROGRESS.clone().unwrap(),
                    thrown_value,
                    lane.clone(),
                );
            }
        }
        match if should_time_slice {
            work_loop_concurrent()
        } else {
            work_loop_sync()
        } {
            Ok(_) => {
                break;
            }
            Err(e) => {
                log!("e {:?}", e);
                handle_throw(root.clone(), e)
            }
        };
    }

    log!("render over {:?}", *root.clone().borrow());

    unsafe {
        // WORK_IN_PROGRESS_ROOT_RENDER_LANE = Lane::NoLane;

        if should_time_slice && WORK_IN_PROGRESS.is_some() {
            return ROOT_INCOMPLETE;
        }

        if !should_time_slice && WORK_IN_PROGRESS.is_some() {
            log!("The WIP is not null when render finishing")
        }
    }

    ROOT_COMPLETED
}

fn perform_concurrent_work_on_root(root: Rc<RefCell<FiberRootNode>>, did_timeout: bool) -> JsValue {
    // 开始执行具体工作前，保证上一次的useEffct都执行了
    // 同时要注意useEffect执行时触发的更新优先级是否大于当前更新的优先级
    let did_flush_passive_effects =
        flush_passive_effects(root.borrow().pending_passive_effects.clone());
    let cur_callback_node = root.borrow().callback_node.clone();

    // 这个分支好像走不到
    // if did_flush_passive_effects {
    //     if root.borrow().callback_node.unwrap().id != cur_callback_node.unwrap().id {
    //         // 调度了更高优更新，这个更新已经被取消了
    //         return null;
    //     }
    // }

    let lanes = root.borrow().get_next_lanes();
    if lanes == Lane::NoLane {
        return JsValue::undefined();
    }

    let should_time_slice = !did_timeout;
    let exit_status = render_root(root.clone(), lanes.clone(), should_time_slice);

    ensure_root_is_scheduled(root.clone());
    if exit_status == ROOT_INCOMPLETE {
        if root.borrow().callback_node.clone().unwrap().borrow().id
            != cur_callback_node.unwrap().borrow().id
        {
            // 调度了更高优更新，这个更新已经被取消了
            return JsValue::undefined();
        }
        let root_cloned = root.clone();
        let closure = Closure::wrap(Box::new(move |did_timeout_js_value: JsValue| {
            let did_timeout = did_timeout_js_value.as_bool().unwrap();
            perform_concurrent_work_on_root(root_cloned.clone(), did_timeout)
        }) as Box<dyn Fn(JsValue) -> JsValue>);
        let function = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        return function.into();
    }

    if exit_status == ROOT_COMPLETED {
        let finished_work = {
            root.clone()
                .borrow()
                .current
                .clone()
                .borrow()
                .alternate
                .clone()
        };
        root.clone().borrow_mut().finished_work = finished_work;
        root.clone().borrow_mut().finished_lanes = lanes;
        unsafe { WORK_IN_PROGRESS_ROOT_RENDER_LANE = Lane::NoLane };
        commit_root(root);
    } else {
        todo!("Unsupported status of concurrent render")
    }

    JsValue::undefined()
}

fn perform_sync_work_on_root(root: Rc<RefCell<FiberRootNode>>, lanes: Lane) {
    let next_lane = root.borrow().get_next_lanes();

    if next_lane != Lane::SyncLane {
        ensure_root_is_scheduled(root.clone());
        return;
    }

    let exit_status = render_root(root.clone(), lanes.clone(), false);

    if exit_status == ROOT_COMPLETED {
        let finished_work = {
            root.clone()
                .borrow()
                .current
                .clone()
                .borrow()
                .alternate
                .clone()
        };
        root.clone().borrow_mut().finished_work = finished_work;
        root.clone().borrow_mut().finished_lanes = lanes;
        unsafe { WORK_IN_PROGRESS_ROOT_RENDER_LANE = Lane::NoLane };
        commit_root(root);
    } else if exit_status == ROOT_DID_NOT_COMPLETE {
        // unsafe { WORK_IN_PROGRESS_ROOT_RENDER_LANE = Lane::NoLane };
        mark_root_suspended(root.clone(), next_lane);
        ensure_root_is_scheduled(root.clone());
    } else {
        todo!("Unsupported status of sync render")
    }
}

fn flush_passive_effects(pending_passive_effects: Rc<RefCell<PendingPassiveEffects>>) -> bool {
    unsafe {
        let mut did_flush_passive_effects = false;
        for effect in &pending_passive_effects.borrow().unmount {
            did_flush_passive_effects = true;
            commit_hook_effect_list_destroy(Flags::Passive, effect.clone());
        }
        pending_passive_effects.borrow_mut().unmount = vec![];

        for effect in &pending_passive_effects.borrow().update {
            did_flush_passive_effects = true;
            commit_hook_effect_list_unmount(Flags::Passive | Flags::HookHasEffect, effect.clone());
        }
        for effect in &pending_passive_effects.borrow().update {
            did_flush_passive_effects = true;
            commit_hook_effect_list_mount(Flags::Passive | Flags::HookHasEffect, effect.clone());
        }
        pending_passive_effects.borrow_mut().update = vec![];
        flush_sync_callbacks();
        did_flush_passive_effects
    }
}

fn commit_root(root: Rc<RefCell<FiberRootNode>>) {
    let cloned = root.clone();
    if cloned.borrow().finished_work.is_none() {
        return;
    }
    let lanes = root.borrow().finished_lanes.clone();

    let finished_work = cloned.borrow().finished_work.clone().unwrap();
    cloned.borrow_mut().finished_work = None;
    cloned.borrow_mut().finished_lanes = Lane::NoLane;
    cloned.borrow_mut().callback_node = None;
    cloned.borrow_mut().callback_priority = Lane::NoLane;

    cloned.borrow_mut().mark_root_finished(lanes.clone());

    if lanes == Lane::NoLane {
        log!("Commit phase finished lane should not be NoLane")
    }

    let subtree_flags = finished_work.borrow().subtree_flags.clone();
    let flags = finished_work.borrow().flags.clone();

    // useEffect
    let root_cloned = root.clone();
    let passive_mask = get_passive_mask();
    if flags.clone() & passive_mask.clone() != Flags::NoFlags
        || subtree_flags.clone() & passive_mask != Flags::NoFlags
    {
        if unsafe { !ROOT_DOES_HAVE_PASSIVE_EFFECTS } {
            unsafe { ROOT_DOES_HAVE_PASSIVE_EFFECTS = true }
            let closure = Closure::wrap(Box::new(move || {
                flush_passive_effects(root_cloned.borrow().pending_passive_effects.clone());
            }) as Box<dyn Fn()>);
            let function = closure.as_ref().unchecked_ref::<Function>().clone();
            closure.forget();
            unstable_schedule_callback_no_delay(Priority::NormalPriority, function);
        }
    }

    let subtree_has_effect = get_mutation_mask().contains(subtree_flags);
    let root_has_effect = get_mutation_mask().contains(flags);

    if subtree_has_effect || root_has_effect {
        // effect

        // 1/3: Before Mutation

        // 2/3: Mutation
        commit_mutation_effects(finished_work.clone(), root.clone());

        // Switch Fiber Tree
        cloned.borrow_mut().current = finished_work.clone();

        // 3/3: Layout
        commit_layout_effects(finished_work.clone(), root.clone());
    } else {
        cloned.borrow_mut().current = finished_work.clone();
    }

    unsafe {
        ROOT_DOES_HAVE_PASSIVE_EFFECTS = false;
    }
    ensure_root_is_scheduled(root);
}

fn prepare_fresh_stack(root: Rc<RefCell<FiberRootNode>>, lane: Lane) {
    let root = root.clone();
    unsafe {
        WORK_IN_PROGRESS = Some(FiberNode::create_work_in_progress(
            root.borrow().current.clone(),
            JsValue::null(),
        ));
        WORK_IN_PROGRESS_ROOT_RENDER_LANE = lane;

        WORK_IN_PROGRESS_ROOT_EXIT_STATUS = ROOT_IN_PROGRESS;
        WORK_IN_PROGRESS_SUSPENDED_REASON = NOT_SUSPENDED;
        WORK_IN_PROGRESS_THROWN_VALUE = None;
    }
}

fn work_loop_sync() -> Result<(), JsValue> {
    unsafe {
        while WORK_IN_PROGRESS.is_some() {
            perform_unit_of_work(WORK_IN_PROGRESS.clone().unwrap())?;
        }
    }
    Ok(())
}

fn work_loop_concurrent() -> Result<(), JsValue> {
    unsafe {
        while WORK_IN_PROGRESS.is_some() && !unstable_should_yield_to_host() {
            perform_unit_of_work(WORK_IN_PROGRESS.clone().unwrap())?;
        }
    }
    Ok(())
}

fn perform_unit_of_work(fiber: Rc<RefCell<FiberNode>>) -> Result<(), JsValue> {
    let next = begin_work(fiber.clone(), unsafe {
        WORK_IN_PROGRESS_ROOT_RENDER_LANE.clone()
    })?;
    let pending_props = { fiber.clone().borrow().pending_props.clone() };
    fiber.clone().borrow_mut().memoized_props = pending_props;
    if next.is_none() {
        complete_unit_of_work(fiber.clone());
    } else {
        // self.work_in_progress = Some(next.unwrap());
        unsafe { WORK_IN_PROGRESS = Some(next.unwrap()) }
    }
    Ok(())
}

fn complete_unit_of_work(fiber: Rc<RefCell<FiberNode>>) {
    let mut node: Option<Rc<RefCell<FiberNode>>> = Some(fiber);

    unsafe {
        loop {
            let next = COMPLETE_WORK
                .as_ref()
                .unwrap()
                .complete_work(node.clone().unwrap().clone());

            if next.is_some() {
                // self.work_in_progress = next.clone();
                WORK_IN_PROGRESS = next.clone();
                return;
            }

            let sibling = node.clone().unwrap().clone().borrow().sibling.clone();
            if sibling.is_some() {
                // self.work_in_progress = next.clone();
                WORK_IN_PROGRESS = sibling.clone();
                return;
            }

            let _return = node.clone().unwrap().clone().borrow()._return.clone();
            // log!("complete_unit_of_work {:?} {:?}", node, _return);
            if _return.is_none() {
                // node = None;
                // self.work_in_progress = None;
                WORK_IN_PROGRESS = None;
                break;
            } else {
                node = _return;
                // self.work_in_progress = node.clone();
                WORK_IN_PROGRESS = node.clone();
            }
        }
    }
}

fn handle_throw(root: Rc<RefCell<FiberRootNode>>, mut thrown_value: JsValue) {
    /*
        throw possibilities:
            1. use thenable
            2. error (Error Boundary)
    */
    if Object::is(&thrown_value, &SUSPENSE_EXCEPTION) {
        unsafe { WORK_IN_PROGRESS_SUSPENDED_REASON = SUSPENDED_ON_DATA };
        thrown_value = get_suspense_thenable();
    } else {
        let is_wakeable = !thrown_value.is_null()
            && type_of(&thrown_value, "object")
            && derive_from_js_value(&thrown_value, "then").is_function();
        unsafe {
            WORK_IN_PROGRESS_SUSPENDED_REASON = if is_wakeable {
                SUSPENDED_ON_DEPRECATED_THROW_PROMISE
            } else {
                SUSPENDED_ON_ERROR
            };
        };
    }

    unsafe {
        WORK_IN_PROGRESS_THROWN_VALUE = Some(thrown_value);
    }
}

fn throw_and_unwind_work_loop(
    root: Rc<RefCell<FiberRootNode>>,
    unit_of_work: Rc<RefCell<FiberNode>>,
    thrown_value: JsValue,
    lane: Lane,
) {
    reset_hooks_on_unwind(unit_of_work.clone());
    throw_exception(
        root.clone(),
        unit_of_work.clone(),
        thrown_value,
        lane.clone(),
    );
    unwind_unit_of_work(unit_of_work);
}

fn unwind_unit_of_work(unit_of_work: Rc<RefCell<FiberNode>>) {
    let mut incomplete_work = Some(unit_of_work);
    loop {
        let unwrapped_work = incomplete_work.clone().unwrap();
        let next = unwind_work(unwrapped_work.clone());
        if next.is_some() {
            let next = next.unwrap();
            next.borrow_mut().flags &= get_host_effect_mask();
            unsafe { WORK_IN_PROGRESS = Some(next) };
            return;
        }

        let return_fiber = unwrapped_work.borrow()._return.clone();
        if return_fiber.is_some() {
            let return_fiber = return_fiber.clone().unwrap();
            // Todo why
            return_fiber.borrow_mut().deletions = vec![];
        }

        incomplete_work = return_fiber.clone();

        if incomplete_work.is_none() {
            break;
        }
    }

    unsafe {
        WORK_IN_PROGRESS = None;
        WORK_IN_PROGRESS_ROOT_EXIT_STATUS = ROOT_DID_NOT_COMPLETE;
    }
}
