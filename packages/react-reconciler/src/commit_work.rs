use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use shared::{derive_from_js_value, log};

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
            finished_work.clone().borrow_mut().flags -= Flags::Placement;
        }

        if flags.contains(Flags::ChildDeletion) {
            let deletions = finished_work.clone().borrow().deletions.clone();
            if deletions.is_some() {
                let deletions = deletions.unwrap();
                for child_to_delete in deletions {
                    self.commit_deletion(child_to_delete);
                }
            }
            finished_work.clone().borrow_mut().flags -= Flags::ChildDeletion;
        }

        if flags.contains(Flags::Update) {
            self.commit_update(finished_work.clone());
            finished_work.clone().borrow_mut().flags -= Flags::Update;
        }
    }

    fn commit_update(&self, finished_work: Rc<RefCell<FiberNode>>) {
        let cloned = finished_work.clone();
        match cloned.borrow().tag {
            WorkTag::HostText => {
                let new_content = derive_from_js_value(&cloned.borrow().pending_props, "content");
                let state_node = FiberNode::derive_state_node(finished_work.clone());
                if let Some(state_node) = state_node.clone() {
                    self.host_config
                        .commit_text_update(state_node.clone(), new_content.as_string().unwrap())
                }
            }
            _ => log!("commit_update, unsupported type"),
        };
    }

    fn commit_deletion(&self, child_to_delete: Rc<RefCell<FiberNode>>) {
        let first_host_fiber: Rc<RefCell<Option<Rc<RefCell<FiberNode>>>>> = Rc::new(RefCell::new(None));
        self.commit_nested_unmounts(child_to_delete.clone(), |unmount_fiber| {
            let cloned = first_host_fiber.clone();
            match unmount_fiber.borrow().tag {
                WorkTag::FunctionComponent => {}
                WorkTag::HostRoot => {}
                WorkTag::HostComponent => {
                    if cloned.borrow().is_none() {
                        *cloned.borrow_mut() = Some(unmount_fiber.clone());
                    }
                }
                WorkTag::HostText => {
                    if cloned.borrow().is_none() {
                        *cloned.borrow_mut() = Some(unmount_fiber.clone());
                    }
                }
            };
        });

        let first_host_fiber = first_host_fiber.clone();
        if first_host_fiber.borrow().is_some() {
            let host_parent_state_node = FiberNode::derive_state_node(
                self.get_host_parent(child_to_delete.clone()).unwrap(),
            );
            let first_host_fiber_state_node =
                FiberNode::derive_state_node((*first_host_fiber.borrow()).clone().unwrap());
            self.host_config.remove_child(
                first_host_fiber_state_node.unwrap(),
                host_parent_state_node.unwrap(),
            )
        }

        child_to_delete.clone().borrow_mut()._return = None;
        child_to_delete.clone().borrow_mut().child = None;
    }

    fn on_commit_unmount(unmount_fiber: Rc<RefCell<FiberNode>>) {
        match unmount_fiber.clone().borrow().tag {
            WorkTag::FunctionComponent => {}
            WorkTag::HostRoot => {}
            WorkTag::HostComponent => {}
            WorkTag::HostText => {}
        };
    }

    fn commit_nested_unmounts<F>(&self, root: Rc<RefCell<FiberNode>>, on_commit_unmount: F)
        where
            F: Fn(Rc<RefCell<FiberNode>>),
    {
        let mut node = root.clone();
        loop {
            on_commit_unmount(node.clone());

            let node_cloned = node.clone();
            if node_cloned.borrow().child.is_some() {
                node_cloned
                    .borrow_mut()
                    .child
                    .clone()
                    .unwrap()
                    .clone()
                    .borrow_mut()
                    ._return = Some(node.clone());
                node = node_cloned.borrow().child.clone().unwrap();
                continue;
            }
            if Rc::ptr_eq(&node, &root.clone()) {
                return;
            }
            while node_cloned.borrow().sibling.is_none() {
                if node_cloned.borrow()._return.is_none()
                    || Rc::ptr_eq(node_cloned.borrow()._return.as_ref().unwrap(), &root)
                {
                    return;
                }
                node = node_cloned.borrow()._return.clone().unwrap();
            }

            node_cloned
                .borrow_mut()
                .sibling
                .clone()
                .unwrap()
                .clone()
                .borrow_mut()
                ._return = node_cloned.borrow()._return.clone();
            node = node_cloned.borrow().sibling.clone().unwrap();
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
            log!("{:?}", fiber.clone().borrow()._type);
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
