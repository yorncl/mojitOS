// arch depedent code to interface with PCI devices config space
// main differences will probably be around IO port or memory mapped space

// This trait represents the inerface for all archs
#[allow(dead_code)]
mod x86 {

    use crate::arch::io;
    use crate::arch::io::port::{PCICONFIG_ADDRESS, PCICONFIG_DATA};

    pub enum BarType {
        // 4 bytes aligned
        IO(u32),
        // 16 bytes aligned
        MMIO(u32),
    }

    // R/W PCI configspace interface
    #[derive(Copy, Clone)]
    pub struct PCIEndpointConfig {
        // base address for registers built from bus number, dev number and fn number
        // TODO might make a separate type for better readability
        base: u32,
        // configspace fields
        pub vendor: u16,
        pub dev_id: u16,
        pub command: PCICommand,
        pub rev_id: u8,
        pub progif: u8,
        pub class: u16,
        pub cache_line_size: u8,
        pub latency_timer: u8,
        pub header_type: u8,
        pub cardbus: u32,
        pub sub_vendor: u16,
        pub sub_system: u16,
        pub rom_base: u32,
        pub caps: Option<Caps>,
        pub int_line: u8,
        pub int_pin: u8,
        pub min_grant: u8,
        pub max_latency: u8,
    }

    // ARCH specific io, move somewhere cleaner
    fn read_reg(base: u32, offset: u32) -> u32 {
        // discards 2 last bit as registers offsets have to be 4 byte aligned
        assert!(offset & 0b11 == 0);
        io::outl(PCICONFIG_ADDRESS, base | offset);
        io::inl(PCICONFIG_DATA)
    }

    fn write_reg(base: u32, offset: u32, data: u32) {
        // assure that write don't happen unaligned
        assert!(offset & 0b11 == 0);
        io::outl(PCICONFIG_ADDRESS, base | offset as u32);
        io::outl(PCICONFIG_DATA, data);
    }

    // Read the full 32 bit registers and extracts the field corresponding to the index
    macro_rules! read_field {
        ($base: expr, $offset: expr, $t: ty) => {
            (read_reg($base, ($offset as u32) & !0b11) >> ((($offset as u32) & 0b11) * 8)) as $t
        };
    }

    // TODO do a new pass on this module
    // My heart ache, having to pass the base address once again
    #[derive(Default, Copy, Clone)]
    pub struct PCICommand {
        base: u32,
        value: u16,
    }

    // TODO something something bitflags something something bloat
    // might make it a more general type for other fields if needed
    impl PCICommand {
        #[inline]
        pub fn get(&self) -> u16 {
            read_reg(self.base, 0x4) as u16
        }

        #[inline]
        pub fn setf(&mut self, flag: u32) {
            write_reg(self.base, 0x4, read_reg(self.base, 0x4) | flag);
        }

        #[inline]
        pub fn unsetf(&mut self, flag: u32) {
            write_reg(self.base, 0x4, read_reg(self.base, 0x4) & !flag);
        }
    }

    // Current capability, pointed by the base value for the io reg
    // TODO should I worry about memory mapping instead ?
    #[derive(Copy, Clone)]
    pub struct Caps {
        base: u32,
    }

    impl Caps {
        pub fn id(&self) -> u8 {
            read_field!(self.base, 0x0, u8)
        }
    }

    // Small iterator interface on the capabilities
    impl Iterator for Caps {
        type Item = Caps;

        fn next(&mut self) -> Option<Self::Item> {
            if self.base == 0 {
                return None;
            }
            self.base = read_field!(self.base, 0x1, u8) as u32;
            Some(*self)
        }
    }

    impl PCIEndpointConfig {
        pub fn from_io_space(base: u32) -> Self {
            // reg offset must be 0
            assert!(base & 0xff == 0);
            let mut caps = Caps { base: 0 };
            // Checking bit 4 of status
            let status = read_field!(base, 0x6, u16);
            // crate::klog!("YEEEEEEEEEEEEEEEEEEEEEEEEEEEEES : {:b}", status);
            if status & (1 << 5) != 0 {
                caps.base = read_field!(base, 0x34, u8) as u32;
            } else {
                Caps { base: 0 };
            }

            PCIEndpointConfig {
                base,
                vendor: read_field!(base, 0x0, u16),
                dev_id: read_field!(base, 0x2, u16),
                command: PCICommand {
                    base,
                    value: read_field!(base, 0x4, u16),
                },
                rev_id: read_field!(base, 0x8, u8),
                progif: read_field!(base, 0x9, u8),
                class: read_field!(base, 0xA, u16),
                cache_line_size: read_field!(base, 0xC, u8),
                latency_timer: read_field!(base, 0xD, u8),
                header_type: read_field!(base, 0xE, u8),
                cardbus: read_field!(base, 0x28, u32),
                sub_vendor: read_field!(base, 0x2C, u16),
                sub_system: read_field!(base, 0x2E, u16),
                rom_base: read_field!(base, 0x30, u32),
                caps: Some(caps),
                int_line: read_field!(base, 0x3C, u8),
                int_pin: read_field!(base, 0x3D, u8),
                min_grant: read_field!(base, 0x3E, u8),
                max_latency: read_field!(base, 0x3F, u8),
            }
        }

        pub fn status(&self) -> u16 {
            read_field!(self.base, 0x6, u16)
        }

        // TODO thought I needed this now, probably will need it later
        pub fn setup_msi(&self) {
            for c in self.caps.into_iter() {
                crate::klog!("Caps base: 0x{:x}, id: {:x}", c.base, c.id());
            }
            loop {}
        }

        // TODO
        // fn get_bist() {}
        // fn set_bist() {}

        pub fn get_bar(&self, i: u32) -> BarType {
            let val = read_reg(self.base, 0x10 + i * 4);
            return if val & 0x1 != 0 {
                BarType::IO(val & !0b11)
            } else {
                BarType::MMIO(val & !0b111)
            };
        }

        pub fn get_bar_raw(&self, i: u32) -> u32 {
            read_reg(self.base, 0x10 + i * 4)
        }
    }
}
#[cfg(target_arch = "x86")]
pub use x86::*;
