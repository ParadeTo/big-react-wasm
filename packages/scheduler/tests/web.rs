//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;

use wasm_bindgen_test::*;
use web_sys::js_sys::Function;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn pass() {
    // 使用假的 Function 实例，因为我们在这里不会真的调用它
    let fake_callback = Function::new_no_args("");

    let start_time = 0.0;
    // 添加任务到堆中
    push(Task::new(fake_callback.clone(), Priority::Normal, start_time, 1.0));
    push(Task::new(fake_callback.clone(), Priority::Normal, start_time, 2.0));
    push(Task::new(fake_callback, Priority::Normal, start_time, 3.0));

    // 按预期顺序弹出任务
    assert_eq!(TASK_QUEUE.pop().unwrap().id, 1);
    assert_eq!(TASK_QUEUE.pop().unwrap().id, 2);
    assert_eq!(TASK_QUEUE.pop().unwrap().id, 3);

    // 堆应该为空
    assert!(TASK_QUEUE.pop().is_none());
}
