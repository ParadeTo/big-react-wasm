use web_sys::{Element, window};

use react_reconciler::host_config::HostConfig;

pub struct DomHostConfig;

impl HostConfig for DomHostConfig {
    fn create_instance(&self, _type: String) -> Element {
        let window = window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        match document.create_element(_type.as_ref()) {
            Ok(element) => element,
            Err(_) => todo!()
        }
    }

    fn append_initial_child(&self, parent: Element, child: Element) {
        match parent.append_child(&*child) {
            Ok(_) => {}
            Err(_) => todo!()
        }
    }
}
