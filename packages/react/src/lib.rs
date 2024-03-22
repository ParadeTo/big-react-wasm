use std::collections::HashMap;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::js_sys;
use web_sys::js_sys::{Object, Reflect};

use shared::REACT_ELEMENT_TYPE;

use crate::utils::set_panic_hook;

mod utils;

const MARK: &str = "ayou";

#[wasm_bindgen]
#[derive(Debug)]
pub struct ReactElement {
    _typeof: REACT_ELEMENT_TYPE,
    _type: Rc<JsValue>,
    key: Option<String>,
    _ref: Option<JsValue>,
    props: Rc<HashMap<String, JsValue>>,
    __mark: String,
}

impl ReactElement {
    fn new(_type: Rc<JsValue>, key: Option<String>, _ref: Option<JsValue>, props: Rc<HashMap<String, JsValue>>) -> Self {
        Self { _typeof: REACT_ELEMENT_TYPE, _type, key, _ref, props, __mark: MARK.to_string() }
    }
}

struct Config {}

#[wasm_bindgen(js_name = jsxDEV)]
pub fn jsx_dev(_type: &JsValue, config: &JsValue) -> ReactElement {
    set_panic_hook();
    let conf = config.dyn_ref::<Object>().unwrap();

    let mut key: Option<String> = None;
    let mut _ref: Option<JsValue> = None;
    let mut props: HashMap<String, JsValue> = HashMap::new();
    for prop in js_sys::Object::keys(conf) {
        let val = Reflect::get(conf, &prop);
        match prop.as_string() {
            None => {}
            Some(k) => {
                if k == "key" && val.is_ok() {
                    key = val.unwrap().as_string();
                } else if k == "ref" && val.is_ok() {
                    _ref = Some(val.unwrap());
                } else if val.is_ok() {
                    props.insert(k, val.unwrap());
                }
            }
        }
    }


    // let a = _type.dyn_ref::<Function>();
    // if a.is_some() {
    //     let this = JsValue::null();
    //     console::log_1(&a.unwrap().call0(&this).unwrap());
    // } else {
    //     let a = _type.dyn_ref::<JsString>();
    //     console::log_1(a.unwrap());
    // }
    ReactElement::new(Rc::new(_type.clone()), key, _ref, Rc::new(props))
}