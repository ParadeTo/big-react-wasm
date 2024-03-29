use std::cell::RefCell;
use std::rc::{Rc, Weak};

use wasm_bindgen::JsValue;

use shared::log;

use crate::begin_work::begin_work;
use crate::fiber::{FiberNode, FiberRootNode, StateNode};
use crate::work_tags::WorkTag;

static mut WORK_IN_PROGRESS: Option<Weak<RefCell<FiberNode>>> = None;

pub fn schedule_update_on_fiber(fiber: Rc<RefCell<FiberNode>>) {
    let root = mark_update_lane_from_fiber_to_root(fiber);
    if root.is_none() {
        return;
    }
    ensure_root_is_scheduled(root.unwrap())
}

pub fn mark_update_lane_from_fiber_to_root(
    fiber: Rc<RefCell<FiberNode>>,
) -> Option<Rc<RefCell<FiberRootNode>>> {
    let mut node = Rc::clone(&fiber);
    let mut parent = Rc::clone(&fiber).borrow()._return.clone();

    while parent.is_some() {
        node = parent.clone().unwrap().upgrade().unwrap();
        let rc = Rc::clone(&parent.unwrap().upgrade().unwrap());
        let rc_ref = rc.borrow();
        let next = match rc_ref._return.as_ref() {
            None => {
                None
            }
            Some(node) => {
                let a = Rc::downgrade(&node.upgrade().unwrap());
                Some(a)
            }
        };
        parent = next;
    }

    let fiber_node_rc = Rc::clone(&node);
    let fiber_node = fiber_node_rc.borrow();
    if fiber_node.tag == WorkTag::HostRoot {
        match fiber_node.state_node.as_ref() {
            None => {}
            Some(state_node) => {
                return match state_node {
                    StateNode::FiberRootNode(fiber_root_node) => {
                        Some(Rc::clone(fiber_root_node))
                    }
                };
            }
        }
    }

    None
}

fn ensure_root_is_scheduled(root: Rc<RefCell<FiberRootNode>>) {
    perform_sync_work_on_root(root);
}

fn perform_sync_work_on_root(root: Rc<RefCell<FiberRootNode>>) {
    prepare_fresh_stack(Rc::clone(&root));

    loop {
        work_loop();
        break;
    }


    // commit
    log!("{:?}", Rc::clone(&root))
}

fn prepare_fresh_stack(root: Rc<RefCell<FiberRootNode>>) {
    let root = Rc::clone(&root);
    unsafe {
        WORK_IN_PROGRESS = Some(FiberNode::create_work_in_progress(
            root.borrow().current.upgrade().unwrap(),
            Rc::new(JsValue::null()),
        ));
    }
}

fn work_loop() {
    unsafe {
        while WORK_IN_PROGRESS.is_some() {
            perform_unit_of_work(WORK_IN_PROGRESS.clone().unwrap().upgrade().unwrap())
        }
    }
}

fn perform_unit_of_work(fiber: Rc<RefCell<FiberNode>>) {
    let next = begin_work(fiber.clone());
    if next.is_none() {
        complete_unit_of_work(fiber.clone())
    } else {
        unsafe {
            WORK_IN_PROGRESS = Some(Rc::downgrade(&next.unwrap()));
        }
    }
}

fn complete_unit_of_work(fiber: Rc<RefCell<FiberNode>>) {}