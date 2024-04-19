use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsValue;

use crate::fiber::FiberNode;
use crate::update_queue::UpdateQueue;

pub struct Hook {
    memoized_state: Option<Rc<JsValue>>,
    update_queue: Option<Rc<RefCell<UpdateQueue>>>,
    next: Option<Rc<RefCell<Hook>>>,
}

pub struct FiberHooks {
    work_in_progress_hook: Option<Rc<RefCell<Hook>>>,
    currently_rendering_fiber: Option<Rc<FiberNode>>,
}