use std::any::Any;
use std::rc::Rc;

use web_sys::{Node, window};

use react_reconciler::HostConfig;
use shared::log;

pub struct ReactDomHostConfig;

impl HostConfig for ReactDomHostConfig {
    fn create_text_instance(&self, content: String) -> Rc<dyn Any> {
        match window() {
            None => {
                log!("no global `window` exists");
                Rc::new(())
            }
            Some(window) => {
                let document = window.document().expect("should have a document on window");
                Rc::new(Node::from(document.create_text_node(content.as_str())))
            }
        }
    }

    fn create_instance(&self, _type: String) -> Rc<dyn Any> {
        match window() {
            None => {
                log!("no global `window` exists");
                Rc::new(())
            }
            Some(window) => {
                let document = window.document().expect("should have a document on window");
                match document.create_element(_type.as_ref()) {
                    Ok(element) => Rc::new(Node::from(element)),
                    Err(_) => todo!(),
                }
            }
        }
    }

    fn append_initial_child(&self, parent: Rc<dyn Any>, child: Rc<dyn Any>) {
        let p = parent.clone().downcast::<Node>();
        let c = child.clone().downcast::<Node>();

        if p.is_err() || c.is_err() {
            return;
        }

        let p = p.unwrap();
        let c = c.unwrap();
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
}
