use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use wasm_bindgen::prelude::wasm_bindgen;

use react::ReactElement;
use react_reconciler::fiber::FiberRootNode;
use shared::log;

#[wasm_bindgen]
pub struct Renderer {
    #[wasm_bindgen(skip)]
    pub root: Rc<RefCell<FiberRootNode>>,
}

impl Renderer {
    pub fn new(root: Rc<RefCell<FiberRootNode>>) -> Self {
        Self { root }
    }
}

#[wasm_bindgen]
impl Renderer {
    pub fn render(&self, element: &JsValue) {
        let element = ReactElement::from_js_value(element);
        log!("{:?}", element);
        // update_container(Rc::new(element.clone()), self.root.borrow())
    }
}
