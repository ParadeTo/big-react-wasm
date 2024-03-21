use wasm_bindgen::prelude::*;
use crate::work_tags::WorkTag;

pub struct FiberNode {
    tag: WorkTag
}

impl FiberNode {
    pub fn new(tag: WorkTag) -> Self {
        Self {tag}

    }
}

#[wasm_bindgen]
pub struct FiberRootNode {
    container: Box<JsValue>
}

impl FiberRootNode {
    pub fn new(container: Box<JsValue>) -> Self {
        Self {container}
    }
}