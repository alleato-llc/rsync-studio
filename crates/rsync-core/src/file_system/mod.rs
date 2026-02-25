pub mod real_file_system;

use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum FsError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("I/O error: {0}")]
    IoError(String),

    #[error("Not a directory: {0}")]
    NotADirectory(String),
}

pub trait FileSystem {
    fn exists(&self, path: &Path) -> bool;
    fn is_dir(&self, path: &Path) -> bool;
    fn is_file(&self, path: &Path) -> bool;
    fn is_symlink(&self, path: &Path) -> bool;

    fn create_dir_all(&self, path: &Path) -> Result<(), FsError>;
    fn remove_dir_all(&self, path: &Path) -> Result<(), FsError>;

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>, FsError>;
    fn read_to_string(&self, path: &Path) -> Result<String, FsError>;
    fn write(&self, path: &Path, content: &str) -> Result<(), FsError>;

    fn create_symlink(&self, original: &Path, link: &Path) -> Result<(), FsError>;
    fn read_link(&self, path: &Path) -> Result<PathBuf, FsError>;
    fn remove_symlink(&self, path: &Path) -> Result<(), FsError>;

    fn available_space(&self, path: &Path) -> Result<u64, FsError>;
    fn dir_size(&self, path: &Path) -> Result<u64, FsError>;

    fn copy_file(&self, from: &Path, to: &Path) -> Result<(), FsError>;
    fn hard_link(&self, original: &Path, link: &Path) -> Result<(), FsError>;
    fn walk_dir(&self, path: &Path) -> Result<Vec<PathBuf>, FsError>;

    fn filesystem_type(&self, path: &Path) -> Option<String>;
}
