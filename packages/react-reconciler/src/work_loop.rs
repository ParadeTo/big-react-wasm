use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsValue;

use shared::{is_dev, log};

use crate::{COMPLETE_WORK, HOST_CONFIG, HostConfig};
use crate::begin_work::begin_work;
use crate::commit_work::CommitWork;
use crate::fiber::{FiberNode, FiberRootNode, StateNode};
use crate::fiber_flags::get_mutation_mask;
use crate::fiber_lanes::{get_highest_priority, Lane, merge_lanes};
use crate::sync_task_queue::{flush_sync_callbacks, schedule_sync_callback};
use crate::work_tags::WorkTag;

static mut WORK_IN_PROGRESS: Option<Rc<RefCell<FiberNode>>> = None;
static mut WORK_IN_PROGRESS_ROOT_RENDER_LANE: Lane = Lane::NoLane;

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
        HOST_CONFIG.as_ref().unwrap()
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

    unsafe { WORK_IN_PROGRESS_ROOT_RENDER_LANE = Lane::NoLane; }
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

    let subtree_has_effect =
        get_mutation_mask().contains(finished_work.clone().borrow().subtree_flags.clone());
    let root_has_effect =
        get_mutation_mask().contains(finished_work.clone().borrow().flags.clone());

    let commit_work = &mut CommitWork::new(unsafe { HOST_CONFIG.clone().unwrap() });
    if subtree_has_effect || root_has_effect {
        commit_work.commit_mutation_effects(finished_work.clone());
        cloned.borrow_mut().current = finished_work.clone();
    } else {
        cloned.borrow_mut().current = finished_work.clone();
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
    // while self.work_in_progress.is_some() {
    //     self.perform_unit_of_work(self.work_in_progress.clone().unwrap())?;
    // }
    unsafe {
        while WORK_IN_PROGRESS.is_some() {
            perform_unit_of_work(WORK_IN_PROGRESS.clone().unwrap())?;
        }
    }
    Ok(())
}

fn perform_unit_of_work(fiber: Rc<RefCell<FiberNode>>) -> Result<(), JsValue> {
    let next = begin_work(fiber.clone(), unsafe { WORK_IN_PROGRESS_ROOT_RENDER_LANE.clone() })?;
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
                .as_ref().unwrap().complete_work(node.clone().unwrap().clone());

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
