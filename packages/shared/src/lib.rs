use web_sys::js_sys::JSON::stringify;
use web_sys::js_sys::Reflect;
use web_sys::wasm_bindgen::JsValue;

pub static REACT_ELEMENT_TYPE: &str = "react.element";

#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

pub fn derive_from_js_value(js_value: &JsValue, str: &str) -> JsValue {
    match Reflect::get(&js_value, &JsValue::from_str(str)) {
        Ok(v) => v,
        Err(_) => {
            log!("derive {} from {:?} error", str, js_value);
            JsValue::undefined()
        }
    }
}

pub fn is_dev() -> bool {
    // env!("ENV") == "dev"
    true
}

pub fn type_of(val: &JsValue, _type: &str) -> bool {
    let t = if val.is_undefined() {
        "undefined".to_string()
    } else if val.is_null() {
        "null".to_string()
    } else if val.as_bool().is_some() {
        "boolean".to_string()
    } else if val.as_f64().is_some() {
        "number".to_string()
    } else if val.as_string().is_some() {
        "string".to_string()
    } else if val.is_function() {
        "function".to_string()
    } else if val.is_object() {
        "object".to_string()
    } else {
        "unknown".to_string()
    };
    t == _type
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
