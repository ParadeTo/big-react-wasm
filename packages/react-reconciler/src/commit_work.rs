use std::cell::RefCell;
use std::rc::Rc;

use crate::fiber::{FiberNode, StateNode};
use crate::fiber_flags::{Flags, get_mutation_mask};
use crate::host_config::get_host_config;
use crate::work_tags::WorkTag;

pub struct CommitWork {
    next_effect: Option<Rc<RefCell<FiberNode>>>,
}

impl CommitWork {
    pub fn new() -> Self {
        Self {
            next_effect: None
        }
    }
    pub fn commit_mutation_effects(&mut self, finished_work: Option<Rc<RefCell<FiberNode>>>) {
        self.next_effect = finished_work.clone();
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
                    let sibling = self.next_effect.clone().clone().unwrap().borrow().sibling.clone();
                    if sibling.is_some() {
                        self.next_effect = sibling;
                        break;
                    }
                    self.next_effect = next_effect.clone().borrow()._return.clone().unwrap().upgrade();
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
        let parent_state_node = match host_parent.clone().unwrap().clone().borrow().tag.clone() {
            WorkTag::FunctionComponent => todo!(),
            WorkTag::HostRoot => host_parent
                .clone()
                .unwrap()
                .clone()
                .borrow()
                .state_node.clone(),
            WorkTag::HostComponent => host_parent
                .clone()
                .unwrap()
                .clone()
                .borrow()
                .state_node.clone(),
            WorkTag::HostText => todo!(),
        };

        if parent_state_node.is_some() {
            self.append_placement_node_into_container(finished_work.clone(), parent_state_node);
        }
    }

    fn append_placement_node_into_container(&self, fiber: Rc<RefCell<FiberNode>>, parent: Option<Rc<StateNode>>) {
        let fiber = fiber.clone();
        let host_config = get_host_config();
        let tag = fiber.borrow().tag.clone();
        if tag == WorkTag::HostComponent || tag == WorkTag::HostText {
            host_config.append_child_to_container(Rc::new(fiber.clone().borrow().state_node.clone()), parent.clone().unwrap());
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
            let p = parent.clone().unwrap().upgrade().unwrap().clone();
            let parent_tag = p.borrow().tag.clone();
            if parent_tag == WorkTag::HostComponent || parent_tag == WorkTag::HostRoot {
                return Some(p);
            }
            parent = p.borrow()._return.clone();
        }

        None
    }
}
