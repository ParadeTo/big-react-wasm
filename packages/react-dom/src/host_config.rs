use std::any::Any;
use std::rc::Rc;

use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, Text, window};

use react_reconciler::host_config::HostConfig;
use shared::log;

pub struct DomHostConfig;

impl HostConfig for DomHostConfig {
    fn create_text_instance(&self, content: String) -> Rc<dyn Any> {
        let window = window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        log!("create_text_instance - {:?}", content.as_str());
        Rc::new(document.create_text_node(content.as_str()))
    }

    fn create_instance(&self, _type: String) -> Rc<dyn Any> {
        let window = window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        match document.create_element(_type.as_ref()) {
            Ok(element) => Rc::new(element),
            Err(_) => todo!(),
        }
    }

    fn append_initial_child(&self, parent: Rc<dyn Any>, child: Rc<dyn Any>) {
        // let cloned = parent.clone();
        // let js_value = parent.clone().downcast::<JsValue>().unwrap().dyn_ref::<Element>();

        let p = match parent.clone().downcast::<Element>() {
            Ok(ele) => {
                let child = child.downcast::<Text>().unwrap();

                match ele.append_child(&child) {
                    Ok(_) => {
                        log!("append_initial_child successfully ele {:?} {:?}", ele, child);
                    }
                    Err(_) => todo!(),
                }
            }
            Err(_) => {
                let p = parent
                    .clone()
                    .downcast::<JsValue>().unwrap();
                let ele = p.dyn_ref::<Element>().unwrap();
                let child = child.downcast::<Element>().unwrap();
                match ele.append_child(&child) {
                    Ok(_) => {
                        log!("append_initial_child successfully jsvalue {:?} {:?}", ele, child);
                    }
                    Err(_) => todo!(),
                }
            }
        };


        // ().unwrap_or_else(
        //     |err| *parent
        //         .clone()
        //         .downcast::<JsValue>()
        //         .unwrap()
        //         .dyn_ref::<Element>().unwrap()
        // );
    }

    fn append_child_to_container(&self, child: Rc<dyn Any>, parent: Rc<dyn Any>) {
        self.append_initial_child(parent, child)
    }
}
