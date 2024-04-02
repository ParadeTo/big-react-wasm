use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsValue;
use web_sys::js_sys::Reflect;

use shared::log;

use crate::fiber::{FiberNode, StateNode};
use crate::fiber_flags::Flags;
use crate::host_config::get_host_config;
use crate::work_tags::WorkTag;

fn append_all_children(parent: Rc<dyn Any>, work_in_progress: Rc<RefCell<FiberNode>>) {
    let work_in_progress = work_in_progress.clone();
    let mut node = work_in_progress.borrow().child.clone();
    while node.is_some() {
        let node_unwrap = node.clone().unwrap();
        let n = node_unwrap.clone();
        if n.borrow().tag == WorkTag::HostComponent {
            match n.borrow().state_node.as_ref() {
                // StateNode::FiberRootNode(_) => {}
                // _ => {}
                None => {}
                Some(state_node) => match &**state_node {
                    StateNode::FiberRootNode(_) => {}
                    _ => {}
                },
            }
            // parent.downcast()::
            // host_config.append_initial_child(parent, )
        } else if n.borrow().child.is_some() {
            let n = node_unwrap.clone();
            let borrowed = n.borrow_mut();
            borrowed
                .child
                .as_ref()
                .unwrap()
                .clone()
                .borrow_mut()
                ._return = Some(Rc::downgrade(&node_unwrap));
            node = node_unwrap.borrow().child.clone();
            continue;
        }

        if Rc::ptr_eq(&node_unwrap, &work_in_progress) {
            return;
        }

        while node_unwrap.borrow().sibling.clone().is_none() {
            if node_unwrap.borrow()._return.is_none()
                || Rc::ptr_eq(
                &node_unwrap
                    .borrow()
                    ._return
                    .as_ref()
                    .unwrap()
                    .upgrade()
                    .unwrap(),
                &work_in_progress,
            )
            {
                return;
            }

            node_unwrap
                .borrow_mut()
                .sibling
                .clone()
                .unwrap()
                .clone()
                .borrow_mut()
                ._return = node_unwrap.borrow()._return.clone();
            node = node_unwrap.borrow().sibling.clone();
        }
    }
}

fn bubble_properties(complete_work: Rc<RefCell<FiberNode>>) {
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
                child_rc.borrow_mut()._return = Some(Rc::downgrade(&complete_work));
            }
            child = child_rc.borrow().sibling.clone();
        }
    }

    complete_work.clone().borrow_mut().subtree_flags |= subtree_flags.clone();
}

pub fn complete_work(work_in_progress: Rc<RefCell<FiberNode>>) -> Option<Rc<RefCell<FiberNode>>> {
    let new_props = { work_in_progress.clone().borrow().pending_props.clone() };
    let host_config = get_host_config();
    let tag = { work_in_progress.clone().borrow().tag.clone() };
    match tag {
        WorkTag::FunctionComponent => {
            log!(
                "complete unknown fibler.tag {:?}",
                work_in_progress.clone().borrow().tag
            );
            None
        }
        WorkTag::HostRoot => {
            bubble_properties(work_in_progress.clone());
            None
        }
        WorkTag::HostComponent => {
            let instance = host_config.create_instance(
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
            append_all_children(instance.clone(), work_in_progress.clone());
            work_in_progress.clone().borrow_mut().state_node =
                Some(Rc::new(StateNode::Element(instance.clone())));
            bubble_properties(work_in_progress.clone());
            None
        }
        WorkTag::HostText => {
            let text_instance = host_config.create_text_instance(
                Reflect::get(&new_props.unwrap(), &JsValue::from_str("content"))
                    .unwrap()
                    .as_string()
                    .unwrap(),
            );
            work_in_progress.clone().borrow_mut().state_node =
                Some(Rc::new(StateNode::Element(text_instance.clone())));
            bubble_properties(work_in_progress.clone());
            None
        }
    }
}
