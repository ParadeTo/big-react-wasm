use std::{cell::RefCell, rc::Rc};

use crate::fiber::FiberNode;

static mut SUSPENSE_HANDLER_STACK: Vec<Rc<RefCell<FiberNode>>> = vec![];

pub fn get_suspense_handler() -> Rc<RefCell<FiberNode>> {
    unsafe { SUSPENSE_HANDLER_STACK[SUSPENSE_HANDLER_STACK.len() - 1].clone() }
}

pub fn push_suspense_handler(handler: Rc<RefCell<FiberNode>>) {
    unsafe { SUSPENSE_HANDLER_STACK.push(handler) }
}

pub fn pop_suspense_handler() {
    unsafe { SUSPENSE_HANDLER_STACK.pop() };
}
