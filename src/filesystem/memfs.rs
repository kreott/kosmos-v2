//! previously used for testing, obsolete

use super::*;
use alloc::string::String;
use alloc::string::ToString;

pub enum INodeKind {
    File(Vec<u8>),
    Directory(BTreeMap<String, INode>),
}

pub struct INode {
    pub name: String,
    pub kind: INodeKind,
}

pub struct MemFs {
    root: INode,
    current_path: String,
}

impl MemFs {
    pub fn new() -> Self {
        Self {
            root: INode {
                name: "/".to_string(),
                kind: INodeKind::Directory(BTreeMap::new()),
            },
            current_path: "/".to_string(),
        }
    }


    fn get_node(&self, path: &str) -> FsResult<&INode> {
        let mut current = &self.root;
        let parts = split_path(path);
        for part in parts {
            match &current.kind {
                INodeKind::Directory(children) => {
                    current = children.get(part).ok_or(FsError::NotFound)?;
                }
                INodeKind::File(_) => return Err(FsError::NotADirectory),
            }
        }
        Ok(current)
    }

    fn get_node_mut(&mut self, path: &str) -> FsResult<&mut INode> {
        let mut current = &mut self.root;
        let parts = split_path(path);
        if let Some((last, parent_parts)) = parts.split_last() {
            for part in parent_parts {
                match &mut current.kind {
                    INodeKind::Directory(children) => {
                        current = children.get_mut(*part).ok_or(FsError::NotFound)?;
                    }
                    INodeKind::File(_) => return Err(FsError::NotADirectory),
                }
            }
            match &mut current.kind {
                INodeKind::Directory(children) => {
                    current = children.get_mut(*last).ok_or(FsError::NotFound)?;
                }
                INodeKind::File(_) => return Err(FsError::NotADirectory),
            }
        }
        Ok(current)
    }

    pub fn read(&self, path: &str) -> FsResult<Vec<u8>> {
        let node = self.get_node(path)?;
        match &node.kind {
            INodeKind::File(contents) => {
                Ok((*contents).clone())
            }
            INodeKind::Directory(_) => return Err(FsError::NotAFile),
        }
    }

    pub fn write(&mut self, path: &str, data: &[u8]) -> FsResult<()> {
        let node = self.get_node_mut(path)?;
        match &mut node.kind {
            INodeKind::File(contents) => {
                *contents = data.to_vec();
            }
            INodeKind::Directory(_) => return Err(FsError::NotAFile),
        }
        Ok(())
    }

    pub fn create(&mut self, path: &str) -> FsResult<()> {
        let parts = split_path(path);
        if let Some((last, parent_parts)) = parts.split_last() {
            let parent_path = "/".to_string() + &parent_parts.join("/");
            let parent = self.get_node_mut(&parent_path)?;
            match &mut parent.kind {
                INodeKind::Directory(dir) => {
                    let new_dir = INode { name: last.to_string(), kind: INodeKind::File(Vec::new())};
                    if dir.contains_key(*last) {
                        return Err(FsError::AlreadyExists);
                    }
                    dir.insert(last.to_string(), new_dir);
                }
                INodeKind::File(_) => return Err(FsError::NotADirectory),
            }
        }
        Ok(())
    }

    pub fn mkdir(&mut self, path: &str) -> FsResult<()> {
        let parts = split_path(path);
        if let Some((last, parent_parts)) = parts.split_last() {
            let parent_path = "/".to_string() + &parent_parts.join("/");
            let parent = self.get_node_mut(&parent_path)?;
            match &mut parent.kind {
                INodeKind::Directory(dir) => {
                    let new_dir = INode { name: last.to_string(), kind: INodeKind::Directory(BTreeMap::new())};
                    if dir.contains_key(*last) {
                        return Err(FsError::AlreadyExists);
                    }
                    dir.insert(last.to_string(), new_dir);
                }
                INodeKind::File(_) => return Err(FsError::NotADirectory),
            }
        };
        Ok(())
    }

    pub fn remove(&mut self, path: &str) -> FsResult<()> {
        let parts = split_path(path);
        if let Some((last, parent_parts)) = parts.split_last() {
            let parent_path = "/".to_string() + &parent_parts.join("/");
            let parent = self.get_node_mut(&parent_path)?;
            match &mut parent.kind {
                INodeKind::Directory(dir) => {
                    if dir.contains_key(*last) {
                        dir.remove(*last);
                    } else {
                        return Err(FsError::NotFound)
                    }
                }
                INodeKind::File(_) => return Err(FsError::NotADirectory),
            }
        };
        Ok(())
    }

    pub fn readdir(&self, path: &str) -> FsResult<Vec<String>> {
        let node= self.get_node(path)?;
        match &node.kind {
            INodeKind::Directory(dir) => {
                Ok(dir.keys().cloned().collect())
            }
            INodeKind::File(_) => return Err(FsError::NotADirectory),
        }
    }

    pub fn exists(&self, path: &str) -> bool {
        if self.get_node(path).is_ok() {
            true
        } else {
            false
        }
    }

    pub fn is_dir(&self, path: &str) -> bool {
        if let Ok(node) = self.get_node(path) {
            match &node.kind {
                INodeKind::Directory(_) => true,
                INodeKind::File(_) => false,
            }
        } else {
            false
        }
    }
}

impl FileSystem for MemFs {
    fn read(&self, path: &str) -> FsResult<Vec<u8>> {
        self.read(path)
    }
    fn write(&mut self, path: &str, data: &[u8]) -> FsResult<()> {
        self.write(path, data)
    }
    fn create(&mut self, path: &str) -> FsResult<()> {
        self.create(path)
    }
    fn mkdir(&mut self, path: &str) -> FsResult<()> {
        self.mkdir(path)
    }
    fn remove(&mut self, path: &str) -> FsResult<()> {
        self.remove(path)
    }
    fn readdir(&self, path: &str) -> FsResult<Vec<String>> {
        self.readdir(path)
    }
    fn exists(&self, path: &str) -> bool {
        self.exists(path)
    }
    fn is_dir(&self, path: &str) -> bool {
        self.is_dir(path)
    }
}

fn split_path(path: &str) -> Vec<&str> {
    path.split('/').filter(|s| !s.is_empty()).collect()
}
