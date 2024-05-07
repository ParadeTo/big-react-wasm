use std::any::Any;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::rc::Rc;

use wasm_bindgen::JsValue;
use web_sys::js_sys::Reflect;

use shared::{derive_from_js_value, log, type_of};

use crate::fiber_flags::Flags;
use crate::fiber_hooks::Hook;
use crate::update_queue::{Update, UpdateQueue};
use crate::work_tags::WorkTag;

#[derive(Debug)]
pub enum StateNode {
    FiberRootNode(Rc<RefCell<FiberRootNode>>),
    Element(Rc<dyn Any>),
}

#[derive(Debug, Clone)]
pub enum MemoizedState {
    MemoizedJsValue(JsValue),
    Hook(Rc<RefCell<Hook>>),
}

impl MemoizedState {
    pub fn js_value(&self) -> Option<JsValue> {
        match self {
            MemoizedState::MemoizedJsValue(js_value) => Some(js_value.clone()),
            MemoizedState::Hook(_) => None,
        }
    }
}

pub struct FiberNode {
    pub index: u32,
    pub tag: WorkTag,
    pub pending_props: JsValue,
    pub key: JsValue,
    pub state_node: Option<Rc<StateNode>>,
    pub update_queue: Option<Rc<RefCell<UpdateQueue>>>,
    pub _return: Option<Rc<RefCell<FiberNode>>>,
    pub sibling: Option<Rc<RefCell<FiberNode>>>,
    pub child: Option<Rc<RefCell<FiberNode>>>,
    pub alternate: Option<Rc<RefCell<FiberNode>>>,
    pub _type: JsValue,
    pub flags: Flags,
    pub subtree_flags: Flags,
    pub memoized_props: JsValue,
    pub memoized_state: Option<MemoizedState>,
    pub deletions: Vec<Rc<RefCell<FiberNode>>>,
}

impl Debug for FiberNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(match self.tag {
            WorkTag::FunctionComponent => {
                write!(
                    f,
                    "{:?}(flags:{:?}, subtreeFlags:{:?})",
                    self._type.as_ref(),
                    self.flags,
                    self.subtree_flags
                )
                    .expect("print error");
            }
            WorkTag::HostRoot => {
                write!(
                    f,
                    "{:?}(subtreeFlags:{:?})",
                    WorkTag::HostRoot,
                    self.subtree_flags
                )
                    .expect("print error");
            }
            WorkTag::HostComponent => {
                write!(
                    f,
                    "{:?}(key:{:?}, flags:{:?}, subtreeFlags:{:?})",
                    self._type,
                    self.key,
                    self.flags,
                    self.subtree_flags
                )
                    .expect("print error");
            }
            WorkTag::HostText => {
                write!(
                    f,
                    "{:?}(state_node:{:?}, flags:{:?})",
                    self.tag,
                    Reflect::get(
                        self.pending_props.as_ref(),
                        &JsValue::from_str("content"),
                    )
                        .unwrap(),
                    self.flags
                )
                    .expect("print error");
            }
        })
    }
}

impl FiberNode {
    pub fn new(tag: WorkTag, pending_props: JsValue, key: JsValue) -> Self {
        Self {
            index: 0,
            tag,
            pending_props,
            key,
            state_node: None,
            update_queue: None,
            _return: None,
            sibling: None,
            child: None,
            alternate: None,
            _type: JsValue::null(),
            memoized_props: JsValue::null(),
            memoized_state: None,
            flags: Flags::NoFlags,
            subtree_flags: Flags::NoFlags,
            deletions: vec![],
        }
    }

    pub fn create_fiber_from_element(ele: &JsValue) -> Self {
        let _type = derive_from_js_value(ele, "type");
        let key = derive_from_js_value(ele, "key");
        let props = derive_from_js_value(ele, "props");

        let mut fiber_tag = WorkTag::FunctionComponent;
        if _type.is_string() {
            fiber_tag = WorkTag::HostComponent
        } else if !type_of(&_type, "function") {
            log!("Unsupported type {:?}", ele);
        }

        let mut fiber = FiberNode::new(fiber_tag, props, key);
        fiber._type = _type;
        fiber
    }

    pub fn enqueue_update(&mut self, update: Update) {
        let update_queue = match &self.update_queue {
            None => {
                return;
            }
            Some(a) => a.clone(),
        };

        let mut u = update_queue.borrow_mut();
        u.shared.pending = Some(update);
    }

    pub fn create_work_in_progress(
        current: Rc<RefCell<FiberNode>>,
        pending_props: JsValue,
    ) -> Rc<RefCell<FiberNode>> {
        let c_rc = Rc::clone(&current);
        let w = {
            let c = c_rc.borrow();
            c.deref().alternate.clone()
        };

        return if w.is_none() {
            let wip = {
                let c = c_rc.borrow();
                let mut wip = FiberNode::new(c.tag.clone(), pending_props, c.key.clone());
                wip._type = c._type.clone();
                wip.state_node = c.state_node.clone();

                wip.update_queue = c.update_queue.clone();
                wip.flags = c.flags.clone();
                wip.child = c.child.clone();
                wip.memoized_props = c.memoized_props.clone();
                wip.memoized_state = c.memoized_state.clone();
                wip.alternate = Some(current);
                wip
            };
            let wip_rc = Rc::new(RefCell::new(wip));
            {
                let mut fibler_node = c_rc.borrow_mut();
                fibler_node.alternate = Some(wip_rc.clone());
            }
            wip_rc
        } else {
            let w = w.clone().unwrap();
            {
                let wip_cloned = w.clone();
                let mut wip = wip_cloned.borrow_mut();
                let c = c_rc.borrow();
                wip.pending_props = pending_props;
                wip.flags = Flags::NoFlags;
                wip.subtree_flags = Flags::NoFlags;
                wip.deletions = vec![];
                wip._type = c._type.clone();

                wip.update_queue = c.update_queue.clone();
                wip.flags = c.flags.clone();
                wip.child = c.child.clone();
                wip.memoized_props = c.memoized_props.clone();
                wip.memoized_state = c.memoized_state.clone();
            }
            w.clone()
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
    pub container: Rc<dyn Any>,
    pub current: Rc<RefCell<FiberNode>>,
    pub finished_work: Option<Rc<RefCell<FiberNode>>>,
}

impl FiberRootNode {
    pub fn new(container: Rc<dyn Any>, host_root_fiber: Rc<RefCell<FiberNode>>) -> Self {
        Self {
            container,
            current: host_root_fiber,
            finished_work: None,
        }
    }
}

struct QueueItem {
    depth: u32,
    node: Rc<RefCell<FiberNode>>,
}

impl QueueItem {
    fn new(node: Rc<RefCell<FiberNode>>, depth: u32) -> Self {
        Self {
            node,
            depth,
        }
    }
}

impl Debug for FiberRootNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let root = self.current.clone().borrow().alternate.clone();
        Ok(if let Some(node) = root {
            let mut queue = VecDeque::new();
            queue.push_back(QueueItem::new(Rc::clone(&node), 0));

            while let Some(QueueItem { node: current, depth }) = queue.pop_front() {
                let current_ref = current.borrow();

                write!(f, "{:?}", current_ref);

                if let Some(ref child) = current_ref.child {
                    queue.push_back(QueueItem::new(Rc::clone(child), depth + 1));
                    let mut sibling = child.clone().borrow().sibling.clone();
                    while sibling.is_some() {
                        queue.push_back(QueueItem::new(Rc::clone(sibling.as_ref().unwrap()), depth + 1));
                        sibling = sibling.as_ref().unwrap().clone().borrow().sibling.clone();
                    }
                }

                if let Some(QueueItem { node: next, depth: next_depth }) = queue.front() {
                    if *next_depth != depth {
                        writeln!(f, "").expect("print error");
                        writeln!(f, "------------------------------------")
                            .expect("print error");
                        continue;
                    }

                    if current_ref._return.is_some() {
                        write!(f, ",").expect("print error");
                    } else {
                        writeln!(f, "").expect("print error");
                        writeln!(f, "------------------------------------").expect("print error");
                    }
                }
            }
        })
    }
}
