
use crate::{io, kprint};
use crate::arch::pci::PCIDevice;
use crate::fs::block;
use crate::klog;
use crate::memory::vmm::mapper;


#[repr(C, packed)]
#[derive(Default, Copy, Clone)]
pub struct PRD  {
    phys_addr: u32,
    bytes_count: u16,
    msb: u16
}

use alloc::vec::Vec;

pub struct IDEDev {
    // TODO allocate in DMA zone ideally
    pio_reg_base: [u16; 2],
    pio_control_base: [u16; 2],
    active_bus: usize,
    dma_reg_base: [u16; 2],
}

impl block::BlockDriver for IDEDev {

    fn read_block(&self, lba: usize) {
    }

}

#[allow(dead_code)]
mod ata_macros {
    // ATA PIO regs offsets
    pub const ATA_REG_DATA: u16 = 0x00;
    pub const ATA_REG_ERROR: u16 = 0x01;
    pub const ATA_REG_FEATURES: u16 = 0x01;
    pub const ATA_REG_SECCOUNT0: u16 = 0x02;
    pub const ATA_REG_LBA0: u16 = 0x03;
    pub const ATA_REG_LBA1: u16 = 0x04;
    pub const ATA_REG_LBA2: u16 = 0x05;
    pub const ATA_REG_HDDEVSEL: u16 = 0x06;
    pub const ATA_REG_COMMAND: u16 = 0x07;
    pub const ATA_REG_STATUS: u16 = 0x07;
    // pub const ATA_REG_SECCOUNT1: u16 = 0x08;
    // pub const ATA_REG_LBA3: u16 = 0x09;
    // pub const ATA_REG_LBA4: u16 = 0x0A;
    // pub const ATA_REG_LBA5: u16 = 0x0B;
    // pub const ATA_REG_CONTROL: u16 = 0x0C;
    // pub const ATA_REG_ALTSTATUS: u16 = 0x0C;
    // pub const ATA_REG_DEVADDRESS: u16 = 0x0D;

    // ATA DMA regs offsets
    pub const DMA_COMMAND_PORT: u16 = 0x0;
    pub const DMA_STATUS_PORT: u16 = 0x2;
    pub const DMA_PRDT_BASE: u16 = 0x4;

    // ATA COMMANDS
    pub const ATA_CMD_READ_PIO: u8 = 0x20;
    pub const ATA_CMD_READ_PIO_EXT: u8 = 0x24;
    pub const ATA_CMD_READ_DMA: u8 = 0xC8;
    pub const ATA_CMD_READ_DMA_EXT: u8 = 0x25;
    pub const ATA_CMD_WRITE_PIO: u8 = 0x30;
    pub const ATA_CMD_WRITE_PIO_EXT: u8 = 0x34;
    pub const ATA_CMD_WRITE_DMA: u8 = 0xCA;
    pub const ATA_CMD_WRITE_DMA_EXT: u8 = 0x35;
    pub const ATA_CMD_CACHE_FLUSH: u8 = 0xE7;
    pub const ATA_CMD_CACHE_FLUSH_EXT: u8 = 0xEA;
    pub const ATA_CMD_PACKET: u8 = 0xA0;
    pub const ATA_CMD_IDENTIFY_PACKET: u8 = 0xA1;
    pub const ATA_CMD_IDENTIFY: u8 = 0xEC;


    // Identify index into 256 buffer
    pub const ATA_IDENT_DEVICETYPE: usize = 0;
    pub const ATA_IDENT_CYLINDERS: usize = 2;
    pub const ATA_IDENT_HEADS: usize = 6;
    pub const ATA_IDENT_SECTORS: usize = 12;
    pub const ATA_IDENT_SERIAL: usize = 20;
    pub const ATA_IDENT_MODEL: usize = 54;
    pub const ATA_IDENT_CAPABILITIES: usize = 98;
    pub const ATA_IDENT_FIELDVALID: usize = 106;
    pub const ATA_IDENT_MAX_LBA: usize = 120;
    pub const ATA_IDENT_COMMANDSETS: usize = 164;
    pub const ATA_IDENT_MAX_LBA_EXT: usize = 200;
}
use alloc::string::String;
use ata_macros::*;

use alloc::boxed::Box;

#[repr(align(4))]
struct AlignedArray {
    array : [PRD; 512]
}

// TODO Put into DMA address space, that's gross right now
static mut PRDT: AlignedArray  = AlignedArray{ array: [PRD {phys_addr: 0, bytes_count: 0, msb: 1 << 15}; 512]};

static mut BUFFER: [u8;512] = [0 as u8; 512];

#[allow(dead_code)]
impl IDEDev {

    // TODO only managing primary but I don't I care enough for the secondary

    pub fn new() -> Self {
        IDEDev {
            // TODO put behind lock
            pio_reg_base: [0; 2],
            pio_control_base: [0; 2],
            dma_reg_base: [0; 2],
            active_bus: 0,
        }
    }

    fn read_pio_reg(&self, port_offset: u16) -> u8 {
        io::inb(self.pio_reg_base[self.active_bus] + port_offset)
    }

    fn write_pio_reg(&self, port_offset: u16, data: u8) {
        io::outb(self.pio_reg_base[self.active_bus] + port_offset, data);
    }

    fn read_dma_reg(&self, port_offset: u16) -> u8 {

        io::inb(self.dma_reg_base[self.active_bus] + port_offset)
    }

    fn write_dma_reg(&self, port_offset: u16, data: u8) {
        io::outb(self.dma_reg_base[self.active_bus] + port_offset, data);
    }

    // TODO clean up 
    fn sel_channel(&mut self, bus: usize, disk: u8) {
        self.active_bus = bus;
        self.write_pio_reg(ATA_REG_HDDEVSEL, disk);
        // Need to wait for a while
        for _ in 0..200000 {
            self.read_pio_reg(ATA_REG_STATUS);
        }
    }

    fn probe_disk(&mut self, bus: usize, disk: u8) {
        self.sel_channel(bus, disk);
        // Preparing to send IDENTIFY command
        self.write_pio_reg(ATA_REG_SECCOUNT0, 0);
        self.write_pio_reg(ATA_REG_LBA0, 0);
        self.write_pio_reg(ATA_REG_LBA1, 0);
        self.write_pio_reg(ATA_REG_LBA2, 0);
        self.write_pio_reg(ATA_REG_COMMAND, ATA_CMD_IDENTIFY);

        let mut status = self.read_pio_reg(ATA_REG_STATUS);
        if status == 0 {
            klog!("IDE: NO DRIVE ON bus {} with function {:x}", bus, disk);
            return
        }
        // Wait for BSY bit to be cleared
        while status & 0x80 != 0 {
            status = self.read_pio_reg(ATA_REG_STATUS);
        }
        if self.read_pio_reg(ATA_REG_LBA1) != 0 || self.read_pio_reg(ATA_REG_LBA2) != 0 {
            // Not an ATA drive
            return
        }
        // DRQ or ERR
        // TODO bitflags

        loop {
            status = self.read_pio_reg(ATA_REG_STATUS);
            if status & 0x1 != 0 {klog!("IDE ERROR"); return;}
            if status & 0x80 == 0 && status & 0x8 != 0  {break}
        }

        // Collect identify response
        let mut id_response: [u16; 256] = [0;256];
        for i in 0..256 {
            id_response[i] = io::inw(self.pio_reg_base[self.active_bus] + ATA_REG_DATA);
        }

        // Parse and build Disck Structure
        klog!("Parsing disk structure");
        let mut model = String::new();
        // check if DMA mode
        for i in 0..20 {
            let bytes = id_response[ATA_IDENT_MODEL/2 + i];
            model.push((bytes >> 8) as u8 as char);
            model.push(bytes as u8 as char);
        }
        klog!("Identifier: '{}'", model);
    }

    fn test_read_pio(&mut self) {
        self.write_pio_reg(ATA_REG_LBA0, 1);
        self.write_pio_reg(ATA_REG_LBA1, 0);
        self.write_pio_reg(ATA_REG_LBA2, 0);
        self.write_pio_reg(ATA_REG_SECCOUNT0, 2);
        self.write_pio_reg(ATA_REG_COMMAND, ATA_CMD_READ_PIO);
        // err = self.read_pio_reg(ATA_REG_STATUS);
        // klog!("ERR {:b} while on 0x{:x}", err, self.pio_reg_base[self.active_bus]);
        for _ in 0..20000 {
            self.read_pio_reg(ATA_REG_STATUS);
        }

        // let buffer = Box::new([0 as u8; 512]);
        for _ in 0..512 {
            kprint!("{}", self.read_pio_reg(ATA_REG_DATA) as char);
        }
        klog!("\nERR {:b}", self.read_pio_reg(ATA_REG_STATUS));

    }

    fn test_read_dma(&mut self) {

        self.sel_channel(1, 0xa0);
        // stop bus master
        let mut com = self.read_dma_reg(DMA_COMMAND_PORT);
        com &= !1;
        self.write_dma_reg(DMA_COMMAND_PORT, com);
        // Preparing PRDT
        // Writing DMA PRDT address
        let prdt_address;
        unsafe {
            klog!("PRDT Virtual  Address 0x{:x}", PRDT.array.as_ptr() as usize);
            prdt_address = mapper::virt_to_phys_kernel(PRDT.array.as_ptr() as usize).unwrap();
            klog!("PRDT Physical Address {:x}", prdt_address);
            PRDT.array[0] = PRD { phys_addr: mapper::virt_to_phys_kernel(BUFFER.as_ptr() as usize).unwrap() as u32, bytes_count: 512, msb: 1<<15 };
            self.write_dma_reg(DMA_PRDT_BASE, (prdt_address) as u8);
            self.write_dma_reg(DMA_PRDT_BASE + 1, (prdt_address >> 8) as u8);
            self.write_dma_reg(DMA_PRDT_BASE + 2, (prdt_address >> 16) as u8);
            self.write_dma_reg(DMA_PRDT_BASE + 3, (prdt_address >> 24) as u8);
        }

        // Set command to read
        let mut com = self.read_dma_reg(DMA_COMMAND_PORT);
        com |= 1 << 3;
        self.write_dma_reg(DMA_COMMAND_PORT, com);

        // Clear interrupt error/interrupt bits
        self.write_dma_reg(DMA_STATUS_PORT, 0b110);


        // select drive
        self.sel_channel(1, 0xa0);

        // klog!("Status reg {:b}", self.read_dma_reg(0x2));
        self.write_pio_reg(ATA_REG_LBA0, 1);
        self.write_pio_reg(ATA_REG_LBA1, 0);
        self.write_pio_reg(ATA_REG_LBA2, 0);
        self.write_pio_reg(ATA_REG_SECCOUNT0, 1);
        self.write_pio_reg(ATA_REG_COMMAND, ATA_CMD_READ_DMA);

        // start bus master
        let mut com = self.read_dma_reg(DMA_COMMAND_PORT);
        com |= 1;
        self.write_dma_reg(DMA_COMMAND_PORT, com);

        for _ in 0..2000000 {
            self.read_pio_reg(ATA_REG_STATUS);
        }


        let error = loop {
            let status = self.read_dma_reg(DMA_STATUS_PORT);

            if status & 1 << 1 != 0 {
                klog!("Error while DMA read, status {:b}", status);
                break true;
            }
            if status & 1 == 0 {
                break false;
            }

        };

        if error {
            klog!("Error while DMA read");
        }
        else {
            for i in 0..512 {
                unsafe {kprint!("{}", BUFFER[i]);}
            }
            klog!("\n END OF TRANSMISSION");
        }

    }

    pub fn probe_controller(pci_dev: &PCIDevice) -> Option<Box<IDEDev>> {
        klog!("Probing IDE controller");

        let caps = pci_dev.header.progif;

        klog!("IDE CAPS : {:b}", caps);
    // TODO hmmm I should probably return errors
        if caps & 1 != 0 || caps & 1 << 2 != 0 {
            panic!("IDE driver only supports compatibility mode");
        }
        if caps & 1 << 7 == 0 {
            panic!("IDE driver needs bus mastering (DMA) to function properly");
        }

        let mut dev: Box<IDEDev> = Box::new(IDEDev::new());


        // Check if there are drives connected

        // Compatibility mode defaults IO ports base
        // I put my trust in god almighty and the osdev wiki
        // Amen
        // Setting up the 2 channels with default value
        dev.pio_reg_base[0] = 0x1f0;
        dev.pio_control_base[0] = 0x3f6;
        dev.pio_reg_base[1] = 0x170;
        dev.pio_control_base[1] = 0x376;

        let source = pci_dev.get_bar(4);
        // klog!("BAR4 {:x} {:b}", source, source);
        let bar4 = (source & 0xFFFFFFFC) as u16;
        // klog!("BAR4 {:x} {:b}", bar4, bar4);
        // loop{}
        dev.dma_reg_base[0] = bar4;
        dev.dma_reg_base[1] = bar4 + 0x8;
        // channel 1 master/slave
        // dev.probe_disk(0, 0xA0);
        // dev.probe_disk(0, 0xB0);
        // // channel 2 master/slave
        // dev.probe_disk(1, 0xA0);
        // dev.probe_disk(1, 0xB0);
        dev.probe_disk(1, 0xA0);
        
        pci_dev.enable_busmaster();
        // dev.test_read_pio();
        dev.test_read_dma();

        Some(dev)
    }
}
