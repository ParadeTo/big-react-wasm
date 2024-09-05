use shared::derive_from_js_value;
use wasm_bindgen::JsValue;
use web_sys::js_sys::{wasm_bindgen::prelude::*, Function, Reflect};

#[wasm_bindgen]
extern "C" {
    pub static SUSPENSE_EXCEPTION: JsValue;
}

static mut SUSPENDED_THENABLE: Option<JsValue> = None;

pub fn get_suspense_thenable() -> JsValue {
    if unsafe { SUSPENDED_THENABLE.is_none() } {
        panic!("Should have SUSPENDED_THENABLE");
    }

    let thenable = unsafe { SUSPENDED_THENABLE.clone() };
    unsafe { SUSPENDED_THENABLE = None };
    thenable.unwrap()
}

pub fn track_used_thenable(thenable: JsValue) -> Result<JsValue, JsValue> {
    let status = derive_from_js_value(&thenable, "status");

    if status.is_string() {
        let status = status.as_string().unwrap();
        if status == "fulfilled" {
            return Ok(derive_from_js_value(&thenable, "value"));
        } else if status == "rejected" {
            return Err(derive_from_js_value(&thenable, "reason"));
        }

        let v = derive_from_js_value(&thenable, "then");
        let then = v.dyn_ref::<Function>().unwrap();

        let closure = Closure::wrap(Box::new(move || {}) as Box<dyn Fn()>);
        let noop = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        then.call2(&thenable, &noop, &noop);
    } else {
        Reflect::set(&thenable, &"status".into(), &"pending".into());
        let v = derive_from_js_value(&thenable, "then");
        let then = v.dyn_ref::<Function>().unwrap();

        let thenable1 = thenable.clone();
        let on_resolve_closure = Closure::wrap(Box::new(move |val: JsValue| {
            if derive_from_js_value(&thenable1, "status") == "pending" {
                Reflect::set(&thenable1, &"status".into(), &"fulfilled".into());
                Reflect::set(&thenable1, &"value".into(), &val);
            }
        }) as Box<dyn Fn(JsValue) -> ()>);
        let on_resolve = on_resolve_closure
            .as_ref()
            .unchecked_ref::<Function>()
            .clone();
        on_resolve_closure.forget();

        let thenable2 = thenable.clone();
        let on_reject_closure = Closure::wrap(Box::new(move |err: JsValue| {
            if derive_from_js_value(&thenable2, "status") == "pending" {
                Reflect::set(&thenable2, &"status".into(), &"rejected".into());
                Reflect::set(&thenable2, &"reason".into(), &err);
            }
        }) as Box<dyn Fn(JsValue) -> ()>);
        let on_reject = on_reject_closure
            .as_ref()
            .unchecked_ref::<Function>()
            .clone();
        on_reject_closure.forget();

        then.call2(&thenable, &on_resolve, &on_reject);
    }
    unsafe { SUSPENDED_THENABLE = Some(thenable.clone()) };
    Err(SUSPENSE_EXCEPTION.__inner.with(JsValue::clone))
}
