use gloo::events::EventListener;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen::closure::Closure;
use web_sys::{Element, Event};
use web_sys::js_sys::{Function, Object, Reflect};

use shared::{derive_from_js_value, is_dev, log};

static VALID_EVENT_TYPE_LIST: [&str; 1] = ["click"];
static ELEMENT_EVENT_PROPS_KEY: &str = "__props";

struct Paths {
    capture: Vec<Function>,
    bubble: Vec<Function>,
}

impl Paths {
    fn new() -> Self {
        Paths {
            capture: vec![],
            bubble: vec![],
        }
    }
}

fn create_synthetic_event(e: Event) -> Event {
    Reflect::set(&*e, &"__stopPropagation".into(), &JsValue::from_bool(false));

    let e_cloned = e.clone();
    let origin_stop_propagation = derive_from_js_value(&*e, "stopPropagation");
    let closure = Closure::wrap(Box::new(move || {
        Reflect::set(
            &*e_cloned,
            &"__stopPropagation".into(),
            &JsValue::from_bool(true),
        );
        if origin_stop_propagation.is_function() {
            let origin_stop_propagation = origin_stop_propagation.dyn_ref::<Function>().unwrap();
            origin_stop_propagation.call0(&JsValue::null());
        }
    }) as Box<dyn Fn()>);
    let function = closure.as_ref().unchecked_ref::<Function>().clone();
    closure.forget();
    Reflect::set(&*e.clone(), &"stopPropagation".into(), &function.into());
    e
}

fn trigger_event_flow(paths: Vec<Function>, se: &Event) {
    for callback in paths {
        callback.call1(&JsValue::null(), se);
        if derive_from_js_value(se, "__stopPropagation")
            .as_bool()
            .unwrap()
        {
            break;
        }
    }
}

fn dispatch_event(container: &Element, event_type: String, e: &Event) {
    if e.target().is_none() {
        log!("Target is none");
        return;
    }

    let target_element = e.target().unwrap().dyn_into::<Element>().unwrap();
    let Paths { capture, bubble } =
        collect_paths(Some(target_element), container, event_type.as_str());

    let se = create_synthetic_event(e.clone());

    if is_dev() {
        log!("Event {} capture phase", event_type);
    }

    trigger_event_flow(capture, &se);
    if !derive_from_js_value(&se, "__stopPropagation")
        .as_bool()
        .unwrap()
    {
        if is_dev() {
            log!("Event {} bubble phase", event_type);
        }
        trigger_event_flow(bubble, &se);
    }
}

fn collect_paths(
    mut target_element: Option<Element>,
    container: &Element,
    event_type: &str,
) -> Paths {
    let mut paths = Paths::new();
    while target_element.is_some() && !Object::is(target_element.as_ref().unwrap(), container) {
        let event_props =
            derive_from_js_value(target_element.as_ref().unwrap(), ELEMENT_EVENT_PROPS_KEY);
        if event_props.is_object() {
            let callback_name_list = get_event_callback_name_from_event_type(event_type);
            if callback_name_list.is_some() {
                for (i, callback_name) in callback_name_list.as_ref().unwrap().iter().enumerate() {
                    let event_callback = derive_from_js_value(&event_props, *callback_name);
                    if event_callback.is_function() {
                        let event_callback = event_callback.dyn_ref::<Function>().unwrap();
                        if i == 0 {
                            paths.capture.insert(0, event_callback.clone());
                        } else {
                            paths.bubble.push(event_callback.clone());
                        }
                    }
                }
            }
        }
        target_element = target_element.unwrap().parent_element();
    }
    paths
}

fn get_event_callback_name_from_event_type(event_type: &str) -> Option<Vec<&str>> {
    if event_type == "click" {
        return Some(vec!["onClickCapture", "onClick"]);
    }
    None
}

pub fn init_event(container: JsValue, event_type: String) {
    if !VALID_EVENT_TYPE_LIST.contains(&event_type.clone().as_str()) {
        log!("Unsupported event type: {:?}", event_type);
        return;
    }

    if is_dev() {
        log!("Init event {:?}", event_type);
    }

    let element = container
        .clone()
        .dyn_into::<Element>()
        .expect("container is not element");
    let on_click = EventListener::new(&element.clone(), event_type.clone(), move |event| {
        dispatch_event(&element, event_type.clone(), event)
    });
    on_click.forget();
}

pub fn update_event_props(node: Element, props: &JsValue) -> Element {
    let js_value = derive_from_js_value(&node, ELEMENT_EVENT_PROPS_KEY);
    let element_event_props = if js_value.is_object() {
        js_value.dyn_into::<Object>().unwrap()
    } else {
        Object::new()
    };
    for event_type in VALID_EVENT_TYPE_LIST {
        let callback_name_list = get_event_callback_name_from_event_type(event_type);
        if callback_name_list.is_none() {
            break;
        }

        for callback_name in callback_name_list.clone().unwrap() {
            if props.is_object()
                && props
                .dyn_ref::<Object>()
                .unwrap()
                .has_own_property(&callback_name.into())
            {
                let callback = derive_from_js_value(props, callback_name);
                Reflect::set(&element_event_props, &callback_name.into(), &callback);
            }
        }
    }
    Reflect::set(&node, &ELEMENT_EVENT_PROPS_KEY.into(), &element_event_props);

    node
}
