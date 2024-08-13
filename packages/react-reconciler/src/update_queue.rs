use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::{JsCast, JsValue};
use web_sys::js_sys::Function;

use shared::log;

use crate::fiber::{FiberNode, MemoizedState};
use crate::fiber_hooks::{basic_state_reducer, Effect};
use crate::fiber_lanes::{is_subset_of_lanes, merge_lanes, Lane};

#[derive(Clone, Debug)]
pub struct UpdateAction;

#[derive(Clone, Debug)]
pub struct Update {
    pub action: Option<JsValue>,
    pub lane: Lane,
    pub next: Option<Rc<RefCell<Update>>>,
    pub has_eager_state: bool,
    pub eager_state: Option<JsValue>,
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
    pub last_rendered_state: Option<JsValue>,
}

pub fn create_update(action: JsValue, lane: Lane) -> Update {
    Update {
        action: Some(action),
        lane,
        next: None,
        has_eager_state: false,
        eager_state: None,
    }
}

pub fn enqueue_update(
    update_queue: Rc<RefCell<UpdateQueue>>,
    mut update: Update,
    fiber: Rc<RefCell<FiberNode>>,
    lane: Lane,
) {
    let pending = update_queue.borrow().shared.pending.clone();
    let update_rc = Rc::new(RefCell::new(update));
    let update_option = Option::from(update_rc.clone());
    if pending.is_none() {
        update_rc.borrow_mut().next = update_option.clone();
    } else {
        let pending = pending.clone().unwrap();
        update_rc.borrow_mut().next = pending.borrow().next.clone();
        pending.borrow_mut().next = update_option.clone();
    }
    update_queue.borrow_mut().shared.pending = update_option.clone();

    let fiber_lane = { fiber.borrow().lanes.clone() };
    fiber.borrow_mut().lanes = merge_lanes(fiber_lane, lane.clone());
    let alternate = fiber.borrow().alternate.clone();
    if alternate.is_some() {
        let alternate = alternate.unwrap();
        let lanes = { alternate.borrow().lanes.clone() };
        alternate.borrow_mut().lanes = merge_lanes(lanes, lane);
    }
}

pub fn create_update_queue() -> Rc<RefCell<UpdateQueue>> {
    Rc::new(RefCell::new(UpdateQueue {
        shared: UpdateType { pending: None },
        dispatch: None,
        last_effect: None,
        last_rendered_state: None,
    }))
}

pub struct ReturnOfProcessUpdateQueue {
    pub memoized_state: Option<MemoizedState>,
    pub base_state: Option<MemoizedState>,
    pub base_queue: Option<Rc<RefCell<Update>>>,
}

pub fn process_update_queue(
    base_state: Option<MemoizedState>,
    pending_update: Option<Rc<RefCell<Update>>>,
    render_lanes: Lane,
    on_skip_update: Option<fn(update: Rc<RefCell<Update>>) -> ()>,
) -> ReturnOfProcessUpdateQueue {
    let mut result = ReturnOfProcessUpdateQueue {
        memoized_state: base_state.clone(),
        base_state: base_state.clone(),
        base_queue: None,
    };

    if pending_update.is_some() {
        let update_option = pending_update.clone().unwrap();
        let first = update_option.borrow().next.clone();
        let mut pending = update_option.borrow().next.clone();

        // 更新后的baseState（有跳过情况下与memoizedState不同）
        let mut new_base_state: Option<MemoizedState> = base_state.clone();
        // 更新后的baseQueue第一个节点
        let mut new_base_queue_first: Option<Rc<RefCell<Update>>> = None;
        // 更新后的baseQueue最后一个节点
        let mut new_base_queue_last: Option<Rc<RefCell<Update>>> = None;
        let mut new_state = base_state.clone();

        loop {
            let mut update = pending.clone().unwrap();
            let update_lane = update.borrow().lane.clone();
            if !is_subset_of_lanes(render_lanes.clone(), update_lane.clone()) {
                // underpriority
                let clone = Rc::new(RefCell::new(create_update(
                    update.borrow().action.clone().unwrap(),
                    update_lane.clone(),
                )));

                if on_skip_update.is_some() {
                    let function = on_skip_update.unwrap();
                    function(clone.clone());
                }

                if new_base_queue_last.is_none() {
                    new_base_queue_first = Some(clone.clone());
                    new_base_queue_last = Some(clone.clone());
                    new_base_state = result.memoized_state.clone();
                } else {
                    new_base_queue_last.clone().unwrap().borrow_mut().next = Some(clone.clone());
                }
            } else {
                if new_base_queue_last.is_some() {
                    let clone = Rc::new(RefCell::new(create_update(
                        update.borrow().action.clone().unwrap(),
                        update_lane.clone(),
                    )));
                    new_base_queue_last.clone().unwrap().borrow_mut().next = Some(clone.clone());
                    new_base_queue_last = Some(clone.clone())
                }

                if update.borrow().has_eager_state {
                    new_state = Some(MemoizedState::MemoizedJsValue(
                        update.borrow().eager_state.clone().unwrap(),
                    ));
                } else {
                    // let b = match base_state.clone() {
                    //     Some(s) => match s {
                    //         MemoizedState::MemoizedJsValue(js_value) => Some(js_value),
                    //         _ => None,
                    //     },
                    //     None => None,
                    // };
                    // new_state = if b.is_none() {
                    //     None
                    // } else {
                    //     Some(MemoizedState::MemoizedJsValue(
                    //         basic_state_reducer(
                    //             b.as_ref().unwrap(),
                    //             &update.borrow().action.clone().unwrap(),
                    //         )
                    //         .unwrap(),
                    //     ))
                    // };
                    new_state = match update.borrow().action.clone() {
                        None => None,
                        Some(action) => {
                            let f = action.dyn_ref::<Function>();
                            match f {
                                None => Some(MemoizedState::MemoizedJsValue(action.clone())),
                                Some(f) => match result.memoized_state.as_ref() {
                                    Some(memoized_state) => {
                                        if let MemoizedState::MemoizedJsValue(base_state) =
                                            memoized_state
                                        {
                                            Some(MemoizedState::MemoizedJsValue(
                                                f.call1(&JsValue::null(), base_state).unwrap(),
                                            ))
                                        } else {
                                            log!("process_update_queue, base_state is not JsValue");
                                            None
                                        }
                                    }
                                    None => Some(MemoizedState::MemoizedJsValue(
                                        f.call1(&JsValue::null(), &JsValue::undefined()).unwrap(),
                                    )),
                                },
                            }
                        }
                    };
                }

                // result.memoized_state = match update.borrow().action.clone() {
                //     None => None,
                //     Some(action) => {
                //         let f = action.dyn_ref::<Function>();
                //         match f {
                //             None => Some(MemoizedState::MemoizedJsValue(action.clone())),
                //             Some(f) => match result.memoized_state.as_ref() {
                //                 Some(memoized_state) => {
                //                     if let MemoizedState::MemoizedJsValue(base_state) =
                //                         memoized_state
                //                     {
                //                         Some(MemoizedState::MemoizedJsValue(
                //                             f.call1(&JsValue::null(), base_state).unwrap(),
                //                         ))
                //                     } else {
                //                         log!("process_update_queue, base_state is not JsValue");
                //                         None
                //                     }
                //                 }
                //                 None => Some(MemoizedState::MemoizedJsValue(
                //                     f.call1(&JsValue::null(), &JsValue::undefined()).unwrap(),
                //                 )),
                //             },
                //         }
                //     }
                // };
            }
            pending = update.clone().borrow().next.clone();
            if Rc::ptr_eq(&pending.clone().unwrap(), &first.clone().unwrap()) {
                break;
            }
        }

        if new_base_queue_last.is_none() {
            new_base_state = new_state.clone();
        } else {
            new_base_queue_last.clone().unwrap().borrow_mut().next = new_base_queue_last.clone();
        }

        result.memoized_state = new_state;
        result.base_state = new_base_state;
        result.base_queue = new_base_queue_last.clone();
    }

    result
}
