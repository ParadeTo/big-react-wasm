use std::rc::Rc;

use js_sys::{Object, Reflect};
use wasm_bindgen::prelude::*;

use shared::REACT_ELEMENT_TYPE;

use crate::current_dispatcher::{Dispatcher, resolve_dispatcher};

pub mod current_dispatcher;

struct MyStruct {
    current: Option<Rc<Dispatcher>>,
}

impl MyStruct {
    pub fn new(current: Option<Rc<Dispatcher>>) -> Self {
        MyStruct { current }
    }
}

#[wasm_bindgen(js_name = jsxDEV)]
pub unsafe fn jsx_dev(_type: &JsValue, config: &JsValue, key: &JsValue) -> JsValue {
    let react_element = Object::new();
    Reflect::set(
        &react_element,
        &"$$typeof".into(),
        &JsValue::from_str(REACT_ELEMENT_TYPE),
    )
        .expect("$$typeof panic");
    Reflect::set(&react_element, &"type".into(), _type).expect("type panic");
    Reflect::set(&react_element, &"key".into(), key).expect("key panic");

    let conf = config.dyn_ref::<Object>().unwrap();
    let props = Object::new();
    for prop in Object::keys(conf) {
        let val = Reflect::get(conf, &prop);
        match prop.as_string() {
            None => {}
            Some(k) => {
                if k == "ref" && val.is_ok() {
                    Reflect::set(&react_element, &"ref".into(), &val.unwrap()).expect("ref panic");
                } else if val.is_ok() {
                    Reflect::set(&props, &JsValue::from(k), &val.unwrap()).expect("props panic");
                }
            }
        }
    }

    Reflect::set(&react_element, &"props".into(), &props).expect("props panic");
    react_element.into()
}


#[wasm_bindgen(js_name = useState)]
pub unsafe fn use_state() -> Vec<JsValue> {
    let dispatcher = resolve_dispatcher();
    (&*dispatcher.as_ref().use_state)()
}

