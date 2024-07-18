
use alloc::vec::Vec;

const ARRAY_REPEAT_VALUE : Vec<fn () -> Result<(),()>> = Vec::new();
static mut TOP_HANDLERS: [Vec<fn () -> Result<(),()>>; 256] = [ARRAY_REPEAT_VALUE; 256];


pub fn top_handlers(irq: u32) -> Result<(), ()> {
    if irq > 255 {
        return Err(())
    }
    unsafe {
        for h in TOP_HANDLERS[irq as usize].iter() {
            // TODO check error
            h();
        }
    }
    Ok(())
}

pub fn request_irq_top(irq_line: u32, handler: fn () -> Result<(),()>) -> Result<(),()> {
    if irq_line > 255 {
        return Err(())
    }
    let i = irq_line as usize;
    unsafe {
        TOP_HANDLERS[i].push(handler);
    }
    Ok(())
}


