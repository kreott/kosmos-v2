use crate::drivers::ata::{self, AtaDrive};
use lazy_static::lazy_static;
use spin::Mutex;

pub trait DiskBackend {
    fn read_sector(&mut self, lba: u32, buf: &mut [u8; 512]);
    fn write_sector(&mut self, lba: u32, buf: &[u8; 512]);
}

pub struct Disk<B: DiskBackend> {
    backend: B,
    pos: u64,
}

impl<B: DiskBackend> Disk<B> {
    pub fn new(backend: B) -> Self {
        Self { backend, pos: 0 }
    }
}

impl DiskBackend for AtaDrive {
    fn read_sector(&mut self, lba: u32, buf: &mut [u8; 512]) {
        self.read_sector(lba, buf);
    }

    fn write_sector(&mut self, lba: u32, buf: &[u8; 512]) {
        self.write_sector(lba, buf);
    }
}

impl DiskBackend for &mut AtaDrive {
    fn read_sector(&mut self, lba: u32, buf: &mut [u8; 512]) {
        AtaDrive::read_sector(self, lba, buf);
    }

    fn write_sector(&mut self, lba: u32, buf: &[u8; 512]) {
        AtaDrive::write_sector(self, lba, buf);
    }
}

/* 
OBSOLETE

impl DiskBackend for Vec<u8> {
    fn read_sector(&mut self, lba: u32, buf: &mut [u8; 512]) {
        let offset = lba as usize * 512;
        buf.copy_from_slice(&self[offset..offset + 512]);
    }

}
*/

impl<B: DiskBackend> fatfs::IoBase for Disk<B> {
    type Error = ();
}

impl<B: DiskBackend> fatfs::Read for Disk<B> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let mut sector = [0u8; 512];
        let lba = (self.pos / 512) as u32;
        let offset = (self.pos % 512) as usize;
        self.backend.read_sector(lba, &mut sector);
        let len = buf.len().min(512 - offset);
        buf[..len].copy_from_slice(&sector[offset..offset + len]);
        self.pos += len as u64;
        Ok(len)
    }
}

impl<B: DiskBackend> fatfs::Seek for Disk<B> {
    fn seek(&mut self, pos: fatfs::SeekFrom) -> Result<u64, Self::Error> {
        self.pos = match pos {
            fatfs::SeekFrom::Start(n) => n,
            fatfs::SeekFrom::Current(n) => (self.pos as i64 + n) as u64,
            fatfs::SeekFrom::End(_) => unimplemented!(),
        };
        Ok(self.pos)
    }
}

impl<B: DiskBackend> fatfs::Write for Disk<B> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
        let mut sector = [0u8; 512];
        let lba = (self.pos / 512) as u32;
        let offset = (self.pos % 512) as usize;
        
        // read the existing sector
        self.backend.read_sector(lba, &mut sector);
        
        // write the new data into it
        let len = buf.len().min(512 - offset);
        sector[offset..offset + len].copy_from_slice(&buf[..len]);
        
        // write it back
        self.backend.write_sector(lba, &sector);
        self.pos += len as u64;
        Ok(len)
    }

    fn flush(&mut self) -> Result<(), ()> {
        Ok(())
    }
}

// public interface
lazy_static! {
    static ref FILESYSTEM: Mutex<Option<fatfs::FileSystem<Disk<&'static mut AtaDrive>>>> = Mutex::new(None);
}

pub fn init_fs() {
    let mut drive_guard = ata::ATA_DRIVE.lock();
    if let Some(drive) = drive_guard.as_mut() {
        // never drop the drive while fs is alive
        let drive_ref: &'static mut AtaDrive = unsafe { &mut *(drive as *mut AtaDrive) };
        let disk = Disk::new(drive_ref);
        let fs = fatfs::FileSystem::new(disk, fatfs::FsOptions::new()).unwrap();
        *FILESYSTEM.lock() = Some(fs);
    }
}

/// Sets up a filesystem for usage
pub fn with_fs<F>(func: F)
where
    F: FnOnce(&fatfs::FileSystem<Disk<&'static mut AtaDrive>>)
{
    let guard = FILESYSTEM.lock();
    if let Some(fs) = guard.as_ref() {
        func(fs);
    }
}