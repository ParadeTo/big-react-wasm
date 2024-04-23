use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsValue;
use web_sys::{Element, window};
use web_sys::js_sys::Reflect;

use crate::fiber::FiberRootNode;

pub mod fiber;

pub trait HostConfig {
    fn create_text_instance(&self, content: String) -> Rc<dyn Any>;
    fn create_instance(&self, _type: String) -> Rc<dyn Any>;
    fn append_initial_child(&self, parent: Rc<dyn Any>, child: Rc<dyn Any>);
    fn append_child_to_container(&self, child: Rc<dyn Any>, parent: Rc<dyn Any>);
}

pub struct Reconciler {
    host_config: Box<dyn HostConfig>,
}

impl Reconciler {
    pub fn new(host_config: Box<dyn HostConfig>) -> Self {
        Reconciler { host_config }
    }
    pub fn create_container(&self, container: &JsValue) -> Rc<RefCell<FiberRootNode>> {
        Rc::new(RefCell::new(FiberRootNode {}))
    }

    pub fn update_container(&self, element: Rc<JsValue>, root: Rc<RefCell<FiberRootNode>>) {
        let props = Reflect::get(&*element, &JsValue::from_str("props")).unwrap();
        let _type = Reflect::get(&*element, &JsValue::from_str("type")).unwrap();
        let children = Reflect::get(&props, &JsValue::from_str("children")).unwrap();
        let text_instance = self.host_config.create_text_instance(children.as_string().unwrap());
        let div_instance = self.host_config.create_instance(_type.as_string().unwrap());
        self.host_config.append_initial_child(div_instance.clone(), text_instance);
        let window = window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().expect("document should have a body");
        body.append_child(&*div_instance.clone().downcast::<Element>().unwrap());
    }
}


