use std::any::Any;
use std::rc::Rc;

use web_sys::{Node, window};

use react_reconciler::HostConfig;
use shared::log;

pub struct ReactDomHostConfig;

impl HostConfig for ReactDomHostConfig {
    fn create_text_instance(&self, content: String) -> Rc<dyn Any> {
        let window = window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        Rc::new(Node::from(document.create_text_node(content.as_str())))
    }

    fn create_instance(&self, _type: String) -> Rc<dyn Any> {
        let window = window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        match document.create_element(_type.as_ref()) {
            Ok(element) => Rc::new(Node::from(element)),
            Err(_) => todo!(),
        }
    }

    fn append_initial_child(&self, parent: Rc<dyn Any>, child: Rc<dyn Any>) {
        let p = parent.clone().downcast::<Node>().unwrap();
        let c = child.clone().downcast::<Node>().unwrap();
        match p.append_child(&c) {
            Ok(_) => {
                log!("append_initial_child successfully {:?} {:?}", p, c);
            }
            Err(_) => todo!(),
        }
    }

    fn append_child_to_container(&self, child: Rc<dyn Any>, parent: Rc<dyn Any>) {
        self.append_initial_child(parent, child)
    }

    fn remove_child(&self, child: Rc<dyn Any>, container: Rc<dyn Any>) {
        let p = container.clone().downcast::<Node>().unwrap();
        let c = child.clone().downcast::<Node>().unwrap();
        match p.remove_child(&c) {
            Ok(_) => {
                log!("remove_child successfully {:?} {:?}", p, c);
            }
            Err(_) => todo!(),
        }
    }

    fn commit_text_update(&self, text_instance: Rc<dyn Any>, content: String) {
        let text_instance = text_instance.clone().downcast::<Node>().unwrap();
        text_instance.set_node_value(Some(content.as_str()));
    }
}
