use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

use react_reconciler::fiber::FiberRootNode;
use react_reconciler::update_container;

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
        update_container(Rc::new(element.clone()), self.root.borrow())
    }
}
