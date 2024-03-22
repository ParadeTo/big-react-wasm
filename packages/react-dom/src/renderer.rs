use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::js_sys::Function;
use react::ReactElement;
use react_reconciler::update_container;
use react_reconciler::fiber::FiberRootNode;
use shared::log;

#[wasm_bindgen]
pub struct Renderer {
    #[wasm_bindgen(skip)]
    pub root: Rc<RefCell<FiberRootNode>>
}

impl Renderer {
    pub fn new(root: Rc<RefCell<FiberRootNode>>) -> Self {
        Self {root}
    }
}

#[wasm_bindgen]
impl Renderer {
    pub fn render(&self, element: &ReactElement) {
        update_container(element, self.root.borrow())
    }
}