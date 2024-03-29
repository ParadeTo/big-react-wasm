use std::cell::RefCell;
use std::rc::Rc;

use web_sys::Element;

use crate::fiber::{FiberNode, Flags, StateNode};
use crate::host_config::get_host_config;
use crate::work_tags::WorkTag;

fn append_all_children(parent: Element, work_in_progress: Rc<RefCell<FiberNode>>) {
    let host_config = get_host_config();
    let work_in_progress = work_in_progress.clone();
    let mut node = work_in_progress.borrow().child.clone();
    while node.is_some() {
        let node_unwrap = node.clone().unwrap();
        let n = node_unwrap.clone();
        if n.borrow().tag == WorkTag::HostComponent {
            match n.borrow().state_node.as_ref().unwrap() { StateNode::FiberRootNode(_) => {} }
            // host_config.append_initial_child(parent, )
        } else if n.borrow().child.is_some() {
            let n = node_unwrap.clone();
            let borrowed = n.borrow_mut();
            borrowed.child.as_ref().unwrap().clone().borrow_mut()._return = Some(Rc::downgrade(&node_unwrap));
            node = node_unwrap.borrow().child.clone();
            continue;
        }

        if Rc::ptr_eq(&node_unwrap, &work_in_progress) {
            return;
        }

        while node_unwrap.borrow().sibling.clone().is_none() {
            if
        }
    }
}

fn bubble_properties(complete_work: Rc<RefCell<FiberNode>>) {
    let mut subtree_flags = Flags::NoFlags;
    let mut child = complete_work.clone().borrow_mut().child.clone();
    while child.is_some() {
        let child_rc = child.clone().unwrap().clone();
        let child_borrowed = child_rc.borrow();
        subtree_flags |= child_borrowed.subtree_flags.clone();
        subtree_flags |= child_borrowed.flags.clone();

        child_rc.borrow_mut()._return = Some(Rc::downgrade(&complete_work));
        child = child_borrowed.sibling.clone();
    }

    complete_work.clone().borrow_mut().subtree_flags |= subtree_flags.clone();
}

pub fn complete_work(work_in_progress: Rc<RefCell<FiberNode>>) {
    let host_config = get_host_config();
    match work_in_progress.clone().borrow().tag {
        WorkTag::FunctionComponent => todo!(),
        WorkTag::HostRoot => bubble_properties(work_in_progress),
        WorkTag::HostComponent => {
            // let render = &host_config;
            host_config.create_instance(
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
            // host_config.app
        }
    }
}
