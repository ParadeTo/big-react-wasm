use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::{wasm_bindgen, Closure};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::js_sys::{Array, Function, Object, Reflect};

use shared::{is_dev, log};

use crate::begin_work::mark_wip_received_update;
use crate::fiber::{FiberNode, MemoizedState};
use crate::fiber_flags::Flags;
use crate::fiber_lanes::{merge_lanes, remove_lanes, request_update_lane, Lane};
use crate::update_queue::{
    create_update, create_update_queue, enqueue_update, process_update_queue,
    ReturnOfProcessUpdateQueue, Update, UpdateQueue,
};
use crate::work_loop::schedule_update_on_fiber;

#[wasm_bindgen]
extern "C" {
    fn updateDispatcher(args: &JsValue);
}

static mut CURRENTLY_RENDERING_FIBER: Option<Rc<RefCell<FiberNode>>> = None;
static mut WORK_IN_PROGRESS_HOOK: Option<Rc<RefCell<Hook>>> = None;
static mut CURRENT_HOOK: Option<Rc<RefCell<Hook>>> = None;
static mut RENDER_LANE: Lane = Lane::NoLane;

#[derive(Debug, Clone)]
pub struct Effect {
    pub tag: Flags,
    pub create: Function,
    pub destroy: JsValue,
    pub deps: JsValue,
    pub next: Option<Rc<RefCell<Effect>>>,
}

impl Effect {
    fn new(
        tag: Flags,
        create: Function,
        destroy: JsValue,
        deps: JsValue,
        next: Option<Rc<RefCell<Effect>>>,
    ) -> Self {
        Self {
            tag,
            create,
            deps,
            destroy,
            next,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Hook {
    memoized_state: Option<MemoizedState>,
    // 对于state，保存update相关数据
    update_queue: Option<Rc<RefCell<UpdateQueue>>>,
    // 对于state，保存开始更新前就存在的updateList（上次更新遗留）
    base_queue: Option<Rc<RefCell<Update>>>,
    // 对于state，基于baseState开始计算更新，与memoizedState的区别在于上次更新是否存在跳过
    base_state: Option<MemoizedState>,
    next: Option<Rc<RefCell<Hook>>>,
}

impl Hook {
    fn new(
        memoized_state: Option<MemoizedState>,
        update_queue: Option<Rc<RefCell<UpdateQueue>>>,
        base_queue: Option<Rc<RefCell<Update>>>,
        base_state: Option<MemoizedState>,
        next: Option<Rc<RefCell<Hook>>>,
    ) -> Self {
        Hook {
            memoized_state,
            update_queue,
            base_queue,
            base_state,
            next,
        }
    }
}

pub fn bailout_hook(wip: Rc<RefCell<FiberNode>>, render_lane: Lane) {
    let current = { wip.borrow().alternate.clone().unwrap() };
    let update_queue = { current.borrow().update_queue.clone() };
    let lanes = { current.borrow().lanes.clone() };
    wip.borrow_mut().update_queue = update_queue;
    wip.borrow_mut().flags -= Flags::PassiveEffect;
    current.borrow_mut().lanes = remove_lanes(lanes, render_lane);
}

fn update_hooks_to_dispatcher(is_update: bool) {
    let object = Object::new();

    // use_state
    let use_state_closure =
        Closure::wrap(Box::new(if is_update { update_state } else { mount_state })
            as Box<dyn Fn(&JsValue) -> Result<Vec<JsValue>, JsValue>>);
    let use_state = use_state_closure
        .as_ref()
        .unchecked_ref::<Function>()
        .clone();
    use_state_closure.forget();

    // use_effect
    let use_effect_closure = Closure::wrap(Box::new(if is_update {
        update_effect
    } else {
        mount_effect
    }) as Box<dyn Fn(Function, JsValue)>);
    let use_effect = use_effect_closure
        .as_ref()
        .unchecked_ref::<Function>()
        .clone();
    use_effect_closure.forget();

    // use_ref
    let use_ref_closure = Closure::wrap(Box::new(if is_update { update_ref } else { mount_ref })
        as Box<dyn Fn(&JsValue) -> JsValue>);
    let use_ref = use_ref_closure.as_ref().unchecked_ref::<Function>().clone();
    use_ref_closure.forget();

    // use_memo
    let use_memo_closure =
        Closure::wrap(Box::new(if is_update { update_memo } else { mount_memo })
            as Box<dyn Fn(Function, JsValue) -> Result<JsValue, JsValue>>);
    let use_memo = use_memo_closure
        .as_ref()
        .unchecked_ref::<Function>()
        .clone();
    use_memo_closure.forget();

    // use_callback
    let use_callback_clusure = Closure::wrap(Box::new(if is_update {
        update_callback
    } else {
        mount_callback
    }) as Box<dyn Fn(Function, JsValue) -> JsValue>);
    let use_callback = use_callback_clusure
        .as_ref()
        .unchecked_ref::<Function>()
        .clone();
    use_callback_clusure.forget();

    Reflect::set(&object, &"use_state".into(), &use_state).expect("TODO: panic set use_state");
    Reflect::set(&object, &"use_effect".into(), &use_effect).expect("TODO: panic set use_effect");
    Reflect::set(&object, &"use_ref".into(), &use_ref).expect("TODO: panic set use_ref");
    Reflect::set(&object, &"use_memo".into(), &use_memo).expect("TODO: panic set use_memo");
    Reflect::set(&object, &"use_callback".into(), &use_callback)
        .expect("TODO: panic set use_callback");

    updateDispatcher(&object.into());
}

pub fn render_with_hooks(
    work_in_progress: Rc<RefCell<FiberNode>>,
    lane: Lane,
) -> Result<JsValue, JsValue> {
    unsafe {
        CURRENTLY_RENDERING_FIBER = Some(work_in_progress.clone());
        RENDER_LANE = lane;
    }

    let work_in_progress_cloned = work_in_progress.clone();
    {
        work_in_progress_cloned.borrow_mut().memoized_state = None;
        work_in_progress_cloned.borrow_mut().update_queue = None;
    }

    let current = work_in_progress_cloned.borrow().alternate.clone();
    if current.is_some() {
        update_hooks_to_dispatcher(true);
    } else {
        update_hooks_to_dispatcher(false);
    }

    let _type;
    let props;
    {
        let work_in_progress_borrow = work_in_progress_cloned.borrow();
        _type = work_in_progress_borrow._type.clone();
        props = work_in_progress_borrow.pending_props.clone();
    }

    let component = JsValue::dyn_ref::<Function>(&_type).unwrap();
    let children = component.call1(&JsValue::null(), &props);

    unsafe {
        CURRENTLY_RENDERING_FIBER = None;
        WORK_IN_PROGRESS_HOOK = None;
        CURRENT_HOOK = None;
        RENDER_LANE = Lane::NoLane;
    }

    children
}

fn mount_work_in_progress_hook() -> Option<Rc<RefCell<Hook>>> {
    let hook = Rc::new(RefCell::new(Hook::new(None, None, None, None, None)));
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

fn update_work_in_progress_hook() -> Option<Rc<RefCell<Hook>>> {
    // case1: Update triggered by interaction, the wip_hook is none, use hook in current_hook to clone wip_hook
    // case2: Update triggered in render process, the wip_hook exists
    let mut next_current_hook: Option<Rc<RefCell<Hook>>> = None;
    let mut next_work_in_progress_hook: Option<Rc<RefCell<Hook>>> = None;

    unsafe {
        next_current_hook = match &CURRENT_HOOK {
            None => {
                let current = CURRENTLY_RENDERING_FIBER
                    .as_ref()
                    .unwrap()
                    .clone()
                    .borrow()
                    .alternate
                    .clone();

                match current {
                    None => None,
                    Some(current) => match current.clone().borrow().memoized_state.clone() {
                        Some(MemoizedState::Hook(memoized_state)) => Some(memoized_state.clone()),
                        _ => None,
                    },
                }
            }
            Some(current_hook) => current_hook.clone().borrow().next.clone(),
        };

        next_work_in_progress_hook = match &WORK_IN_PROGRESS_HOOK {
            None => match CURRENTLY_RENDERING_FIBER.clone() {
                Some(current) => match current.clone().borrow().memoized_state.clone() {
                    Some(MemoizedState::Hook(memoized_state)) => Some(memoized_state.clone()),
                    _ => None,
                },
                _ => None,
            },
            Some(work_in_progress_hook) => work_in_progress_hook.clone().borrow().next.clone(),
        };

        if next_work_in_progress_hook.is_some() {
            WORK_IN_PROGRESS_HOOK = next_work_in_progress_hook.clone();
            CURRENT_HOOK = next_current_hook.clone();
        } else {
            if next_current_hook.is_none() {
                log!(
                    "{:?} hooks is more than last",
                    CURRENTLY_RENDERING_FIBER
                        .as_ref()
                        .unwrap()
                        .clone()
                        .borrow()
                        ._type
                );
            }

            CURRENT_HOOK = next_current_hook;
            let cloned = CURRENT_HOOK.clone().unwrap().clone();
            let current_hook = cloned.borrow();
            let new_hook = Rc::new(RefCell::new(Hook::new(
                current_hook.memoized_state.clone(),
                current_hook.update_queue.clone(),
                current_hook.base_queue.clone(),
                current_hook.base_state.clone(),
                None,
            )));

            if WORK_IN_PROGRESS_HOOK.is_none() {
                WORK_IN_PROGRESS_HOOK = Some(new_hook.clone());
                CURRENTLY_RENDERING_FIBER
                    .as_ref()
                    .unwrap()
                    .clone()
                    .borrow_mut()
                    .memoized_state = Some(MemoizedState::Hook(new_hook.clone()));
            } else {
                let wip_hook = WORK_IN_PROGRESS_HOOK.clone().unwrap();
                wip_hook.borrow_mut().next = Some(new_hook.clone());
                WORK_IN_PROGRESS_HOOK = Some(new_hook.clone());
            }
        }
        WORK_IN_PROGRESS_HOOK.clone()
    }
}

fn mount_state(initial_state: &JsValue) -> Result<Vec<JsValue>, JsValue> {
    let hook = mount_work_in_progress_hook();
    let memoized_state: JsValue;

    if initial_state.is_function() {
        memoized_state = initial_state
            .dyn_ref::<Function>()
            .unwrap()
            .call0(&JsValue::null())?;
    } else {
        memoized_state = initial_state.clone();
    }
    hook.as_ref().unwrap().clone().borrow_mut().memoized_state =
        Some(MemoizedState::MemoizedJsValue(memoized_state.clone()));
    hook.as_ref().unwrap().clone().borrow_mut().base_state =
        Some(MemoizedState::MemoizedJsValue(memoized_state.clone()));

    unsafe {
        if CURRENTLY_RENDERING_FIBER.is_none() {
            log!("mount_state, currentlyRenderingFiber is empty");
        }
    }
    let queue = create_update_queue();
    hook.as_ref().unwrap().clone().borrow_mut().update_queue = Some(queue.clone());
    let q_rc = Rc::new(queue.clone());
    let q_rc_cloned = q_rc.clone();
    let fiber = unsafe { CURRENTLY_RENDERING_FIBER.clone().unwrap() };
    let closure = Closure::wrap(Box::new(move |action: &JsValue| {
        dispatch_set_state(fiber.clone(), (*q_rc_cloned).clone(), action)
    }) as Box<dyn Fn(&JsValue)>);
    let function: Function = closure.as_ref().unchecked_ref::<Function>().clone();
    closure.forget();

    queue.clone().borrow_mut().dispatch = Some(function.clone());
    queue.clone().borrow_mut().last_rendered_state = Some(memoized_state.clone());
    Ok(vec![memoized_state, function.into()])
}

fn update_state(_: &JsValue) -> Result<Vec<JsValue>, JsValue> {
    let hook = update_work_in_progress_hook();

    if hook.is_none() {
        panic!("update_state hook is none")
    }

    let hook_cloned = hook.clone().unwrap().clone();
    let queue = hook_cloned.borrow().update_queue.clone();
    let base_state = hook_cloned.borrow().base_state.clone();

    let mut base_queue = unsafe { CURRENT_HOOK.clone().unwrap().borrow().base_queue.clone() };
    let pending = queue.clone().unwrap().borrow().shared.pending.clone();

    if pending.is_some() {
        if base_queue.is_some() {
            let base_queue = base_queue.clone().unwrap();
            let pending = pending.clone().unwrap();
            // baseQueue = b2 -> b0 -> b1 -> b2
            // pending = p2 -> p0 -> p1 -> p2

            // b0
            let base_first = base_queue.borrow().next.clone();
            // p0
            let pending_first = pending.borrow().next.clone();
            // baseQueue = b2 -> p0 -> p1 -> p2
            base_queue.borrow_mut().next = pending_first;
            // pending = p2 -> b0 -> b1 -> b2
            pending.borrow_mut().next = base_first;
            // 拼接完成后：先pending，再baseQueue
            // baseQueue = b2 -> p0 -> p1 -> p2 -> b0 -> b1 -> b2
        }
        // pending保存在current中，因为commit阶段不完成，current不会变为wip
        // 所以可以保证多次render阶段（只要不进入commit）都能从current恢复pending
        unsafe { CURRENT_HOOK.clone().unwrap().borrow_mut().base_queue = pending.clone() };
        base_queue = pending;
        queue.clone().unwrap().borrow_mut().shared.pending = None;
    }

    if base_queue.is_some() {
        let pre_state = hook.as_ref().unwrap().borrow().memoized_state.clone();

        let ReturnOfProcessUpdateQueue {
            memoized_state,
            base_state: new_base_state,
            base_queue: new_base_queue,
        } = process_update_queue(
            base_state,
            base_queue,
            unsafe { RENDER_LANE.clone() },
            Some(|update: Rc<RefCell<Update>>| {
                let skipped_lane = update.borrow().lane.clone();
                let fiber = unsafe { CURRENTLY_RENDERING_FIBER.clone().unwrap().clone() };
                let lanes = { fiber.borrow().lanes.clone() };
                fiber.borrow_mut().lanes = merge_lanes(lanes, skipped_lane);
            }),
        );

        if !(memoized_state.is_none() && pre_state.is_none()) {
            let memoized_state = memoized_state.clone().unwrap();
            let pre_state = pre_state.unwrap();
            if let MemoizedState::MemoizedJsValue(ms_value) = memoized_state {
                if let MemoizedState::MemoizedJsValue(ps_value) = pre_state {
                    if !Object::is(&ms_value, &ps_value) {
                        mark_wip_received_update();
                    }
                }
            }
        }

        hook_cloned.borrow_mut().memoized_state = memoized_state.clone();
        hook_cloned.borrow_mut().base_state = new_base_state;
        hook_cloned.borrow_mut().base_queue = new_base_queue;

        queue.clone().unwrap().borrow_mut().last_rendered_state = Some(match memoized_state {
            Some(m) => match m {
                MemoizedState::MemoizedJsValue(js_value) => js_value,
                _ => todo!(),
            },
            None => todo!(),
        });
    }

    Ok(vec![
        hook.clone()
            .unwrap()
            .clone()
            .borrow()
            .memoized_state
            .clone()
            .unwrap()
            .js_value()
            .unwrap()
            .clone(),
        queue.clone().unwrap().borrow().dispatch.clone().into(),
    ])
}

pub fn basic_state_reducer(state: &JsValue, action: &JsValue) -> Result<JsValue, JsValue> {
    if action.is_function() {
        let function = action.dyn_ref::<Function>().unwrap();
        return function.call1(&JsValue::null(), state);
    }
    Ok(action.into())
}

fn dispatch_set_state(
    fiber: Rc<RefCell<FiberNode>>,
    update_queue: Rc<RefCell<UpdateQueue>>,
    action: &JsValue,
) {
    let lane = request_update_lane();
    let mut update = create_update(action.clone(), lane.clone());
    let current = { fiber.borrow().alternate.clone() };
    log!(
        "dispatch_set_state {:?} {:?}",
        fiber.borrow().lanes.clone(),
        if current.is_none() {
            Lane::NoLane
        } else {
            current.clone().unwrap().borrow().lanes.clone()
        }
    );
    if fiber.borrow().lanes == Lane::NoLane
        && (current.is_none() || current.unwrap().borrow().lanes == Lane::NoLane)
    {
        log!("sdadgasd");
        let current_state = update_queue.borrow().last_rendered_state.clone();
        if current_state.is_none() {
            panic!("current state is none")
        }
        let current_state = current_state.unwrap();
        let eager_state = basic_state_reducer(&current_state, &action);
        // if not ok, the update will be handled in render phase, means the error will be handled in render phase
        if eager_state.is_ok() {
            let eager_state = eager_state.unwrap();
            update.has_eager_state = true;
            update.eager_state = Some(eager_state.clone());
            if Object::is(&current_state, &eager_state) {
                enqueue_update(update_queue.clone(), update, fiber.clone(), Lane::NoLane);
                if is_dev() {
                    log!("Hit eager state")
                }
                return;
            }
        }
    }

    enqueue_update(update_queue.clone(), update, fiber.clone(), lane.clone());
    schedule_update_on_fiber(fiber.clone(), lane);
}

fn push_effect(
    hook_flags: Flags,
    create: Function,
    destroy: JsValue,
    deps: JsValue,
) -> Rc<RefCell<Effect>> {
    let mut effect = Rc::new(RefCell::new(Effect::new(
        hook_flags, create, destroy, deps, None,
    )));
    let fiber = unsafe { CURRENTLY_RENDERING_FIBER.clone().unwrap() };
    let update_queue = { fiber.borrow().update_queue.clone() };
    if update_queue.is_none() {
        let update_queue = create_update_queue();
        fiber.borrow_mut().update_queue = Some(update_queue.clone());
        effect.borrow_mut().next = Option::from(effect.clone());
        update_queue.borrow_mut().last_effect = Option::from(effect.clone());
    } else {
        let update_queue = update_queue.unwrap();
        let last_effect = update_queue.borrow().last_effect.clone();
        if last_effect.is_none() {
            effect.borrow_mut().next = Some(effect.clone());
            update_queue.borrow_mut().last_effect = Some(effect.clone());
        } else {
            let last_effect = last_effect.unwrap();
            let first_effect = last_effect.borrow().next.clone();
            last_effect.borrow_mut().next = Some(effect.clone());
            effect.borrow_mut().next = first_effect;
            update_queue.borrow_mut().last_effect = Some(effect.clone());
        }
    }
    return effect;
}

fn mount_effect(create: Function, deps: JsValue) {
    let hook = mount_work_in_progress_hook();
    let next_deps = if deps.is_undefined() {
        JsValue::null()
    } else {
        deps
    };

    // 注意区分PassiveEffect与Passive，PassiveEffect是针对fiber.flags
    // Passive是effect类型，代表useEffect。类似的，Layout代表useLayoutEffect
    let currently_rendering_fiber = unsafe { CURRENTLY_RENDERING_FIBER.clone().unwrap() };
    currently_rendering_fiber.borrow_mut().flags |= Flags::PassiveEffect;
    hook.as_ref().unwrap().clone().borrow_mut().memoized_state =
        Some(MemoizedState::Effect(push_effect(
            Flags::Passive | Flags::HookHasEffect,
            create,
            JsValue::null(),
            next_deps,
        )));
}

fn update_effect(create: Function, deps: JsValue) {
    let hook = update_work_in_progress_hook();
    let next_deps = if deps.is_undefined() {
        JsValue::null()
    } else {
        deps
    };

    let mut destroy = JsValue::null();
    unsafe {
        if CURRENT_HOOK.is_some() {
            let current_hook = CURRENT_HOOK.clone().unwrap();
            let prev_effect = current_hook.borrow().memoized_state.clone();
            if let MemoizedState::Effect(prev_effect) = prev_effect.unwrap() {
                destroy = prev_effect.borrow().destroy.clone();
                if !next_deps.is_null() {
                    let prev_deps = prev_effect.borrow().deps.clone();

                    if are_hook_inputs_equal(&prev_deps, &next_deps) {
                        hook.as_ref().unwrap().borrow_mut().memoized_state =
                            Some(MemoizedState::Effect(push_effect(
                                Flags::Passive,
                                create,
                                destroy,
                                next_deps,
                            )));
                        return;
                    }
                }
            } else {
                panic!("memoized_state is not Effect")
            }
        }

        CURRENTLY_RENDERING_FIBER
            .as_ref()
            .unwrap()
            .borrow_mut()
            .flags |= Flags::PassiveEffect;
        log!("CURRENTLY_RENDERING_FIBER.as_ref().unwrap().borrow_mut()");

        hook.as_ref().unwrap().clone().borrow_mut().memoized_state =
            Some(MemoizedState::Effect(push_effect(
                Flags::Passive | Flags::HookHasEffect,
                create,
                destroy.clone(),
                next_deps,
            )));
    }
}

fn are_hook_inputs_equal(next_deps: &JsValue, pre_deps: &JsValue) -> bool {
    if next_deps.is_null() || pre_deps.is_null() {
        return false;
    }

    let next_deps = next_deps.dyn_ref::<Array>().unwrap();
    let pre_deps = pre_deps.dyn_ref::<Array>().unwrap();

    let len = next_deps.length();

    for i in 0..len {
        if Object::is(&pre_deps.get(i), &next_deps.get(i)) {
            continue;
        }
        return false;
    }
    return true;
}

fn mount_ref(initial_value: &JsValue) -> JsValue {
    let hook = mount_work_in_progress_hook();
    let ref_obj: Object = Object::new();
    Reflect::set(&ref_obj, &"current".into(), initial_value);
    hook.as_ref().unwrap().borrow_mut().memoized_state =
        Some(MemoizedState::MemoizedJsValue(ref_obj.clone().into()));
    ref_obj.into()
}

fn update_ref(initial_value: &JsValue) -> JsValue {
    let hook = update_work_in_progress_hook();
    match hook.unwrap().borrow_mut().memoized_state.clone() {
        Some(MemoizedState::MemoizedJsValue(value)) => value,
        _ => panic!("ref is none"),
    }
}

fn mount_memo(create: Function, deps: JsValue) -> Result<JsValue, JsValue> {
    let hook = mount_work_in_progress_hook();
    let next_deps = if deps.is_undefined() {
        JsValue::null()
    } else {
        deps
    };
    let next_value = create.call0(&JsValue::null())?;
    let array = Array::new();
    array.push(&next_value);
    array.push(&next_deps);
    hook.as_ref().unwrap().clone().borrow_mut().memoized_state =
        Some(MemoizedState::MemoizedJsValue(array.into()));
    Ok(next_value)
}

fn update_memo(create: Function, deps: JsValue) -> Result<JsValue, JsValue> {
    let hook = update_work_in_progress_hook();
    let next_deps = if deps.is_undefined() {
        JsValue::null()
    } else {
        deps
    };

    if let MemoizedState::MemoizedJsValue(prev_state) = hook
        .clone()
        .unwrap()
        .borrow()
        .memoized_state
        .as_ref()
        .unwrap()
    {
        if !next_deps.is_null() {
            let arr = prev_state.dyn_ref::<Array>().unwrap();
            let prev_deps = arr.get(1);
            if are_hook_inputs_equal(&next_deps, &prev_deps) {
                return Ok(arr.get(0));
            }
        }
        let next_value = create.call0(&JsValue::null())?;
        let array = Array::new();
        array.push(&next_value);
        array.push(&next_deps);
        hook.as_ref().unwrap().clone().borrow_mut().memoized_state =
            Some(MemoizedState::MemoizedJsValue(array.into()));
        return Ok(next_value);
    }
    panic!("update_memo, memoized_state is not JsValue");
}

fn mount_callback(callback: Function, deps: JsValue) -> JsValue {
    let hook = mount_work_in_progress_hook();
    let next_deps = if deps.is_undefined() {
        JsValue::null()
    } else {
        deps
    };
    let array = Array::new();
    array.push(&callback);
    array.push(&next_deps);
    hook.as_ref().unwrap().clone().borrow_mut().memoized_state =
        Some(MemoizedState::MemoizedJsValue(array.into()));
    callback.into()
}

fn update_callback(callback: Function, deps: JsValue) -> JsValue {
    let hook = update_work_in_progress_hook();
    let next_deps = if deps.is_undefined() {
        JsValue::null()
    } else {
        deps
    };

    if let MemoizedState::MemoizedJsValue(prev_state) = hook
        .clone()
        .unwrap()
        .borrow()
        .memoized_state
        .as_ref()
        .unwrap()
    {
        if !next_deps.is_null() {
            let arr = prev_state.dyn_ref::<Array>().unwrap();
            let prev_deps = arr.get(1);
            if are_hook_inputs_equal(&next_deps, &prev_deps) {
                return arr.get(0);
            }
        }
        let array = Array::new();
        array.push(&callback);
        array.push(&next_deps);
        hook.as_ref().unwrap().clone().borrow_mut().memoized_state =
            Some(MemoizedState::MemoizedJsValue(array.into()));
        return callback.into();
    }
    panic!("update_callback, memoized_state is not JsValue");
}
