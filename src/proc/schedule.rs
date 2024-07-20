use core::mem;

use crate::arch::context;
use crate::arch::context::Context;
use crate::irq::request_irq_top;
use crate::{klog, lock};

use alloc::vec::Vec;

#[derive(Default)]
pub struct  Task {
    pub context: Context,
}

impl Task {
    fn new() -> Self {
        Task {
            context: Context::default(),
        }
    }
}

// TODO put behind a lock
static mut TASKS: Vec<Task> = Vec::new();
static mut CURRENT: usize = 0;

// static mut TASKS: Vec<Task>: 

#[no_mangle]
pub static mut CONTEXT_CHANGE: u32 = 0;

pub fn schedule() -> Result<(), ()> {
    unsafe {
        let prev = CURRENT;
        CURRENT += 1;
        if prev  == CURRENT {
            return Ok(())
        }
        if CURRENT == TASKS.len() {
            CURRENT = 0;
        }
        context::switch(&mut TASKS[prev].context, &mut TASKS[CURRENT].context);
    }
    Ok(())
}


pub extern "C" fn unlock_scheduler() {

    // TODO proper unlocking
}

use core::arch::asm;
extern "C" fn new_task_wrapper() {
    unsafe {
        // loop{}
        asm!("iretd", options(noreturn));
    }
}

pub fn new_kernel_thread(entry_point: fn ()) {
    // Create a new stack for that thread
    let mut task = Task::new();
    // Push the handler's address onto the new stack
    let cont = &mut task.context;
    cont.init_stack();

    // building iret frame

    let mut eflags: u32;
    unsafe {
        asm!(
        "pushf",
        "mov eax, [esp]",
        "popf",
        out("eax") eflags
        );
    }
    klog!("Eflags before: {:b}", eflags);
    eflags |= 0x200;
    klog!("Eflags after : {:b}", eflags);

    // cont.push(eflags); // EFLAGS
    // cont.push(0x8); // CS
    // cont.push(entry_point as u32); // EIP

    cont.push(eflags); // EFLAGS
    cont.push(0x8); // CS
    cont.push(entry_point as u32); // EIP
    klog!("ENTRYPOINT {:x}", entry_point as u32);

    cont.push(new_task_wrapper as u32); // Return address from context_switch
    

    // TODO lock
    unsafe {
        TASKS.push(task);
    }
}

fn idle_task() {
    loop{}
}

pub fn init() {
    request_irq_top(50, schedule);
    new_kernel_thread(idle_task);
}
