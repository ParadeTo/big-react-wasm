use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use wasm_bindgen::{JsCast, JsValue};
use web_sys::js_sys::{Array, Object, Reflect};

use shared::{derive_from_js_value, log, type_of, REACT_ELEMENT_TYPE};

use crate::fiber::FiberNode;
use crate::fiber_flags::Flags;
use crate::work_tags::WorkTag;
use crate::work_tags::WorkTag::HostText;

fn use_fiber(fiber: Rc<RefCell<FiberNode>>, pending_props: JsValue) -> Rc<RefCell<FiberNode>> {
    let clone = FiberNode::create_work_in_progress(fiber, pending_props);
    clone.borrow_mut().index = 0;
    clone.borrow_mut().sibling = None;
    clone
}

fn place_single_child(
    fiber: Rc<RefCell<FiberNode>>,
    should_track_effects: bool,
) -> Rc<RefCell<FiberNode>> {
    if should_track_effects && fiber.clone().borrow().alternate.is_none() {
        let fiber = fiber.clone();
        let mut fiber = fiber.borrow_mut();
        fiber.flags |= Flags::Placement;
    }
    return fiber;
}

fn delete_child(
    return_fiber: Rc<RefCell<FiberNode>>,
    child_to_delete: Rc<RefCell<FiberNode>>,
    should_track_effects: bool,
) {
    if !should_track_effects {
        return;
    }

    let deletions = {
        let return_fiber_borrowed = return_fiber.borrow();
        return_fiber_borrowed.deletions.clone()
    };
    if deletions.is_empty() {
        return_fiber.borrow_mut().deletions = vec![child_to_delete.clone()];
        return_fiber.borrow_mut().flags |= Flags::ChildDeletion;
    } else {
        let del = &mut return_fiber.borrow_mut().deletions;
        del.push(child_to_delete.clone());
    }
}

fn delete_remaining_children(
    return_fiber: Rc<RefCell<FiberNode>>,
    current_first_child: Option<Rc<RefCell<FiberNode>>>,
    should_track_effects: bool,
) -> Option<Rc<RefCell<FiberNode>>> {
    if !should_track_effects {
        return None;
    }

    let mut child_to_delete = current_first_child;
    while child_to_delete.as_ref().is_some() {
        delete_child(
            return_fiber.clone(),
            child_to_delete.clone().unwrap(),
            should_track_effects,
        );
        child_to_delete = child_to_delete
            .clone()
            .unwrap()
            .clone()
            .borrow()
            .sibling
            .clone();
    }

    return None;
}

fn reconcile_single_element(
    return_fiber: Rc<RefCell<FiberNode>>,
    current_first_child: Option<Rc<RefCell<FiberNode>>>,
    element: Option<JsValue>,
    should_track_effects: bool,
) -> Rc<RefCell<FiberNode>> {
    if element.is_none() {
        panic!("reconcile_single_element, element is none")
    }

    let element = element.as_ref().unwrap();
    let key = derive_from_js_value(&(*element).clone(), "key");
    let mut current = current_first_child;
    while current.is_some() {
        let current_cloned = current.clone().unwrap().clone();
        // Be careful, it is different with ===
        // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Equality_comparisons_and_sameness#same-value_equality_using_object.is
        if Object::is(&current_cloned.borrow().key, &key) {
            if derive_from_js_value(&(*element).clone(), "$$typeof") != REACT_ELEMENT_TYPE {
                panic!("Undefined $$typeof");
            }

            if Object::is(
                &current_cloned.borrow()._type,
                &derive_from_js_value(&(*element).clone(), "type"),
            ) {
                // type is the same, update props
                let existing = use_fiber(
                    current_cloned.clone(),
                    derive_from_js_value(&(*element).clone(), "props"),
                );
                existing.clone().borrow_mut()._return = Some(return_fiber.clone());
                delete_remaining_children(
                    return_fiber.clone(),
                    current.clone().unwrap().borrow().sibling.clone(),
                    should_track_effects,
                );
                return existing;
            }
            delete_remaining_children(return_fiber.clone(), current.clone(), should_track_effects);
            break;
        } else {
            delete_child(
                return_fiber.clone(),
                current_cloned.clone(),
                should_track_effects,
            );
            current = current_cloned.borrow().sibling.clone();
        }
    }

    let mut fiber = FiberNode::create_fiber_from_element(element);
    fiber._return = Some(return_fiber.clone());
    Rc::new(RefCell::new(fiber))
}

fn create_props_with_content(content: JsValue) -> JsValue {
    let props = Object::new();
    Reflect::set(&props, &JsValue::from("content"), &content).expect("props panic");
    props.into()
}

fn reconcile_single_text_node(
    return_fiber: Rc<RefCell<FiberNode>>,
    current_first_child: Option<Rc<RefCell<FiberNode>>>,
    content: Option<JsValue>,
    should_track_effects: bool,
) -> Rc<RefCell<FiberNode>> {
    let props = create_props_with_content(content.unwrap());
    let mut current = current_first_child;
    while current.is_some() {
        let current_rc = current.clone().unwrap();
        if current_rc.borrow().tag == HostText {
            let existing = use_fiber(current_rc.clone(), props.clone());
            existing.borrow_mut()._return = Some(return_fiber.clone());
            delete_remaining_children(
                return_fiber.clone(),
                current_rc.borrow().sibling.clone(),
                should_track_effects,
            );
            return existing;
        }
        delete_child(
            return_fiber.clone(),
            current_rc.clone(),
            should_track_effects,
        );
        current = current_rc.borrow().sibling.clone();
    }

    let mut created = FiberNode::new(
        WorkTag::HostText,
        props.clone(),
        JsValue::null(),
        JsValue::null(),
    );
    created._return = Some(return_fiber.clone());
    Rc::new(RefCell::new(created))
}

struct Key(JsValue);

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        Object::is(&self.0, &other.0)
    }
}

impl Eq for Key {}

impl Hash for Key {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if self.0.is_string() {
            self.0.as_string().unwrap().hash(state)
        } else if let Some(n) = self.0.as_f64() {
            n.to_bits().hash(state)
        } else if self.0.is_null() {
            "null".hash(state)
        }
    }
}

fn update_from_map(
    return_fiber: Rc<RefCell<FiberNode>>,
    existing_children: &mut HashMap<Key, Rc<RefCell<FiberNode>>>,
    index: u32,
    element: &JsValue,
    should_track_effects: bool,
) -> Option<Rc<RefCell<FiberNode>>> {
    let key_to_use;
    if type_of(element, "string") || type_of(element, "null") || type_of(element, "number") {
        key_to_use = JsValue::from(index);
    } else {
        let key = derive_from_js_value(element, "key");
        key_to_use = match key.is_null() {
            true => JsValue::from(index),
            false => key.clone(),
        }
    }
    let before = existing_children.get(&Key(key_to_use.clone())).clone();
    if type_of(element, "null") || type_of(element, "string") || type_of(element, "number") {
        let props = create_props_with_content(element.clone());
        if before.is_some() {
            let before = (*before.clone().unwrap()).clone();
            existing_children.remove(&Key(key_to_use.clone()));
            if before.borrow().tag == HostText {
                return Some(use_fiber(before.clone(), props.clone()));
            } else {
                delete_child(return_fiber, before, should_track_effects);
            }
        }
        return if type_of(element, "null") {
            None
        } else {
            Some(Rc::new(RefCell::new(FiberNode::new(
                WorkTag::HostText,
                props.clone(),
                JsValue::null(),
                JsValue::null(),
            ))))
        };
    } else if type_of(element, "object") && !element.is_null() {
        if derive_from_js_value(&(*element).clone(), "$$typeof") != REACT_ELEMENT_TYPE {
            panic!("Undefined $$typeof");
        }

        if before.is_some() {
            let before = (*before.clone().unwrap()).clone();
            existing_children.remove(&Key(key_to_use.clone()));
            if Object::is(
                &before.borrow()._type,
                &derive_from_js_value(&(*element).clone(), "type"),
            ) {
                return Some(use_fiber(
                    before.clone(),
                    derive_from_js_value(element, "props"),
                ));
            } else {
                delete_child(return_fiber, before, should_track_effects);
            }
        }

        return Some(Rc::new(RefCell::new(FiberNode::create_fiber_from_element(
            element,
        ))));
    }
    None
}

fn reconcile_children_array(
    return_fiber: Rc<RefCell<FiberNode>>,
    current_first_child: Option<Rc<RefCell<FiberNode>>>,
    new_child: &Array,
    should_track_effects: bool,
) -> Option<Rc<RefCell<FiberNode>>> {
    // 遍历到的最后一个可复用fiber在before中的index
    let mut last_placed_index = 0;
    // 创建的最后一个fiber
    let mut last_new_fiber: Option<Rc<RefCell<FiberNode>>> = None;
    // 创建的第一个fiber
    let mut first_new_fiber: Option<Rc<RefCell<FiberNode>>> = None;

    let mut existing_children: HashMap<Key, Rc<RefCell<FiberNode>>> = HashMap::new();
    let mut current = current_first_child;
    while current.is_some() {
        let current_rc = current.unwrap();
        let key_to_use = match current_rc.clone().borrow().key.is_null() {
            true => JsValue::from(current_rc.borrow().index),
            false => current_rc.borrow().key.clone(),
        };
        existing_children.insert(Key(key_to_use), current_rc.clone());
        current = current_rc.borrow().sibling.clone();
    }

    let length = new_child.length();
    for i in 0..length {
        let after = new_child.get(i);
        let new_fiber = update_from_map(
            return_fiber.clone(),
            &mut existing_children,
            i,
            &after,
            should_track_effects,
        );

        if new_fiber.is_none() {
            continue;
        }

        let new_fiber = new_fiber.unwrap();

        {
            new_fiber.borrow_mut().index = i;
            new_fiber.borrow_mut()._return = Some(return_fiber.clone());
        }

        if last_new_fiber.is_none() {
            last_new_fiber = Some(new_fiber.clone());
            first_new_fiber = Some(new_fiber.clone());
        } else {
            last_new_fiber.clone().unwrap().clone().borrow_mut().sibling = Some(new_fiber.clone());
            last_new_fiber = Some(new_fiber.clone());
        }

        if !should_track_effects {
            continue;
        }

        let current = { new_fiber.borrow().alternate.clone() };
        if current.is_some() {
            let old_index = current.clone().unwrap().borrow().index;
            if old_index < last_placed_index {
                new_fiber.borrow_mut().flags |= Flags::Placement;
                continue;
            } else {
                last_placed_index = old_index;
            }
        } else {
            new_fiber.borrow_mut().flags |= Flags::Placement;
        }
    }

    for (_, fiber) in existing_children {
        delete_child(return_fiber.clone(), fiber, should_track_effects);
    }

    first_new_fiber
}

fn _reconcile_child_fibers(
    return_fiber: Rc<RefCell<FiberNode>>,
    current_first_child: Option<Rc<RefCell<FiberNode>>>,
    new_child: Option<JsValue>,
    should_track_effects: bool,
) -> Option<Rc<RefCell<FiberNode>>> {
    if new_child.is_some() {
        let new_child: &JsValue = &new_child.unwrap();

        if type_of(new_child, "string") || type_of(new_child, "number") {
            return Some(place_single_child(
                reconcile_single_text_node(
                    return_fiber,
                    current_first_child,
                    Some(new_child.clone()),
                    should_track_effects,
                ),
                should_track_effects,
            ));
        } else if new_child.is_array() {
            return reconcile_children_array(
                return_fiber,
                current_first_child,
                new_child.dyn_ref::<Array>().unwrap(),
                should_track_effects,
            );
        } else if new_child.is_object() {
            if let Some(_typeof) = derive_from_js_value(&new_child, "$$typeof").as_string() {
                if _typeof == REACT_ELEMENT_TYPE {
                    return Some(place_single_child(
                        reconcile_single_element(
                            return_fiber,
                            current_first_child,
                            Some(new_child.clone()),
                            should_track_effects,
                        ),
                        should_track_effects,
                    ));
                }
            }
        }
    }

    delete_remaining_children(return_fiber, current_first_child, should_track_effects)
}

pub fn clone_child_fiblers(wip: Rc<RefCell<FiberNode>>) {
    if wip.borrow().child.is_none() {
        return;
    }

    let mut current_child = { wip.borrow().child.clone().unwrap() };
    let pending_props = { current_child.borrow().pending_props.clone() };
    let mut new_child = FiberNode::create_work_in_progress(current_child.clone(), pending_props);
    wip.borrow_mut().child = Some(new_child.clone());
    new_child.borrow_mut()._return = Some(wip);

    while current_child.borrow().sibling.is_some() {
        let sibling = { current_child.borrow().sibling.clone().unwrap() };
        let pending_props = { sibling.borrow().pending_props.clone() };
        let new_slibing = FiberNode::create_work_in_progress(sibling.clone(), pending_props);
        new_child.borrow_mut().sibling = Some(new_slibing.clone());

        current_child = sibling;
        new_child = new_slibing;
    }
}

pub fn reconcile_child_fibers(
    return_fiber: Rc<RefCell<FiberNode>>,
    current_first_child: Option<Rc<RefCell<FiberNode>>>,
    new_child: Option<JsValue>,
) -> Option<Rc<RefCell<FiberNode>>> {
    _reconcile_child_fibers(return_fiber, current_first_child, new_child, true)
}

pub fn mount_child_fibers(
    return_fiber: Rc<RefCell<FiberNode>>,
    current_first_child: Option<Rc<RefCell<FiberNode>>>,
    new_child: Option<JsValue>,
) -> Option<Rc<RefCell<FiberNode>>> {
    _reconcile_child_fibers(return_fiber, current_first_child, new_child, false)
}
