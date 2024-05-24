use std::cell::RefCell;
use std::rc::Rc;

use bitflags::bitflags;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen::closure::Closure;
use web_sys::js_sys::Function;

use scheduler::{Priority, unstable_schedule_callback_no_delay};
use shared::{is_dev, log};

use crate::{COMMIT_WORK, COMPLETE_WORK, HOST_CONFIG, HostConfig};
use crate::begin_work::begin_work;
use crate::commit_work::CommitWork;
use crate::fiber::{FiberNode, FiberRootNode, PendingPassiveEffects, StateNode};
use crate::fiber_flags::{Flags, get_mutation_mask, get_passive_mask};
use crate::fiber_lanes::{get_highest_priority, Lane, merge_lanes};
use crate::sync_task_queue::{flush_sync_callbacks, schedule_sync_callback};
use crate::work_tags::WorkTag;

bitflags! {
    #[derive(Debug, Clone)]
    pub struct ExecutionContext: u8 {
        const NoContext = 0b0000;
        const RenderContext = 0b0010;
        const CommitContext = 0b0100;
        const ChildDeletion = 0b00010000;
    }
}

static mut WORK_IN_PROGRESS: Option<Rc<RefCell<FiberNode>>> = None;
static mut WORK_IN_PROGRESS_ROOT_RENDER_LANE: Lane = Lane::NoLane;
static mut EXECUTION_CONTEXT: ExecutionContext = ExecutionContext::NoContext;
static mut ROOT_DOES_HAVE_PASSIVE_EFFECTS: bool = false;

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

    let node_lanes = { node.borrow().lanes.clone() };
    node.borrow_mut().lanes = merge_lanes(node_lanes, lane.clone());
    let alternate = node.borrow().alternate.clone();
    if alternate.is_some() {
        let alternate = alternate.unwrap();
        let alternate_lanes = { alternate.borrow().lanes.clone() };
        alternate.borrow_mut().lanes = merge_lanes(alternate_lanes, lane);
    }

    while parent.is_some() {
        node = parent.clone().unwrap();
        let rc = Rc::clone(&parent.unwrap());
        let rc_ref = rc.borrow();
        let next = match rc_ref._return.as_ref() {
            None => None,
            Some(node) => {
                let a = node.clone();
                Some(a)
            }
        };
        parent = next;
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
    let update_lane = get_highest_priority(root.borrow().pending_lanes.clone());
    if update_lane == Lane::NoLane {
        return;
    }
    if update_lane == Lane::SyncLane {
        if is_dev() {
            log!("Schedule in microtask, priority {:?}", update_lane);
        }
    }
    schedule_sync_callback(Box::new(move || {
        perform_sync_work_on_root(root_cloned.clone(), update_lane.clone());
    }));
    unsafe {
        HOST_CONFIG
            .as_ref()
            .unwrap()
            .schedule_microtask(Box::new(|| flush_sync_callbacks()));
    }
}

fn perform_sync_work_on_root(root: Rc<RefCell<FiberRootNode>>, lane: Lane) {
    let next_lane = get_highest_priority(root.borrow().pending_lanes.clone());
    log!("perform_sync_work_on_root {:?}", next_lane);
    if next_lane != Lane::SyncLane {
        ensure_root_is_scheduled(root.clone());
        return;
    }

    let prev_execution_context: ExecutionContext;
    unsafe {
        prev_execution_context = EXECUTION_CONTEXT.clone();
        EXECUTION_CONTEXT |= ExecutionContext::RenderContext;
    }

    prepare_fresh_stack(root.clone(), lane.clone());

    loop {
        match work_loop() {
            Ok(_) => {
                break;
            }
            Err(e) => unsafe {
                log!("work_loop error {:?}", e);
                WORK_IN_PROGRESS = None
            },
        };
    }

    log!("{:?}", *root.clone().borrow());

    unsafe {
        EXECUTION_CONTEXT = prev_execution_context;
        WORK_IN_PROGRESS_ROOT_RENDER_LANE = Lane::NoLane;
    }
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
    root.clone().borrow_mut().finished_lane = lane;

    commit_root(root);
}

fn flush_passive_effects(pending_passive_effects: Rc<RefCell<PendingPassiveEffects>>) {
    unsafe {
        if EXECUTION_CONTEXT
            .contains(ExecutionContext::RenderContext | ExecutionContext::CommitContext)
        {
            log!("Cannot execute useEffect callback in React work loop")
        }

        for effect in &pending_passive_effects.borrow().unmount {
            CommitWork::commit_hook_effect_list_destroy(Flags::Passive, effect.clone());
        }
        pending_passive_effects.borrow_mut().unmount = vec![];

        for effect in &pending_passive_effects.borrow().update {
            CommitWork::commit_hook_effect_list_unmount(
                Flags::Passive | Flags::HookHasEffect,
                effect.clone(),
            );
        }
        for effect in &pending_passive_effects.borrow().update {
            CommitWork::commit_hook_effect_list_mount(
                Flags::Passive | Flags::HookHasEffect,
                effect.clone(),
            );
        }
        pending_passive_effects.borrow_mut().update = vec![];
    }
}

fn commit_root(root: Rc<RefCell<FiberRootNode>>) {
    let cloned = root.clone();
    if cloned.borrow().finished_work.is_none() {
        return;
    }
    let lane = root.borrow().finished_lane.clone();

    let finished_work = cloned.borrow().finished_work.clone().unwrap();
    cloned.borrow_mut().finished_work = None;
    cloned.borrow_mut().finished_lane = Lane::NoLane;

    cloned.borrow_mut().mark_root_finished(lane.clone());

    if lane == Lane::NoLane {
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

        unsafe {
            COMMIT_WORK
                .as_mut()
                .unwrap()
                .commit_mutation_effects(finished_work.clone(), root.clone());
        }

        cloned.borrow_mut().current = finished_work.clone();

        unsafe {
            EXECUTION_CONTEXT |= prev_execution_context;
        }
    } else {
        cloned.borrow_mut().current = finished_work.clone();
    }

    unsafe {
        ROOT_DOES_HAVE_PASSIVE_EFFECTS = false;
    }
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

fn work_loop() -> Result<(), JsValue> {
    unsafe {
        while WORK_IN_PROGRESS.is_some() {
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
