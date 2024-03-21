mod utils;
mod renderer;

use wasm_bindgen::prelude::*;
use web_sys::{console};
use react_reconciler::create_container;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn create_root(container: &JsValue) {
    let root = create_container(container);

}
