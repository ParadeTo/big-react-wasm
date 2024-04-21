use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen::prelude::{Closure, wasm_bindgen};
use web_sys::js_sys::{Function, Object, Reflect};

use shared::log;

use crate::fiber::{FiberNode, MemoizedState};
use crate::update_queue::{create_update, create_update_queue, enqueue_update, UpdateQueue};

#[wasm_bindgen]
extern "C" {
    fn updateDispatcher(args: &JsValue);
}

static mut CURRENTLY_RENDERING_FIBER: Option<Rc<RefCell<FiberNode>>> = None;
static mut WORK_IN_PROGRESS_HOOK: Option<Rc<RefCell<Hook>>> = None;

#[derive(Debug, Clone)]
pub struct Hook {
    memoized_state: Option<MemoizedState>,
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

fn update_mount_hooks_to_dispatcher() {
    let object = Object::new();

    let closure = Closure::wrap(Box::new(mount_state) as Box<dyn Fn(&JsValue) -> Vec<JsValue>>);
    let function = closure.as_ref().unchecked_ref::<Function>().clone();
    closure.forget();
    Reflect::set(&object, &"use_state".into(), &function).expect("TODO: panic set use_state");

    updateDispatcher(&object.into());
}

pub fn render_with_hooks(work_in_progress: Rc<RefCell<FiberNode>>) -> Result<JsValue, JsValue> {
    unsafe {
        CURRENTLY_RENDERING_FIBER = Some(work_in_progress.clone());
    }

    let work_in_progress_cloned = work_in_progress.clone();
    {
        work_in_progress_cloned.borrow_mut().memoized_state = None;
        work_in_progress_cloned.borrow_mut().update_queue = None;
    }

    let current = work_in_progress_cloned.borrow().alternate.clone();
    if current.is_some() {
        log!("还未实现update时renderWithHooks");
    } else {
        update_mount_hooks_to_dispatcher();
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
                CURRENTLY_RENDERING_FIBER
                    .as_ref()
                    .unwrap()
                    .clone()
                    .borrow_mut()
                    .memoized_state = Some(MemoizedState::Hook(hook.clone()));
                WORK_IN_PROGRESS_HOOK = Some(hook.clone());
            }
        } else {
            WORK_IN_PROGRESS_HOOK
                .as_ref()
                .unwrap()
                .clone()
                .borrow_mut()
                .next = Some(hook.clone());
            WORK_IN_PROGRESS_HOOK = Some(hook.clone());
        }
        WORK_IN_PROGRESS_HOOK.clone()
    }
}

fn mount_state(initial_state: &JsValue) -> Vec<JsValue> {
    let hook = mount_work_in_progress_nook();
    let memoized_state: JsValue;
    if initial_state.is_function() {
        memoized_state = initial_state
            .dyn_ref::<Function>()
            .unwrap()
            .call0(&JsValue::null())
            .unwrap();
    } else {
        memoized_state = initial_state.clone();
    }
    hook.as_ref().unwrap().clone().borrow_mut().memoized_state =
        Some(MemoizedState::JsValue(Rc::new((memoized_state))));

    unsafe {
        if CURRENTLY_RENDERING_FIBER.is_none() {
            log!("mount_state, currentlyRenderingFiber is empty");
        }
    }
    let queue = create_update_queue();
    hook.as_ref().unwrap().clone().borrow_mut().update_queue = Option::from(queue);

    let closure = Closure::wrap(Box::new(move |action: &JsValue| unsafe {
        dispatch_set_state(CURRENTLY_RENDERING_FIBER.clone(), queue.clone(), action)
    }) as Box<dyn Fn(&JsValue)>);
    let function = closure.as_ref().unchecked_ref::<Function>().clone();
    closure.forget();

    return vec![initial_state.to_owned(), function.into()];
}

fn dispatch_set_state(fiber: Option<Rc<RefCell<FiberNode>>>, update_queue: Rc<RefCell<UpdateQueue>>, action: &JsValue) {
    let update = create_update(Rc::new(*action.clone()));
    enqueue_update(update_queue, update);
    let a = fiber.as_ref().unwrap().borrow();
    // a.s
    // scheduleUpdateOnFiber(fiber);
}
