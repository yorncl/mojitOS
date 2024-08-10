use crate::driver::pci::{config::BarType, PCIDevice};
use crate::fs::block;
use crate::io::{Pio, PortIO};
use crate::klog;
use crate::klib::lock::RwLock;

use alloc::sync::Arc;
use alloc::boxed::Box;
use alloc::vec::Vec;

#[allow(dead_code)]
mod ata_macros {
    // ATA IO regs offsets
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

    // ATA Control reg offsets
    pub const ATA_REG_CONTROL: u16 = 0x0;
    pub const ATA_REG_ALTSTATUS: u16 = 0x0;
    pub const ATA_REG_DEVADDRESS: u16 = 0x1;

    // ATA DMA regs offsets
    pub const ATA_REG_DMA_COMMAND: u16 = 0x0;
    pub const ATA_REG_DMA_STATUS: u16 = 0x2;
    pub const ATA_REG_DMA_PRDT: u16 = 0x4;

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

// The 3 are always 1, the middle one is to enable LBA addressing
pub const IDE_DISK_MASTER: u8 = 0xE0; // 0b11100000
pub const IDE_DISK_SLAVE: u8 = 0xF0; //  0b11110000

#[repr(C, packed)]
#[derive(Default, Copy, Clone)]
pub struct PRD {
    phys_addr: u32,
    bytes_count: u16,
    msb: u16,
}

#[repr(align(4))]
struct AlignedArray {
    array: [PRD; 512],
}

pub struct IDEController {
    // only two bus supported, sorry ATA/IDE/PATA afficionados
    pub buses: [Arc<RwLock<Bus>>; 2],
}

// One ATA bus, used to interact with two drives
#[allow(dead_code)]
pub struct Bus {
    // IO reg
    pub data: Pio<u16>,
    pub error: Pio<u8>,
    pub features: Pio<u8>,
    pub seccount: Pio<u8>,
    pub lba0: Pio<u8>,
    pub lba1: Pio<u8>,
    pub lba2: Pio<u8>,
    drive_select: Pio<u8>,
    pub status: Pio<u8>,
    pub command: Pio<u8>,
    // Control regs
    pub altstatus: Pio<u8>,
    pub devctrl: Pio<u8>,
    pub driveaddrs: Pio<u8>,
    // DMA regs
    pub dma_command: Pio<u8>,
    pub dma_status: Pio<u8>,
    pub dma_prdt: [Pio<u8>; 4],
    // Remember if master or slave is active
    pub active_drivesel: u8,

    pub disks: RwLock<Vec<Arc<IDEDisk>>>,
}

pub struct IDEDiskInfo {
    pub model: [u8; 40],
    pub slot: u8,
    // 28bit addressable sectors
    pub seccount: u32,
}

pub struct IDEDisk {
    info: IDEDiskInfo,
    bus: Arc<RwLock<Bus>>,
}

impl block::BlockDriver for IDEDisk {
    fn read(&self, lba: usize, buffer: &mut [u8]) -> Result<(), ()> {

        let mut bus = self.bus.write().unwrap();

        bus.select_slot(self.info.slot);
        // TODO optimize to DMA more than 512 bytes at a time
        for chunk in buffer.chunks_mut(512) {
            bus.read_dma(lba.try_into().unwrap(), chunk)?;
        }
        Ok(())
    }

    fn sector_size(&self) -> usize {
        todo!()
    }
}

impl Bus {
    pub fn new(iobase: u16, controlbase: u16, dmabase: u16) -> Self {
        Bus {
            active_drivesel: 0,
            // PIO regs
            data: Pio::new(iobase + ATA_REG_DATA),
            error: Pio::new(iobase + ATA_REG_ERROR),
            features: Pio::new(iobase + ATA_REG_FEATURES),
            seccount: Pio::new(iobase + ATA_REG_SECCOUNT0),
            lba0: Pio::new(iobase + ATA_REG_LBA0),
            lba1: Pio::new(iobase + ATA_REG_LBA1),
            lba2: Pio::new(iobase + ATA_REG_LBA2),
            drive_select: Pio::new(iobase + ATA_REG_HDDEVSEL),
            status: Pio::new(iobase + ATA_REG_STATUS),
            command: Pio::new(iobase + ATA_REG_COMMAND),
            altstatus: Pio::new(controlbase + ATA_REG_ALTSTATUS),
            devctrl: Pio::new(controlbase + ATA_REG_CONTROL),
            driveaddrs: Pio::new(controlbase + ATA_REG_DEVADDRESS),
            dma_command: Pio::new(dmabase + ATA_REG_DMA_COMMAND),
            dma_status: Pio::new(dmabase + ATA_REG_DMA_STATUS),
            dma_prdt: [
                Pio::new(dmabase + ATA_REG_DMA_PRDT),
                Pio::new(dmabase + ATA_REG_DMA_PRDT + 1),
                Pio::new(dmabase + ATA_REG_DMA_PRDT + 2),
                Pio::new(dmabase + ATA_REG_DMA_PRDT + 3),
            ],
            disks: RwLock::new(vec![]),
        }
    }

    fn read_dma(&mut self, lba: u32, buffer: &mut [u8]) -> Result<(), ()> {
        if (lba >> 28) > 0 {
            // TODO better error managemetn
            return Err(())
        }
        let bus = &self;
        // stop bus master
        let mut com = bus.dma_command.read();
        com &= !1;
        bus.dma_command.write(com);
        // Preparing PRDT
        // Writing DMA PRDT address
        let prdt_address;
        unsafe {
            prdt_address = mapper::virt_to_phys_kernel(PRDT.array.as_ptr() as usize).unwrap();
            PRDT.array[0] = PRD {
                phys_addr: mapper::virt_to_phys_kernel(BUFFER.as_ptr() as usize).unwrap() as u32,
                bytes_count: 512,
                msb: 1 << 15,
            };
            bus.dma_prdt[0].write(prdt_address as u8);
            bus.dma_prdt[1].write((prdt_address >> 8) as u8);
            bus.dma_prdt[2].write((prdt_address >> 16) as u8);
            bus.dma_prdt[3].write((prdt_address >> 24) as u8);
        }

        // Set command to read
        let mut com = bus.dma_command.read();
        com |= 1 << 3;
        bus.dma_command.write(com);

        // Clear interrupt error/interrupt bits
        bus.dma_status.write(0b110);

        // Writing the 28 bit logical address
        // lowe 24 bits
        bus.lba0.write(lba as u8);
        bus.lba1.write((lba >> 8) as u8);
        bus.lba2.write((lba >> 16) as u8);

        bus.drive_select.write(bus.drive_select.read() | (lba >> 24) as u8);


        bus.seccount.write(1);
        bus.command.write(ATA_CMD_READ_DMA);

        // start bus master
        let mut com = bus.dma_command.read();
        com |= 1;
        bus.dma_command.write(com);

        self.poll();
        // TODO check for error
        // need to read it after each operation
        self.dma_status.read();

        // TODO Hmm move somewhere else
        let error = loop {
            let status = bus.dma_status.read();

            if status & 1 == 0 {
                break false;
            }
            if status & 1 << 1 != 0 {
                klog!("Error while DMA read, status {:b}", status);
                break true;
            }
        };
        if error {
            klog!("Error while DMA read");
            return Err(());
        }
        // TODO most certainly very much garbage code
        // Copy into before
        for i in 0..512 {
            unsafe { buffer[i] = BUFFER[i] }
        }
        Ok(())
    }

    fn poll(&self) {
        loop {
            let status = self.status.read();
            // Check if not busy anymore
            // TODO more extensive error checking
            if status & 0x80 == 0 {
                break;
            }
        }
    }

    pub fn select_slot(&mut self, id: u8) {
        self.active_drivesel = id;
        self.drive_select.write(id);
        self.wait();
    }

    // TODO should add some error checking probably
    fn wait(&self) {
        for _ in 0..2000 {
            let _ = self.status.read();
        }
    }

    // TODO maybe deeper error management
    pub fn probe(&mut self) -> Option<IDEDiskInfo> {
        // Preparing to send IDENTIFY command
        self.seccount.write(0);
        self.lba0.write(0);
        self.lba1.write(0);
        self.lba2.write(0);
        self.command.write(ATA_CMD_IDENTIFY);

        if self.status.read() == 0 {
            return None;
        }
        // Wait for BSY bit to be cleared
        while self.status.read() & 0x80 != 0 {
            continue;
        }
        if self.lba1.read() != 0 || self.lba2.read() != 0 {
            // Not an ATA drive
            return None;
        }
        // DRQ or ERR
        // TODO bitflags

        loop {
            let status = self.status.read();
            if status & 0x1 != 0 {
                klog!("IDE ERROR");
                return None;
            }
            if status & 0x80 == 0 && status & 0x8 != 0 {
                break;
            }
        }

        // Collect IDENTIFY response
        let mut id_response: [u16; 256] = [0; 256];
        for i in 0..256 {
            id_response[i] = self.data.read();
        }

        let mut diskinfo = IDEDiskInfo {
            model: [0; 40],
            slot: self.active_drivesel,
            seccount: (id_response[61] as u32) << 16 | id_response[60] as u32
        };

        // Parse and build Disck Structure
        for i in 0..20 {
            let bytes = id_response[ATA_IDENT_MODEL / 2 + i];
            diskinfo.model[i * 2] = (bytes >> 8) as u8;
            diskinfo.model[i * 2 + 1] = bytes as u8;
        }

        // Check if lba 28 bit addressing is supported
        if diskinfo.seccount == 0 {
            // TODO drop the drive instead
            panic!("Lba 28 mode not supported on IDE drive!");
        }
        Some(diskinfo)
    }
}

// TODO Put into DMA address space, that's gross right now
static mut PRDT: AlignedArray = AlignedArray {
    array: [PRD {
        phys_addr: 0,
        bytes_count: 0,
        msb: 1 << 15,
    }; 512],
};
static mut BUFFER: [u8; 512] = [0 as u8; 512];

use crate::memory::vmm::mapper;

#[allow(dead_code)]
impl IDEController {
    // TODO support PIO mode
    // fn test_read_pio(bus: &mut Bus) {
    //     bus.select_slot(0xA0);
    //     bus.lba0.write(1);
    //     bus.lba1.write(0);
    //     bus.lba2.write(0);
    //     bus.seccount.write(1);
    //     bus.command.write(ATA_CMD_READ_PIO);

    //     for _ in 0..20000 {
    //         bus.status.read();
    //     }

    //     let err = bus.status.read();
    //     klog!("status {:b}", err);
    //     if err & 0x1 != 0 {
    //         klog!("ERR {:b}", err);
    //         loop {}
    //     }

    //     for _ in 0..512 {
    //         crate::kprint!("{}", bus.data.read());
    //     }
    //     klog!("\nERR {:b}", bus.status.read());
    // }

    pub fn probe_controller(pci_dev: &mut PCIDevice) -> Option<Box<IDEController>> {
        // Extracting info from the PCI config
        let caps = pci_dev.config.progif;
        // TODO hmmm I should probably return errors
        if caps & 1 != 0 || caps & 1 << 2 != 0 {
            panic!("IDE driver only supports compatibility mode");
        }
        if caps & 1 << 7 == 0 {
            panic!("IDE driver needs bus mastering (DMA) to function properly");
        }
        let dma_base = match pci_dev.config.get_bar(4) {
            BarType::IO(val) => val as u16,
            BarType::MMIO(_val) => {
                panic!("Unsupported MMIO on PCI IDE controller")
            }
        };

        // Compatibility mode defaults IO ports base
        // Setting up the 2 buses with default value
        // I put my trust in god almighty and the osdev wiki
        // Amen
        let mut controller: Box<IDEController> = Box::new(IDEController {
            buses: [
                Arc::new(RwLock::new(Bus::new(0x1f0, 0x3f6, dma_base))),
                Arc::new(RwLock::new(Bus::new(0x170, 0x376, dma_base + 0x8))),
            ],
        });

        // Check if there are drives connected
        for bus_lock in controller.buses.iter_mut() {
            // Master
            let mut bus = bus_lock.write().unwrap();
            bus.select_slot(IDE_DISK_MASTER);

            if let Some(diskinfo) = bus.probe() {
                let mut disks = bus.disks.write().unwrap();
                disks.push(Arc::new(IDEDisk {
                    info: diskinfo,
                    bus: bus_lock.clone(),
                }));
            }

            // Slave
            bus.select_slot(IDE_DISK_SLAVE);
            if let Some(diskinfo) = bus.probe() {
                let mut disks = bus.disks.write().unwrap();
                disks.push(Arc::new(IDEDisk {
                    info: diskinfo,
                    bus: bus_lock.clone(),
                }));
            }
        }

        // enabling bustmatering
        pci_dev.config.command.setf(0x4);
        Some(controller)
    }
}
