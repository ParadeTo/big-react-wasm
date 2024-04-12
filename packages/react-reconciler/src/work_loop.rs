use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsValue;

use shared::log;

use crate::begin_work::begin_work;
use crate::fiber::{FiberNode, FiberRootNode, StateNode};
use crate::HostConfig;
use crate::work_tags::WorkTag;

pub struct WorkLoop {
    work_in_progress: Option<Rc<RefCell<FiberNode>>>,
}

impl WorkLoop {
    pub fn new(host_config: Rc<dyn HostConfig>) -> Self {
        Self {
            work_in_progress: None,
        }
    }

    pub fn schedule_update_on_fiber(&mut self, fiber: Rc<RefCell<FiberNode>>) {
        let root = self.mark_update_lane_from_fiber_to_root(fiber);
        if root.is_none() {
            return;
        }
        log!(
            "schedule_update_on_fiber - root container: {:?}",
            root.clone().unwrap().clone().borrow().container
        );

        self.ensure_root_is_scheduled(root.unwrap())
    }

    pub fn mark_update_lane_from_fiber_to_root(
        &self,
        fiber: Rc<RefCell<FiberNode>>,
    ) -> Option<Rc<RefCell<FiberRootNode>>> {
        let mut node = Rc::clone(&fiber);
        let mut parent = Rc::clone(&fiber).borrow()._return.clone();

        while parent.is_some() {
            node = parent.clone().unwrap();
            let rc = Rc::clone(&parent.unwrap());
            let rc_ref = rc.borrow();
            let next = match rc_ref._return.as_ref() {
                None => None,
                Some(node) => {
                    let a = node.clone();
                    Some(a)
                }
            };
            parent = next;
        }

        let fiber_node_rc = Rc::clone(&node);
        let fiber_node = fiber_node_rc.borrow();
        if fiber_node.tag == WorkTag::HostRoot {
            // match fiber_node.state_node.clone() {
            //     None => {}
            //     Some(state_node) => {
            //         return match &*state_node {
            //             StateNode::FiberRootNode(fiber_root_node) => {
            //                 Some(Rc::clone(&fiber_root_node))
            //             }
            //             StateNode::Element(_) => todo!(),
            //         };
            //     }
            // }
            //
            let Some(StateNode::FiberRootNode(fiber_root_node)) = fiber_node.state_node.clone();
            return Some(Rc::clone(&fiber_root_node));
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

        log!("{:?}", *root.clone().borrow());
        // commit
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
            log!(
                "work_loop - work_in_progress {:?}",
                self.work_in_progress.clone().unwrap().clone().borrow().tag
            );
            self.perform_unit_of_work(self.work_in_progress.clone().unwrap());
        }
    }

    fn perform_unit_of_work(&mut self, fiber: Rc<RefCell<FiberNode>>) {
        let next = begin_work(fiber.clone());

        if next.is_none() {
            // self.complete_unit_of_work(fiber.clone())
            self.work_in_progress = None;
        } else {
            log!(
                "perform_unit_of_work - next {:?}",
                next.clone().unwrap().clone().borrow().tag
            );
            self.work_in_progress = Some(next.unwrap());
        }
    }
}
