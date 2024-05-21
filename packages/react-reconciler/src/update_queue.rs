use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::{JsCast, JsValue};
use web_sys::js_sys::Function;

use shared::log;

use crate::fiber::{FiberNode, MemoizedState};
use crate::fiber_hooks::Effect;
use crate::fiber_lanes::Lane;

#[derive(Clone, Debug)]
pub struct UpdateAction;

#[derive(Clone, Debug)]
pub struct Update {
    pub action: Option<JsValue>,
    pub lane: Lane,
    pub next: Option<Rc<RefCell<Update>>>,
}

#[derive(Clone, Debug)]
pub struct UpdateType {
    pub pending: Option<Rc<RefCell<Update>>>,
}

#[derive(Clone, Debug)]
pub struct UpdateQueue {
    pub shared: UpdateType,
    pub dispatch: Option<Function>,
    pub last_effect: Option<Rc<RefCell<Effect>>>,
}

pub fn create_update(action: JsValue, lane: Lane) -> Update {
    Update {
        action: Some(action),
        lane,
        next: None,
    }
}

pub fn enqueue_update(update_queue: Rc<RefCell<UpdateQueue>>, mut update: Update) {
    let pending = update_queue.borrow().shared.pending.clone();
    let update_rc = Rc::new(RefCell::new(update));
    let update_option = Option::from(update_rc.clone());
    if pending.is_none() {
        update_rc.borrow_mut().next = update_option.clone();
    } else {
        let pending = pending.clone().unwrap();
        update_rc.borrow_mut().next = { pending.borrow().next.clone() };
        pending.borrow_mut().next = update_option.clone();
    }
    update_queue.borrow_mut().shared.pending = update_option.clone();
}

pub fn create_update_queue() -> Rc<RefCell<UpdateQueue>> {
    Rc::new(RefCell::new(UpdateQueue {
        shared: UpdateType { pending: None },
        dispatch: None,
        last_effect: None,
    }))
}

pub fn process_update_queue(
    mut base_state: Option<MemoizedState>,
    update_queue: Option<Rc<RefCell<UpdateQueue>>>,
    fiber: Rc<RefCell<FiberNode>>,
    render_lane: Lane,
) -> Option<MemoizedState> {
    if update_queue.is_some() {
        let update_queue = update_queue.clone().unwrap().clone();
        let pending = update_queue.borrow().shared.pending.clone();
        update_queue.borrow_mut().shared.pending = None;
        if pending.is_some() {
            let pending_update = pending.clone().unwrap();
            let mut update = pending_update.clone();
            loop {
                let update_lane = update.borrow().lane.clone();
                if render_lane == update_lane {
                    let action = update.borrow().action.clone();
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
                let next = update.clone().borrow().next.clone();
                if next.is_none() || Rc::ptr_eq(&next.clone().unwrap(), &pending_update.clone()) {
                    break;
                }
                update = next.unwrap();
            }
        }
    } else {
        log!("{:?} process_update_queue, update_queue is empty", fiber)
    }

    base_state
}
