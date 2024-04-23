use wasm_bindgen::prelude::*;

use react_reconciler::Reconciler;

use crate::host_config::ReactDomHostConfig;
use crate::renderer::Renderer;
use crate::utils::set_panic_hook;

mod utils;
mod renderer;
mod host_config;

#[wasm_bindgen(js_name = createRoot)]
pub fn create_root(container: &JsValue) -> Renderer {
    set_panic_hook();
    let reconciler = Reconciler::new(Box::new(ReactDomHostConfig));
    let root = reconciler.create_container(container);
    let renderer = Renderer::new(root, reconciler);
    renderer
}