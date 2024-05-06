use js_sys::{Array, JSON, Object, Reflect};
use wasm_bindgen::prelude::*;

use shared::{derive_from_js_value, log, REACT_ELEMENT_TYPE};

use crate::current_dispatcher::CURRENT_DISPATCHER;

pub mod current_dispatcher;

fn resolve_key(val: &JsValue) -> JsValue {
    if val.is_undefined() {
        JsValue::null()
    } else if val.is_string() {
        val.clone()
    } else {
        JSON::stringify(val).unwrap().into()
    }
}

fn resolve_ref(val: &JsValue) -> JsValue {
    if val.is_undefined() {
        JsValue::null()
    } else {
        val.clone()
    }
}

#[wasm_bindgen(js_name = jsxDEV)]
pub fn jsx_dev(_type: &JsValue, config: &JsValue, key: &JsValue) -> JsValue {
    let react_element = Object::new();
    let mut _ref = JsValue::null();
    let mut key = resolve_key(key);
    Reflect::set(
        &react_element,
        &"$$typeof".into(),
        &JsValue::from_str(REACT_ELEMENT_TYPE),
    )
        .expect("$$typeof panic");
    Reflect::set(&react_element, &"type".into(), _type).expect("type panic");

    let props = Object::new();
    if let Some(conf) = config.dyn_ref::<Object>() {
        for prop in Object::keys(conf) {
            let val = Reflect::get(conf, &prop);
            match prop.as_string() {
                None => {}
                Some(k) => {
                    if k == "ref" && val.is_ok() {
                        _ref = resolve_ref(&val.unwrap());
                    } else if k == "key" && val.is_ok() {
                        key = resolve_key(&val.unwrap());
                    } else if val.is_ok() {
                        Reflect::set(&props, &JsValue::from(k), &val.unwrap())
                            .expect("props panic");
                    }
                }
            }
        }
        Reflect::set(&react_element, &"props".into(), &props).expect("props panic");
    } else {
        // const config = Object.create(null, {foo: {value: 1, enumerable: true}});
        if config.is_object() {
            Reflect::set(&react_element, &"props".into(), &config).expect("props panic");
        } else {
            Reflect::set(&react_element, &"props".into(), &props).expect("props panic");
        }
    }

    Reflect::set(&react_element, &"ref".into(), &_ref).expect("ref panic");
    Reflect::set(&react_element, &"key".into(), &key).expect("key panic");
    log!("react_element2 {:?}", react_element);
    react_element.into()
}

#[wasm_bindgen(js_name = createElement, variadic)]
pub fn create_element(_type: &JsValue, config: &JsValue, maybe_children: &JsValue) -> JsValue {
    jsx(_type, config, maybe_children)
}

#[wasm_bindgen(variadic)]
pub fn jsx(_type: &JsValue, config: &JsValue, maybe_children: &JsValue) -> JsValue {
    let length = derive_from_js_value(maybe_children, "length");
    let obj = Object::new();
    let config = if config.is_object() { config } else { &*obj };
    match length.as_f64() {
        None => {}
        Some(length) => {
            if length != 0.0 {
                if length == 1.0 {
                    let children = maybe_children.dyn_ref::<Array>().unwrap();
                    log!("children {:?}", children.get(0));

                    Reflect::set(&config, &"children".into(), &children.get(0)).expect("TODO: panic children");
                } else {
                    Reflect::set(&config, &"children".into(), maybe_children);
                }
            }
        }
    };
    log!("react_element1 config {:?}", config);

    jsx_dev(_type, config, &JsValue::undefined())
}


#[wasm_bindgen(js_name = isValidElement)]
pub fn is_valid_element(object: &JsValue) -> bool {
    object.is_object()
        && !object.is_null()
        && Reflect::get(&object, &"$$typeof".into())
        .unwrap_or("".into())
        .as_string()
        .unwrap_or("".into())
        .as_str()
        == REACT_ELEMENT_TYPE
}

#[wasm_bindgen(js_name = useState)]
pub unsafe fn use_state(initial_state: &JsValue) -> Result<JsValue, JsValue> {
    let use_state = &CURRENT_DISPATCHER.current.as_ref().unwrap().use_state;
    use_state.call1(&JsValue::null(), initial_state)
}
