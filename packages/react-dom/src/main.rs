use wasm_bindgen::JsValue;

use react_dom::create_root;

fn main() {
    // let _jsx = jsx(&JsValue::from_str("div"), &Object::new());
    let renderer = create_root(&JsValue::null());
    renderer.render(&JsValue::null());
}