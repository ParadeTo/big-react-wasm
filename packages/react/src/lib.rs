use js_sys::{Object, Reflect};
use wasm_bindgen::prelude::*;

use shared::REACT_ELEMENT_TYPE;

use crate::current_dispatcher::resolve_dispatche;

pub mod current_dispatcher;

#[wasm_bindgen(typescript_custom_section)]
const TS_TYPE: &'static str = r#"
export type Action<State> = State | ((prevState: State) => State);
export type Disptach<State> = (action: Action<State>) => void;
export type useState = <State>(initialState: (() => State) | State) => [State, Disptach<State>];
"#;

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

#[wasm_bindgen(js_name = useState, skip_typescript)]
pub unsafe fn use_state() -> Vec<JsValue> {
    let dispatcher = resolve_dispatche();
    (&*dispatcher.as_ref().use_state)()
}