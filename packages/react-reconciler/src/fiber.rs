use std::cell::RefCell;
use std::ops::Deref;
use std::rc::{Rc, Weak};

use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

use crate::update_queue::{Update, UpdateQueue, UpdateQueueTrait, UpdateType};
use crate::work_tags::WorkTag;

trait Node {}

static mut WORK_IN_PROGRESS: Option<Weak<RefCell<FiberNode>>> = None;

#[derive(Debug, Clone)]
pub enum StateNode {
    FiberRootNode(Rc<RefCell<FiberRootNode>>),
}

#[derive(Debug, Clone)]
pub enum Flags {
    NoFlags = 0b00000000000000000000000000,
    Placement = 0b00000000000000000000000010,
    Update = 0b00000000000000000000000100,
    Deletion = 0b00000000000000000000001000,
}

#[derive(Debug, Clone)]
pub struct FiberNode {
    tag: WorkTag,
    pending_props: Option<JsValue>,
    key: Option<String>,
    pub state_node: Option<StateNode>,
    pub update_queue: Option<Weak<RefCell<UpdateQueue>>>,
    _return: Option<Weak<RefCell<FiberNode>>>,
    sibling: Option<Rc<RefCell<FiberNode>>>,
    child: Option<Rc<RefCell<FiberNode>>>,
    alternate: Option<Weak<RefCell<FiberNode>>>,
    _type: JsValue,
    flags: Flags,
    memoized_props: JsValue,
    memoized_state: JsValue,
}

impl Node for FiberNode {}

impl FiberNode {
    pub fn new(tag: WorkTag, pending_props: &JsValue, key: Option<String>) -> Self {
        Self {
            tag,
            pending_props: Some(pending_props.clone()),
            key,
            state_node: None,
            update_queue: None,
            _return: None,
            sibling: None,
            child: None,
            alternate: None,
            _type: JsValue::null(),
            memoized_props: JsValue::null(),
            memoized_state: JsValue::null(),
            flags: Flags::NoFlags,
        }
    }

    pub fn enqueue_update(&mut self, update: Update) {
        let mut update_queue = match &self.update_queue {
            None => {
                return;
            }
            Some(a) => {
                let b = a.upgrade().clone().unwrap().borrow();
                b
            }
        };

        let mut u = &mut update_queue;
        u.shared.pending = update;
    }

    pub fn initialize_update_queue(&mut self) {
        self.update_queue = Some(Rc::downgrade(&Rc::new(RefCell::new((UpdateQueue {
            shared: UpdateType {
                pending: Update { action: None },
            },
        })))));
    }

    pub fn schedule_update_on_fiber(fiber: Rc<RefCell<FiberNode>>) {
        let root = FiberNode::mark_update_lane_from_fiber_to_root(fiber);
        if root.is_none() {
            return;
        }
    }

    pub fn mark_update_lane_from_fiber_to_root(
        fiber: Rc<RefCell<FiberNode>>,
    ) -> Option<Rc<RefCell<FiberRootNode>>> {
        let mut node = Rc::clone(&fiber);
        let mut parent = Rc::clone(&fiber).borrow()._return.as_ref();

        while parent.is_some() {
            node = parent.unwrap().upgrade().unwrap();
            parent = Rc::clone(&parent.unwrap().upgrade().unwrap())
                .borrow()
                ._return
                .as_ref();
        }

        let node = Rc::clone(&node).borrow();
        let a = Rc::clone(&node).borrow().deref();
        if node.tag == WorkTag::HostRoot {
            match Rc::clone(&node).borrow().state_node {
                None => {}
                Some(state_node) => {
                    return match state_node {
                        StateNode::FiberRootNode(fiber_root_node) => Some(fiber_root_node),
                    };
                }
            }
        }

        None
    }

    pub fn create_work_in_progress(current: Rc<RefCell<FiberNode>>, pending_props: &JsValue) -> Weak<RefCell<FiberNode>> {
        let c = Rc::clone(&current).borrow().deref();
        let w = c.alternate.as_ref();
        if w.is_none() {
            let mut fiberNode = Rc::clone(&current).borrow_mut();
            let mut wip = fiberNode.clone();
            wip.alternate = Some(Rc::downgrade(&current));
            let wipRc = Rc::new(RefCell::new(wip));
            fiberNode.alternate = Some(Rc::downgrade(&wipRc));
            return Rc::downgrade(&wipRc);
        } else {
            let mut wip = w.unwrap().upgrade().unwrap().borrow_mut().clone();
            wip.pending_props = Some(pending_props.clone());
            wip.update_queue = Some(c.update_queue.as_ref().unwrap().clone());
            wip.flags = c.flags.clone();
            wip.child = Some(Rc::clone(c.child.as_ref().unwrap()));
            wip.memoized_props = c.memoized_props.clone();
            wip.memoized_state = c.memoized_state.clone();
            return Rc::downgrade(&Rc::new(RefCell::new(wip)));
        }
    }
}

#[derive(Debug)]
pub struct FiberRootNode {
    container: Box<JsValue>,
    pub current: Weak<RefCell<FiberNode>>,
}

impl Node for FiberRootNode {}

impl FiberRootNode {
    pub fn new(container: Box<JsValue>, host_root_fiber: Rc<RefCell<FiberNode>>) -> Self {
        Self {
            container,
            current: Rc::downgrade(&host_root_fiber),
        }
    }

    fn ensure_root_is_scheduled(&self) {
        self.perform_sync_work_on_root();
    }

    fn perform_sync_work_on_root(&self) {
        self.prepare_fresh_stack();

        loop {}
    }

    fn work_loop(&self) {}

    fn prepare_fresh_stack(&self) {
        unsafe { WORK_IN_PROGRESS = Some(FiberNode::create_work_in_progress(self.current.upgrade().unwrap(), &JsValue::null())); }
    }
}
