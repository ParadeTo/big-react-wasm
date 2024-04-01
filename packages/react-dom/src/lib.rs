use wasm_bindgen::prelude::*;

use react_reconciler::create_container;
use react_reconciler::host_config::{get_host_config, init_host_config};
use shared::log;

use crate::host_config::DomHostConfig;
use crate::renderer::DomRenderer;
use crate::utils::set_panic_hook;

mod renderer;
mod utils;
mod host_config;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}


#[wasm_bindgen(js_name = createRoot)]
pub fn create_root(container: &JsValue) -> DomRenderer {
    set_panic_hook();
    let root = create_container(container);
    let renderer = DomRenderer::new(root);
    init_host_config(Box::new(DomHostConfig));
    let a = get_host_config();
    log!("{:?}", a.create_instance("div".to_string()));
    renderer
}


