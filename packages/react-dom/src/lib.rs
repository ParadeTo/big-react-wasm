use std::rc::Rc;

use js_sys::{Array, Function};
use wasm_bindgen::prelude::*;
use web_sys::Node;

use react_reconciler::Reconciler;
use scheduler::{
    Priority, unstable_cancel_callback,
    unstable_schedule_callback as origin_unstable_schedule_callback, unstable_should_yield_to_host,
};

use crate::host_config::ReactDomHostConfig;
use crate::renderer::Renderer;
use crate::utils::set_panic_hook;

mod host_config;
mod renderer;
mod synthetic_event;
mod utils;

#[wasm_bindgen(js_name = createRoot)]
pub fn create_root(container: &JsValue) -> Renderer {
    set_panic_hook();
    let reconciler = Reconciler::new(Rc::new(ReactDomHostConfig));
    let node = match container.clone().dyn_into::<Node>() {
        Ok(node) => node,
        Err(_) => {
            panic!("container should be Node")
        }
    };
    let root = reconciler.create_container(Rc::new(node));
    let renderer = Renderer::new(root, reconciler, container);
    renderer
}

#[wasm_bindgen(js_name = scheduleCallback, variadic)]
pub fn unstable_schedule_callback(
    priority_level: Priority,
    callback: Function,
    delay: &JsValue,
) -> u32 {
    let delay = delay.dyn_ref::<Array>().unwrap();
    let d = delay.get(0).as_f64().unwrap_or_else(|| 0.0);
    origin_unstable_schedule_callback(priority_level, callback, d)
}

#[wasm_bindgen(js_name = cancelCallback)]
pub fn cancel_callback(id: u32) {
    unstable_cancel_callback(id)
}

#[wasm_bindgen(js_name = shouldYieldToHost)]
pub fn should_yield_to_host() -> bool {
    unstable_should_yield_to_host()
}
