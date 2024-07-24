use crate::io;
use crate::arch::pci::PCIDevice;
use crate::fs::block;
use crate::klog;


#[repr(C, packed)]
#[derive(Default, Copy, Clone)]
pub struct PRD  {
    phys_addr: u32,
    bytes_count: u16,
    msb: u16
}

#[derive(Default)]
pub struct IDEDev {
    // TODO allocate in DMA zone ideally
    prdt: Option<Box<[PRD;512]>>,
    prdt_tail: usize,
    pio_reg_base: [u16; 2],
    pio_control_base: [u16; 2],
    dma_reg: u16,
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
use ata_macros::*;

use alloc::boxed::Box;

#[allow(dead_code)]
impl IDEDev {

    // TODO only managing primary but I don't I care enough for the secondary

    pub fn new() -> Self {
        IDEDev {
            prdt: Some(Box::<[PRD;512]>::new([PRD::default(); 512])),
            prdt_tail: 0,
            pio_reg_base: [0; 2],
            pio_control_base: [0; 2],
            dma_reg: 0
        }
    }

    pub fn read_sector() {
        todo!()
    }

    pub fn write_sector() {
        // TODO should flush by ending 0xE7 command after write
        todo!()
    }

    fn read_pio_reg(&self, channel: usize, port_offset: u16) -> u8 {
        io::inb(self.pio_reg_base[channel as usize] + port_offset)
    }

    fn write_pio_reg(&self, channel: usize, port_offset: u16, data: u8) {
        io::outb(self.pio_reg_base[channel as usize] + port_offset, data);
    }

    // Might need to run that after a drive select
    fn poll_status(&self, channel: usize) {
        for _ in 0..30 {
            self.read_pio_reg(channel, ATA_REG_STATUS);
        }
    }

    fn probe_disk(&mut self, channel: usize, disk_function: u8) {
        // Select the drive
        self.write_pio_reg(channel,ATA_REG_HDDEVSEL, disk_function);
        self.poll_status(channel);

        // Preparing to send IDENTIFY command
        self.write_pio_reg(channel, ATA_REG_SECCOUNT0, 0);
        self.write_pio_reg(channel, ATA_REG_LBA0, 0);
        self.write_pio_reg(channel, ATA_REG_LBA1, 0);
        self.write_pio_reg(channel, ATA_REG_LBA2, 0);
        self.write_pio_reg(channel, ATA_REG_COMMAND, ATA_CMD_IDENTIFY);

        let mut status = self.read_pio_reg(channel, ATA_REG_STATUS);
        if status == 0 {
            klog!("IDE: NO DRIVE ON CHANNEL {} with function {:x}", channel, disk_function);
            return
        }
        // Wait for BSY bit to be cleared
        while status & 0x80 != 0 {
            status = self.read_pio_reg(channel, ATA_REG_STATUS);
        }
        if self.read_pio_reg(channel, ATA_REG_LBA1) != 0 || self.read_pio_reg(channel, ATA_REG_LBA2) != 0 {
            // Not an ATA drive
            return
        }
        // DRQ or ERR
        // TODO bitflags

        loop {
            status = self.read_pio_reg(channel, ATA_REG_STATUS);
            if status & 0x1 != 0 {klog!("IDE ERROR"); return;}
            if status & 0x80 == 0 && status & 0x8 != 0  {break}
        }

        // Collect identify response
        let mut id_response: [u8; 512] = [0;512];
        for i in 0..256 {
            let bytes = io::inw(ATA_REG_DATA);
            // klog!("Reading response bytes: {:x}", byteanics);
            id_response[i] = bytes as u8;
            id_response[i + 1] = (bytes >> 8) as u8;
        }
        klog!("IDE disk serial {:x}{:x}", id_response[ATA_IDENT_SERIAL + 1], id_response[ATA_IDENT_SERIAL]);
    }

    pub fn probe_controller(pci_dev: &PCIDevice) -> Option<Box<IDEDev>> {
        klog!("Probing IDE controller");

        let caps = pci_dev.h.progif;
    // TODO hmmm I should probably return errors
        if caps & 1 != 0 || caps & 1 << 2 != 0 {
            panic!("IDE driver only supports compatibility mode");
        }
        if caps & 1 << 7 == 0 {
            panic!("IDE driver needs bus mastering to function properly");
        }

        let mut dev: Box<IDEDev> = Box::new(IDEDev::default());

        // Check if there are drives connected

        // Compatibility mode defaults IO ports base
        // I put my trust in god almighty and the osdev wiki
        // Amen
        // Setting up the 2 channels with default value
        dev.pio_reg_base[0] = 0x1f0;
        dev.pio_control_base[0] = 0x3f6;
        dev.pio_reg_base[1] = 0x170;
        dev.pio_control_base[1] = 0x376;

        dev.dma_reg = pci_dev.get_bar(4).try_into().unwrap();

        // channel 1 master/slave
        dev.probe_disk(0, 0xA0);
        dev.probe_disk(0, 0xB0);
        // channel 2 master/slave
        dev.probe_disk(1, 0xA0);
        dev.probe_disk(1, 0xB0);

        Some(dev)
    }
}
