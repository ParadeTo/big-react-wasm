use std::{cell::RefCell, rc::Rc};

use shared::derive_from_js_value;

use crate::{
    fiber::FiberNode,
    fiber_context::pop_provider,
    fiber_flags::Flags,
    suspense_context::pop_suspense_handler,
    work_tags::WorkTag::{ContextProvider, SuspenseComponent},
};

pub fn unwind_work(wip: Rc<RefCell<FiberNode>>) -> Option<Rc<RefCell<FiberNode>>> {
    let flags = wip.borrow().flags.clone();
    let tag = wip.borrow().tag.clone();
    match tag {
        SuspenseComponent => {
            pop_suspense_handler();
            if (flags.clone() & Flags::ShouldCapture) != Flags::NoFlags
                && (flags.clone() & Flags::DidCapture) == Flags::NoFlags
            {
                wip.borrow_mut().flags = (flags - Flags::ShouldCapture) | Flags::DidCapture;
                return Some(wip.clone());
            }
            None
        }
        ContextProvider => {
            let context = derive_from_js_value(&wip.borrow()._type, "_context");
            pop_provider(&context);
            None
        }
        _ => None,
    }
}
