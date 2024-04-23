use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsValue;
use web_sys::js_sys::Reflect;

use crate::fiber::{FiberNode, StateNode};
use crate::fiber_flags::Flags;
use crate::work_tags::WorkTag;
use crate::HostConfig;

pub struct CompleteWork {
    pub host_config: Rc<dyn HostConfig>,
}

impl CompleteWork {
    pub(crate) fn new(host_config: Rc<dyn HostConfig>) -> Self {
        Self { host_config }
    }

    fn append_all_children(&self, parent: Rc<dyn Any>, work_in_progress: Rc<RefCell<FiberNode>>) {
        let work_in_progress = work_in_progress.clone();
        let mut node = work_in_progress.borrow().child.clone();
        while node.is_some() {
            let node_unwrap = node.clone().unwrap();
            let n = node_unwrap.clone();
            if n.borrow().tag == WorkTag::HostComponent || n.borrow().tag == WorkTag::HostText {
                self.host_config.append_initial_child(
                    parent.clone(),
                    FiberNode::derive_state_node(node.clone().unwrap()).unwrap(),
                )
            } else if n.borrow().child.is_some() {
                let n = node_unwrap.clone();
                {
                    let borrowed = n.borrow_mut();
                    borrowed
                        .child
                        .as_ref()
                        .unwrap()
                        .clone()
                        .borrow_mut()
                        ._return = Some(node_unwrap.clone());
                }

                node = node_unwrap.clone().borrow().child.clone();
                continue;
            }

            if Rc::ptr_eq(&node_unwrap, &work_in_progress) {
                return;
            }

            while node
                .clone()
                .unwrap()
                .clone()
                .borrow()
                .sibling
                .clone()
                .is_none()
            {
                let node_cloned = node.clone().unwrap().clone();
                if node_cloned.borrow()._return.is_none()
                    || Rc::ptr_eq(
                        &node_cloned.borrow()._return.as_ref().unwrap(),
                        &work_in_progress,
                    )
                {
                    return;
                }

                node = node_cloned.borrow()._return.clone();
            }

            {
                node.clone()
                    .unwrap()
                    .borrow()
                    .sibling
                    .clone()
                    .unwrap()
                    .clone()
                    .borrow_mut()
                    ._return = node_unwrap.borrow()._return.clone();
            }

            node = node.clone().unwrap().borrow().sibling.clone();
        }
    }

    fn bubble_properties(&self, complete_work: Rc<RefCell<FiberNode>>) {
        let mut subtree_flags = Flags::NoFlags;
        {
            let mut child = complete_work.clone().borrow().child.clone();
            while child.is_some() {
                let child_rc = child.clone().unwrap().clone();
                {
                    let child_borrowed = child_rc.borrow();
                    subtree_flags |= child_borrowed.subtree_flags.clone();
                    subtree_flags |= child_borrowed.flags.clone();
                }
                {
                    child_rc.borrow_mut()._return = Some(complete_work.clone());
                }
                child = child_rc.borrow().sibling.clone();
            }
        }

        complete_work.clone().borrow_mut().subtree_flags |= subtree_flags.clone();
    }

    pub fn complete_work(
        &self,
        work_in_progress: Rc<RefCell<FiberNode>>,
    ) -> Option<Rc<RefCell<FiberNode>>> {
        let new_props = { work_in_progress.clone().borrow().pending_props.clone() };
        let tag = { work_in_progress.clone().borrow().tag.clone() };
        match tag {
            WorkTag::FunctionComponent => {
                self.bubble_properties(work_in_progress.clone());
                None
            }
            WorkTag::HostRoot => {
                self.bubble_properties(work_in_progress.clone());
                None
            }
            WorkTag::HostComponent => {
                let instance = self.host_config.create_instance(
                    work_in_progress
                        .clone()
                        .borrow()
                        ._type
                        .clone()
                        .unwrap()
                        .clone()
                        .as_string()
                        .unwrap(),
                );
                self.append_all_children(instance.clone(), work_in_progress.clone());
                work_in_progress.clone().borrow_mut().state_node =
                    Some(Rc::new(StateNode::Element(instance.clone())));
                self.bubble_properties(work_in_progress.clone());
                None
            }
            WorkTag::HostText => {
                let text_instance = self.host_config.create_text_instance(
                    Reflect::get(&new_props.unwrap(), &JsValue::from_str("content"))
                        .unwrap()
                        .as_string()
                        .unwrap(),
                );
                work_in_progress.clone().borrow_mut().state_node =
                    Some(Rc::new(StateNode::Element(text_instance.clone())));
                self.bubble_properties(work_in_progress.clone());
                None
            }
        }
    }
}
