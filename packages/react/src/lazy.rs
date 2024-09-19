use js_sys::{Function, Reflect};
use shared::derive_from_js_value;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};

pub static UNINITIALIZED: i8 = -1;
static PENDING: i8 = 0;
static RESOLVED: i8 = 1;
static REJECTED: i8 = 2;

pub fn lazy_initializer(payload: JsValue) -> Result<JsValue, JsValue> {
    let status = derive_from_js_value(&payload, "_status");
    if status == UNINITIALIZED {
        let ctor = derive_from_js_value(&payload, "_result");
        let ctor_fn = ctor.dyn_ref::<Function>().unwrap();
        let thenable = ctor_fn.call0(ctor_fn).unwrap();
        let then_jsvalue = derive_from_js_value(&thenable, "then");
        let then = then_jsvalue.dyn_ref::<Function>().unwrap();

        let payload1 = payload.clone();
        let on_resolve_closure = Closure::wrap(Box::new(move |module: JsValue| {
            Reflect::set(&payload1, &"_status".into(), &JsValue::from(RESOLVED));
            Reflect::set(&payload1, &"_result".into(), &module);
        }) as Box<dyn Fn(JsValue) -> ()>);
        let on_resolve = on_resolve_closure
            .as_ref()
            .unchecked_ref::<Function>()
            .clone();
        on_resolve_closure.forget();

        let payload2 = payload.clone();
        let on_reject_closure = Closure::wrap(Box::new(move |err: JsValue| {
            Reflect::set(&payload2, &"_status".into(), &JsValue::from(REJECTED));
            Reflect::set(&payload2, &"_result".into(), &err);
        }) as Box<dyn Fn(JsValue) -> ()>);
        let on_reject = on_reject_closure
            .as_ref()
            .unchecked_ref::<Function>()
            .clone();

        then.call2(&thenable, &on_resolve, &on_reject);

        Reflect::set(&payload, &"_status".into(), &JsValue::from(PENDING));
        Reflect::set(&payload, &"_result".into(), &thenable);
    }

    if status == RESOLVED {
        let module = derive_from_js_value(&payload, "_result");
        return Ok(derive_from_js_value(&module, "default"));
    } else {
        return Err(derive_from_js_value(&payload, "_result"));
    }
}
