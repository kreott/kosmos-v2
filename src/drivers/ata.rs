use x86_64::instructions::port::Port;
use crate::macros::*;

const ATA_PRIMARY_DATA: u16 = 0x1F0;
const ATA_PRIMARY_STATUS: u16 = 0x1F7;
const ATA_PRIMARY_COMMAND: u16 = 0x1F7;
const ATA_PRIMARY_DRIVE: u16 = 0x1F6;
const ATA_PRIMARY_SECCOUNT: u16 = 0x1F2;
const ATA_PRIMARY_LBA_LO: u16 = 0x1F3;
const ATA_PRIMARY_LBA_MID: u16 = 0x1F4;
const ATA_PRIMARY_LBA_HI: u16 = 0x1F5;

pub struct AtaDrive {
    data: Port<u16>,
    status: Port<u8>,
    command: Port<u8>,
    drive: Port<u8>,
    sector_count: Port<u8>,
    lba_lo: Port<u8>,
    lba_mid: Port<u8>,
    lba_hi: Port<u8>,
}

impl AtaDrive {
    pub fn new() -> Self {
        Self {
            data: Port::new(ATA_PRIMARY_DATA),
            status: Port::new(ATA_PRIMARY_STATUS),
            command: Port::new(ATA_PRIMARY_COMMAND),
            drive: Port::new(ATA_PRIMARY_DRIVE),
            sector_count: Port::new(ATA_PRIMARY_SECCOUNT),
            lba_lo: Port::new(ATA_PRIMARY_LBA_LO),
            lba_mid: Port::new(ATA_PRIMARY_LBA_MID),
            lba_hi: Port::new(ATA_PRIMARY_LBA_HI),
        }
    }

    pub fn read_sector(&mut self, lba: u32, buf: &mut [u8]) {
        unsafe {
            self.drive.write(0xF0 | ((lba >> 24) as u8 & 0x0F));
            self.sector_count.write(1);
            self.lba_lo.write(lba as u8);
            self.lba_mid.write((lba >> 8) as u8);
            self.lba_hi.write((lba >> 16) as u8);
            self.command.write(0x20);
    
            serial_println!("reading data");
            // read 256 u16s
            for i in 0..256 {
                let word = self.data.read();
                buf[i * 2] = word as u8;
                buf[i * 2 + 1] = (word >> 8) as u8;
            }

            serial_println!("done reading");
        }
    }

    pub fn test_port(&mut self) {
        unsafe {
            let status = self.status.read();
            serial_println!("ata status: {:x}", status);
        }
    }
}

