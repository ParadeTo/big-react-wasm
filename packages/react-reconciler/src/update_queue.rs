use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::{JsCast, JsValue};
use web_sys::js_sys::Function;

use shared::log;

use crate::fiber::{FiberNode, MemoizedState};
use crate::fiber_hooks::Effect;
use crate::fiber_lanes::{is_subset_of_lanes, Lane};

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
        update_rc.borrow_mut().next = pending.borrow().next.clone();
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

struct ReturnOfProcessUpdateQueue {
    memoized_state: Option<MemoizedState>,
    base_state: Option<MemoizedState>,
    base_queue: Option<Rc<RefCell<Update>>>,
    skipped_update_lanes: Lane,
}

pub fn process_update_queue(
    mut base_state: Option<MemoizedState>,
    pending_update: Option<Update>,
    render_lanes: Lane,
) -> Option<ReturnOfProcessUpdateQueue> {
    let result = ReturnOfProcessUpdateQueue {
        memoized_state: base_state,
        base_state,
        base_queue: None,
        skipped_update_lanes: Lane::NoLane,
    };

    if pending_update.is_some() {
        let update = pending_update.clone().unwrap();
        // 更新后的baseState（有跳过情况下与memoizedState不同）
        let new_base_state = base_state;
        // 更新后的baseQueue第一个节点
        let new_base_queue_first: Option<Update> = None;
        // 更新后的baseQueue最后一个节点
        let new_base_queue_last: Option<Update> = None;

        loop {
            let update_lane = update.lane;
            if !is_subset_of_lanes(render_lanes, update_lane) {
                // underpriority
                let clone = create_update(update.action.unwrap(), update.lane);
            }
        }
    }

    None
    // if update_queue.is_some() {
    //     let update_queue = update_queue.clone().unwrap().clone();
    //     let pending = update_queue.borrow().shared.pending.clone();
    //     update_queue.borrow_mut().shared.pending = None;
    //     if pending.is_some() {
    //         let pending_update = pending.clone().unwrap();
    //         let mut update = pending_update.clone();
    //         loop {
    //             let update_lane = update.borrow().lane.clone();
    //             if render_lane == update_lane {
    //                 let action = update.borrow().action.clone();
    //                 match action {
    //                     None => {}
    //                     Some(action) => {
    //                         let f = action.dyn_ref::<Function>();
    //                         base_state = match f {
    //                             None => Some(MemoizedState::MemoizedJsValue(action.clone())),
    //                             Some(f) => {
    //                                 if let MemoizedState::MemoizedJsValue(base_state) =
    //                                     base_state.as_ref().unwrap()
    //                                 {
    //                                     Some(MemoizedState::MemoizedJsValue(
    //                                         f.call1(&JsValue::null(), base_state).unwrap(),
    //                                     ))
    //                                 } else {
    //                                     log!("process_update_queue, base_state is not JsValue");
    //                                     None
    //                                 }
    //                             }
    //                         }
    //                     }
    //                 }
    //             }
    //             let next = update.clone().borrow().next.clone();
    //             if next.is_none() || Rc::ptr_eq(&next.clone().unwrap(), &pending_update.clone()) {
    //                 break;
    //             }
    //             update = next.unwrap();
    //         }
    //     }
    // } else {
    //     log!("{:?} process_update_queue, update_queue is empty", fiber)
    // }

    // base_state
}
