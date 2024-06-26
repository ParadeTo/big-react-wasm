use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::{wasm_bindgen, Closure};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::js_sys::{Array, Function, Object, Reflect};

use shared::log;

use crate::fiber::{FiberNode, MemoizedState};
use crate::fiber_flags::Flags;
use crate::fiber_lanes::{merge_lanes, request_update_lane, Lane};
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

    Reflect::set(&object, &"use_state".into(), &use_state).expect("TODO: panic set use_state");
    Reflect::set(&object, &"use_effect".into(), &use_effect).expect("TODO: panic set use_effect");

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
        log!("base state is {:?}", base_state);
        let ReturnOfProcessUpdateQueue {
            memoized_state,
            base_state: new_base_state,
            base_queue: new_base_queue,
            skipped_update_lanes,
        } = process_update_queue(base_state, base_queue, unsafe { RENDER_LANE.clone() });
        unsafe {
            let lanes = {
                CURRENTLY_RENDERING_FIBER
                    .clone()
                    .unwrap()
                    .borrow()
                    .lanes
                    .clone()
            };
            CURRENTLY_RENDERING_FIBER
                .clone()
                .unwrap()
                .borrow_mut()
                .lanes = merge_lanes(lanes, skipped_update_lanes)
        };
        hook_cloned.borrow_mut().memoized_state = memoized_state;
        hook_cloned.borrow_mut().base_state = new_base_state;
        hook_cloned.borrow_mut().base_queue = new_base_queue;
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

fn dispatch_set_state(
    fiber: Rc<RefCell<FiberNode>>,
    update_queue: Rc<RefCell<UpdateQueue>>,
    action: &JsValue,
) {
    let lane = request_update_lane();
    let update = create_update(action.clone(), lane.clone());
    enqueue_update(update_queue.clone(), update);
    unsafe {
        schedule_update_on_fiber(fiber.clone(), lane);
    }
}

fn create_fc_update_queue() -> Rc<RefCell<UpdateQueue>> {
    let update_queue = create_update_queue();
    update_queue.borrow_mut().last_effect = None;
    update_queue
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
        let update_queue = create_fc_update_queue();
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
