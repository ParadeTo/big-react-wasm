use std::cell::RefCell;
use std::rc::{Rc, Weak};

use wasm_bindgen::prelude::*;

use crate::update_queue::{Update, UpdateQueue, UpdateType};
use crate::work_tags::WorkTag;

#[derive(Debug)]
pub enum StateNode {
    FiberRootNode(Weak<RefCell<FiberRootNode>>),
}

#[derive(Debug)]
pub struct FiberNode {
    tag: WorkTag,
    pub state_node: Option<StateNode>,
    pub update_queue: Option<Box<UpdateQueue>>,
}

impl FiberNode {
    pub fn new(tag: WorkTag) -> Self {
        Self { tag, state_node: None, update_queue: None }
    }

    pub fn enqueue_update(&mut self, update: Update) {
        let mut update_queue = match &self.update_queue {
            None => {
                return;
            }
            Some(a) => (**a).clone()
        };

        let mut u = &mut update_queue;
        u.shared.pending = update;
    }

    pub fn initialize_update_queue(&mut self) {
        self.update_queue = Some(Box::new(UpdateQueue {
            shared: UpdateType { pending: Update { action: None } },
        }))
    }
}

#[derive(Debug)]
pub struct FiberRootNode {
    container: Box<JsValue>,
    pub current: Rc<RefCell<FiberNode>>,
}

impl FiberRootNode {
    pub fn new(container: Box<JsValue>, host_root_fiber: Rc<RefCell<FiberNode>>) -> Self {
        Self { container, current: host_root_fiber }
    }
}