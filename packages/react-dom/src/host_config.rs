use std::any::Any;
use std::rc::Rc;

use js_sys::JSON::stringify;
use wasm_bindgen::JsValue;
use web_sys::{Node, window};

use react_reconciler::HostConfig;
use shared::{log, type_of};

use crate::synthetic_event::update_fiber_props;

pub struct ReactDomHostConfig;

pub fn to_string(js_value: &JsValue) -> String {
    js_value.as_string().unwrap_or_else(|| {
        if js_value.is_undefined() {
            "undefined".to_owned()
        } else if js_value.is_null() {
            "null".to_owned()
        } else if type_of(js_value, "boolean") {
            let bool_value = js_value.as_bool().unwrap();
            bool_value.to_string()
        } else if js_value.as_f64().is_some() {
            let num_value = js_value.as_f64().unwrap();
            num_value.to_string()
        } else {
            let js_string = stringify(&js_value).unwrap();
            js_string.into()
        }
    })
}

impl HostConfig for ReactDomHostConfig {
    fn create_text_instance(&self, content: &JsValue) -> Rc<dyn Any> {
        let window = window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        Rc::new(Node::from(
            document.create_text_node(to_string(content).as_str()),
        ))
    }

    fn create_instance(&self, _type: String, props: Rc<dyn Any>) -> Rc<dyn Any> {
        let window = window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        match document.create_element(_type.as_ref()) {
            Ok(element) => {
                let element = update_fiber_props(
                    element.clone(),
                    &*props.clone().downcast::<JsValue>().unwrap(),
                );
                Rc::new(Node::from(element))
            }
            Err(_) => {
                panic!("Failed to create_instance {:?}", _type);
            }
        }
    }

    fn append_initial_child(&self, parent: Rc<dyn Any>, child: Rc<dyn Any>) {
        let p = parent.clone().downcast::<Node>().unwrap();
        let c = child.clone().downcast::<Node>().unwrap();
        match p.append_child(&c) {
            Ok(_) => {
                log!(
                    "append_initial_child {:?} {:?}",
                    p,
                    if c.first_child().is_some() {
                        c.first_child().clone().unwrap().text_content()
                    } else {
                        c.text_content()
                    }
                );
            }
            Err(_) => {
                log!("Failed to append_initial_child {:?} {:?}", p, c);
            }
        }
    }

    fn append_child_to_container(&self, child: Rc<dyn Any>, parent: Rc<dyn Any>) {
        self.append_initial_child(parent, child)
    }

    fn remove_child(&self, child: Rc<dyn Any>, container: Rc<dyn Any>) {
        let p = container.clone().downcast::<Node>().unwrap();
        let c = child.clone().downcast::<Node>().unwrap();
        match p.remove_child(&c) {
            Ok(_) => {}
            Err(e) => {
                log!("Failed to remove_child {:?} {:?} {:?} ", e, p, c);
            }
        }
    }

    fn commit_text_update(&self, text_instance: Rc<dyn Any>, content: String) {
        let text_instance = text_instance.clone().downcast::<Node>().unwrap();
        text_instance.set_node_value(Some(content.as_str()));
    }

    fn insert_child_to_container(
        &self,
        child: Rc<dyn Any>,
        container: Rc<dyn Any>,
        before: Rc<dyn Any>,
    ) {
        let parent = container.clone().downcast::<Node>().unwrap();
        let before = before.clone().downcast::<Node>().unwrap();
        let child = child.clone().downcast::<Node>().unwrap();
        match parent.insert_before(&before, Some(&child)) {
            Ok(_) => {
                log!(
                    "insert_child_to_container {:?} {:?} {:?}",
                    parent,
                    if before.first_child().is_some() {
                        before.first_child().clone().unwrap().text_content()
                    } else {
                        before.text_content()
                    },
                    if child.first_child().is_some() {
                        child.first_child().clone().unwrap().text_content()
                    } else {
                        child.text_content()
                    }
                );
            }
            Err(_) => {
                log!(
                    "Failed to insert_child_to_container {:?} {:?}",
                    parent,
                    child
                );
            }
        }
    }
}
