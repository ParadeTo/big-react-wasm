use std::any::Any;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::rc::Rc;

use wasm_bindgen::JsValue;
use web_sys::js_sys::Reflect;

use shared::derive_from_js_value;

use crate::fiber_flags::Flags;
use crate::update_queue::{Update, UpdateQueue, UpdateType};
use crate::work_tags::WorkTag;

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
    pub _return: Option<Rc<RefCell<FiberNode>>>,
    pub sibling: Option<Rc<RefCell<FiberNode>>>,
    pub child: Option<Rc<RefCell<FiberNode>>>,
    pub alternate: Option<Rc<RefCell<FiberNode>>>,
    pub _type: Option<Rc<JsValue>>,
    pub flags: Flags,
    pub subtree_flags: Flags,
    pub memoized_props: Option<Rc<JsValue>>,
    pub memoized_state: Option<Rc<JsValue>>,
}

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
            memoized_props: None,
            memoized_state: None,
            flags: Flags::NoFlags,
            subtree_flags: Flags::NoFlags,
        }
    }

    pub fn create_fiber_from_element(ele: Rc<JsValue>) -> Self {
        let _type = derive_from_js_value(ele.clone(), "type");
        let key = match derive_from_js_value(ele.clone(), "key") {
            None => None,
            Some(k) => k.as_string(),
        };
        let props = derive_from_js_value(ele.clone(), "props");

        let mut fiber_tag = WorkTag::FunctionComponent;
        if _type.is_some() && (*_type.as_ref().unwrap()).is_string() {
            fiber_tag = WorkTag::HostComponent
        }
        let mut fiber = FiberNode::new(fiber_tag, props, key);
        fiber._type = _type;
        fiber
    }

    pub fn enqueue_update(&mut self, update: Update) {
        let mut update_queue = match &self.update_queue {
            None => {
                return;
            }
            Some(a) => a.clone(),
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
                let mut wip = FiberNode::new(c.tag.clone(), c.pending_props.clone(), c.key.clone());
                wip.update_queue = Some(c.update_queue.as_ref().unwrap().clone());
                wip.flags = c.flags.clone();
                wip.child = c.child.clone();
                wip.memoized_props = c.memoized_props.clone();
                wip.memoized_state = c.memoized_state.clone();
                wip
            };
            wip._type = c_rc.borrow()._type.clone();
            wip.state_node = c_rc.borrow().state_node.clone();
            wip.alternate = Some(current);
            let wip_rc = Rc::new(RefCell::new(wip));
            let mut fibler_node = c_rc.borrow_mut();
            fibler_node.alternate = Some(wip_rc.clone());
            wip_rc
        } else {
            let c = c_rc.borrow();
            let a = w.clone().unwrap();
            let mut wip = a.borrow_mut();

            wip.pending_props = Some(pending_props.clone());
            wip.update_queue = Some(c.update_queue.as_ref().unwrap().clone());
            wip.flags = c.flags.clone();
            wip.child = Some(Rc::clone(c.child.as_ref().unwrap()));
            wip.memoized_props = c.memoized_props.clone();
            wip.memoized_state = c.memoized_state.clone();
            w.clone().unwrap()
        };
    }

    pub fn derive_state_node(fiber: Rc<RefCell<FiberNode>>) -> Option<Rc<dyn Any>> {
        let state_node = fiber.clone().borrow().state_node.clone();
        if state_node.is_none() {
            return None;
        }

        Some(match &*state_node.unwrap().clone() {
            StateNode::FiberRootNode(root) => root.clone().borrow().container.clone(),
            StateNode::Element(ele) => ele.clone(),
        })
    }
}

pub struct FiberRootNode {
    pub container: Rc<JsValue>,
    pub current: Rc<RefCell<FiberNode>>,
    pub finished_work: Option<Rc<RefCell<FiberNode>>>,
}

impl FiberRootNode {
    pub fn new(container: Rc<JsValue>, host_root_fiber: Rc<RefCell<FiberNode>>) -> Self {
        Self {
            container,
            current: host_root_fiber,
            finished_work: None,
        }
    }
}

impl Debug for FiberRootNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut root = self.current.clone().borrow().alternate.clone();
        Ok(if let Some(node) = root {
            let mut queue = VecDeque::new();
            queue.push_back(Rc::clone(&node));

            while let Some(current) = queue.pop_front() {
                let current_ref = current.borrow();

                match current_ref.tag {
                    WorkTag::FunctionComponent => {
                        write!(f, "{:?}", current.borrow()._type.as_ref().unwrap());
                    }
                    WorkTag::HostRoot => {
                        write!(f, "{:?}", WorkTag::HostRoot);
                    }
                    WorkTag::HostComponent => {
                        let current_borrowed = current.borrow();
                        write!(
                            f,
                            "{:?}({:?})",
                            current_borrowed
                                ._type
                                .as_ref()
                                .unwrap()
                                .as_string()
                                .unwrap(),
                            current_borrowed.state_node
                        );
                    }
                    WorkTag::HostText => {
                        let current_borrowed = current.borrow();

                        write!(
                            f,
                            "{:?}({:?})",
                            current_borrowed.tag,
                            Reflect::get(
                                current_borrowed.pending_props.as_ref().unwrap(),
                                &JsValue::from_str("content"),
                            )
                                .unwrap()
                                .as_string()
                                .unwrap(),
                        );
                    }
                };
                if let Some(ref child) = current_ref.child {
                    queue.push_back(Rc::clone(child));
                    let mut sibling = child.clone().borrow().sibling.clone();
                    while sibling.is_some() {
                        queue.push_back(Rc::clone(sibling.as_ref().unwrap()));
                        sibling = sibling.as_ref().unwrap().clone().borrow().sibling.clone();
                    }
                }

                if let Some(next) = queue.front() {
                    let next_ref = next.borrow();
                    if let (Some(current_parent), Some(next_parent)) =
                        (current_ref._return.as_ref(), next_ref._return.as_ref())
                    {
                        if !Rc::ptr_eq(current_parent, next_parent) {
                            writeln!(f, "");
                            writeln!(f, "------------------------------------");
                            continue;
                        }
                    }

                    if current_ref._return.is_some() {
                        write!(f, ",");
                    } else {
                        writeln!(f, "");
                        writeln!(f, "------------------------------------");
                    }
                }
            }
        })
    }
}
