use alloc::{vec::Vec, collections::VecDeque};

use crate::klog;


#[derive(Debug)]
pub enum InputEvent {
    Keyboard(u32)
}

// TODO thread safety
static mut QUEUE: VecDeque<InputEvent> = VecDeque::new();

pub fn push_event(ev: InputEvent) {
    unsafe {
        QUEUE.push_back(ev);
    }
}

// TODO implement limit ?
pub fn process_input_events() {
    unsafe {
        while !QUEUE.is_empty() {
            let v = QUEUE.pop_front().unwrap();
            klog!("{:?}", v);
        }
    }
}
