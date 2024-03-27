use std::cell::RefCell;
use std::ops::Deref;
use std::rc::{Rc, Weak};

use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

use crate::update_queue::{Update, UpdateQueue, UpdateType};
use crate::work_tags::WorkTag;

trait Node {}


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
    pub tag: WorkTag,
    pub pending_props: Option<JsValue>,
    key: Option<String>,
    pub state_node: Option<StateNode>,
    pub update_queue: Option<Weak<RefCell<UpdateQueue>>>,
    pub _return: Option<Weak<RefCell<FiberNode>>>,
    pub sibling: Option<Rc<RefCell<FiberNode>>>,
    pub child: Option<Rc<RefCell<FiberNode>>>,
    pub alternate: Option<Weak<RefCell<FiberNode>>>,
    pub _type: JsValue,
    pub flags: Flags,
    pub memoized_props: JsValue,
    pub memoized_state: JsValue,
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
                let b = a.upgrade().clone().unwrap();

                b
            }
        };

        let mut u = update_queue.borrow_mut();
        u.shared.pending = update;
    }

    pub fn initialize_update_queue(&mut self) {
        self.update_queue = Some(Rc::downgrade(&Rc::new(RefCell::new(UpdateQueue {
            shared: UpdateType {
                pending: Update { action: None },
            },
        }))));
    }


    pub fn create_work_in_progress(
        current: Rc<RefCell<FiberNode>>,
        pending_props: &JsValue,
    ) -> Weak<RefCell<FiberNode>> {
        let c_rc = Rc::clone(&current);
        let c = c_rc.borrow();
        let w = c.deref().alternate.as_ref();
        return if w.is_none() {
            let mut wip = Rc::clone(&current).borrow_mut().deref().clone();
            wip.alternate = Some(Rc::downgrade(&current));
            let wip_rc = Rc::new(RefCell::new(wip));
            let mut fibler_node = c_rc.borrow_mut();
            fibler_node.alternate = Some(Rc::downgrade(&wip_rc));
            Rc::downgrade(&wip_rc)
        } else {
            let mut wip = w.unwrap().upgrade().unwrap().borrow_mut().clone();

            wip.pending_props = Some(pending_props.clone());
            wip.update_queue = Some(c.update_queue.as_ref().unwrap().clone());
            wip.flags = c.flags.clone();
            wip.child = Some(Rc::clone(c.child.as_ref().unwrap()));
            wip.memoized_props = c.memoized_props.clone();
            wip.memoized_state = c.memoized_state.clone();
            Rc::downgrade(&Rc::new(RefCell::new(wip)))
        };
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


    fn begin_work(fiber: Weak<RefCell<FiberNode>>) {}
}
