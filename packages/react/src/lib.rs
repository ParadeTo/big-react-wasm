use js_sys::{Array, Object, Reflect, JSON};
use wasm_bindgen::prelude::*;

use shared::{
    derive_from_js_value, REACT_CONTEXT_TYPE, REACT_ELEMENT_TYPE, REACT_MEMO_TYPE,
    REACT_PROVIDER_TYPE,
};

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
                    Reflect::set(&config, &"children".into(), &children.get(0))
                        .expect("TODO: panic children");
                } else {
                    Reflect::set(&config, &"children".into(), maybe_children)
                        .expect("TODO: panic set children");
                }
            }
        }
    };
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

#[wasm_bindgen(js_name = useEffect)]
pub unsafe fn use_effect(create: &JsValue, deps: &JsValue) {
    let use_effect = &CURRENT_DISPATCHER.current.as_ref().unwrap().use_effect;
    use_effect.call2(&JsValue::null(), create, deps);
}

#[wasm_bindgen(js_name = useRef)]
pub unsafe fn use_ref(initial_value: &JsValue) -> Result<JsValue, JsValue> {
    let use_ref = &CURRENT_DISPATCHER.current.as_ref().unwrap().use_ref;
    use_ref.call1(&JsValue::null(), initial_value)
}

#[wasm_bindgen(js_name = useMemo)]
pub unsafe fn use_memo(create: &JsValue, deps: &JsValue) -> Result<JsValue, JsValue> {
    let use_memo = &CURRENT_DISPATCHER.current.as_ref().unwrap().use_memo;
    use_memo.call2(&JsValue::null(), create, deps)
}

#[wasm_bindgen(js_name = useCallback)]
pub unsafe fn use_callback(callback: &JsValue, deps: &JsValue) -> Result<JsValue, JsValue> {
    let use_callback = &CURRENT_DISPATCHER.current.as_ref().unwrap().use_callback;
    use_callback.call2(&JsValue::null(), callback, deps)
}

#[wasm_bindgen(js_name = useContext)]
pub unsafe fn use_context(context: &JsValue) -> Result<JsValue, JsValue> {
    let use_context = &CURRENT_DISPATCHER.current.as_ref().unwrap().use_context;
    use_context.call1(&JsValue::null(), context)
}

#[wasm_bindgen(js_name = createContext)]
pub unsafe fn create_context(default_value: &JsValue) -> JsValue {
    let context = Object::new();
    Reflect::set(
        &context,
        &"$$typeof".into(),
        &JsValue::from_str(REACT_CONTEXT_TYPE),
    );
    Reflect::set(&context, &"_currentValue".into(), default_value);
    let provider = Object::new();
    Reflect::set(
        &provider,
        &"$$typeof".into(),
        &JsValue::from_str(REACT_PROVIDER_TYPE),
    );
    Reflect::set(&provider, &"_context".into(), &context);
    Reflect::set(&context, &"Provider".into(), &provider);
    context.into()
}

#[wasm_bindgen]
pub unsafe fn memo(_type: &JsValue, compare: &JsValue) -> JsValue {
    let fiber_type = Object::new();

    Reflect::set(
        &fiber_type,
        &"$$typeof".into(),
        &JsValue::from_str(REACT_MEMO_TYPE),
    );
    Reflect::set(&fiber_type, &"type".into(), _type);

    let null = JsValue::null();
    Reflect::set(
        &fiber_type,
        &"compare".into(),
        if compare.is_undefined() {
            &null
        } else {
            compare
        },
    );
    fiber_type.into()
}
