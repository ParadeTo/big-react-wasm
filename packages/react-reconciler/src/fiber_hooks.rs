use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::{JsCast, JsValue};
use web_sys::js_sys::Function;

use shared::log;

use crate::fiber::FiberNode;

//
// use wasm_bindgen::JsValue;
//
// use crate::fiber::FiberNode;
// use crate::update_queue::UpdateQueue;
//
// pub struct Hook {
//     memoized_state: Option<Rc<JsValue>>,
//     update_queue: Option<Rc<RefCell<UpdateQueue>>>,
//     next: Option<Rc<RefCell<Hook>>>,
// }
//
pub struct FiberHooks {
    currently_rendering_fiber: Option<Rc<RefCell<FiberNode>>>,
}

impl FiberHooks {
    pub fn new() -> Self {
        FiberHooks {
            currently_rendering_fiber: None
        }
    }

    pub fn render_with_hooks(&mut self, work_in_progress: Rc<RefCell<FiberNode>>) -> Result<JsValue, JsValue> {
        self.currently_rendering_fiber = Some(work_in_progress.clone());

        let work_in_progress_cloned = work_in_progress.clone();
        {
            work_in_progress_cloned.borrow_mut().memoized_state = None;
            work_in_progress_cloned.borrow_mut().update_queue = None;
        }


        let current = work_in_progress_cloned.borrow().alternate.clone();
        if current.is_some() {
            log!("还未实现update时renderWithHooks");
        } else {}

        let work_in_progress_borrow = work_in_progress_cloned.borrow();
        let _type = work_in_progress_borrow._type.as_ref().unwrap();
        let props = work_in_progress_borrow.pending_props.as_ref().unwrap();
        let component = JsValue::dyn_ref::<Function>(_type).unwrap();
        let children = component.call1(&JsValue::null(), props);
        children
    }
}

