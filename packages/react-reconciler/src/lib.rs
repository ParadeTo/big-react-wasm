mod utils;
mod fiber;
mod work_tags;

use wasm_bindgen::prelude::*;
use web_sys::{console};
use react::ReactElement;
use crate::fiber::{FiberNode, FiberRootNode};
use crate::work_tags::WorkTag;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}


pub fn create_container(container: &JsValue) -> FiberRootNode {
    let host_root_fiber = FiberNode::new(WorkTag::HostRoot);
    let root = FiberRootNode::new(Box::new(container.clone()));
    return root
}

pub fn update_container(element: ReactElement, root: &FiberRootNode) {

}

