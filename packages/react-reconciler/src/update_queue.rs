use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::{JsCast, JsValue};
use web_sys::js_sys::Function;

use shared::log;

use crate::fiber::{FiberNode, MemoizedState};

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
    Update {
        action: Some(action),
    }
}

pub fn enqueue_update(update_queue: Rc<RefCell<UpdateQueue>>, update: Update) {
    update_queue.borrow_mut().shared.pending = Option::from(update);
}

pub fn create_update_queue() -> Rc<RefCell<UpdateQueue>> {
    Rc::new(RefCell::new(UpdateQueue {
        shared: UpdateType {
            pending: None,
        },
    }))
}

pub fn process_update_queue(fiber: Rc<RefCell<FiberNode>>) {
    let rc_fiber = fiber.clone();
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
                            None => Some(MemoizedState::JsValue(action.clone())),
                            Some(f) => Some(MemoizedState::JsValue(Rc::new(
                                f.call0(&JsValue::null()).unwrap(),
                            ))),
                        }
                    }
                }
            }
        }
    }

    fiber.memoized_state = new_state
}
