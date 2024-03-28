use std::cell::Ref;
use std::rc::Rc;

use react::ReactElement;

use crate::fiber::FiberNode;

#[derive(Clone, Debug)]
pub struct UpdateAction;

#[derive(Clone, Debug)]
pub enum Action {
    ReactElement(Rc<ReactElement>)
}

#[derive(Clone, Debug)]
pub struct Update {
    pub action: Option<Action>,
}

#[derive(Clone, Debug)]
pub struct UpdateType {
    pub pending: Update,
}


#[derive(Clone, Debug)]
pub struct UpdateQueue {
    pub shared: UpdateType,
}


pub fn create_update(action: Rc<ReactElement>) -> Update {
    Update { action: Some(Action::ReactElement(action)) }
}

pub fn enqueue_update(fiber: Ref<FiberNode>, update: Update) {
    // let update_queue = fiber.update_queue.borrow_mut();
}