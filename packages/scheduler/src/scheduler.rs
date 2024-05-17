use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use web_sys::{MessageChannel, MessagePort};
use web_sys::js_sys::Function;

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
#[wasm_bindgen]
pub enum Priority {
    ImmediatePriority = 1,
    UserBlockingPriority = 2,
    NormalPriority = 3,
    LowPriority = 4,
    IdlePriority = 5,
}

static FRAME_YIELD_MS: f64 = 5.0;


#[derive(Clone, Debug)]
struct Task {
    id: u32,
    callback: JsValue,
    priority_level: Priority,
    start_time: f64,
    expiration_time: f64,
    sort_index: f64,
}

impl Task {
    fn new(
        id: u32,
        callback: Function,
        priority_level: Priority,
        start_time: f64,
        expiration_time: f64,
    ) -> Self {
        Self {
            id,
            callback: JsValue::from(callback),
            priority_level,
            start_time,
            expiration_time,
            sort_index: -1.0,
        }
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Task) -> bool {
        self.id == other.id
    }
}

struct Scheduler<'a> {
    task_id_counter: u32,
    task_queue: Vec<Task>,
    timer_queue: Vec<Task>,
    is_host_timeout_scheduled: bool,
    is_host_callback_scheduled: bool,
    is_performing_work: bool,
    task_timeout_id: f64,
    scheduled_host_callback: Option<fn(bool, f64) -> bool>,
    is_message_loop_running: bool,
    message_channel: Option<MessageChannel>,
    start_time: f64,
    current_priority_level: Priority,
    current_task: Option<&'a Task>,
    port1: Option<MessagePort>,
    port2: Option<MessagePort>,
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

impl Scheduler {
    fn cancel_host_timeout(&mut self) {
        clearTimeout(self.task_timeout_id);
        self.task_timeout_id = -1.0;
    }

    pub fn schedule_perform_work_until_deadline(&self) {
        let perform_work_closure =
            Closure::wrap(Box::new(self.perform_work_until_deadline) as Box<dyn FnMut()>);
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

    fn perform_work_until_deadline(&self) {
        // unsafe {
        //     if SCHEDULED_HOST_CALLBACK.is_some() {
        //         let scheduled_host_callback = SCHEDULED_HOST_CALLBACK.unwrap();
        //         let current_time = unstable_now();
        //
        //         START_TIME = current_time;
        //         let has_time_remaining = true;
        //         let has_more_work = scheduled_host_callback(has_time_remaining, current_time);
        //         if has_more_work {
        //             schedule_perform_work_until_deadline();
        //         } else {
        //             IS_MESSAGE_LOOP_RUNNING = false;
        //             SCHEDULED_HOST_CALLBACK = None;
        //         }
        //     } else {
        //         IS_MESSAGE_LOOP_RUNNING = false
        //     }
        // }
    }
    fn advance_timers(current_time: f64) {
        unsafe {
            let mut timer = peek(&mut TIMER_QUEUE);
            while timer.is_some() {
                let task = timer.unwrap().as_mut_any().downcast_mut::<Task>().unwrap();
                if task.callback.is_null() {
                    pop(&mut TIMER_QUEUE);
                } else if task.start_time <= current_time {
                    let t = pop(&mut TIMER_QUEUE);
                    task.sort_index = task.expiration_time;
                    push(&mut TASK_QUEUE, Box::new(task.clone()));
                } else {
                    return;
                }
                timer = peek(&mut TIMER_QUEUE);
            }
        }
    }

    fn flush_work(has_time_remaining: bool, initial_time: f64) -> bool {
        unsafe {
            IS_HOST_CALLBACK_SCHEDULED = false;
            if IS_HOST_TIMEOUT_SCHEDULED {
                log!("IS_HOST_TIMEOUT_SCHEDULED");
                IS_HOST_TIMEOUT_SCHEDULED = false;
                cancel_host_timeout();
            }

            IS_PERFORMING_WORK = true;
            let previous_priority_level = CURRENT_PRIORITY_LEVEL.clone();

            let has_more = work_loop(has_time_remaining, initial_time).unwrap_or_else(|_| {
                log!("work_loop error");
                false
            });

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

    fn work_loop(has_time_remaining: bool, initial_time: f64) -> Result<bool, JsValue> {
        unsafe {
            let mut current_time = initial_time;
            advance_timers(current_time);
            let mut current_task = peek(&mut TASK_QUEUE);
            log!(
            "current_task {:?}",
            current_task.as_ref()
                .unwrap()
                .as_any()
                .downcast_ref::<Task>()
                .unwrap()
        );

            CURRENT_TASK = peek(&mut TASK_QUEUE);
            while current_task.is_some() {
                let mut t = current_task
                    .unwrap()
                    .as_mut_any()
                    .downcast_mut::<Task>()
                    .unwrap();
                if t.expiration_time > current_time && (!has_time_remaining || unstable_should_yield_to_host()) {
                    break;
                }

                let callback = t.callback.clone();
                if callback.is_function() {
                    t.callback = JsValue::null();
                    // CURRENT_TASK = Some(&mut (Box::new(t.clone()) as Box<dyn Comparable>));
                    CURRENT_PRIORITY_LEVEL = t.priority_level.clone();
                    let did_user_callback_timeout = t.expiration_time <= current_time;
                    let continuation_callback = callback
                        .dyn_ref::<Function>()
                        .unwrap()
                        .call1(&JsValue::null(), &JsValue::from(did_user_callback_timeout))?;
                    current_time = unstable_now();

                    if continuation_callback.is_function() {
                        t.callback = continuation_callback;
                        // let mut boxed_t = Box::new(t.clone()) as Box<dyn Comparable>;
                        // CURRENT_TASK = Some(&mut boxed_t.clone());
                    } else {
                        if match peek(&mut TASK_QUEUE) {
                            None => false,
                            Some(task) => {
                                let task = task.as_any().downcast_ref::<Task>().unwrap();
                                log!("{:?} {:?} {:?}", task, t, task == t);
                                task == t
                            }
                        } {
                            pop(&mut TASK_QUEUE);
                        }
                        // if t == peek(&mut TASK_QUEUE) {
                        //     pop(&mut TASK_QUEUE);
                        // }
                    }

                    advance_timers(current_time);
                } else {
                    pop(&mut TASK_QUEUE);
                }

                current_task = peek(&mut TASK_QUEUE);
                CURRENT_TASK = peek(&mut TASK_QUEUE);
            }

            if CURRENT_TASK.is_some() {
                return Ok(true);
            } else {
                let first_timer = peek(&mut TIMER_QUEUE);
                log!("request_host_timeout");
                if first_timer.is_some() {
                    let task = first_timer
                        .unwrap()
                        .as_any()
                        .downcast_ref::<Task>()
                        .unwrap();
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
                log!("handle_timeout0 {:?}", TASK_QUEUE.len());
                if peek(&mut TASK_QUEUE).is_some() {
                    log!("handle_timeout1");
                    IS_HOST_CALLBACK_SCHEDULED = true;
                    request_host_callback(flush_work);
                } else {
                    log!("handle_timeout2");

                    let first_timer = peek(&mut TIMER_QUEUE);
                    if first_timer.is_some() {
                        let first_timer_task = first_timer
                            .unwrap()
                            .as_any()
                            .downcast_ref::<Task>()
                            .unwrap();
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

    pub fn unstable_cancel_callback(id: u32) {
        unsafe {
            for mut task in &mut TASK_QUEUE {
                let task = task.as_mut_any().downcast_mut::<Task>().unwrap();
                if task.id == id {
                    task.callback = JsValue::null();
                }
            }

            for mut task in &mut TIMER_QUEUE {
                let task = task.as_mut_any().downcast_mut::<Task>().unwrap();
                if task.id == id {
                    task.callback = JsValue::null();
                }
            }
        }
    }

    pub fn unstable_schedule_callback(&self, priority_level: Priority, callback: Function, delay: f64) -> u32 {
        let current_time = unstable_now();
        let mut start_time = current_time;
        log!("starttime {:?} {:?} {:?}", start_time, delay, start_time + delay);
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
        let id = new_task.id;
        unsafe {
            if start_time > current_time {
                new_task.sort_index = start_time;
                push(&mut TIMER_QUEUE, Box::new(new_task.clone()));

                if peek(&mut TASK_QUEUE).is_none() {
                    if let Some(task) = peek(&mut TIMER_QUEUE) {
                        let task = task.as_any().downcast_ref::<Task>().unwrap();
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
                push(&mut TASK_QUEUE, Box::new(new_task));

                if !IS_HOST_CALLBACK_SCHEDULED && !IS_PERFORMING_WORK {
                    IS_HOST_CALLBACK_SCHEDULED = true;
                    request_host_callback(flush_work);
                }
            }
        }

        id
    }

    pub fn unstable_schedule_callback_no_delay(&self, priority_level: Priority, callback: Function) -> u32 {
        self.unstable_schedule_callback(priority_level, callback, 0.0)
    }
}