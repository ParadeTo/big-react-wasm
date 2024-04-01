use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsValue;

use shared::log;

use crate::begin_work::begin_work;
use crate::commit_work::CommitWork;
use crate::complete_work::complete_work;
use crate::fiber::{FiberNode, FiberRootNode, StateNode};
use crate::fiber_flags::get_mutation_mask;
use crate::work_tags::WorkTag;

static mut WORK_IN_PROGRESS: Option<Rc<RefCell<FiberNode>>> = None;

pub struct WorkLoop {
    work_in_progress: Option<Rc<RefCell<FiberNode>>>,
}

impl WorkLoop {
    pub fn new() -> Self {
        Self {
            work_in_progress: None,
        }
    }
    pub fn schedule_update_on_fiber(&mut self, fiber: Rc<RefCell<FiberNode>>) {
        let root = self.mark_update_lane_from_fiber_to_root(fiber);
        if root.is_none() {
            return;
        }
        self.ensure_root_is_scheduled(root.unwrap())
    }

    pub fn mark_update_lane_from_fiber_to_root(
        &self,
        fiber: Rc<RefCell<FiberNode>>,
    ) -> Option<Rc<RefCell<FiberRootNode>>> {
        let mut node = Rc::clone(&fiber);
        let mut parent = Rc::clone(&fiber).borrow()._return.clone();

        while parent.is_some() {
            node = parent.clone().unwrap().upgrade().unwrap();
            let rc = Rc::clone(&parent.unwrap().upgrade().unwrap());
            let rc_ref = rc.borrow();
            let next = match rc_ref._return.as_ref() {
                None => None,
                Some(node) => {
                    let a = Rc::downgrade(&node.upgrade().unwrap());
                    Some(a)
                }
            };
            parent = next;
        }

        let fiber_node_rc = Rc::clone(&node);
        let fiber_node = fiber_node_rc.borrow();
        if fiber_node.tag == WorkTag::HostRoot {
            match fiber_node.state_node.clone() {
                None => {}
                Some(state_node) => {
                    let state_node = state_node;
                    return match &*state_node {
                        StateNode::FiberRootNode(fiber_root_node) => {
                            Some(Rc::clone(&fiber_root_node))
                        }
                        StateNode::Element(_) => todo!()
                    };
                    // return match state_node.clone() {
                    //     // StateNode::FiberRootNode(fiber_root_node) => {
                    //     //     Some(Rc::clone(fiber_root_node))
                    //     // }
                    //     // _ => todo!(),
                    //     Rc { .. } => {}
                    // };
                }
            }
        }

        None
    }

    fn ensure_root_is_scheduled(&mut self, root: Rc<RefCell<FiberRootNode>>) {
        self.perform_sync_work_on_root(root);
    }

    fn perform_sync_work_on_root(&mut self, root: Rc<RefCell<FiberRootNode>>) {
        self.prepare_fresh_stack(Rc::clone(&root));

        loop {
            self.work_loop();
            break;
        }

        // commit
        log!(
            "commit {:?}",
            Rc::clone(&root).borrow().current.clone().borrow().tag
        );
        self.commit_root(root);
    }

    fn commit_root(&self, root: Rc<RefCell<FiberRootNode>>) {
        let cloned = root.clone();
        if cloned.borrow().finished_work.is_none() {
            return;
        }

        cloned.borrow_mut().finished_work = None;

        let finished_work = cloned.borrow().finished_work.clone();
        let subtree_has_effect = get_mutation_mask().contains(finished_work.clone().unwrap().borrow().flags.clone());
        let root_has_effect = get_mutation_mask().contains(finished_work.clone().unwrap().borrow().flags.clone());

        let mut commit_work = &mut CommitWork::new();
        if subtree_has_effect || root_has_effect {
            commit_work.commit_mutation_effects(root.borrow().finished_work.clone());
            cloned.borrow_mut().current = cloned.borrow().finished_work.clone().unwrap();
        } else {
            cloned.borrow_mut().current = cloned.borrow().finished_work.clone().unwrap();
        }
    }

    fn prepare_fresh_stack(&mut self, root: Rc<RefCell<FiberRootNode>>) {
        let root = Rc::clone(&root);

        self.work_in_progress = Some(FiberNode::create_work_in_progress(
            root.borrow().current.clone(),
            Rc::new(JsValue::null()),
        ));
    }

    fn work_loop(&mut self) {
        while self.work_in_progress.is_some() {
            self.perform_unit_of_work(self.work_in_progress.clone().unwrap());
        }
    }

    fn perform_unit_of_work(&mut self, fiber: Rc<RefCell<FiberNode>>) {
        let next = begin_work(fiber.clone());
        if next.is_none() {
            self.complete_unit_of_work(fiber.clone())
        } else {
            self.work_in_progress = Some(next.unwrap());
        }
    }

    fn complete_unit_of_work(&mut self, fiber: Rc<RefCell<FiberNode>>) {
        let mut node: Option<Rc<RefCell<FiberNode>>> = Some(fiber);

        loop {
            let next = complete_work(node.clone().unwrap().clone());

            if next.is_some() {
                self.work_in_progress = next.clone();
                return;
            }

            let sibling = node.clone().unwrap().clone().borrow().sibling.clone();
            if sibling.is_some() {
                self.work_in_progress = next.clone();
                return;
            }

            let _return = node
                .clone()
                .unwrap()
                .clone()
                .borrow()
                ._return
                .clone();

            if _return.is_some() {
                self.work_in_progress = _return.unwrap().upgrade();
            } else {
                break;
            }
        }
    }
}
