use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen::prelude::{Closure, wasm_bindgen};
use web_sys::js_sys::Function;

use shared::log;

use crate::current_dispatcher::{CURRENT_DISPATCHER, Dispatcher};
use crate::fiber::{FiberNode, MemoizedState};
use crate::update_queue::UpdateQueue;

//
// use wasm_bindgen::JsValue;
//
// use crate::fiber::FiberNode;
// use crate::update_queue::UpdateQueue;
//
#[derive(Debug, Clone)]
pub struct Hook {
    memoized_state: Option<Rc<JsValue>>,
    update_queue: Option<Rc<RefCell<UpdateQueue>>>,
    next: Option<Rc<RefCell<Hook>>>,
}

impl Hook {
    fn new() -> Self {
        Hook {
            memoized_state: None,
            update_queue: None,
            next: None,
        }
    }
}

static mut CURRENTLY_RENDERING_FIBER: Option<Rc<RefCell<FiberNode>>> = None;
static mut WORK_IN_PROGRESS_HOOK: Option<Rc<RefCell<Hook>>> = None;

pub fn render_with_hooks(work_in_progress: Rc<RefCell<FiberNode>>) -> Result<JsValue, JsValue> {
    unsafe { CURRENTLY_RENDERING_FIBER = Some(work_in_progress.clone()); }

    let work_in_progress_cloned = work_in_progress.clone();
    {
        work_in_progress_cloned.borrow_mut().memoized_state = None;
        work_in_progress_cloned.borrow_mut().update_queue = None;
    }


    let current = work_in_progress_cloned.borrow().alternate.clone();
    if current.is_some() {
        log!("还未实现update时renderWithHooks");
    } else {
        let use_callback = || {
            log!("use_callback");
        };
        let b = Box::new(Dispatcher::new(&mount_state, &use_callback));
        unsafe {
            CURRENT_DISPATCHER.current = Some(b);
        }
    }

    let _type;
    let props;
    {
        let work_in_progress_borrow = work_in_progress_cloned.borrow();
        _type = work_in_progress_borrow._type.clone().unwrap();
        props = work_in_progress_borrow.pending_props.clone().unwrap();
    }

    let component = JsValue::dyn_ref::<Function>(&_type).unwrap();
    let children = component.call1(&JsValue::null(), &props);
    children
}

fn mount_work_in_progress_nook() -> Option<Rc<RefCell<Hook>>> {
    let hook = Rc::new(RefCell::new(Hook::new()));
    unsafe {
        if WORK_IN_PROGRESS_HOOK.is_none() {
            if CURRENTLY_RENDERING_FIBER.is_none() {
                log!("WORK_IN_PROGRESS_HOOK and CURRENTLY_RENDERING_FIBER is empty")
            } else {
                CURRENTLY_RENDERING_FIBER.as_ref().unwrap().clone().borrow_mut().memoized_state = Some(MemoizedState::Hook(hook.clone()));
                WORK_IN_PROGRESS_HOOK = Some(hook.clone());
            }
        } else {
            WORK_IN_PROGRESS_HOOK.as_ref().unwrap().clone().borrow_mut().next = Some(hook.clone());
            WORK_IN_PROGRESS_HOOK = Some(hook.clone());
        }
        WORK_IN_PROGRESS_HOOK.clone()
    }
}

fn mount_state(initial_state: &JsValue) -> Vec<JsValue> {
    let hook = mount_work_in_progress_nook();
    // let memoizedState: State;
    // if (initialState instanceof Function) {
    //     memoizedState = initialState();
    // } else {
    //     memoizedState = initialState;
    // }
    // hook.memoizedState = memoizedState;
    //
    // if (currentlyRenderingFiber === null) {
    //     console.error('mountState时currentlyRenderingFiber不存在');
    // }
    // const queue = createUpdateQueue<State>();
    // hook.updateQueue = queue;

    // let closure = Closure::wrap(Box::new(|| unsafe {
    //     // dispatch_set_state1(CURRENTLY_RENDERING_FIBER.clone());
    //     log!("closure")
    // }));

    let closure = Closure::wrap(Box::new(move |action: &JsValue| unsafe {
        // web_sys::console::log_1(&"Hello, world!".into());
        dispatch_set_state(CURRENTLY_RENDERING_FIBER.clone(), action)
    }) as Box<dyn Fn(&JsValue)>);

    let function = closure.as_ref().unchecked_ref::<Function>().clone();

    // Don't forget to forget the closure or it will be cleaned up when it goes out of scope.
    closure.forget();


    return vec![
        initial_state.to_owned(),
        function.into(),
    ];
}

fn dispatch_set_state1(fiber: Option<Rc<RefCell<FiberNode>>>) {
    log!("dispatch_set_state {:?}", fiber)
}

fn dispatch_set_state(fiber: Option<Rc<RefCell<FiberNode>>>, action: &JsValue) {
    log!("dispatch_set_state {:?}", action)
}


// pub fn update_current_dispatcher() {
//     unsafe {
//         let use_state = || {
//             log!("use_state");
//             vec![JsValue::null(), JsValue::null()]
//         };
//         let use_callback = || {
//             log!("use_callback");
//         };
//         let b = Box::new(Dispatcher::new(&use_state, &use_callback));
//         CURRENT_DISPATCHER.current = Some(b);
//     }
// }

#[wasm_bindgen(js_name = useStateImpl)]
pub unsafe fn use_state(initial_state: &JsValue) -> Vec<JsValue> {
    let dispatcher = CURRENT_DISPATCHER.current.as_ref();
    if dispatcher.is_none() {
        log!("dispatcher doesn't exist")
    }
    let use_state = dispatcher.unwrap().use_state;
    (*use_state)(initial_state)
}
