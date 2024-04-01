use std::cell::{Ref, RefCell};
use std::rc::Rc;

use wasm_bindgen::{JsCast, JsValue};
use web_sys::js_sys::Function;

use shared::log;

use crate::fiber::FiberNode;

#[derive(Clone, Debug)]
pub struct UpdateAction;


#[derive(Clone, Debug)]
pub struct Update {
    pub action: Option<Rc<JsValue>>,
}

#[derive(Clone, Debug)]
pub struct UpdateType {
    pub pending: Option<Update>,
}


#[derive(Clone, Debug)]
pub struct UpdateQueue {
    pub shared: UpdateType,
}


pub fn create_update(action: Rc<JsValue>) -> Update {
    Update { action: Some(action) }
}

pub fn enqueue_update(fiber: Ref<FiberNode>, update: Update) {
    if fiber.update_queue.is_some() {
        // let update_queue = fiber.update_queue.as_ref().unwrap();
        // let update_queue = update_queue;
        let uq = fiber.update_queue.clone().unwrap();
        let mut update_queue = uq.borrow_mut();
        update_queue.shared.pending = Some(update);
    }
}

pub fn process_update_queue(fiber: Rc<RefCell<FiberNode>>) {
    let mut rc_fiber = fiber.clone();
    let mut fiber = rc_fiber.borrow_mut();
    let mut new_state = None;
    match fiber.update_queue.clone() {
        None => {
            log!("{:?} process_update_queue, update_queue is empty", fiber)
        }
        Some(q) => {
            let update_queue = q.clone();
            let pending = update_queue.clone().borrow().shared.pending.clone();
            update_queue.borrow_mut().shared.pending = None;
            if pending.is_some() {
                let action = pending.unwrap().action;
                match action {
                    None => {}
                    Some(action) => {
                        let f = action.dyn_ref::<Function>();
                        new_state = match f {
                            None => Some(action.clone()),
                            Some(f) => Some(Rc::new(f.call0(&JsValue::null()).unwrap())),
                        }
                    }
                }
            }
        }
    }

    fiber.memoized_state = new_state
}