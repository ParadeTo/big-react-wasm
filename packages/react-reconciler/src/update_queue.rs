use std::cell::Ref;
use std::rc::Rc;

use wasm_bindgen::JsValue;

use crate::fiber::FiberNode;

#[derive(Clone, Debug)]
pub struct UpdateAction;

#[derive(Clone, Debug)]
pub struct Update {
    pub action: Option<Rc<JsValue>>,
}

#[derive(Clone, Debug)]
pub struct UpdateType {
    pub pending: Update,
}


#[derive(Clone, Debug)]
pub struct UpdateQueue {
    pub shared: UpdateType,
}


pub fn create_update(action: Rc<JsValue>) -> Update {
    Update { action: Some(action) }
}

pub fn enqueue_update(fiber: Ref<FiberNode>, update: Update) {
    // let update_queue = fiber.update_queue.borrow_mut();
}