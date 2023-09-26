// use crate::VGA_INSTANCE;
// use core::fmt::Write;

#[repr(C)]
#[repr(packed)]
pub struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_mid: u8,
    access: u8,
    flags_limit_high: u8,
    base_high: u8,
}

impl Default for GdtEntry {
    fn default() -> Self {
        GdtEntry {
            limit_low: 0,
            base_low: 0,
            base_mid: 0,
            access: 0,
            flags_limit_high: 0,
            base_high: 0,
        }
    }
}

impl Clone for GdtEntry 
{
    fn clone(&self) -> Self {
        GdtEntry {
            limit_low: self.limit_low,
            base_low: self.base_low,
            base_mid: self.base_mid,
            access: self.access,
            flags_limit_high: self.flags_limit_high,
            base_high: self.base_high,
        }
    }
}


impl Copy for GdtEntry {}

const NENTRIES : usize = 6;


static mut GDT : [GdtEntry; NENTRIES] = [
    GdtEntry { limit_low: 0, base_low: 0, base_mid: 0, access: 0, flags_limit_high: 0, base_high: 0, }; NENTRIES];

#[repr(C)]
#[repr(packed)]
struct Gdtr {
    size : u16, // size in bytes - 1
    offset : u32, // offset of GDT (linear address, paging applies)
}

static mut GDTR : Gdtr = Gdtr {size : 0, offset : 0}; // pointer to the gdt table

pub fn format_entry(base: u32, limit : u32, access_byte: u8, flags: u8) -> GdtEntry
{
    let mut entry = GdtEntry::default();
    entry.base_low = (base & 0xffff) as u16;
    entry.base_mid = ((base >> 16) & 0xff) as u8;
    entry.base_high = ((base >> 24) & 0xff) as u8;
    entry.limit_low = (limit & 0xffff) as u16;
    entry.flags_limit_high = (((limit >> 16) & 0xf) as u8) | (flags & 0xf0);
    entry.access = access_byte;
    entry
}

extern "C" {
    fn load_gdt(gdtr: *const Gdtr);
}

pub fn load()
{
    // setup basic segments
    unsafe {
        // GDT[0] = format_entry(0, 0, 0, 0);
        // GDT[1] = format_entry(0, 0xffffffff, 0x9a, 0xc);
        // GDT[2] = format_entry(0, 0xffffffff, 0x92, 0xc);
        // GDT[3] = format_entry(0, 0xffffffff, 0xfa, 0xc);
        // GDT[4] = format_entry(0, 0xffffffff, 0xf2, 0xc);
        // // GDT[5] = format_entry(0, 0xffffffff, 0x89, 0x0);// TODO tss entry
        // GDTR.size = ((GDT.len() * core::mem::size_of::<GdtEntry>()) - 1) as u16;
        // GDTR.offset = &GDT as *const _ as u32;
        // // write!(VGA_INSTANCE.as_mut().unwrap(), "GDTR size : {:x}, offset : {:x}\n", GDTR.size, GDTR.offset);
        // // print gdt address
        // write!(VGA_INSTANCE.as_mut().unwrap(), "GDT address : {:x}\n", &GDT as *const _ as u32).unwrap();
        load_gdt(&GDTR);
    }
    // unsafe {
    //     write!(VGA_INSTANCE.as_mut().unwrap(), "GDT poitner : {:x}\n", &GDT as *const _ as u32).unwrap();
    // }
    
}


