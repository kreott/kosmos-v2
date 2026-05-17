use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

mod memfs;

#[derive(Debug)]
pub enum FsError {
    NotFound,
    NotADirectory,
    NotAFile,
    AlreadyExists,
    PermissionDenied,
}

pub type FsResult<T> = Result<T, FsError>;

pub trait FileSystem {
    fn read(&self, path: &str) -> FsResult<Vec<u8>>;
    fn write(&mut self, path: &str, data: &[u8]) -> FsResult<()>;
    fn create(&mut self, path: &str) -> FsResult<()>;
    fn mkdir(&mut self, path: &str) -> FsResult<()>;
    fn remove(&mut self, path: &str) -> FsResult<()>;
    fn readdir(&self, path: &str) -> FsResult<Vec<String>>;
    fn exists(&self, path: &str) -> bool;
    fn is_dir(&self, path: &str) -> bool;
}