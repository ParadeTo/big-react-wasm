use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use wasm_bindgen::{JsCast, JsValue};
use web_sys::js_sys::{Array, Object, Reflect};

use shared::{derive_from_js_value, log, REACT_ELEMENT_TYPE, type_of};

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
    if deletions.is_none() {
        return_fiber.borrow_mut().deletions = Some(vec![child_to_delete.clone()]);
        return_fiber.borrow_mut().flags |= Flags::ChildDeletion;
    } else {
        let mut del = return_fiber.borrow_mut().deletions.clone().unwrap();
        del.push(child_to_delete.clone());
    }
}

fn delete_remaining_children(
    return_fiber: Rc<RefCell<FiberNode>>,
    current_first_child: Option<Rc<RefCell<FiberNode>>>,
    should_track_effects: bool,
) {
    if !should_track_effects {
        return;
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

    let mut created = FiberNode::new(WorkTag::HostText, props.clone(), JsValue::null());
    created._return = Some(return_fiber.clone());
    Rc::new(RefCell::new(created))
}

fn update_from_map(
    return_fiber: Rc<RefCell<FiberNode>>,
    mut existing_children: HashMap<String, Rc<RefCell<FiberNode>>>,
    index: u32,
    element: &JsValue,
    should_track_effects: bool,
) -> Rc<RefCell<FiberNode>> {
    let key_to_use;
    if type_of(element, "string") {
        key_to_use = index.to_string();
    } else {
        let key = derive_from_js_value(element, "key");
        key_to_use = match key.is_null() {
            true => index.to_string(),
            false => match key.as_string() {
                None => {
                    log!(
                        "update_from_map, key is not string {:?}",
                        derive_from_js_value(element, "key")
                    );
                    "".to_string()
                }
                Some(k) => k,
            },
        }
    }

    let before = existing_children.get(&key_to_use).clone();
    if type_of(element, "string") {
        let props = create_props_with_content(element.clone());
        if before.is_some() {
            let before = (*before.clone().unwrap()).clone();
            existing_children.remove(&key_to_use);
            if before.borrow().tag == HostText {
                return use_fiber(before.clone(), props.clone());
            } else {
                delete_child(return_fiber, before, should_track_effects);
            }
        }
        return Rc::new(RefCell::new(FiberNode::new(
            WorkTag::HostText,
            props.clone(),
            JsValue::null(),
        )));
    } else if type_of(element, "object") && !element.is_null() {
        if derive_from_js_value(&(*element).clone(), "$$typeof") != REACT_ELEMENT_TYPE {
            panic!("Undefined $$typeof");
        }

        if before.is_some() {
            let before = (*before.clone().unwrap()).clone();
            existing_children.remove(&key_to_use);
            if Object::is(
                &before.borrow()._type,
                &derive_from_js_value(&(*element).clone(), "type"),
            ) {
                return use_fiber(before.clone(), derive_from_js_value(element, "props"));
            } else {
                delete_child(return_fiber, before, should_track_effects);
            }
        }

        return Rc::new(RefCell::new(FiberNode::create_fiber_from_element(element)));
    }
    panic!("update_from_map unsupported");
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

    let mut existing_children: HashMap<String, Rc<RefCell<FiberNode>>> = HashMap::new();
    let mut current = current_first_child;
    while current.is_some() {
        let current_rc = current.unwrap();
        let key_to_use = match current_rc.clone().borrow().key.is_null() {
            true => current_rc.borrow().index.to_string(),
            false => current_rc
                .borrow()
                .key
                .as_string()
                .expect("key is not string"),
        };
        existing_children.insert(key_to_use, current_rc.clone());
        current = current_rc.borrow().sibling.clone();
    }

    let length = new_child.length();
    for i in 0..length {
        let after = new_child.get(i);
        let new_fiber = update_from_map(
            return_fiber.clone(),
            existing_children.clone(),
            i,
            &after,
            should_track_effects,
        );
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
    log!("Unsupported child type when reconcile");
    None
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
