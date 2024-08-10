use core::mem;

use crate::arch;
use crate::arch::context;
use crate::arch::context::Context;
use crate::irq::request_irq_top;
use crate::klib::lock::{RwLock, RwLockWriteGuard};
use crate::klog;

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

static TASKS: RwLock<Vec<Task>> = RwLock::new(Vec::new());
static mut GUARD: Option<RwLockWriteGuard<'static, Vec<Task>>> = None;
static mut CURRENT: usize = 0;
// static mut TASKS: Vec<Task>: 

#[no_mangle]
pub static mut CONTEXT_CHANGE: u32 = 0;

pub fn schedule() -> Result<(), ()> {
    // klog!("Shedule tick start");
    arch::disable_interrupts();
    unsafe {
        GUARD = Some(TASKS.write().unwrap());
        let tasks = GUARD.as_mut().unwrap();
        let prev = CURRENT;
        CURRENT += 1;
        if prev  == CURRENT {
            return Ok(())
        }
        if CURRENT == tasks.len() {
            CURRENT = 0;
        }
        let c2 = tasks[prev].context.clone();
        let c1 = &mut tasks[prev].context;
        context::switch(c1, c2);
    }
    Ok(())
}

pub extern "C" fn unlock_scheduler() {
    unsafe {
        match &GUARD {
            Some(guard) => {drop(guard); GUARD = None;},
            None => {panic!("This should not happen")},
        }
        arch::enable_interrupts();
    }
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
    // klog!("Eflags before: {:b}", eflags);
    eflags |= 0x200;
    // klog!("Eflags after : {:b}", eflags);

    // cont.push(eflags); // EFLAGS
    // cont.push(0x8); // CS
    // cont.push(entry_point as u32); // EIP

    cont.push(eflags); // EFLAGS
    cont.push(0x8); // CS
    cont.push(entry_point as u32); // EIP
    // klog!("ENTRYPOINT {:x}", entry_point as u32);

    cont.push(new_task_wrapper as u32); // Return address from context_switch
    
    let mut tasks = TASKS.write().unwrap();
    tasks.push(task);
}

fn idle_task() {
    loop{
    }
}

pub fn init() {
    request_irq_top(50, schedule);
    new_kernel_thread(idle_task);
}
