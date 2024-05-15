use std::cmp::PartialEq;

use wasm_bindgen::prelude::*;
use web_sys::js_sys::Function;

use crate::heap::Comparable;

mod heap;

static FRAME_YIELD_MS: u32 = 5;
static mut TASK_ID_COUNTER: u32 = 1;
static mut TASK_QUEUE: Vec<Task> = vec![];

#[derive(Clone)]
enum Priority {
    Normal = 3
}

#[wasm_bindgen]
extern "C" {
    type Performance;

    #[wasm_bindgen(static_method_of = Performance, catch, js_namespace = performance, js_name = now)]
    fn now() -> Result<f64, JsValue>;

    #[wasm_bindgen(js_namespace = Date, js_name = now)]
    fn date_now() -> f64;
}

struct Task {
    id: u32,
    callback: Function,
    priority_level: Priority,
    start_time: f64,
    expiration_time: f64,
    sort_index: f64,
}

impl Task {
    fn new(callback: Function, priority_level: Priority, start_time: f64, expiration_time: f64) -> Self {
        unsafe {
            let s = Self {
                id: TASK_ID_COUNTER,
                callback,
                priority_level,
                start_time,
                expiration_time,
                sort_index: -1.0,
            };
            TASK_ID_COUNTER += TASK_ID_COUNTER;
            s
        }
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Task) -> bool {
        self.id == other.id
    }
}

// impl Comparable for Task {
//     fn compare(&self, b: &dyn Comparable) -> bool {
//         let diff = self.sort_index - b.sort_index;
//         if diff != 0.0 {
//             return diff < 0.0;
//         }
//         (self.id - b.id) < 0
//     }
// }


fn unstable_now() -> f64 {
    Performance::now().unwrap_or_else(|_| date_now())
}

fn get_priority_timeout(priority_level: Priority) -> f64 {
    match priority_level {
        Priority::Normal => 5000.0
    }
}

fn _unstable_schedule_callback(priority_level: Priority, callback: Function, delay: f64) {
    let current_time = unstable_now();
    let mut start_time = current_time;

    if delay > 0.0 {
        start_time += delay;
    }

    let timeout = get_priority_timeout(priority_level.clone());
    let expiration_time = start_time + timeout;
    let mut new_task = Task::new(callback, priority_level.clone(), start_time, expiration_time);

    if start_time > current_time {
        new_task.sort_index = start_time;
    }
}

fn unstable_schedule_callback_no_delay(priority_level: Priority, callback: Function) {
    _unstable_schedule_callback(priority_level, callback, 0.0)
}

