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

    pub fn mkdir(&self, path: &str) -> FsResult<&mut INode> {

    }
}

fn split_path(path: &str) -> Vec<&str> {
    path.split('/').filter(|s| !s.is_empty()).collect()
}
