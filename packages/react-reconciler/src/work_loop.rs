use std::cell::RefCell;
use std::rc::Rc;

use bitflags::bitflags;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::js_sys::Function;

use scheduler::{
    unstable_cancel_callback, unstable_schedule_callback_no_delay, unstable_should_yield_to_host,
    Priority,
};
use shared::{is_dev, log};

use crate::begin_work::begin_work;
use crate::commit_work::{
    commit_hook_effect_list_destroy, commit_hook_effect_list_mount,
    commit_hook_effect_list_unmount, commit_layout_effects, commit_mutation_effects,
};
use crate::fiber::{FiberNode, FiberRootNode, PendingPassiveEffects, StateNode};
use crate::fiber_flags::{get_mutation_mask, get_passive_mask, Flags};
use crate::fiber_lanes::{get_highest_priority, lanes_to_scheduler_priority, merge_lanes, Lane};
use crate::sync_task_queue::{flush_sync_callbacks, schedule_sync_callback};
use crate::work_tags::WorkTag;
use crate::{COMPLETE_WORK, HOST_CONFIG};

bitflags! {
    #[derive(Debug, Clone)]
    pub struct ExecutionContext: u8 {
        const NoContext = 0b0000;
        const RenderContext = 0b0010;
        const CommitContext = 0b0100;
        const ChildDeletion = 0b00010000;
    }
}

impl PartialEq for ExecutionContext {
    fn eq(&self, other: &Self) -> bool {
        self.bits() == other.bits()
    }
}

static mut WORK_IN_PROGRESS: Option<Rc<RefCell<FiberNode>>> = None;
static mut WORK_IN_PROGRESS_ROOT_RENDER_LANE: Lane = Lane::NoLane;
static mut EXECUTION_CONTEXT: ExecutionContext = ExecutionContext::NoContext;
static mut ROOT_DOES_HAVE_PASSIVE_EFFECTS: bool = false;

static ROOT_INCOMPLETE: u8 = 1;
static ROOT_COMPLETED: u8 = 2;

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
        log!("mark_update_lane_from_fiber_to_root {:?}", p);
        let alternate = p.borrow().alternate.clone();
        if alternate.is_some() {
            let alternate = alternate.unwrap();
            let child_lanes = { alternate.borrow().child_lanes.clone() };
            alternate.borrow_mut().child_lanes = merge_lanes(child_lanes, lane.clone());
            log!("mark_update_lane_from_fiber_to_root alternate {:?}", p);
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

fn ensure_root_is_scheduled(root: Rc<RefCell<FiberRootNode>>) {
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
        let scheduler_priority = lanes_to_scheduler_priority(cur_priority.clone());
        let closure = Closure::wrap(Box::new(move |did_timeout_js_value: JsValue| {
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

fn render_root(root: Rc<RefCell<FiberRootNode>>, lanes: Lane, should_time_slice: bool) -> u8 {
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

    let prev_execution_context: ExecutionContext;
    unsafe {
        prev_execution_context = EXECUTION_CONTEXT.clone();
        EXECUTION_CONTEXT |= ExecutionContext::RenderContext;
    }

    prepare_fresh_stack(root.clone(), lanes.clone());

    loop {
        match if should_time_slice {
            work_loop_concurrent()
        } else {
            work_loop_sync()
        } {
            Ok(_) => {
                break;
            }
            Err(e) => unsafe {
                log!("work_loop error {:?}", e);
                WORK_IN_PROGRESS = None
            },
        };
    }

    // log!("render over {:?}", *root.clone().borrow());
    log!("render over {:?}", unsafe { WORK_IN_PROGRESS.clone() });
    // log!("render over");

    unsafe {
        EXECUTION_CONTEXT = prev_execution_context;
        WORK_IN_PROGRESS_ROOT_RENDER_LANE = Lane::NoLane;

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
    unsafe {
        if EXECUTION_CONTEXT.clone()
            & (ExecutionContext::RenderContext | ExecutionContext::CommitContext)
            != ExecutionContext::NoContext
        {
            panic!("No in React work process {:?}", EXECUTION_CONTEXT)
        }
    }

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
        if root.borrow().callback_node.as_ref().unwrap().id != cur_callback_node.unwrap().id {
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

        commit_root(root);
    } else {
        todo!("Unsupported status of concurrent render")
    }

    JsValue::undefined()
}

fn perform_sync_work_on_root(root: Rc<RefCell<FiberRootNode>>, lanes: Lane) {
    let next_lane = get_highest_priority(root.borrow().pending_lanes.clone());

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

        commit_root(root);
    } else {
        todo!("Unsupported status of sync render")
    }
}

fn flush_passive_effects(pending_passive_effects: Rc<RefCell<PendingPassiveEffects>>) -> bool {
    unsafe {
        if EXECUTION_CONTEXT
            .contains(ExecutionContext::RenderContext | ExecutionContext::CommitContext)
        {
            log!("Cannot execute useEffect callback in React work loop")
        }

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
        let prev_execution_context: ExecutionContext;
        unsafe {
            prev_execution_context = EXECUTION_CONTEXT.clone();
            EXECUTION_CONTEXT |= ExecutionContext::CommitContext;
        }

        // effect

        // 1/3: Before Mutation

        // 2/3: Mutation
        commit_mutation_effects(finished_work.clone(), root.clone());

        // Switch Fiber Tree
        cloned.borrow_mut().current = finished_work.clone();

        // 3/3: Layout
        commit_layout_effects(finished_work.clone(), root.clone());

        unsafe {
            EXECUTION_CONTEXT = prev_execution_context;
        }
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
            log!("work_loop_concurrent");
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
// }
