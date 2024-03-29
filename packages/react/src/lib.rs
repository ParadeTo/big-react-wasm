use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::js_sys;
use web_sys::js_sys::{Object, Reflect};

use shared::{log, REACT_ELEMENT};

use crate::utils::set_panic_hook;

mod utils;

const MARK: &str = "ayou";

#[derive(Debug)]
pub struct ReactElement {
    _typeof: String,
    pub _type: Rc<JsValue>,
    pub key: Option<String>,
    _ref: Option<JsValue>,
    pub props: Option<Rc<JsValue>>,
    __mark: String,
}

impl ReactElement {
    fn new(
        _type: Rc<JsValue>,
        key: Option<String>,
        _ref: Option<JsValue>,
        props: Option<Rc<JsValue>>,
    ) -> Self {
        Self {
            _typeof: REACT_ELEMENT.to_string(),
            _type,
            key,
            _ref,
            props,
            __mark: MARK.to_string(),
        }
    }

    pub fn from_js_value(js_value: &JsValue) -> Self {
        let _type =
            Rc::new(Reflect::get(js_value, &JsValue::from_str("_type")).expect("_type err"));
        let key = Reflect::get(js_value, &JsValue::from_str("key"))
            .expect("key err")
            .as_string();
        let _ref = Reflect::get(js_value, &JsValue::from_str("_ref")).ok();

        let props = Reflect::get(js_value, &JsValue::from_str("props")).unwrap();
        // let mut props: HashMap<String, JsValue> = HashMap::new();
        // for prop in js_sys::Object::keys(props_js_value.dyn_ref::<Object>().unwrap()) {
        //     let val = Reflect::get(&props_js_value, &prop);
        //     match prop.as_string() {
        //         None => {}
        //         Some(k) => {
        //             if val.is_ok() {
        //                 props.insert(k, val.unwrap());
        //             }
        //         }
        //     }
        // }

        Self {
            _typeof: REACT_ELEMENT.to_string(),
            _type,
            key,
            _ref,
            props: Some(Rc::new(props)),
            __mark: MARK.to_string(),
        }
    }
}

struct Config {}

#[wasm_bindgen(js_name = jsxDEV)]
pub fn jsx_dev(_type: &JsValue, config: &JsValue) -> JsValue {
    set_panic_hook();
    let obj = Object::new();
    Reflect::set(&obj, &"_typeof".into(), &JsValue::from_str(REACT_ELEMENT))
        .expect("_typeof panic");
    Reflect::set(&obj, &"_type".into(), _type).expect("_type panic");

    let conf = config.dyn_ref::<Object>().unwrap();
    let mut props = Object::new();
    for prop in js_sys::Object::keys(conf) {
        let val = Reflect::get(conf, &prop);
        match prop.as_string() {
            None => {}
            Some(k) => {
                log!("{} {:?}", k, val.clone().unwrap().as_string());
                if k == "key" && val.is_ok() {
                    Reflect::set(&obj, &"key".into(), &val.unwrap()).expect("key panic");
                } else if k == "ref" && val.is_ok() {
                    Reflect::set(&obj, &"_ref".into(), &val.unwrap()).expect("_ref panic");
                } else if val.is_ok() {
                    Reflect::set(&props, &JsValue::from(k), &val.unwrap()).expect("props panic");
                }
            }
        }
    }

    Reflect::set(&obj, &"props".into(), &props).expect("props panic");
    obj.into()
}
