use std::any::Any;
use std::rc::Rc;

use web_sys::{Element, Text, window};

use react_reconciler::host_config::HostConfig;
use shared::log;

pub struct DomHostConfig;

impl HostConfig for DomHostConfig {
    fn create_text_instance(&self, content: String) -> Rc<dyn Any> {
        let window = window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        log!("create_text_instance - {:?}", content.as_str());
        Rc::new(document.create_text_node(content.as_str()))
    }

    fn create_instance(&self, _type: String) -> Rc<dyn Any> {
        let window = window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        match document.create_element(_type.as_ref()) {
            Ok(element) => Rc::new(element),
            Err(_) => todo!()
        }
    }

    fn append_initial_child(&self, parent: Rc<dyn Any>, child: Rc<dyn Any>) {
        let parent = parent.downcast::<Element>().unwrap();
        let child = child.downcast::<Text>().unwrap();

        match parent.append_child(&***child) {
            Ok(_) => {
                log!("append_initial_child successfully {:?} {:?}", parent, child);
            }
            Err(_) => todo!()
        }
    }

    fn append_child_to_container(&self, child: Rc<dyn Any>, parent: Rc<dyn Any>) {
        self.append_initial_child(parent, child)
    }
}


