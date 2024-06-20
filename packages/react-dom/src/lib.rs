use gloo::console::log;
use js_sys::{Array, Function, Object, Reflect};
use react_reconciler::fiber::FiberRootNode;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::Node;

use react_reconciler::Reconciler;
use scheduler::{
    unstable_cancel_callback, unstable_schedule_callback as origin_unstable_schedule_callback,
    unstable_should_yield_to_host, Priority,
};

use crate::host_config::ReactDomHostConfig;
use crate::renderer::Renderer;
use crate::utils::set_panic_hook;

mod host_config;
mod renderer;
mod synthetic_event;
mod utils;

// static mut CONTAINER_TO_ROOT: Option<HashMap<JsValue, Rc<RefCell<FiberRootNode>>>> = None;

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

    // TODO cache the container
    // let mut root;
    // unsafe {
    //     if CONTAINER_TO_ROOT.is_none() {
    //         CONTAINER_TO_ROOT = Some(HashMap::new());
    //     }
    // };
    // log!(
    //     "ptr {:?}",
    //     Reflect::get(container, &JsValue::from_str("ptr")).unwrap()
    // );
    // unsafe {
    //     CONTAINER_TO_ROOT.unwrap().insert(container.clone(), root);
    // }

    let root = reconciler.create_container(Rc::new(node));
    let renderer = Renderer::new(root, reconciler, container);
    renderer
}
