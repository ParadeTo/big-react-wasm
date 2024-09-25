use std::cmp::{Ordering, PartialEq};

use shared::log;
use wasm_bindgen::prelude::*;
use web_sys::js_sys::{global, Function};
use web_sys::{MessageChannel, MessagePort};

use crate::heap::{peek, peek_mut, pop, push};

mod heap;

static FRAME_YIELD_MS: f64 = 5.0;
static mut TASK_ID_COUNTER: u32 = 1;
static mut TASK_QUEUE: Vec<Task> = vec![];
static mut TIMER_QUEUE: Vec<Task> = vec![];
static mut IS_HOST_TIMEOUT_SCHEDULED: bool = false;
static mut IS_HOST_CALLBACK_SCHEDULED: bool = false;
static mut IS_PERFORMING_WORK: bool = false;
static mut TASK_TIMEOUT_ID: f64 = -1.0;
static mut SCHEDULED_HOST_CALLBACK: Option<fn(bool, f64) -> bool> = None;
static mut IS_MESSAGE_LOOP_RUNNING: bool = false;
static mut MESSAGE_CHANNEL: Option<MessageChannel> = None;
// static mut MESSAGE_CHANNEL_LISTENED: bool = false;
static mut START_TIME: f64 = -1.0;
static mut CURRENT_PRIORITY_LEVEL: Priority = Priority::NormalPriority;
static mut CURRENT_TASK: Option<&Task> = None;
static mut PORT1: Option<MessagePort> = None;
static mut PORT2: Option<MessagePort> = None;

#[derive(Clone, Debug)]
#[wasm_bindgen]
pub enum Priority {
    ImmediatePriority = 1,
    UserBlockingPriority = 2,
    NormalPriority = 3,
    LowPriority = 4,
    IdlePriority = 5,
}

#[wasm_bindgen]
extern "C" {
    type Performance;
    type Global;
    #[wasm_bindgen(static_method_of = Performance, catch, js_namespace = performance, js_name = now)]
    fn now() -> Result<f64, JsValue>;
    #[wasm_bindgen]
    fn clearTimeout(id: f64);
    #[wasm_bindgen]
    fn setTimeout(closure: &Function, timeout: f64) -> f64;
    #[wasm_bindgen(js_namespace = Date, js_name = now)]
    fn date_now() -> f64;

    #[wasm_bindgen]
    fn setImmediate(f: &Function);

    #[wasm_bindgen(method, getter, js_name = setImmediate)]
    fn hasSetImmediate(this: &Global) -> JsValue;
}

#[derive(Clone, Debug)]
pub struct Task {
    pub id: u32,
    callback: JsValue,
    priority_level: Priority,
    start_time: f64,
    expiration_time: f64,
    sort_index: f64,
}

impl Task {
    fn new(
        callback: Function,
        priority_level: Priority,
        start_time: f64,
        expiration_time: f64,
    ) -> Self {
        unsafe {
            let s = Self {
                id: TASK_ID_COUNTER,
                callback: JsValue::from(callback),
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

impl Eq for Task {}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.id.cmp(&other.id) == Ordering::Equal
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let mut sort_index_ordering;

        if self.sort_index.is_nan() {
            if other.sort_index.is_nan() {
                sort_index_ordering = Ordering::Equal
            } else {
                sort_index_ordering = Ordering::Less
            }
        } else if other.sort_index.is_nan() {
            sort_index_ordering = (Ordering::Greater)
        } else {
            sort_index_ordering = self.sort_index.partial_cmp(&other.sort_index).unwrap()
        }

        if sort_index_ordering != Ordering::Equal {
            return Some(sort_index_ordering);
        }
        return self.id.partial_cmp(&other.id);
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

fn unstable_now() -> f64 {
    Performance::now().unwrap_or_else(|_| date_now())
}

fn get_priority_timeout(priority_level: Priority) -> f64 {
    match priority_level {
        Priority::NormalPriority => 5000.0,
        Priority::ImmediatePriority => -1.0,
        Priority::UserBlockingPriority => 250.0,
        Priority::IdlePriority => 1073741823.0,
        Priority::LowPriority => 10000.0,
    }
}

fn cancel_host_timeout() {
    unsafe {
        clearTimeout(TASK_TIMEOUT_ID);
        TASK_TIMEOUT_ID = -1.0;
    }
}

pub fn schedule_perform_work_until_deadline() {
    let perform_work_closure =
        Closure::wrap(Box::new(perform_work_until_deadline) as Box<dyn FnMut()>);
    let perform_work_function = perform_work_closure
        .as_ref()
        .unchecked_ref::<Function>()
        .clone();
    // let schedule_closure = Closure::wrap(Box::new(schedule_perform_work_until_deadline) as Box<dyn FnMut()>);

    if global()
        .unchecked_into::<Global>()
        .hasSetImmediate()
        .is_function()
    {
        setImmediate(&perform_work_function);
    } else if let Ok(message_channel) = MessageChannel::new() {
        unsafe {
            if PORT1.is_none() {
                PORT1 = Some(message_channel.port1());
                PORT2 = Some(message_channel.port2())
            }
            PORT1
                .as_ref()
                .unwrap()
                .set_onmessage(Some(&perform_work_function));
            PORT2
                .as_ref()
                .unwrap()
                .post_message(&JsValue::null())
                .expect("port post message panic");
        }
    } else {
        setTimeout(&perform_work_function, 0.0);
    }

    perform_work_closure.forget();
}

fn perform_work_until_deadline() {
    unsafe {
        if SCHEDULED_HOST_CALLBACK.is_some() {
            let scheduled_host_callback = SCHEDULED_HOST_CALLBACK.unwrap();
            let current_time = unstable_now();

            START_TIME = current_time;
            let has_time_remaining = true;
            let has_more_work = scheduled_host_callback(has_time_remaining, current_time);
            if has_more_work {
                schedule_perform_work_until_deadline();
            } else {
                IS_MESSAGE_LOOP_RUNNING = false;
                SCHEDULED_HOST_CALLBACK = None;
            }
        } else {
            IS_MESSAGE_LOOP_RUNNING = false
        }
    }
}

/**
static mut MY_V: Vec<Box<Task>> = vec![];

#[derive(Debug)]
struct Task {
    id: f64,
}

fn peek<'a>(v: &'a mut Vec<Box<Task>>) -> &'a Box<Task> {
    &v[0]
}

fn pop<'a>(v: &'a mut Vec<Box<Task>>) -> Box<Task> {
    let t = v.swap_remove(0);
    t
}

fn main() {
    unsafe {
        MY_V = vec![Box::new(Task {
            id: 10000.0
        })];

        let t = peek(&mut MY_V);

        println!("{:?}", t);

        pop(&mut MY_V);
        // let a = pop(&mut MY_V);

        println!("{:?}", t);
    };
}

 */
fn advance_timers(current_time: f64) {
    unsafe {
        let mut timer = peek_mut(&mut TIMER_QUEUE);
        while timer.is_some() {
            let task = timer.unwrap();
            if task.callback.is_null() {
                pop(&mut TIMER_QUEUE);
            } else if task.start_time <= current_time {
                let t = pop(&mut TIMER_QUEUE);
                task.sort_index = task.expiration_time;
                push(&mut TASK_QUEUE, task.clone());
            } else {
                return;
            }
            timer = peek_mut(&mut TIMER_QUEUE);
        }
    }
}

fn flush_work(has_time_remaining: bool, initial_time: f64) -> bool {
    unsafe {
        IS_HOST_CALLBACK_SCHEDULED = false;
        if IS_HOST_TIMEOUT_SCHEDULED {
            IS_HOST_TIMEOUT_SCHEDULED = false;
            cancel_host_timeout();
        }

        IS_PERFORMING_WORK = true;
        let previous_priority_level = CURRENT_PRIORITY_LEVEL.clone();

        let has_more = work_loop(has_time_remaining, initial_time).unwrap();
        //     .unwrap_or_else(|_| {
        //     log!("work_loop error");
        //     false
        // });

        CURRENT_TASK = None;
        CURRENT_PRIORITY_LEVEL = previous_priority_level.clone();
        IS_PERFORMING_WORK = false;

        return has_more;
    }
}

pub fn unstable_should_yield_to_host() -> bool {
    unsafe {
        let time_elapsed = unstable_now() - START_TIME;
        if time_elapsed < FRAME_YIELD_MS {
            return false;
        }
    }
    return true;
}

pub fn unstable_run_with_priority(priority_level: Priority, event_handler: &Function) {
    let previous_priority_level = unsafe { CURRENT_PRIORITY_LEVEL.clone() };
    unsafe { CURRENT_PRIORITY_LEVEL = priority_level.clone() };

    event_handler.call0(&JsValue::null());
    unsafe { CURRENT_PRIORITY_LEVEL = previous_priority_level.clone() };
}

fn work_loop(has_time_remaining: bool, initial_time: f64) -> Result<bool, JsValue> {
    unsafe {
        let mut current_time = initial_time;
        advance_timers(current_time);
        let mut current_task = peek_mut(&mut TASK_QUEUE);

        CURRENT_TASK = peek(&mut TASK_QUEUE);
        while current_task.is_some() {
            let mut t = current_task.unwrap();

            if t.expiration_time > current_time
                && (!has_time_remaining || unstable_should_yield_to_host())
            {
                break;
            }

            let callback = t.callback.clone();
            if callback.is_function() {
                t.callback = JsValue::null();
                CURRENT_PRIORITY_LEVEL = t.priority_level.clone();
                let did_user_callback_timeout = t.expiration_time <= current_time;
                let continuation_callback = callback
                    .dyn_ref::<Function>()
                    .unwrap()
                    .call1(&JsValue::null(), &JsValue::from(did_user_callback_timeout))?;
                current_time = unstable_now();

                if continuation_callback.is_function() {
                    t.callback = continuation_callback;
                } else {
                    if match peek(&TASK_QUEUE) {
                        None => false,
                        Some(task) => task == t,
                    } {
                        pop(&mut TASK_QUEUE);
                    }
                }

                advance_timers(current_time);
            } else {
                pop(&mut TASK_QUEUE);
            }

            current_task = peek_mut(&mut TASK_QUEUE);
            CURRENT_TASK = peek(&TASK_QUEUE);
        }

        if CURRENT_TASK.is_some() {
            return Ok(true);
        } else {
            let first_timer = peek(&mut TIMER_QUEUE);
            if first_timer.is_some() {
                let task = first_timer.unwrap();

                request_host_timeout(handle_timeout, task.start_time - current_time);
            }

            return Ok(false);
        }
    }
}

fn request_host_callback(callback: fn(bool, f64) -> bool) {
    unsafe {
        SCHEDULED_HOST_CALLBACK = Some(callback);
        if !IS_MESSAGE_LOOP_RUNNING {
            IS_MESSAGE_LOOP_RUNNING = true;
            schedule_perform_work_until_deadline();
        }
    }
}

fn handle_timeout(current_time: f64) {
    unsafe {
        IS_HOST_TIMEOUT_SCHEDULED = false;
        advance_timers(current_time);

        if !IS_HOST_TIMEOUT_SCHEDULED {
            if peek(&mut TASK_QUEUE).is_some() {
                IS_HOST_CALLBACK_SCHEDULED = true;
                request_host_callback(flush_work);
            } else {
                let first_timer = peek(&mut TIMER_QUEUE);
                if first_timer.is_some() {
                    let first_timer_task = first_timer.unwrap();
                    request_host_timeout(
                        handle_timeout,
                        first_timer_task.start_time - current_time,
                    );
                }
            }
        }
    }
}

fn request_host_timeout(callback: fn(f64), ms: f64) {
    unsafe {
        let closure = Closure::wrap(Box::new(move || {
            callback(unstable_now());
        }) as Box<dyn Fn()>);
        let function = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        TASK_TIMEOUT_ID = setTimeout(&function, ms);
    }
}

pub fn unstable_cancel_callback(task: Task) {
    let id = task.id;
    unsafe {
        for mut task in &mut TASK_QUEUE {
            if task.id == id {
                task.callback = JsValue::null();
            }
        }

        for mut task in &mut TIMER_QUEUE {
            if task.id == id {
                task.callback = JsValue::null();
            }
        }
    }
}

pub fn unstable_schedule_callback(
    priority_level: Priority,
    callback: Function,
    delay: f64,
) -> Task {
    let current_time = unstable_now();
    let mut start_time = current_time;

    if delay > 0.0 {
        start_time += delay;
    }

    let timeout = get_priority_timeout(priority_level.clone());
    let expiration_time = start_time + timeout;
    let mut new_task = Task::new(
        callback,
        priority_level.clone(),
        start_time,
        expiration_time,
    );
    let cloned = new_task.clone();
    unsafe {
        if start_time > current_time {
            new_task.sort_index = start_time;
            push(&mut TIMER_QUEUE, new_task.clone());

            if peek(&mut TASK_QUEUE).is_none() {
                if let Some(task) = peek(&mut TIMER_QUEUE) {
                    if task == &new_task {
                        if IS_HOST_TIMEOUT_SCHEDULED {
                            cancel_host_timeout();
                        } else {
                            IS_HOST_TIMEOUT_SCHEDULED = true;
                        }
                        request_host_timeout(handle_timeout, start_time - current_time);
                    }
                }
            }
        } else {
            new_task.sort_index = expiration_time;
            push(&mut TASK_QUEUE, new_task);

            if !IS_HOST_CALLBACK_SCHEDULED && !IS_PERFORMING_WORK {
                IS_HOST_CALLBACK_SCHEDULED = true;
                request_host_callback(flush_work);
            }
        }
    }

    cloned
}

pub fn unstable_schedule_callback_no_delay(priority_level: Priority, callback: Function) -> Task {
    unstable_schedule_callback(priority_level, callback, 0.0)
}

pub fn unstable_get_current_priority_level() -> Priority {
    unsafe { CURRENT_PRIORITY_LEVEL.clone() }
}
