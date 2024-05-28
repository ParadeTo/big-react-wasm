use std::any::Any;
use std::rc::Rc;

use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use web_sys::js_sys::{Array, Object, Reflect};
use web_sys::js_sys::JSON::stringify;

use react_reconciler::HostConfig;
use shared::derive_from_js_value;

static mut INSTANCE_COUNTER: u32 = 0;

pub struct ReactNoopHostConfig;


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

fn type_of(p0: &JsValue, p1: &str) -> bool {
    todo!()
}

fn getCounter() -> u32 {
    let mut counter;
    unsafe {
        counter = INSTANCE_COUNTER;
        INSTANCE_COUNTER += 1;
    }
    counter
}

pub fn create_container() -> JsValue {
    let container = Object::new();
    Reflect::set(&container, &"rootId".into(), getCounter().into());
    Reflect::set(&container, &"pendingChildren".into(), &**Array::new());
    Reflect::set(&container, &"children".into(), &**Array::new());
    container.into()
}

impl HostConfig for ReactNoopHostConfig {
    fn create_text_instance(&self, content: &JsValue) -> Rc<dyn Any> {
        let obj = Object::new();
        Reflect::set(&obj, &"id".into(), getCounter().into());
        Reflect::set(&obj, &"text".into(), content);
        Reflect::set(&obj, &"parent".into(), -1.0.into());
        Rc::new(obj)
    }

    fn create_instance(&self, _type: String, props: Rc<dyn Any>) -> Rc<dyn Any> {
        let obj = Object::new();
        Reflect::set(&obj, "id".into(), getCounter().into());
        Reflect::set(&obj, "type".into(), _type.into());
        Reflect::set(&obj, "chidren".into(), &**Array::new());
        Reflect::set(&obj, "parent".into(), -1.0.into());
        Reflect::set(&obj, "props".into(), &*props.clone().downcast::<JsValue>().unwrap());
        Rc::new(obj)
    }

    fn append_initial_child(&self, parent: Rc<dyn Any>, child: Rc<dyn Any>) {
        let p = parent.clone().downcast::<JsValue>().unwrap();
        let c = child.clone().downcast::<JsValue>().unwrap();
        let prev_parent = derive_from_js_value(&c, "parent").as_f64().unwrap();
        let parent_id = derive_from_js_value(&p, "id").as_f64().unwrap();
        if prev_parent != -1.0 && prev_parent != parent_id {
            panic!("Cannot mount child repeatedly")
        }
        Reflect::set(&c, "parent".into(), parent_id.into());
        let children = derive_from_js_value(&p, "children").dyn_ref::<Array>().unwrap();
        children.push(&c);
    }

    fn append_child_to_container(&self, child: Rc<dyn Any>, container: Rc<dyn Any>) {
        let container = container.clone().downcast::<JsValue>().unwrap();
        let c = child.clone().downcast::<JsValue>().unwrap();
        let prev_parent = derive_from_js_value(&c, "parent").as_f64().unwrap();
        let root_id = derive_from_js_value(&container, "rootId").as_f64().unwrap();
        if prev_parent != -1.0 && prev_parent != root_id {
            panic!("Cannot mount child repeatedly")
        }
        Reflect::set(&c, "parent".into(), root_id.into());
        let children = derive_from_js_value(&container, "children").dyn_ref::<Array>().unwrap();
        let index = children.index_of(&c, 0);
        if index != -1 {
            children.splice(index as u32, 1, &JsValue::undefined());
        }
        children.push(&c);
    }

    fn remove_child(&self, child: Rc<dyn Any>, container: Rc<dyn Any>) {
        let container = container.clone().downcast::<JsValue>().unwrap();
        let children = derive_from_js_value(&container, "children").dyn_ref::<Array>().unwrap();
        let child = child.clone().downcast::<JsValue>().unwrap();
        let index = children.index_of(&child, 0);
        if index == -1 {
            panic!("Child does not exist")
        }
        children.splice(index as u32, 1, &JsValue::undefined());
    }

    fn commit_text_update(&self, text_instance: Rc<dyn Any>, content: &JsValue) {
        let text_instance = text_instance.clone().downcast::<JsValue>().unwrap();
        Reflect::set(&text_instance, &"text".into(), content);
    }

    fn insert_child_to_container(&self, child: Rc<dyn Any>, container: Rc<dyn Any>, before: Rc<dyn Any>) {
        let container = container.clone().downcast::<JsValue>().unwrap();
        let child = child.clone().downcast::<JsValue>().unwrap();
        let children = derive_from_js_value(&container, "children").dyn_ref::<Array>().unwrap();
        let index = children.index_of(&child, 0);
        if index != -1 {
            children.splice(index as u32, 1, &JsValue::undefined());
        }
        let before = before.clone().downcast::<JsValue>().unwrap();
        let before_index = children.index_of(&before, 0);
        if before_index != -1 {
            panic!("Before does not exist")
        }

        children.splice(before_index as u32, 0, &child);
    }

    fn schedule_microtask(&self, callback: Box<dyn FnMut()>) {
        todo!()
    }

    // fn create_instance(&self, _type: String, props: Rc<dyn Any>) -> Rc<dyn Any> {
    //     let window = window().expect("no global `window` exists");
    //     let document = window.document().expect("should have a document on window");
    //     match document.create_element(_type.as_ref()) {
    //         Ok(element) => {
    //             let element = update_fiber_props(
    //                 element.clone(),
    //                 &*props.clone().downcast::<JsValue>().unwrap(),
    //             );
    //             Rc::new(Node::from(element))
    //         }
    //         Err(_) => {
    //             panic!("Failed to create_instance {:?}", _type);
    //         }
    //     }
    // }
    //
    // fn append_initial_child(&self, parent: Rc<dyn Any>, child: Rc<dyn Any>) {
    //     let p = parent.clone().downcast::<Node>().unwrap();
    //     let c = child.clone().downcast::<Node>().unwrap();
    //     match p.append_child(&c) {
    //         Ok(_) => {
    //             log!(
    //                 "append_initial_child {:?} {:?}",
    //                 p,
    //                 if c.first_child().is_some() {
    //                     c.first_child().clone().unwrap().text_content()
    //                 } else {
    //                     c.text_content()
    //                 }
    //             );
    //         }
    //         Err(_) => {
    //             log!("Failed to append_initial_child {:?} {:?}", p, c);
    //         }
    //     }
    // }
    //
    // fn append_child_to_container(&self, child: Rc<dyn Any>, parent: Rc<dyn Any>) {
    //     self.append_initial_child(parent, child)
    // }
    //
    // fn remove_child(&self, child: Rc<dyn Any>, container: Rc<dyn Any>) {
    //     let p = container.clone().downcast::<Node>().unwrap();
    //     let c = child.clone().downcast::<Node>().unwrap();
    //     match p.remove_child(&c) {
    //         Ok(_) => {
    //             log!("remove_child {:?} {:?}", p, c);
    //         }
    //         Err(e) => {
    //             log!("Failed to remove_child {:?} {:?} {:?} ", e, p, c);
    //         }
    //     }
    // }
    //
    // fn commit_text_update(&self, text_instance: Rc<dyn Any>, content: &JsValue) {
    //     let text_instance = text_instance.clone().downcast::<Node>().unwrap();
    //     text_instance.set_node_value(Some(to_string(content).as_str()));
    // }
    //
    // fn insert_child_to_container(
    //     &self,
    //     child: Rc<dyn Any>,
    //     container: Rc<dyn Any>,
    //     before: Rc<dyn Any>,
    // ) {
    //     let parent = container.clone().downcast::<Node>().unwrap();
    //     let before = before.clone().downcast::<Node>().unwrap();
    //     let child = child.clone().downcast::<Node>().unwrap();
    //     match parent.insert_before(&child, Some(&before)) {
    //         Ok(_) => {
    //             log!(
    //                 "insert_child_to_container {:?} {:?} {:?}",
    //                 parent,
    //                 if before.first_child().is_some() {
    //                     before.first_child().clone().unwrap().text_content()
    //                 } else {
    //                     before.text_content()
    //                 },
    //                 if child.first_child().is_some() {
    //                     child.first_child().clone().unwrap().text_content()
    //                 } else {
    //                     child.text_content()
    //                 }
    //             );
    //         }
    //         Err(_) => {
    //             log!(
    //                 "Failed to insert_child_to_container {:?} {:?}",
    //                 parent,
    //                 child
    //             );
    //         }
    //     }
    // }
    //
    // fn schedule_microtask(&self, callback: Box<dyn FnMut()>) {
    //     let closure = Rc::new(RefCell::new(Some(Closure::wrap(callback))));
    //
    //     if global()
    //         .unchecked_into::<Global>()
    //         .hasQueueMicrotask()
    //         .is_function()
    //     {
    //         let closure_clone = closure.clone();
    //         queueMicrotask(&closure_clone.borrow_mut().as_ref().unwrap().as_ref().unchecked_ref::<JsValue>());
    //         closure_clone.borrow_mut().take().unwrap_throw().forget();
    //     } else if js_sys::Reflect::get(&*global(), &JsValue::from_str("Promise"))
    //         .map(|value| value.is_function())
    //         .unwrap_or(false)
    //     {
    //         let promise = Promise::resolve(&JsValue::NULL);
    //         let closure_clone = closure.clone();
    //         let c = Closure::wrap(Box::new(move |_v| {
    //             let b = closure_clone.borrow_mut();
    //             let function = b.as_ref().unwrap().as_ref().unchecked_ref::<Function>();
    //             let _ = function.call0(&JsValue::NULL);
    //         }) as Box<dyn FnMut(JsValue)>);
    //         let _ = promise.then(&c);
    //         c.forget();
    //     } else {
    //         let closure_clone = closure.clone();
    //         setTimeout(&closure_clone.borrow_mut().as_ref().unwrap().as_ref().unchecked_ref::<JsValue>(), 0);
    //         closure_clone.borrow_mut().take().unwrap_throw().forget();
    //     }
    // }
}
