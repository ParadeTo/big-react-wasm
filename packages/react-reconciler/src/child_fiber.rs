use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsValue;
use web_sys::js_sys::{Object, Reflect};

use shared::{derive_from_js_value, log, REACT_ELEMENT_TYPE};

use crate::fiber::FiberNode;
use crate::fiber_flags::Flags;
use crate::work_tags::WorkTag;

fn place_single_child(
    fiber: Rc<RefCell<FiberNode>>,
    should_track_effect: bool,
) -> Rc<RefCell<FiberNode>> {
    if should_track_effect {
        let fiber = fiber.clone();
        let mut fiber = fiber.borrow_mut();
        fiber.flags |= Flags::Placement;
    }
    return fiber;
}

fn reconcile_single_element(
    return_fiber: Rc<RefCell<FiberNode>>,
    current_first_child: Option<Rc<RefCell<FiberNode>>>,
    element: Option<Rc<JsValue>>,
) -> Rc<RefCell<FiberNode>> {
    let mut fiber = FiberNode::create_fiber_from_element(element.unwrap());
    fiber._return = Some(return_fiber.clone());
    Rc::new(RefCell::new(fiber))
}

fn reconcile_single_text_node(
    return_fiber: Rc<RefCell<FiberNode>>,
    current_first_child: Option<Rc<RefCell<FiberNode>>>,
    content: Option<Rc<JsValue>>,
) -> Rc<RefCell<FiberNode>> {
    let props = Object::new();
    Reflect::set(
        &props,
        &JsValue::from("content"),
        &content.unwrap().clone(),
    )
        .expect("props panic");
    let mut created = FiberNode::new(WorkTag::HostText, Some(Rc::new(Object::into(props))), None);
    created._return = Some(return_fiber.clone());
    Rc::new(RefCell::new(created))
}

fn _reconcile_child_fibers(
    return_fiber: Rc<RefCell<FiberNode>>,
    current_first_child: Option<Rc<RefCell<FiberNode>>>,
    new_child: Option<Rc<JsValue>>,
    should_track_effect: bool,
) -> Option<Rc<RefCell<FiberNode>>> {
    if new_child.is_some() {
        let new_child = Rc::clone(&new_child.unwrap());

        if new_child.is_string() {
            return Some(place_single_child(
                reconcile_single_text_node(
                    return_fiber,
                    current_first_child,
                    Some(new_child.clone()),
                ),
                should_track_effect,
            ));
        } else if new_child.is_object() {
            log!("{:?}", new_child);
            let _typeof = Rc::clone(&derive_from_js_value(new_child.clone(), "$$typeof").unwrap())
                .as_string()
                .unwrap();
            if _typeof == REACT_ELEMENT_TYPE {
                return Some(place_single_child(
                    reconcile_single_element(
                        return_fiber,
                        current_first_child,
                        Some(new_child.clone()),
                    ),
                    should_track_effect,
                ));
            }
        }
    }
    log!("Unsupported child type when reconcile");
    return None;
}

pub fn reconcile_child_fibers(
    return_fiber: Rc<RefCell<FiberNode>>,
    current_first_child: Option<Rc<RefCell<FiberNode>>>,
    new_child: Option<Rc<JsValue>>,
) -> Option<Rc<RefCell<FiberNode>>> {
    _reconcile_child_fibers(return_fiber, current_first_child, new_child, true)
}

pub fn mount_child_fibers(
    return_fiber: Rc<RefCell<FiberNode>>,
    current_first_child: Option<Rc<RefCell<FiberNode>>>,
    new_child: Option<Rc<JsValue>>,
) -> Option<Rc<RefCell<FiberNode>>> {
    _reconcile_child_fibers(return_fiber, current_first_child, new_child, false)
}