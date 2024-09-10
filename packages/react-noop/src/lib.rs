use std::rc::Rc;

use wasm_bindgen::prelude::*;

use react_reconciler::Reconciler;

use crate::host_config::{create_container, ReactNoopHostConfig};
use crate::renderer::Renderer;
use crate::utils::set_panic_hook;

mod utils;
mod renderer;
mod host_config;


#[wasm_bindgen(js_name = createRoot)]
pub fn create_root() -> Renderer {
    set_panic_hook();
    let container = create_container();
    let reconciler = Reconciler::new(Rc::new(ReactNoopHostConfig));
    let root = reconciler.create_container(Rc::new(container.clone()));
    let renderer = Renderer::new(root, reconciler, container);
    renderer
}