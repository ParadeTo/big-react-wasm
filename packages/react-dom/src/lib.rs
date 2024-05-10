use std::rc::Rc;

use wasm_bindgen::prelude::*;
use web_sys::Node;

use react_reconciler::Reconciler;
use shared::log;

use crate::host_config::ReactDomHostConfig;
use crate::renderer::Renderer;
use crate::utils::set_panic_hook;

mod host_config;
mod renderer;
mod synthetic_event;
mod utils;

struct T {
    name: String,
}

impl T {
    fn new() -> Self {
        T { name: "ayou".to_string() }
    }

    fn callback(&self) {
        log!("{:?}", self.name);
    }
}

#[wasm_bindgen(js_name = createRoot)]
pub fn create_root(container: &JsValue) -> Renderer {
    set_panic_hook();
    let reconciler = Reconciler::new(Rc::new(ReactDomHostConfig));
    let t = Box::new(T::new());
    reconciler.host_config.schedule_microtask(Box::new(move || t.callback()));
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
