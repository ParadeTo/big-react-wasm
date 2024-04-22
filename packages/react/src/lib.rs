use js_sys::{Object, Reflect};
use wasm_bindgen::prelude::*;

use shared::{log, REACT_ELEMENT_TYPE};

use crate::current_dispatcher::CURRENT_DISPATCHER;

pub mod current_dispatcher;

#[wasm_bindgen(js_name = jsxDEV)]
pub fn jsx_dev(_type: &JsValue, config: &JsValue, key: &JsValue) -> JsValue {
    let react_element = Object::new();
    Reflect::set(
        &react_element,
        &"$$typeof".into(),
        &JsValue::from_str(REACT_ELEMENT_TYPE),
    )
        .expect("$$typeof panic");
    Reflect::set(&react_element, &"type".into(), _type).expect("type panic");
    Reflect::set(&react_element, &"key".into(), key).expect("key panic");

    let props = Object::new();

    if let Some(conf) = config.dyn_ref::<Object>() {
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
    }
    
    Reflect::set(&react_element, &"props".into(), &props).expect("props panic");
    react_element.into()
}

#[wasm_bindgen(js_name = createElement)]
pub fn create_element(_type: &JsValue, config: &JsValue, key: &JsValue) -> JsValue {
    log!("create_element {:?} {:?} {:?}", _type, config, key);
    jsx_dev(_type, config, key)
}


#[wasm_bindgen(js_name = useState)]
pub unsafe fn use_state(initial_state: &JsValue) -> Result<JsValue, JsValue> {
    let use_state = &CURRENT_DISPATCHER.current.as_ref().unwrap().use_state;
    use_state.call1(&JsValue::null(), initial_state)
}

