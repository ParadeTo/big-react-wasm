use std::cell::RefCell;
use std::rc::{Rc, Weak};

use wasm_bindgen::prelude::*;

use crate::update_queue::{Update, UpdateQueue, UpdateType};
use crate::work_tags::WorkTag;

trait Node {}

static let work_in_progress;

#[derive(Debug, Clone)]
pub enum StateNode {
    FiberRootNode(Rc<RefCell<FiberRootNode>>),
}

#[derive(Debug, Clone)]
pub struct FiberNode {
    tag: WorkTag,
    pending_props: Option<JsValue>,
    key: Option<String>,
    pub state_node: Option<StateNode>,
    pub update_queue: Option<Box<UpdateQueue>>,
    _return: Option<Weak<RefCell<FiberNode>>>,
    sibling: Option<Rc<RefCell<FiberNode>>>,
    child: Option<Rc<RefCell<FiberNode>>>,
    alternate: Option<Weak<RefCell<dyn Node>>>,
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
        }
    }

    pub fn enqueue_update(&mut self, update: Update) {
        let mut update_queue = match &self.update_queue {
            None => {
                return;
            }
            Some(a) => (**a).clone(),
        };

        let mut u = &mut update_queue;
        u.shared.pending = update;
    }

    pub fn initialize_update_queue(&mut self) {
        self.update_queue = Some(Box::new(UpdateQueue {
            shared: UpdateType {
                pending: Update { action: None },
            },
        }))
    }

    pub fn schedule_update_on_fiber(fiber: Rc<RefCell<FiberNode>>) {
        let root = FiberNode::mark_update_lane_from_fiber_to_root(fiber);
        if root == None {
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
        // let w = self.alternate.as_ref();
        // let mut wip: FiberNode = FiberNode
        // if w.is_none() {
        //     wip = self.clone();
        // }

        let mut wip = Rc::clone(&current).borrow().clone();
        wip.alternate = Some(Rc::downgrade(&current));
        let wip = Rc::downgrade(&Rc::new(RefCell::new(wip)));
        Rc::clone(&current).borrow_mut().alternate = Some(wip);
        wip
        // wip
        // if (wip === null) {
        //     // mount
        //     wip = new FiberNode(current.tag, pendingProps, current.key);
        //     wip.type = current.type;
        //     wip.stateNode = current.stateNode;
        //
        //     wip.alternate = current;
        //     current.alternate = wip;
        // } else {
        //     // update
        //     wip.pendingProps = pendingProps;
        // }
        // wip.updateQueue = current.updateQueue;
        // wip.flags = current.flags;
        // wip.child = current.child;
        //
        // // 数据
        // wip.memoizedProps = current.memoizedProps;
        // wip.memoizedState = current.memoizedState;
        //
        // return wip;
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

    fn perform_sync_work_on_root(&self) {}

    fn prepare_fresh_stack(&self) {}
}
