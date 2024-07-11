
use alloc::vec::Vec;

const ARRAY_REPEAT_VALUE : Vec<fn () -> Result<(),()>> = Vec::new();
static mut HANDLERS: [Vec<fn () -> Result<(),()>>; 256] = [ARRAY_REPEAT_VALUE; 256];


pub fn raise_irq(irq: u32) -> Result<(), ()> {
    if irq > 255 {
        return Err(())
    }
    unsafe {
        for h in HANDLERS[irq as usize].iter() {
            // TODO check error
            h();
        }
    }
    Ok(())
}

pub fn request_irq(irq_line: u32, handler: fn () -> Result<(),()>) -> Result<(),()> {
    if irq_line > 255 {
        return Err(())
    }
    let i = irq_line as usize;
    unsafe {
        HANDLERS[i].push(handler);
    }
    Ok(())
}


