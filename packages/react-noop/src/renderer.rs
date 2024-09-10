use std::cell::RefCell;
use std::rc::Rc;

use shared::REACT_ELEMENT_TYPE;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::js_sys::Array;

use react_reconciler::fiber::FiberRootNode;
use react_reconciler::Reconciler;
use shared::{derive_from_js_value, to_string, type_of};
use web_sys::js_sys::Object;
use web_sys::js_sys::Reflect;

#[wasm_bindgen]
pub struct Renderer {
    container: JsValue,
    root: Rc<RefCell<FiberRootNode>>,
    reconciler: Reconciler,
}

impl Renderer {
    pub fn new(
        root: Rc<RefCell<FiberRootNode>>,
        reconciler: Reconciler,
        container: JsValue,
    ) -> Self {
        Self {
            root,
            reconciler,
            container,
        }
    }
}

fn child_to_jsx(child: JsValue) -> JsValue {
    if child.is_null() {
        return JsValue::null();
    }

    if type_of(&child, "string") || type_of(&child, "number") {
        return child.clone();
    }

    if child.is_array() {
        let child = child.dyn_ref::<Array>().unwrap();
        if child.length() == 0 {
            return JsValue::null();
        }

        if child.length() == 1 {
            return child_to_jsx(child.get(0));
        }

        let children: Array = child
            .iter()
            .map(|child_value| child_to_jsx(child_value))
            .collect::<Array>()
            .into();

        if children
            .iter()
            .all(|c| type_of(&child, "string") || type_of(&child, "number"))
        {
            let joined_children = children
                .iter()
                .map(|c| to_string(&c))
                .collect::<Vec<String>>()
                .join("");
            return JsValue::from_str(&joined_children);
        }

        return children.into();
    }

    let children = derive_from_js_value(&child, "children");
    if children.is_array() {
        let childrenChildren = child_to_jsx(children);
        let props = derive_from_js_value(&child, "props");
        if (!childrenChildren.is_null()) {
            Reflect::set(&props, &"children".into(), &childrenChildren);
        }

        let obj = Object::new();
        Reflect::set(&obj, &"$$typeof".into(), &REACT_ELEMENT_TYPE.into());
        Reflect::set(&obj, &"type".into(), &derive_from_js_value(&child, "type"));
        Reflect::set(&obj, &"key".into(), &JsValue::null());
        Reflect::set(&obj, &"ref".into(), &JsValue::null());
        Reflect::set(&obj, &"props".into(), &props);
        return obj.into();
    }

    derive_from_js_value(&child, "text")
}

#[wasm_bindgen]
impl Renderer {
    pub fn render(&self, element: &JsValue) -> JsValue {
        self.reconciler
            .update_container(element.clone(), self.root.clone())
    }

    pub fn getChildrenAsJSX(&self) -> JsValue {
        let mut children = derive_from_js_value(&self.container, "children");
        if children.is_undefined() {
            children = JsValue::null();
        }
        children = child_to_jsx(children);

        if children.is_null() {
            return JsValue::null();
        }
        if children.is_array() {
            todo!("Fragment")
        }
        return children;
    }
}
