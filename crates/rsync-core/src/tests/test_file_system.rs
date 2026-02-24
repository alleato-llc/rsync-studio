use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::file_system::{FileSystem, FsError};

#[derive(Debug, Clone)]
enum FsNode {
    File { content: String, inode: u64 },
    Directory,
    Symlink { target: PathBuf },
}

#[derive(Debug)]
struct Inner {
    nodes: HashMap<PathBuf, FsNode>,
    next_inode: u64,
    inode_paths: HashMap<u64, HashSet<PathBuf>>,
    available_space: u64,
}

impl Inner {
    fn new() -> Self {
        let mut nodes = HashMap::new();
        nodes.insert(PathBuf::from("/"), FsNode::Directory);
        Self {
            nodes,
            next_inode: 1,
            inode_paths: HashMap::new(),
            available_space: u64::MAX,
        }
    }

    fn allocate_inode(&mut self) -> u64 {
        let inode = self.next_inode;
        self.next_inode += 1;
        inode
    }

    fn ensure_parents(&mut self, path: &Path) {
        let mut current = PathBuf::new();
        for component in path.parent().unwrap_or(Path::new("/")).components() {
            current.push(component);
            if !self.nodes.contains_key(&current) {
                self.nodes.insert(current.clone(), FsNode::Directory);
            }
        }
    }

    fn register_inode_path(&mut self, inode: u64, path: PathBuf) {
        self.inode_paths
            .entry(inode)
            .or_default()
            .insert(path);
    }

    fn unregister_inode_path(&mut self, inode: u64, path: &Path) {
        if let Some(paths) = self.inode_paths.get_mut(&inode) {
            paths.remove(path);
            if paths.is_empty() {
                self.inode_paths.remove(&inode);
            }
        }
    }
}

pub struct TestFileSystem {
    inner: RefCell<Inner>,
}

impl TestFileSystem {
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(Inner::new()),
        }
    }

    pub fn with_dir(self, path: &str) -> Self {
        let mut inner = self.inner.borrow_mut();
        let path = PathBuf::from(path);
        inner.ensure_parents(&path);
        inner.nodes.insert(path, FsNode::Directory);
        drop(inner);
        self
    }

    pub fn with_file(self, path: &str, content: &str) -> Self {
        let mut inner = self.inner.borrow_mut();
        let path = PathBuf::from(path);
        inner.ensure_parents(&path);
        let inode = inner.allocate_inode();
        inner.register_inode_path(inode, path.clone());
        inner.nodes.insert(
            path,
            FsNode::File {
                content: content.to_string(),
                inode,
            },
        );
        drop(inner);
        self
    }

    pub fn with_available_space(self, bytes: u64) -> Self {
        self.inner.borrow_mut().available_space = bytes;
        self
    }

    pub fn files_under(&self, path: &str) -> Vec<PathBuf> {
        let inner = self.inner.borrow();
        let base = PathBuf::from(path);
        let mut result: Vec<PathBuf> = inner
            .nodes
            .iter()
            .filter(|(p, node)| {
                p.starts_with(&base) && *p != &base && matches!(node, FsNode::File { .. })
            })
            .map(|(p, _)| p.clone())
            .collect();
        result.sort();
        result
    }

    pub fn are_hard_linked(&self, path1: &str, path2: &str) -> bool {
        let inner = self.inner.borrow();
        let p1 = PathBuf::from(path1);
        let p2 = PathBuf::from(path2);

        match (inner.nodes.get(&p1), inner.nodes.get(&p2)) {
            (
                Some(FsNode::File { inode: i1, .. }),
                Some(FsNode::File { inode: i2, .. }),
            ) => i1 == i2,
            _ => false,
        }
    }

    pub fn symlink_target(&self, path: &str) -> Option<PathBuf> {
        let inner = self.inner.borrow();
        match inner.nodes.get(&PathBuf::from(path)) {
            Some(FsNode::Symlink { target }) => Some(target.clone()),
            _ => None,
        }
    }

    pub fn file_content(&self, path: &str) -> Option<String> {
        let inner = self.inner.borrow();
        match inner.nodes.get(&PathBuf::from(path)) {
            Some(FsNode::File { content, .. }) => Some(content.clone()),
            _ => None,
        }
    }
}

impl Default for TestFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystem for TestFileSystem {
    fn exists(&self, path: &Path) -> bool {
        self.inner.borrow().nodes.contains_key(path)
    }

    fn is_dir(&self, path: &Path) -> bool {
        matches!(self.inner.borrow().nodes.get(path), Some(FsNode::Directory))
    }

    fn is_file(&self, path: &Path) -> bool {
        matches!(self.inner.borrow().nodes.get(path), Some(FsNode::File { .. }))
    }

    fn is_symlink(&self, path: &Path) -> bool {
        matches!(self.inner.borrow().nodes.get(path), Some(FsNode::Symlink { .. }))
    }

    fn create_dir_all(&self, path: &Path) -> Result<(), FsError> {
        let mut inner = self.inner.borrow_mut();
        let mut current = PathBuf::new();
        for component in path.components() {
            current.push(component);
            if !inner.nodes.contains_key(&current) {
                inner.nodes.insert(current.clone(), FsNode::Directory);
            }
        }
        Ok(())
    }

    fn remove_dir_all(&self, path: &Path) -> Result<(), FsError> {
        let mut inner = self.inner.borrow_mut();
        if !inner.nodes.contains_key(path) {
            return Err(FsError::NotFound(path.display().to_string()));
        }

        let paths_to_remove: Vec<PathBuf> = inner
            .nodes
            .keys()
            .filter(|p| p.starts_with(path))
            .cloned()
            .collect();

        for p in paths_to_remove {
            if let Some(FsNode::File { inode, .. }) = inner.nodes.get(&p) {
                let inode = *inode;
                inner.unregister_inode_path(inode, &p);
            }
            inner.nodes.remove(&p);
        }
        Ok(())
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>, FsError> {
        let inner = self.inner.borrow();
        if !matches!(inner.nodes.get(path), Some(FsNode::Directory)) {
            return Err(FsError::NotADirectory(path.display().to_string()));
        }

        let mut entries: Vec<PathBuf> = inner
            .nodes
            .keys()
            .filter(|p| {
                if let Some(parent) = p.parent() {
                    parent == path && *p != path
                } else {
                    false
                }
            })
            .cloned()
            .collect();
        entries.sort();
        Ok(entries)
    }

    fn read_to_string(&self, path: &Path) -> Result<String, FsError> {
        let inner = self.inner.borrow();
        match inner.nodes.get(path) {
            Some(FsNode::File { content, .. }) => Ok(content.clone()),
            Some(_) => Err(FsError::IoError(format!(
                "{} is not a file",
                path.display()
            ))),
            None => Err(FsError::NotFound(path.display().to_string())),
        }
    }

    fn write(&self, path: &Path, content: &str) -> Result<(), FsError> {
        let mut inner = self.inner.borrow_mut();
        inner.ensure_parents(path);

        // If the file already exists, update content but keep inode
        if let Some(FsNode::File { inode, .. }) = inner.nodes.get(path) {
            let inode = *inode;
            inner.nodes.insert(
                path.to_path_buf(),
                FsNode::File {
                    content: content.to_string(),
                    inode,
                },
            );
        } else {
            let inode = inner.allocate_inode();
            inner.register_inode_path(inode, path.to_path_buf());
            inner.nodes.insert(
                path.to_path_buf(),
                FsNode::File {
                    content: content.to_string(),
                    inode,
                },
            );
        }
        Ok(())
    }

    fn create_symlink(&self, original: &Path, link: &Path) -> Result<(), FsError> {
        let mut inner = self.inner.borrow_mut();
        inner.ensure_parents(link);
        inner.nodes.insert(
            link.to_path_buf(),
            FsNode::Symlink {
                target: original.to_path_buf(),
            },
        );
        Ok(())
    }

    fn read_link(&self, path: &Path) -> Result<PathBuf, FsError> {
        let inner = self.inner.borrow();
        match inner.nodes.get(path) {
            Some(FsNode::Symlink { target }) => Ok(target.clone()),
            Some(_) => Err(FsError::IoError(format!(
                "{} is not a symlink",
                path.display()
            ))),
            None => Err(FsError::NotFound(path.display().to_string())),
        }
    }

    fn remove_symlink(&self, path: &Path) -> Result<(), FsError> {
        let mut inner = self.inner.borrow_mut();
        match inner.nodes.get(path) {
            Some(FsNode::Symlink { .. }) => {
                inner.nodes.remove(path);
                Ok(())
            }
            Some(_) => Err(FsError::IoError(format!(
                "{} is not a symlink",
                path.display()
            ))),
            None => Err(FsError::NotFound(path.display().to_string())),
        }
    }

    fn available_space(&self, _path: &Path) -> Result<u64, FsError> {
        Ok(self.inner.borrow().available_space)
    }

    fn dir_size(&self, path: &Path) -> Result<u64, FsError> {
        let inner = self.inner.borrow();
        if !matches!(inner.nodes.get(path), Some(FsNode::Directory)) {
            return Err(FsError::NotADirectory(path.display().to_string()));
        }

        let size: u64 = inner
            .nodes
            .iter()
            .filter(|(p, _)| p.starts_with(path) && *p != path)
            .filter_map(|(_, node)| match node {
                FsNode::File { content, .. } => Some(content.len() as u64),
                _ => None,
            })
            .sum();
        Ok(size)
    }

    fn copy_file(&self, from: &Path, to: &Path) -> Result<(), FsError> {
        let content = {
            let inner = self.inner.borrow();
            match inner.nodes.get(from) {
                Some(FsNode::File { content, .. }) => content.clone(),
                Some(_) => {
                    return Err(FsError::IoError(format!(
                        "{} is not a file",
                        from.display()
                    )))
                }
                None => return Err(FsError::NotFound(from.display().to_string())),
            }
        };

        let mut inner = self.inner.borrow_mut();
        inner.ensure_parents(to);
        let inode = inner.allocate_inode();
        inner.register_inode_path(inode, to.to_path_buf());
        inner
            .nodes
            .insert(to.to_path_buf(), FsNode::File { content, inode });
        Ok(())
    }

    fn hard_link(&self, original: &Path, link: &Path) -> Result<(), FsError> {
        let mut inner = self.inner.borrow_mut();
        let (content, inode) = match inner.nodes.get(original) {
            Some(FsNode::File { content, inode }) => (content.clone(), *inode),
            Some(_) => {
                return Err(FsError::IoError(format!(
                    "{} is not a file",
                    original.display()
                )))
            }
            None => return Err(FsError::NotFound(original.display().to_string())),
        };

        inner.ensure_parents(link);
        inner.register_inode_path(inode, link.to_path_buf());
        inner
            .nodes
            .insert(link.to_path_buf(), FsNode::File { content, inode });
        Ok(())
    }

    fn walk_dir(&self, path: &Path) -> Result<Vec<PathBuf>, FsError> {
        let inner = self.inner.borrow();
        if !matches!(inner.nodes.get(path), Some(FsNode::Directory)) {
            return Err(FsError::NotADirectory(path.display().to_string()));
        }

        let mut result: Vec<PathBuf> = inner
            .nodes
            .keys()
            .filter(|p| p.starts_with(path) && *p != path)
            .cloned()
            .collect();
        result.sort();
        Ok(result)
    }
}
