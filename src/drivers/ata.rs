use x86_64::instructions::port::Port;
use crate::macros::*;
use lazy_static::lazy_static;
use spin::Mutex;


#[derive(Debug, Clone, Copy)]
pub enum AtaBus {
    Primary,
    Secondary,
}

#[derive(Debug, Clone, Copy)]
pub enum AtaUnit {
    Master,
    Slave,
}

pub struct AtaDrive {
    data: Port<u16>,
    status: Port<u8>,
    alt_status: Port<u8>,
    command: Port<u8>,
    drive: Port<u8>,
    sector_count: Port<u8>,
    lba_lo: Port<u8>,
    lba_mid: Port<u8>,
    lba_hi: Port<u8>,
    drive_select: u8,
}

impl AtaDrive {
    pub fn new_with(bus: AtaBus, unit: AtaUnit) -> Self {
        let base = match bus {
            AtaBus::Primary   => 0x1F0u16,
            AtaBus::Secondary => 0x170u16,
        };
        let alt = match bus {
            AtaBus::Primary   => 0x3F6u16,
            AtaBus::Secondary => 0x376u16,
        };
        let drive_select = match unit {
            AtaUnit::Master => 0xE0u8,
            AtaUnit::Slave  => 0xF0u8,
        };
        Self {
            data: Port::new(base),
            status: Port::new(base + 7),
            alt_status: Port::new(alt),
            command: Port::new(base + 7),
            drive: Port::new(base + 6),
            sector_count: Port::new(base + 2),
            lba_lo: Port::new(base + 3),
            lba_mid: Port::new(base + 4),
            lba_hi: Port::new(base + 5),
            drive_select,
        }
    }

    fn is_fat_volume(&mut self) -> bool {
        let mut buf = [0u8; 512];
        self.read_sector(0, &mut buf);
        
        // boot signature
        if buf[510] != 0x55 || buf[511] != 0xAA {
            return false;
        }
    
        // bytes per sector should be 512
        let bytes_per_sector = u16::from_le_bytes([buf[11], buf[12]]);
        if bytes_per_sector != 512 {
            return false;
        }
    
        // FAT32 signature at offset 66
        if buf[66] != 0x28 && buf[66] != 0x29 {
            return false;
        }
    
        true
    }

    pub fn scan() -> Option<Self> {
        let locations = [
            (AtaBus::Primary, AtaUnit::Master),
            (AtaBus::Primary, AtaUnit::Slave),
            (AtaBus::Secondary, AtaUnit::Master),
            (AtaBus::Secondary, AtaUnit::Slave),
        ];
        for (bus, unit) in locations {
            let mut drive = Self::new_with(bus, unit);
            if drive.detect() && drive.is_fat_volume() {
                serial_println!("ata: found fat volume on {:?} {:?}", bus, unit);
                return Some(drive);
            }
        }
        serial_println!("ata: no fat volume found");
        None
    }

    fn delay(&mut self) {
        unsafe {
            for _ in 0..4 { self.alt_status.read(); }
        }
    }

    fn wait_ready(&mut self) {
        unsafe {
            loop {
                let status = self.status.read();
                if status & 0x80 != 0 { continue; }
                if status & 0x01 != 0 { panic!("ata error"); }
                if status & 0x08 != 0 { break; }
            }
        }
    }

    pub fn detect(&mut self) -> bool {
        unsafe {
            self.drive.write(self.drive_select);
            self.delay();
            let status = self.status.read();
            status != 0xFF && status != 0x00
        }
    }

    pub fn read_sector(&mut self, lba: u32, buf: &mut [u8; 512]) {
        unsafe {
            self.drive.write(self.drive_select | ((lba >> 24) as u8 & 0x0F));
            loop {
                let status = self.status.read();
                if status & 0x80 == 0 { break; }
            }
            self.sector_count.write(1);
            self.lba_lo.write(lba as u8);
            self.lba_mid.write((lba >> 8) as u8);
            self.lba_hi.write((lba >> 16) as u8);
            self.command.write(0x20);
            self.wait_ready();
            for i in 0..256 {
                let word = self.data.read();
                buf[i * 2]     = word as u8;
                buf[i * 2 + 1] = (word >> 8) as u8;
            }
        }
    }

    pub fn write_sector(&mut self, lba: u32, buf: &[u8; 512]) {
        unsafe {
            self.drive.write(self.drive_select | ((lba >> 24) as u8 & 0x0F));
            self.sector_count.write(1);
            self.lba_lo.write(lba as u8);
            self.lba_mid.write((lba >> 8) as u8);
            self.lba_hi.write((lba >> 16) as u8);
            self.command.write(0x30);
            self.wait_ready();
            for i in 0..256 {
                let word = (buf[i * 2] as u16) | ((buf[i * 2 + 1] as u16) << 8);
                self.data.write(word);
            }
            self.command.write(0xE7);
        }
    }
}

// public interface

lazy_static! {
    pub static ref ATA_DRIVE: Mutex<Option<AtaDrive>> = Mutex::new(None);
}

pub fn init() {
    *ATA_DRIVE.lock() = AtaDrive::scan();
}