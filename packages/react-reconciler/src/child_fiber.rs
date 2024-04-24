use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsValue;
use web_sys::js_sys::{Object, Reflect};

use shared::{derive_from_js_value, log, REACT_ELEMENT_TYPE};

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
    should_track_effect: bool,
) -> Rc<RefCell<FiberNode>> {
    if should_track_effect && fiber.clone().borrow().alternate.is_none() {
        let fiber = fiber.clone();
        let mut fiber = fiber.borrow_mut();
        fiber.flags |= Flags::Placement;
    }
    return fiber;
}

fn delete_child(
    return_fiber: Rc<RefCell<FiberNode>>,
    child_to_delete: Rc<RefCell<FiberNode>>,
    should_track_effect: bool,
) {
    if !should_track_effect {
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

fn reconcile_single_element(
    return_fiber: Rc<RefCell<FiberNode>>,
    current_first_child: Option<Rc<RefCell<FiberNode>>>,
    element: Option<JsValue>,
    should_track_effect: bool,
) -> Rc<RefCell<FiberNode>> {
    if element.is_none() {
        panic!("reconcile_single_element, element is none")
    }

    let element = element.as_ref().unwrap();
    let key = derive_from_js_value(&(*element).clone(), "key");

    if current_first_child.is_some() {
        let current_first_child_cloned = current_first_child.clone().unwrap().clone();
        ;
        // Be careful, it is different with ===
        // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Equality_comparisons_and_sameness#same-value_equality_using_object.is
        if Object::is(&current_first_child_cloned.borrow().key, &key) {
            if derive_from_js_value(&(*element).clone(), "$$typeof") != REACT_ELEMENT_TYPE {
                panic!("Undefined $$typeof");
            }

            if Object::is(
                &current_first_child_cloned.borrow()._type,
                &derive_from_js_value(&(*element).clone(), "type"),
            ) {
                // type is the same, update props
                let existing = use_fiber(
                    current_first_child.clone().unwrap().clone(),
                    derive_from_js_value(&(*element).clone(), "props"),
                );
                existing.clone().borrow_mut()._return = Some(return_fiber);
                return existing;
            }
            delete_child(
                return_fiber.clone(),
                current_first_child.clone().unwrap().clone(),
                should_track_effect,
            );
        } else {
            delete_child(
                return_fiber.clone(),
                current_first_child.clone().unwrap().clone(),
                should_track_effect,
            );
        }
    }

    let mut fiber = FiberNode::create_fiber_from_element(element);
    fiber._return = Some(return_fiber.clone());
    Rc::new(RefCell::new(fiber))
}

fn reconcile_single_text_node(
    return_fiber: Rc<RefCell<FiberNode>>,
    current_first_child: Option<Rc<RefCell<FiberNode>>>,
    content: Option<JsValue>,
    should_track_effect: bool,
) -> Rc<RefCell<FiberNode>> {
    let props = Object::new();
    Reflect::set(&props, &JsValue::from("content"), &content.unwrap().clone())
        .expect("props panic");

    if current_first_child.is_some() && current_first_child.as_ref().unwrap().borrow().tag == HostText {
        let existing = use_fiber(current_first_child.as_ref().unwrap().clone(), (*props).clone());
        existing.borrow_mut()._return = Some(return_fiber.clone());
        return existing;
    }

    if current_first_child.is_some() {
        delete_child(return_fiber.clone(), current_first_child.clone().unwrap(), should_track_effect);
    }


    let mut created = FiberNode::new(WorkTag::HostText, (*props).clone(), JsValue::null());
    created._return = Some(return_fiber.clone());
    Rc::new(RefCell::new(created))
}

fn _reconcile_child_fibers(
    return_fiber: Rc<RefCell<FiberNode>>,
    current_first_child: Option<Rc<RefCell<FiberNode>>>,
    new_child: Option<JsValue>,
    should_track_effect: bool,
) -> Option<Rc<RefCell<FiberNode>>> {
    if new_child.is_some() {
        let new_child = &new_child.unwrap();

        if new_child.is_string() {
            return Some(place_single_child(
                reconcile_single_text_node(
                    return_fiber,
                    current_first_child,
                    Some(new_child.clone()),
                    should_track_effect,
                ),
                should_track_effect,
            ));
        } else if new_child.is_object() {
            if let Some(_typeof) = derive_from_js_value(&new_child, "$$typeof").as_string() {
                if _typeof == REACT_ELEMENT_TYPE {
                    return Some(place_single_child(
                        reconcile_single_element(
                            return_fiber,
                            current_first_child,
                            Some(new_child.clone()),
                            should_track_effect,
                        ),
                        should_track_effect,
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
