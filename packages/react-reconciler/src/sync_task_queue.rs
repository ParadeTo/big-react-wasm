static mut SYNC_QUEUE: Vec<Box<dyn FnMut()>> = vec![];
static mut IS_FLUSHING_SYNC_QUEUE: bool = false;

pub fn schedule_sync_callback(callback: Box<dyn FnMut()>) {
    unsafe { SYNC_QUEUE.push(callback) }
}

pub fn flush_sync_callbacks() {
    unsafe {
        if !IS_FLUSHING_SYNC_QUEUE && !SYNC_QUEUE.is_empty() {
            IS_FLUSHING_SYNC_QUEUE = true;
            for callback in SYNC_QUEUE.iter_mut() {
                callback();
            }
            SYNC_QUEUE = vec![];
            IS_FLUSHING_SYNC_QUEUE = false;
        }
    }
}
