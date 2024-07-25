use std::{cell::RefCell, rc::Rc};

use shared::{derive_from_js_value, log};
use wasm_bindgen::JsValue;
use web_sys::js_sys::{Object, Reflect};

use crate::{
    begin_work::mark_wip_received_update,
    fiber::{FiberDependencies, FiberNode},
    fiber_lanes::{include_some_lanes, is_subset_of_lanes, merge_lanes, Lane},
    work_tags::WorkTag,
};

static mut PREV_CONTEXT_VALUE: JsValue = JsValue::null();
static mut PREV_CONTEXT_VALUE_STACK: Vec<JsValue> = vec![];
static mut LAST_CONTEXT_DEP: Option<Rc<RefCell<ContextItem>>> = None;

#[derive(Clone, Debug)]
pub struct ContextItem {
    context: JsValue,
    memoized_state: JsValue,
    next: Option<Rc<RefCell<ContextItem>>>,
}

pub fn push_provider(context: &JsValue, new_value: JsValue) {
    unsafe {
        PREV_CONTEXT_VALUE_STACK.push(PREV_CONTEXT_VALUE.clone());
        PREV_CONTEXT_VALUE = Reflect::get(context, &"_currentValue".into()).unwrap();
        Reflect::set(context, &"_currentValue".into(), &new_value);
    }
}

pub fn pop_provider(context: &JsValue) {
    unsafe {
        Reflect::set(context, &"_currentValue".into(), &PREV_CONTEXT_VALUE);
        let top = PREV_CONTEXT_VALUE_STACK.pop();
        if top.is_none() {
            PREV_CONTEXT_VALUE = JsValue::null();
        } else {
            PREV_CONTEXT_VALUE = top.unwrap();
        }
    }
}

pub fn prepare_to_read_context(wip: Rc<RefCell<FiberNode>>, render_lane: Lane) {
    unsafe { LAST_CONTEXT_DEP = None };

    let deps = { wip.borrow().dependencies.clone() };

    if deps.is_some() {
        let deps = deps.unwrap();
        if deps.borrow().first_context.is_some() {
            if include_some_lanes(deps.borrow().lanes.clone(), render_lane) {
                mark_wip_received_update()
            }
            deps.borrow_mut().first_context = None;
        }
    }
}

pub fn read_context(consumer: Option<Rc<RefCell<FiberNode>>>, context: JsValue) -> JsValue {
    if consumer.is_none() {
        panic!("Can only call useContext in Function Component");
    }
    let consumer = consumer.unwrap();
    let value = derive_from_js_value(&context, "_currentValue");

    let context_item = Rc::new(RefCell::new(ContextItem {
        context,
        next: None,
        memoized_state: value.clone(),
    }));

    if unsafe { LAST_CONTEXT_DEP.is_none() } {
        unsafe { LAST_CONTEXT_DEP = Some(context_item.clone()) };
        consumer.borrow_mut().dependencies = Some(Rc::new(RefCell::new(FiberDependencies {
            first_context: Some(context_item),
            lanes: Lane::NoLane,
        })));
    } else {
        let next = Some(context_item.clone());
        unsafe {
            LAST_CONTEXT_DEP.clone().unwrap().borrow_mut().next = next.clone();
            LAST_CONTEXT_DEP = next;
        }
    }
    value
}

// DFS
pub fn propagate_context_change(wip: Rc<RefCell<FiberNode>>, context: JsValue, render_lane: Lane) {
    let mut fiber = { wip.borrow().child.clone() };
    if fiber.is_some() {
        fiber.as_ref().unwrap().borrow_mut()._return = Some(wip.clone());
    }

    while fiber.is_some() {
        let mut next_fiber = None;
        let fiber_unwrapped = fiber.clone().unwrap();
        let deps = { fiber_unwrapped.borrow().dependencies.clone() };
        if deps.is_some() {
            let deps = deps.unwrap();
            next_fiber = fiber_unwrapped.borrow().child.clone();
            let mut context_item = deps.borrow().first_context.clone();
            while context_item.is_some() {
                let context_item_unwrapped = context_item.unwrap();
                if Object::is(&context_item_unwrapped.borrow().context, &context) {
                    // find the FiberNode which depend on wip(context.Provider)
                    let lanes = { fiber_unwrapped.borrow().lanes.clone() };
                    fiber_unwrapped.borrow_mut().lanes = merge_lanes(lanes, render_lane.clone());
                    let alternate = { fiber_unwrapped.borrow().alternate.clone() };
                    if alternate.is_some() {
                        let alternate = alternate.unwrap();
                        let lanes = { alternate.borrow().lanes.clone() };
                        alternate.borrow_mut().lanes = merge_lanes(lanes, render_lane.clone());
                    }
                    // update ancestors' child_lanes
                    schedule_context_work_on_parent_path(
                        fiber_unwrapped.borrow()._return.clone(),
                        wip.clone(),
                        render_lane.clone(),
                    );
                    let lanes = { deps.borrow().lanes.clone() };
                    deps.borrow_mut().lanes = merge_lanes(lanes, render_lane.clone());
                    break;
                }
                context_item = context_item_unwrapped.borrow().next.clone();
            }
        } else if fiber_unwrapped.borrow().tag == WorkTag::ContextProvider {
            /*
             * const ctx = createContext()
             * <ctx.Provider> // propagate context change
             *  <div>
             *    <ctx.Provider> // stop here
             *       <div></div>
             *    <ctx.Provider>
             *  </div>
             * </ctx.Provider>
             */
            next_fiber = if Object::is(&fiber_unwrapped.borrow()._type, &wip.borrow()._type) {
                None
            } else {
                fiber_unwrapped.borrow().child.clone()
            };
        } else {
            next_fiber = fiber_unwrapped.borrow().child.clone();
        }

        if next_fiber.is_some() {
            next_fiber.clone().unwrap().borrow_mut()._return = fiber;
        } else {
            // Leaf Node
            next_fiber = fiber.clone();
            while next_fiber.is_some() {
                let next_fiber_unwrapped = next_fiber.unwrap();
                if Rc::ptr_eq(&next_fiber_unwrapped, &wip) {
                    next_fiber = None;
                    break;
                }

                let sibling = next_fiber_unwrapped.borrow().sibling.clone();
                if sibling.is_some() {
                    let sibling_unwrapped = sibling.clone().unwrap();
                    sibling_unwrapped.borrow_mut()._return =
                        next_fiber_unwrapped.borrow()._return.clone();
                    next_fiber = sibling.clone();
                    break;
                }
                next_fiber = next_fiber_unwrapped.borrow()._return.clone();
            }
        }

        fiber = next_fiber.clone();
    }
}

fn schedule_context_work_on_parent_path(
    from: Option<Rc<RefCell<FiberNode>>>,
    to: Rc<RefCell<FiberNode>>,
    render_lane: Lane,
) {
    let mut node = from;

    while node.is_some() {
        let node_unwrapped = node.unwrap();
        let alternate = { node_unwrapped.borrow().alternate.clone() };
        let child_lanes = { node_unwrapped.borrow().child_lanes.clone() };

        if !is_subset_of_lanes(child_lanes.clone(), render_lane.clone()) {
            node_unwrapped.borrow_mut().child_lanes =
                merge_lanes(child_lanes.clone(), render_lane.clone());
            if alternate.is_some() {
                let alternate_unwrapped = alternate.unwrap();
                let child_lanes = { alternate_unwrapped.borrow().child_lanes.clone() };
                alternate_unwrapped.borrow_mut().child_lanes =
                    merge_lanes(child_lanes.clone(), render_lane.clone());
            }
        } else if alternate.is_some() {
            let alternate_unwrapped = alternate.unwrap();
            let child_lanes = { alternate_unwrapped.borrow().child_lanes.clone() };
            if !is_subset_of_lanes(child_lanes.clone(), render_lane.clone()) {
                alternate_unwrapped.borrow_mut().child_lanes =
                    merge_lanes(child_lanes.clone(), render_lane.clone());
            }
        }

        if Rc::ptr_eq(&node_unwrapped, &to) {
            break;
        }

        node = node_unwrapped.borrow()._return.clone();
    }
}
