use crate::irq;
use crate::proc;


// TODO put under lock
static mut jiffies: u64 = 0;


pub fn do_timer() -> Result<(), ()>{
    // TODO remove unsafe
    unsafe {
        jiffies += 1;
    }
    proc::schedule::schedule();
    Ok(())
}


pub fn init() {

    irq::request_irq_top(50, do_timer);
}
