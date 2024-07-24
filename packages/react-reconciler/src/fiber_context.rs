use wasm_bindgen::JsValue;
use web_sys::js_sys::Reflect;

static mut PREV_CONTEXT_VALUE: JsValue = JsValue::null();
static mut PREV_CONTEXT_VALUE_STACK: Vec<JsValue> = vec![];

pub fn push_provider(context: &JsValue, new_value: JsValue) {
    unsafe {
        PREV_CONTEXT_VALUE_STACK.push(PREV_CONTEXT_VALUE.clone());
        PREV_CONTEXT_VALUE = Reflect::get(context, &"_currentValue".into()).unwrap();
        Reflect::set(context, &"_currentValue".into(), &new_value);
    }
}

pub fn pop_provider(context: &JsValue) {
    unsafe {
        Reflect::set(context, &"_currentValue".into(), &PREV_CONTEXT_VALUE);
        let top = PREV_CONTEXT_VALUE_STACK.pop();
        if top.is_none() {
            PREV_CONTEXT_VALUE = JsValue::null();
        } else {
            PREV_CONTEXT_VALUE = top.unwrap();
        }
    }
}
