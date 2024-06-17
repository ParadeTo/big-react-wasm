use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

use react_reconciler::fiber::FiberRootNode;
use react_reconciler::Reconciler;
use web_sys::Element;

use crate::synthetic_event::init_event;

#[wasm_bindgen]
pub struct Renderer {
    container: JsValue,
    root: Rc<RefCell<FiberRootNode>>,
    reconciler: Reconciler,
}

impl Renderer {
    pub fn new(
        root: Rc<RefCell<FiberRootNode>>,
        reconciler: Reconciler,
        container: &JsValue,
    ) -> Self {
        Self {
            root,
            reconciler,
            container: container.clone(),
        }
    }

    // fn clear_container_dom(&self) {
    //     let ele = self.container.dyn_ref::<Element>().unwrap();
    //     if !ele.has_child_nodes() {
    //         return;
    //     }

    //     ele.child_nodes
    // }
}

#[wasm_bindgen]
impl Renderer {
    pub fn render(&self, element: &JsValue) -> JsValue {
        init_event(self.container.clone(), "click".to_string());
        self.reconciler
            .update_container(element.clone(), self.root.clone())
    }

    pub fn unmount(&self) -> JsValue {
        self.reconciler
            .update_container(JsValue::null(), self.root.clone())
    }
}
