use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use js_sys::JSON::stringify;
use js_sys::{global, Function, Promise};
use react_reconciler::work_tags::WorkTag;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::{window, Element, Node};

use react_reconciler::fiber::FiberNode;
use react_reconciler::HostConfig;
use shared::{derive_from_js_value, log, type_of};

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

#[wasm_bindgen]
extern "C" {
    type Global;

    #[wasm_bindgen]
    fn queueMicrotask(closure: &JsValue);

    #[wasm_bindgen]
    fn setTimeout(closure: &JsValue, timeout: i32);

    #[wasm_bindgen(method, getter, js_name = queueMicrotask)]
    fn hasQueueMicrotask(this: &Global) -> JsValue;
}

impl ReactDomHostConfig {
    fn commit_text_update(&self, text_instance: Rc<dyn Any>, content: &JsValue) {
        let text_instance = text_instance.clone().downcast::<Node>().unwrap();
        text_instance.set_node_value(Some(to_string(content).as_str()));
    }
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
                update_fiber_props(
                    &element.clone(),
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
                // log!(
                //     "append_initial_child {:?} {:?}",
                //     p,
                //     if c.first_child().is_some() {
                //         c.first_child().clone().unwrap().text_content()
                //     } else {
                //         c.text_content()
                //     }
                // );
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
            Ok(_) => {
                // log!("remove_child {:?} {:?}", p, c);
            }
            Err(e) => {
                log!("Failed to remove_child {:?} {:?} {:?} ", e, p, c);
            }
        }
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
        match parent.insert_before(&child, Some(&before)) {
            Ok(_) => {
                // log!(
                //     "insert_child_to_container {:?} {:?} {:?}",
                //     parent,
                //     if before.first_child().is_some() {
                //         before.first_child().clone().unwrap().text_content()
                //     } else {
                //         before.text_content()
                //     },
                //     if child.first_child().is_some() {
                //         child.first_child().clone().unwrap().text_content()
                //     } else {
                //         child.text_content()
                //     }
                // );
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

    fn schedule_microtask(&self, callback: Box<dyn FnMut()>) {
        let closure = Rc::new(RefCell::new(Some(Closure::wrap(callback))));

        if global()
            .unchecked_into::<Global>()
            .hasQueueMicrotask()
            .is_function()
        {
            let closure_clone = closure.clone();
            queueMicrotask(
                &closure_clone
                    .borrow_mut()
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .unchecked_ref::<JsValue>(),
            );
            closure_clone.borrow_mut().take().unwrap_throw().forget();
        } else if js_sys::Reflect::get(&*global(), &JsValue::from_str("Promise"))
            .map(|value| value.is_function())
            .unwrap_or(false)
        {
            let promise = Promise::resolve(&JsValue::NULL);
            let closure_clone = closure.clone();
            let c = Closure::wrap(Box::new(move |_v| {
                let b = closure_clone.borrow_mut();
                let function = b.as_ref().unwrap().as_ref().unchecked_ref::<Function>();
                let _ = function.call0(&JsValue::NULL);
            }) as Box<dyn FnMut(JsValue)>);
            let _ = promise.then(&c);
            c.forget();
        } else {
            let closure_clone = closure.clone();
            setTimeout(
                &closure_clone
                    .borrow_mut()
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .unchecked_ref::<JsValue>(),
                0,
            );
            closure_clone.borrow_mut().take().unwrap_throw().forget();
        }
    }

    fn commit_update(&self, fiber: Rc<RefCell<FiberNode>>) {
        let instance = FiberNode::derive_state_node(fiber.clone());
        let memoized_props = fiber.borrow().memoized_props.clone();
        match fiber.borrow().tag {
            WorkTag::HostText => {
                let text = derive_from_js_value(&memoized_props, "content");
                self.commit_text_update(instance.unwrap(), &text);
            }
            WorkTag::HostComponent => {
                update_fiber_props(
                    instance
                        .unwrap()
                        .downcast::<Node>()
                        .unwrap()
                        .dyn_ref::<Element>()
                        .unwrap(),
                    &memoized_props,
                );
            }
            _ => {
                log!("Unsupported update type")
            }
        };
    }
}
