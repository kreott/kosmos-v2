use alloc::vec::Vec;
use fatfs::{FileSystem, FsOptions};
use crate::macros::*;

static DISK_IMAGE: &[u8] = include_bytes!("../../fat32.img");

pub struct MemDisk {
    data: Vec<u8>,
    pos: usize,
}

impl MemDisk {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            pos: 0,
        }
    }
}

impl fatfs::IoBase for MemDisk {
    type Error = ();
}

impl fatfs::Read for MemDisk {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let len = buf.len().min(self.data.len() - self.pos);
        buf[..len].copy_from_slice(&self.data[self.pos..self.pos + len]);
        self.pos += len;
        Ok(len)
    }
}

impl fatfs::Seek for MemDisk {
    fn seek(&mut self, pos: fatfs::SeekFrom) -> Result<u64, Self::Error> {
        self.pos = match pos {
            fatfs::SeekFrom::Start(n) => n as usize,
            fatfs::SeekFrom::End(n) => (self.data.len() as i64 + n) as usize,
            fatfs::SeekFrom::Current(n) => (self.pos as i64 + n) as usize,
        };
        Ok(self.pos as u64)
    }
}

impl fatfs::Write for MemDisk {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let len = buf.len().min(self.data.len() - self.pos);
        self.data[self.pos..self.pos + len].copy_from_slice(&buf[..len]);
        self.pos += len;
        Ok(len)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub fn test_fat32() {
    let disk = MemDisk::new(DISK_IMAGE.to_vec());
    serial_println!("disk size: {}", disk.data.len());
    serial_println!("first bytes: {:x} {:x} {:x} {:x}", disk.data[0], disk.data[1], disk.data[2], disk.data[3]);
    let fs = FileSystem::new(disk, FsOptions::new()).unwrap();
    let root = fs.root_dir();
    for entry in root.iter() {
        let entry = entry.unwrap();
        serial_println!("{}", entry.file_name());
    }
}