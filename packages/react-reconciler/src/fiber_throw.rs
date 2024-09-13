use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use shared::{derive_from_js_value, type_of};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::js_sys::Function;

use crate::{
    fiber::{FiberNode, FiberRootNode},
    fiber_flags::Flags,
    fiber_lanes::Lane,
    suspense_context::get_suspense_handler,
    work_loop::{ensure_root_is_scheduled, mark_update_lane_from_fiber_to_root},
    JsValueKey,
};

fn attach_ping_listener(
    root: Rc<RefCell<FiberRootNode>>,
    source_fiber: Rc<RefCell<FiberNode>>,
    wakeable: JsValue,
    lane: Lane,
) {
    let mut ping_cache_option: Option<HashMap<JsValueKey, Rc<RefCell<HashSet<Lane>>>>> =
        root.borrow().ping_cache.clone();
    let mut ping_cache: HashMap<JsValueKey, Rc<RefCell<HashSet<Lane>>>>;

    if ping_cache_option.is_none() {
        ping_cache = HashMap::new();
        ping_cache.insert(
            JsValueKey(wakeable.clone()),
            Rc::new(RefCell::new(HashSet::new())),
        );
    } else {
        ping_cache = ping_cache_option.unwrap();
        let _thread_ids = ping_cache.get(&JsValueKey(wakeable.clone()));
        if _thread_ids.is_none() {
            ping_cache.insert(
                JsValueKey(wakeable.clone()),
                Rc::new(RefCell::new(HashSet::new())),
            );
            // thread_ids = &mut ids;
        }
        // } else {
        //     thread_ids = &mut _thread_ids.unwrap();
        // }
    }

    let mut thread_ids = ping_cache.get(&JsValueKey(wakeable.clone())).unwrap();

    if !thread_ids.borrow().contains(&lane) {
        thread_ids.borrow_mut().insert(lane.clone());
        let then_value = derive_from_js_value(&wakeable, "then");
        let then = then_value.dyn_ref::<Function>().unwrap();
        let wakable1 = wakeable.clone();
        let closure = Closure::wrap(Box::new(move || {
            let mut ping_cache = { root.borrow().ping_cache.clone() };
            if ping_cache.is_some() {
                let mut ping_cache = ping_cache.unwrap();
                ping_cache.remove(&JsValueKey(wakable1.clone()));
            }
            root.clone().borrow_mut().mark_root_updated(lane.clone());
            root.clone().borrow_mut().mark_root_pinged(lane.clone());
            mark_update_lane_from_fiber_to_root(source_fiber.clone(), lane.clone());
            ensure_root_is_scheduled(root.clone());
        }) as Box<dyn Fn()>);
        let ping = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        then.call2(&wakeable, &ping, &ping)
            .expect("failed to call then function");
    }
}

pub fn throw_exception(
    root: Rc<RefCell<FiberRootNode>>,
    source_fiber: Rc<RefCell<FiberNode>>,
    value: JsValue,
    lane: Lane,
) {
    if !value.is_null()
        && type_of(&value, "object")
        && derive_from_js_value(&value, "then").is_function()
    {
        let suspense_boundary = get_suspense_handler();
        if suspense_boundary.is_some() {
            let suspense_boundary = suspense_boundary.unwrap();
            suspense_boundary.borrow_mut().flags |= Flags::ShouldCapture;
        }

        attach_ping_listener(root, source_fiber, value, lane)
    }
}
