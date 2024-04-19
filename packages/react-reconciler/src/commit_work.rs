use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use crate::fiber::{FiberNode, StateNode};
use crate::fiber_flags::{Flags, get_mutation_mask};
use crate::HostConfig;
use crate::work_tags::WorkTag;

pub struct CommitWork {
    next_effect: Option<Rc<RefCell<FiberNode>>>,
    host_config: Rc<dyn HostConfig>,
}

impl CommitWork {
    pub fn new(host_config: Rc<dyn HostConfig>) -> Self {
        Self {
            next_effect: None,
            host_config,
        }
    }
    pub fn commit_mutation_effects(&mut self, finished_work: Rc<RefCell<FiberNode>>) {
        self.next_effect = Some(finished_work);
        while self.next_effect.is_some() {
            let next_effect = self.next_effect.clone().unwrap().clone();
            let child = next_effect.borrow().child.clone();
            if child.is_some()
                && get_mutation_mask().contains(next_effect.borrow().subtree_flags.clone())
            {
                self.next_effect = child;
            } else {
                while self.next_effect.is_some() {
                    self.commit_mutation_effects_on_fiber(self.next_effect.clone().unwrap());
                    let sibling = self
                        .next_effect
                        .clone()
                        .clone()
                        .unwrap()
                        .borrow()
                        .sibling
                        .clone();
                    if sibling.is_some() {
                        self.next_effect = sibling;
                        break;
                    }

                    let _return = self
                        .next_effect
                        .clone()
                        .unwrap()
                        .clone()
                        .borrow()
                        ._return
                        .clone();

                    if _return.is_none() {
                        self.next_effect = None
                    } else {
                        self.next_effect = _return;
                    }
                }
            }
        }
    }

    fn commit_mutation_effects_on_fiber(&self, finished_work: Rc<RefCell<FiberNode>>) {
        let flags = finished_work.clone().borrow().flags.clone();
        if flags.contains(Flags::Placement) {
            self.commit_placement(finished_work.clone());
            finished_work.clone().borrow_mut().flags -= Flags::Placement
        }
    }

    fn commit_placement(&self, finished_work: Rc<RefCell<FiberNode>>) {
        let host_parent = self.get_host_parent(finished_work.clone());
        if host_parent.is_none() {
            return;
        }
        let parent_state_node = FiberNode::derive_state_node(host_parent.unwrap());

        if parent_state_node.is_some() {
            self.append_placement_node_into_container(
                finished_work.clone(),
                parent_state_node.unwrap(),
            );
        }
    }

    fn get_element_from_state_node(&self, state_node: Rc<StateNode>) -> Rc<dyn Any> {
        match &*state_node {
            StateNode::FiberRootNode(root) => root.clone().borrow().container.clone(),
            StateNode::Element(ele) => ele.clone(),
        }
    }

    fn append_placement_node_into_container(
        &self,
        fiber: Rc<RefCell<FiberNode>>,
        parent: Rc<dyn Any>,
    ) {
        let fiber = fiber.clone();
        let tag = fiber.borrow().tag.clone();
        if tag == WorkTag::HostComponent || tag == WorkTag::HostText {
            let state_node = fiber.clone().borrow().state_node.clone().unwrap();
            self.host_config.append_child_to_container(
                self.get_element_from_state_node(state_node),
                parent.clone(),
            );
            return;
        }

        let child = fiber.borrow().child.clone();
        if child.is_some() {
            self.append_placement_node_into_container(child.clone().unwrap(), parent.clone());
            let mut sibling = child.unwrap().clone().borrow().sibling.clone();
            while sibling.is_some() {
                self.append_placement_node_into_container(sibling.clone().unwrap(), parent.clone());
                sibling = sibling.clone().unwrap().clone().borrow().sibling.clone();
            }
        }
    }

    fn get_host_parent(&self, fiber: Rc<RefCell<FiberNode>>) -> Option<Rc<RefCell<FiberNode>>> {
        let mut parent = fiber.clone().borrow()._return.clone();
        while parent.is_some() {
            let p = parent.clone().unwrap();
            let parent_tag = p.borrow().tag.clone();
            if parent_tag == WorkTag::HostComponent || parent_tag == WorkTag::HostRoot {
                return Some(p);
            }
            parent = p.borrow()._return.clone();
        }

        None
    }
}
