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
    pub action: Option<JsValue>,
}

#[derive(Clone, Debug)]
pub struct UpdateType {
    pub pending: Option<Update>,
}

#[derive(Clone, Debug)]
pub struct UpdateQueue {
    pub shared: UpdateType,
    pub dispatch: Option<Function>,
}

pub fn create_update(action: JsValue) -> Update {
    Update {
        action: Some(action),
    }
}

pub fn enqueue_update(update_queue: Rc<RefCell<UpdateQueue>>, update: Update) {
    update_queue.borrow_mut().shared.pending = Option::from(update);
}

pub fn create_update_queue() -> Rc<RefCell<UpdateQueue>> {
    Rc::new(RefCell::new(UpdateQueue {
        shared: UpdateType { pending: None },
        dispatch: None,
    }))
}

pub fn process_update_queue(
    mut base_state: Option<MemoizedState>,
    update_queue: Option<Rc<RefCell<UpdateQueue>>>,
    fiber: Rc<RefCell<FiberNode>>,
) -> Option<MemoizedState> {
    if update_queue.is_some() {
        let update_queue = update_queue.clone().unwrap().clone();
        let pending = update_queue.borrow().shared.pending.clone();
        update_queue.borrow_mut().shared.pending = None;
        if pending.is_some() {
            let action = pending.unwrap().action;
            match action {
                None => {}
                Some(action) => {
                    let f = action.dyn_ref::<Function>();
                    base_state = match f {
                        None => Some(MemoizedState::MemoizedJsValue(action.clone())),
                        Some(f) => {
                            if let MemoizedState::MemoizedJsValue(base_state) =
                                base_state.as_ref().unwrap()
                            {
                                Some(MemoizedState::MemoizedJsValue(
                                    f.call1(&JsValue::null(), base_state).unwrap(),
                                ))
                            } else {
                                log!("process_update_queue, base_state is not JsValue");
                                None
                            }
                        }
                    }
                }
            }
        }
    } else {
        log!("{:?} process_update_queue, update_queue is empty", fiber)
    }

    base_state
}
