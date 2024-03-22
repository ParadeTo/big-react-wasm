use std::cell::Ref;

use crate::fiber::FiberNode;

pub trait Action {}

#[derive(Clone, Debug)]
pub struct UpdateAction;

#[derive(Clone, Debug)]
pub struct Update {
    pub action: Option<dyn Action>,
}

#[derive(Clone, Debug)]
pub struct UpdateType {
    pub pending: Update,
}


#[derive(Clone, Debug)]
pub struct UpdateQueue {
    pub shared: UpdateType,
}

pub fn create_update(action: dyn Action) -> Update {
    Update { action: Some(action) }
}

pub fn enqueue_update(fiber: Ref<FiberNode>, update: Update) {
    let update_queue = fiber.update_queue.borrow_mut();
}