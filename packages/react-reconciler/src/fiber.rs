use std::any::Any;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::{Rc, Weak};

use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

use react::ReactElement;

use crate::fiber_flags::Flags;
use crate::update_queue::{Update, UpdateQueue, UpdateType};
use crate::work_tags::WorkTag;

trait Node {}


#[derive(Debug)]
pub enum StateNode {
    FiberRootNode(Rc<RefCell<FiberRootNode>>),
    Element(Rc<dyn Any>),
}


#[derive(Debug)]
pub struct FiberNode {
    pub tag: WorkTag,
    pub pending_props: Option<Rc<JsValue>>,
    key: Option<String>,
    pub state_node: Option<Rc<StateNode>>,
    pub update_queue: Option<Rc<RefCell<UpdateQueue>>>,
    pub _return: Option<Weak<RefCell<FiberNode>>>,
    pub sibling: Option<Rc<RefCell<FiberNode>>>,
    pub child: Option<Rc<RefCell<FiberNode>>>,
    pub alternate: Option<Weak<RefCell<FiberNode>>>,
    pub _type: Option<Rc<JsValue>>,
    pub flags: Flags,
    pub subtree_flags: Flags,
    pub memoized_props: JsValue,
    pub memoized_state: Option<Rc<JsValue>>,
}

impl Node for FiberNode {}

impl FiberNode {
    pub fn new(tag: WorkTag, pending_props: Option<Rc<JsValue>>, key: Option<String>) -> Self {
        Self {
            tag,
            pending_props,
            key,
            state_node: None,
            update_queue: None,
            _return: None,
            sibling: None,
            child: None,
            alternate: None,
            _type: None,
            memoized_props: JsValue::null(),
            memoized_state: None,
            flags: Flags::NoFlags,
            subtree_flags: Flags::NoFlags,
        }
    }

    // pub fn initialize_update_queue(&self) {}

    pub fn create_fiber_from_element(element: Rc<JsValue>) -> Self {
        let element = ReactElement::from_js_value(element.deref().as_ref());
        let ReactElement { _type, key, props, .. } = element;
        let mut fiberTag = WorkTag::FunctionComponent;
        if _type.is_string() {
            fiberTag = WorkTag::HostComponent
        }
        let mut fiber = FiberNode::new(fiberTag, props, key);
        fiber._type = Some(_type);
        fiber
    }

    pub fn enqueue_update(&mut self, update: Update) {
        let mut update_queue = match &self.update_queue {
            None => {
                return;
            }
            Some(a) => {
                let b = a.clone();

                b
            }
        };

        let mut u = update_queue.borrow_mut();
        u.shared.pending = Some(update);
    }

    pub fn initialize_update_queue(&mut self) {
        self.update_queue = Some(Rc::new(RefCell::new(UpdateQueue {
            shared: UpdateType {
                pending: Some(Update { action: None }),
            },
        })));
    }


    pub fn create_work_in_progress(
        current: Rc<RefCell<FiberNode>>,
        pending_props: Rc<JsValue>,
    ) -> Rc<RefCell<FiberNode>> {
        let c_rc = Rc::clone(&current);
        let w = {
            let c = c_rc.borrow();
            c.deref().alternate.clone()
        };
        return if w.is_none() {
            let mut wip = {
                let c = c_rc.borrow();
                FiberNode::new(c.tag.clone(), c.pending_props.clone(), c.key.clone())
            };
            wip.alternate = Some(Rc::downgrade(&current));
            let wip_rc = Rc::new(RefCell::new(wip));
            let mut fibler_node = c_rc.borrow_mut();
            fibler_node.alternate = Some(Rc::downgrade(&wip_rc));
            wip_rc
        } else {
            let c = c_rc.borrow();
            let a = w.clone().unwrap().upgrade().clone().unwrap();
            let mut wip = a.borrow_mut();

            wip.pending_props = Some(pending_props.clone());
            wip.update_queue = Some(c.update_queue.as_ref().unwrap().clone());
            wip.flags = c.flags.clone();
            wip.child = Some(Rc::clone(c.child.as_ref().unwrap()));
            wip.memoized_props = c.memoized_props.clone();
            wip.memoized_state = c.memoized_state.clone();
            w.clone().unwrap().upgrade().unwrap()
        };
    }
}

#[derive(Debug)]
pub struct FiberRootNode {
    container: Box<JsValue>,
    pub current: Rc<RefCell<FiberNode>>,
    pub finished_work: Option<Rc<RefCell<FiberNode>>>,
}

impl Node for FiberRootNode {}

impl FiberRootNode {
    pub fn new(container: Box<JsValue>, host_root_fiber: Rc<RefCell<FiberNode>>) -> Self {
        Self {
            container,
            current: host_root_fiber,
            finished_work: None,
        }
    }


    fn begin_work(fiber: Weak<RefCell<FiberNode>>) {}
}
