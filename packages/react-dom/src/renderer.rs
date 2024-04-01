use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use wasm_bindgen::prelude::wasm_bindgen;

use react_reconciler::fiber::FiberRootNode;
use react_reconciler::update_container;

#[wasm_bindgen]
#[derive(Clone)]
pub struct DomRenderer {
    #[wasm_bindgen(skip)]
    pub root: Rc<RefCell<FiberRootNode>>,
}

impl DomRenderer {
    pub fn new(root: Rc<RefCell<FiberRootNode>>) -> Self {
        Self { root }
    }
}

#[wasm_bindgen]
impl DomRenderer {
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

        update_container(Rc::new(element.clone()), self.root.clone())
    }
}


#[cfg(test)]
mod tests {
    use wasm_bindgen::JsValue;
    use web_sys::js_sys::Object;

    use react::jsx_dev;

    use crate::create_root;

    fn renderer() {
        let jsx = jsx_dev(&JsValue::from_str("div"), &Object::new());
        let renderer = create_root(&jsx);
        renderer.render(&JsValue::null());
    }
}


