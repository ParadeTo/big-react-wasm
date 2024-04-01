use std::any::Any;
use std::sync::{Mutex, MutexGuard, Once};

use once_cell::sync::OnceCell;

pub trait HostConfig {
    fn create_instance(&self, _type: String) -> Box<dyn Any>;
    fn append_initial_child(&self, parent: Box<dyn Any>, child: Box<dyn Any>);
}

pub trait Ele {}

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
