use alloc::{collections::BTreeMap, string::String, vec::Vec};
use spin::Mutex;
use lazy_static::lazy_static;

pub struct FileDescriptor {
    pub data: Vec<u8>,
    pub offset: usize,
    pub is_dir: bool,
    pub path: String,
}

lazy_static! {
    pub static ref FD_TABLE: Mutex<BTreeMap<u64, FileDescriptor>> = Mutex::new(BTreeMap::new());
}

pub fn alloc_fd(fd: FileDescriptor) -> u64 {
    let mut table = FD_TABLE.lock();
    // start from 3, since 0=stdin 1=stdout 2=stderr
    let id = (3..).find(|id| !table.contains_key(id)).unwrap();
    table.insert(id, fd);
    id
}

pub fn free_fd(fd: u64) {
    FD_TABLE.lock().remove(&fd);
}

pub fn open(path: &str) -> Option<u64> {
    let mut data = Vec::new();
    let mut is_dir = false;
    let mut success = false;

    crate::filesystem::fat32::with_fs(|fs| {
        // try as file first
        match fs.root_dir().open_file(path.trim_start_matches('/')) {
            Ok(mut file) => {
                let mut buf = [0u8; 512];
                loop {
                    let n = fatfs::Read::read(&mut file, &mut buf).unwrap();
                    if n == 0 { break; }
                    data.extend_from_slice(&buf[..n]);
                }
                success = true;
            }
            Err(_) => {
                // try as dir
                if fs.root_dir().open_dir(path.trim_start_matches('/')).is_ok() {
                    is_dir = true;
                    success = true;
                }
            }
        }
    });

    if success {
        Some(alloc_fd(FileDescriptor {
            data,
            offset: 0,
            is_dir,
            path: path.into(),
        }))
    } else {
        None
    }
}

pub fn read(fd: u64, buf: &mut [u8]) -> isize {
    let mut table = FD_TABLE.lock();
    match table.get_mut(&fd) {
        None => -1,
        Some(file) => {
            let remaining = file.data.len() - file.offset;
            let len = buf.len().min(remaining);
            buf[..len].copy_from_slice(&file.data[file.offset..file.offset + len]);
            file.offset += len;
            len as isize
        }
    }
}

pub fn close(fd: u64) -> isize {
    if FD_TABLE.lock().remove(&fd).is_some() {
        0
    } else {
        -1
    }
}