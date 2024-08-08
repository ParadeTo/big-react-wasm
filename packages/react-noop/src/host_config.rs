use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use react_reconciler::work_tags::WorkTag;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::js_sys;
use web_sys::js_sys::JSON::stringify;
use web_sys::js_sys::{global, Array, Function, Object, Promise, Reflect};

use react_reconciler::fiber::FiberNode;
use react_reconciler::HostConfig;
use shared::{derive_from_js_value, log};

static mut INSTANCE_COUNTER: u32 = 0;

pub struct ReactNoopHostConfig;

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
    Reflect::set(&container, &"rootId".into(), &JsValue::from(getCounter()));
    Reflect::set(&container, &"pendingChildren".into(), &**Array::new());
    Reflect::set(&container, &"children".into(), &**Array::new());
    container.into()
}

impl ReactNoopHostConfig {
    fn commit_text_update(&self, text_instance: Rc<dyn Any>, content: &JsValue) {
        let text_instance = text_instance.clone().downcast::<JsValue>().unwrap();
        Reflect::set(&text_instance, &"text".into(), content);
    }
}

impl HostConfig for ReactNoopHostConfig {
    fn create_text_instance(&self, content: &JsValue) -> Rc<dyn Any> {
        let obj = Object::new();
        Reflect::set(&obj, &"id".into(), &getCounter().into());
        Reflect::set(&obj, &"text".into(), &content);
        Reflect::set(&obj, &"parent".into(), &JsValue::from(-1.0));
        Rc::new(JsValue::from(obj))
    }

    fn create_instance(&self, _type: String, props: Rc<dyn Any>) -> Rc<dyn Any> {
        let obj = Object::new();
        Reflect::set(&obj, &"id".into(), &getCounter().into());
        Reflect::set(&obj, &"type".into(), &_type.into());
        Reflect::set(&obj, &"children".into(), &**Array::new());
        Reflect::set(&obj, &"parent".into(), &JsValue::from(-1.0));
        Reflect::set(
            &obj,
            &"props".into(),
            &*props.clone().downcast::<JsValue>().unwrap(),
        );
        Rc::new(JsValue::from(obj))
    }

    fn append_initial_child(&self, parent: Rc<dyn Any>, child: Rc<dyn Any>) {
        let p = parent.clone().downcast::<JsValue>().unwrap();
        let c = child.clone().downcast::<JsValue>().unwrap();
        let prev_parent = derive_from_js_value(&c, "parent").as_f64().unwrap();
        let parent_id = derive_from_js_value(&p, "id").as_f64().unwrap();
        if prev_parent != -1.0 && prev_parent != parent_id {
            panic!("Cannot mount child repeatedly")
        }
        Reflect::set(&c, &"parent".into(), &parent_id.into());
        let children_js_value = derive_from_js_value(&p, "children");
        let children = children_js_value.dyn_ref::<Array>().unwrap();
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
        Reflect::set(&c, &"parent".into(), &JsValue::from(root_id));
        let children_js_value = derive_from_js_value(&container, "children");
        let children = children_js_value.dyn_ref::<Array>().unwrap();
        let index = children.index_of(&c, 0);
        if index != -1 {
            children.splice(index as u32, 1, &JsValue::undefined());
        }
        children.push(&c);
    }

    fn remove_child(&self, child: Rc<dyn Any>, container: Rc<dyn Any>) {
        let container = container.clone().downcast::<JsValue>().unwrap();
        let children_js_value = derive_from_js_value(&container, "children");
        let children = children_js_value.dyn_ref::<Array>().unwrap();
        let child = child.clone().downcast::<JsValue>().unwrap();
        let index = children.index_of(&child, 0);
        if index == -1 {
            panic!("Child does not exist")
        }
        children.splice(index as u32, 1, &JsValue::undefined());
    }

    fn insert_child_to_container(
        &self,
        child: Rc<dyn Any>,
        container: Rc<dyn Any>,
        before: Rc<dyn Any>,
    ) {
        let container = container.clone().downcast::<JsValue>().unwrap();
        let child = child.clone().downcast::<JsValue>().unwrap();
        let children_js_value = derive_from_js_value(&container, "children");
        let children = children_js_value.dyn_ref::<Array>().unwrap();
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
        match fiber.borrow().tag {
            WorkTag::HostText => {
                let text = derive_from_js_value(&fiber.borrow().memoized_props, "content");
                let instance = FiberNode::derive_state_node(fiber.clone());
                self.commit_text_update(instance.unwrap(), &text);
            }
            _ => {
                log!("Unsupported update type")
            }
        }
    }
}
