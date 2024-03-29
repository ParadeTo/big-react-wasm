use std::sync::{Mutex, MutexGuard, Once};

use once_cell::sync::OnceCell;
use web_sys::Element;

pub trait HostConfig {
    fn create_instance(&self, _type: String) -> Element;
    fn append_initial_child(&self, parent: Element, child: Element);
}

static INIT: Once = Once::new();

static HOST_CONFIG: OnceCell<Mutex<Box<dyn HostConfig + Send + Sync>>> = OnceCell::new();

pub fn init_host_config(renderer: Box<dyn HostConfig + Send + Sync>) {
    INIT.call_once(|| {
        let instance = Mutex::new(renderer);
        HOST_CONFIG.set(instance);
    });
}

pub fn get_host_config() -> MutexGuard<'static, Box<dyn HostConfig + Send + Sync>> {
    HOST_CONFIG.get().unwrap().lock().unwrap()
}
