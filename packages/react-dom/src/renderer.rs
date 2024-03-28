use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use wasm_bindgen::prelude::wasm_bindgen;

use react_reconciler::fiber::FiberRootNode;
use react_reconciler::update_container;
use shared::{compare_js_value, derive_from_js_value, log, REACT_ELEMENT};

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
        // let element = Rc::new(ReactElement::from_js_value(element));

        // let _typeof = Rc::clone(&derive_from_js_value(Rc::new(element.clone()), "_typeof").unwrap());
        // let b = JsValue::from_str(REACT_ELEMENT);
        // log!("{:?} {:?} {:?}", _typeof, b, compare_js_value(_typeof.deref(), &b));
        // if compare_js_value(_typeof.deref(), &b) {
        //     log!("equal")
        // }
        // Reflect::equals(_typeof, b)
        // let c = _typeof.dyn_ref::<Symbol>().unwrap();
        // if b.value_of() == c.value_of() {
        //     log!("equal");
        // }

        update_container(Rc::new(element.clone()), self.root.borrow())
    }
}
