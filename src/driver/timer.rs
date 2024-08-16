use crate::irq;
use crate::proc;

static mut JIFFIES: u64 = 0;

#[allow(dead_code)]
pub fn do_timer() -> Result<(), ()>{
    // TODO remove unsafe
    unsafe {
        JIFFIES += 1;
    }
    // TODO error handling
    let _ = proc::schedule::schedule();
    Ok(())
}

#[allow(dead_code)]
pub fn get_jiffies() -> u64 {
    unsafe { JIFFIES }
}

#[allow(dead_code)]
pub fn init() {
    // TODO error handling
    let _ = irq::request_irq_top(50, do_timer);
}
