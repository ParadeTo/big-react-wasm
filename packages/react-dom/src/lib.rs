use wasm_bindgen::prelude::*;

use react_reconciler::create_container;

use crate::renderer::Renderer;
use crate::utils::set_panic_hook;

mod renderer;
mod utils;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen(js_name = createRoot)]
pub fn create_root(container: &JsValue) -> Renderer {
    set_panic_hook();
    let root = create_container(container);
    Renderer::new(root)
}
