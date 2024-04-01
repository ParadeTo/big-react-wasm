use std::any::Any;
use std::ops::Deref;

use web_sys::{Element, window};

use react_reconciler::host_config::HostConfig;

pub struct DomHostConfig;

impl HostConfig for DomHostConfig {
    fn create_instance(&self, _type: String) -> Box<dyn Any> {
        let window = window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        match document.create_element(_type.as_ref()) {
            Ok(element) => Box::new(element),
            Err(_) => todo!()
        }
    }

    fn append_initial_child(&self, parent: Box<dyn Any>, child: Box<dyn Any>) {
        let parent = parent.downcast::<Element>().unwrap();
        let child = child.downcast::<Element>().unwrap();
        match parent.append_child(child.deref()) {
            Ok(_) => {}
            Err(_) => todo!()
        }
    }
}


