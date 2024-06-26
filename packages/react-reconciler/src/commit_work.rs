use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::{JsCast, JsValue};
use web_sys::js_sys::Function;

use shared::{derive_from_js_value, log};

use crate::fiber::{FiberNode, FiberRootNode, StateNode};
use crate::fiber_flags::{Flags, get_mutation_mask, get_passive_mask};
use crate::fiber_hooks::Effect;
use crate::HostConfig;
use crate::work_tags::WorkTag;
use crate::work_tags::WorkTag::{HostComponent, HostRoot, HostText};

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

    fn commit_passive_effect(
        finished_work: Rc<RefCell<FiberNode>>,
        root: Rc<RefCell<FiberRootNode>>,
        _type: &str,
    ) {
        let finished_work_b = finished_work.borrow();
        if finished_work_b.tag != WorkTag::FunctionComponent
            || (_type == "update"
            && (finished_work_b.flags.clone() & Flags::PassiveEffect == Flags::NoFlags))
        {
            return;
        }

        let update_queue = &finished_work_b.update_queue;
        if update_queue.is_some() {
            let update_queue = update_queue.clone().unwrap();
            if update_queue.borrow().last_effect.is_none() {
                log!("When FC has PassiveEffect, the effect should exist.")
            }
            if _type == "unmount" {
                root.borrow()
                    .pending_passive_effects
                    .borrow_mut()
                    .unmount
                    .push(update_queue.borrow().last_effect.clone().unwrap());
            } else {
                root.borrow()
                    .pending_passive_effects
                    .borrow_mut()
                    .update
                    .push(update_queue.borrow().last_effect.clone().unwrap());
            }
        }
    }

    pub fn commit_hook_effect_list(
        flags: Flags,
        last_effect: Rc<RefCell<Effect>>,
        callback: fn(effect: Rc<RefCell<Effect>>),
    ) {
        let mut effect = last_effect.borrow().next.clone();
        loop {
            let mut effect_rc = effect.clone().unwrap();
            if effect_rc.borrow().tag.clone() & flags.clone() == flags.clone() {
                callback(effect_rc.clone())
            }
            effect = effect_rc.borrow().next.clone();
            if Rc::ptr_eq(
                &effect.clone().unwrap(),
                last_effect.borrow().next.as_ref().unwrap(),
            ) {
                break;
            }
        }
    }
    pub fn commit_hook_effect_list_destroy(flags: Flags, last_effect: Rc<RefCell<Effect>>) {
        CommitWork::commit_hook_effect_list(flags, last_effect, |effect: Rc<RefCell<Effect>>| {
            let destroy = { effect.borrow().destroy.clone() };
            if destroy.is_function() {
                destroy
                    .dyn_ref::<Function>()
                    .unwrap()
                    .call0(&JsValue::null());
            }
            effect.borrow_mut().tag &= !Flags::HookHasEffect;
        });
    }

    pub fn commit_hook_effect_list_unmount(flags: Flags, last_effect: Rc<RefCell<Effect>>) {
        CommitWork::commit_hook_effect_list(flags, last_effect, |effect: Rc<RefCell<Effect>>| {
            let destroy = &effect.borrow().destroy;
            if destroy.is_function() {
                destroy
                    .dyn_ref::<Function>()
                    .unwrap()
                    .call0(&JsValue::null());
            }
        });
    }

    pub fn commit_hook_effect_list_mount(flags: Flags, last_effect: Rc<RefCell<Effect>>) {
        CommitWork::commit_hook_effect_list(flags, last_effect, |effect: Rc<RefCell<Effect>>| {
            let create = { effect.borrow().create.clone() };
            if create.is_function() {
                let destroy = create.call0(&JsValue::null()).unwrap();
                effect.borrow_mut().destroy = destroy;
            }
        });
    }

    pub fn commit_mutation_effects(
        &mut self,
        finished_work: Rc<RefCell<FiberNode>>,
        root: Rc<RefCell<FiberRootNode>>,
    ) {
        self.next_effect = Some(finished_work);
        while self.next_effect.is_some() {
            let next_effect = self.next_effect.clone().unwrap().clone();
            let child = next_effect.borrow().child.clone();
            if child.is_some()
                && next_effect.borrow().subtree_flags.clone()
                & (get_mutation_mask() | get_passive_mask())
                != Flags::NoFlags
            {
                self.next_effect = child;
            } else {
                while self.next_effect.is_some() {
                    self.commit_mutation_effects_on_fiber(
                        self.next_effect.clone().unwrap(),
                        root.clone(),
                    );
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

    fn commit_mutation_effects_on_fiber(
        &self,
        finished_work: Rc<RefCell<FiberNode>>,
        root: Rc<RefCell<FiberRootNode>>,
    ) {
        let flags = finished_work.borrow().flags.clone();
        if flags.contains(Flags::Placement) {
            self.commit_placement(finished_work.clone());
            finished_work.borrow_mut().flags -= Flags::Placement;
        }

        if flags.contains(Flags::ChildDeletion) {
            {
                let deletions = &finished_work.borrow().deletions;
                if !deletions.is_empty() {
                    for child_to_delete in deletions {
                        self.commit_deletion(child_to_delete.clone(), root.clone());
                    }
                }
            }

            finished_work.borrow_mut().flags -= Flags::ChildDeletion;
        }

        if flags.contains(Flags::Update) {
            self.commit_update(finished_work.clone());
            finished_work.borrow_mut().flags -= Flags::Update;
        }

        if flags & Flags::PassiveEffect != Flags::NoFlags {
            CommitWork::commit_passive_effect(finished_work.clone(), root, "update");
            finished_work.borrow_mut().flags -= Flags::PassiveEffect;
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
                        .commit_text_update(state_node.clone(), &new_content)
                }
            }
            _ => log!("commit_update, unsupported type"),
        };
    }

    fn commit_deletion(
        &self,
        child_to_delete: Rc<RefCell<FiberNode>>,
        root: Rc<RefCell<FiberRootNode>>,
    ) {
        let first_host_fiber: Rc<RefCell<Option<Rc<RefCell<FiberNode>>>>> =
            Rc::new(RefCell::new(None));
        self.commit_nested_unmounts(child_to_delete.clone(), |unmount_fiber| {
            let cloned = first_host_fiber.clone();
            match unmount_fiber.borrow().tag {
                WorkTag::FunctionComponent => {
                    CommitWork::commit_passive_effect(
                        unmount_fiber.clone(),
                        root.clone(),
                        "unmount",
                    );
                }
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
            while node.clone().borrow().sibling.is_none() {
                if node.clone().borrow()._return.is_none()
                    || Rc::ptr_eq(node.clone().borrow()._return.as_ref().unwrap(), &root)
                {
                    return;
                }
                node = node.clone().borrow()._return.clone().unwrap();
            }

            let node_cloned = node.clone();
            let _return = { node_cloned.borrow()._return.clone() };
            node_cloned
                .borrow_mut()
                .sibling
                .clone()
                .unwrap()
                .clone()
                .borrow_mut()
                ._return = _return;
            node = node_cloned.borrow().sibling.clone().unwrap();
        }
    }

    fn commit_placement(&self, finished_work: Rc<RefCell<FiberNode>>) {
        let host_parent = self.get_host_parent(finished_work.clone());
        if host_parent.is_none() {
            return;
        }
        let parent_state_node = FiberNode::derive_state_node(host_parent.unwrap());
        let sibling = self.get_host_sibling(finished_work.clone());

        if parent_state_node.is_some() {
            self.insert_or_append_placement_node_into_container(
                finished_work.clone(),
                parent_state_node.unwrap(),
                sibling,
            );
        }
    }

    fn get_element_from_state_node(&self, state_node: Rc<StateNode>) -> Rc<dyn Any> {
        match &*state_node {
            StateNode::FiberRootNode(root) => root.clone().borrow().container.clone(),
            StateNode::Element(ele) => ele.clone(),
        }
    }

    fn insert_or_append_placement_node_into_container(
        &self,
        fiber: Rc<RefCell<FiberNode>>,
        parent: Rc<dyn Any>,
        before: Option<Rc<dyn Any>>,
    ) {
        let fiber = fiber.clone();
        let tag = fiber.borrow().tag.clone();
        if tag == WorkTag::HostComponent || tag == WorkTag::HostText {
            let state_node = fiber.clone().borrow().state_node.clone().unwrap();
            let state_node = self.get_element_from_state_node(state_node);

            if before.is_some() {
                self.host_config.insert_child_to_container(
                    state_node,
                    parent,
                    before.clone().unwrap(),
                );
            } else {
                self.host_config
                    .append_child_to_container(state_node, parent.clone());
            }

            return;
        }

        let child = fiber.borrow().child.clone();
        if child.is_some() {
            self.insert_or_append_placement_node_into_container(
                child.clone().unwrap(),
                parent.clone(),
                before.clone(),
            );
            let mut sibling = child.unwrap().clone().borrow().sibling.clone();
            while sibling.is_some() {
                self.insert_or_append_placement_node_into_container(
                    sibling.clone().unwrap(),
                    parent.clone(),
                    before.clone(),
                );
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

    /**
     * 难点在于目标fiber的hostSibling可能并不是他的同级sibling
     * 比如： <A/><B/> 其中：function B() {return <div/>} 所以A的hostSibling实际是B的child
     * 实际情况层级可能更深
     * 同时：一个fiber被标记Placement，那他就是不稳定的（他对应的DOM在本次commit阶段会移动），也不能作为hostSibling
     */
    fn get_host_sibling(&self, fiber: Rc<RefCell<FiberNode>>) -> Option<Rc<dyn Any>> {
        let mut node = Some(fiber);
        'find_sibling: loop {
            let node_rc = node.clone().unwrap();
            while node_rc.borrow().sibling.is_none() {
                let parent = node_rc.borrow()._return.clone();
                let tag = parent.clone().unwrap().borrow().tag.clone();
                if parent.is_none() || tag == HostComponent || tag == HostRoot {
                    return None;
                }
                node = parent.clone();
            }

            let node_rc = node.clone().unwrap();
            let _return = { node_rc.borrow()._return.clone() };
            node_rc
                .borrow_mut()
                .sibling
                .clone()
                .unwrap()
                .borrow_mut()
                ._return = _return;
            node = node_rc.borrow().sibling.clone();

            let node_rc = node.clone().unwrap();
            let tag = node_rc.borrow().tag.clone();
            while tag != HostText && tag != HostComponent {
                if node_rc.borrow().flags.contains(Flags::Placement) {
                    continue 'find_sibling;
                }
                if node_rc.borrow().child.is_none() {
                    continue 'find_sibling;
                } else {
                    node_rc
                        .borrow_mut()
                        .child
                        .clone()
                        .unwrap()
                        .borrow_mut()
                        ._return = node.clone();
                    node = node_rc.borrow().child.clone();
                }
            }
            if !node
                .clone()
                .unwrap()
                .borrow()
                .flags
                .contains(Flags::Placement)
            {
                return Some(self.get_element_from_state_node(
                    node.clone().unwrap().borrow().state_node.clone().unwrap(),
                ));
            }
        }
    }
}
