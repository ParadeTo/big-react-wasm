use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsValue;

use shared::{derive_from_js_value, REACT_ELEMENT};

use crate::fiber::FiberNode;
use crate::fiber::Flags;

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
    fiber._return = Some(Rc::downgrade(&return_fiber));
    Rc::new(RefCell::new(fiber))
}

pub fn reconcile_child_fibers(
    return_fiber: Rc<RefCell<FiberNode>>,
    current_first_child: Option<Rc<RefCell<FiberNode>>>,
    new_child: Option<Rc<JsValue>>,
) -> Option<Rc<RefCell<FiberNode>>> {
    if (new_child.is_some()) {
        let new_child = Rc::clone(&new_child.unwrap());
        let _typeof = Rc::clone(&derive_from_js_value(new_child.clone(), "_typeof").unwrap())
            .as_string()
            .unwrap();
        if _typeof == REACT_ELEMENT {
            return Some(place_single_child(reconcile_single_element(return_fiber, current_first_child, Some(new_child.clone())), true));
        }
    }
    return None;
}

pub fn mount_child_fibers(
    return_fiber: Rc<RefCell<FiberNode>>,
    current_first_child: Option<Rc<RefCell<FiberNode>>>,
    new_child: Option<Rc<JsValue>>,
) {}
