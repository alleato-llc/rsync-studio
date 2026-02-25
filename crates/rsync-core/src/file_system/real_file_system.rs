use std::ffi::CString;
use std::fs;
use std::path::{Path, PathBuf};

use super::{FileSystem, FsError};

pub struct RealFileSystem;

impl RealFileSystem {
    pub fn new() -> Self {
        Self
    }

    fn map_io_error(e: std::io::Error, path: &Path) -> FsError {
        match e.kind() {
            std::io::ErrorKind::NotFound => FsError::NotFound(path.display().to_string()),
            std::io::ErrorKind::PermissionDenied => {
                FsError::PermissionDenied(path.display().to_string())
            }
            _ => FsError::IoError(format!("{}: {}", path.display(), e)),
        }
    }
}

impl Default for RealFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystem for RealFileSystem {
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn is_symlink(&self, path: &Path) -> bool {
        path.symlink_metadata()
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)
    }

    fn create_dir_all(&self, path: &Path) -> Result<(), FsError> {
        fs::create_dir_all(path).map_err(|e| Self::map_io_error(e, path))
    }

    fn remove_dir_all(&self, path: &Path) -> Result<(), FsError> {
        fs::remove_dir_all(path).map_err(|e| Self::map_io_error(e, path))
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>, FsError> {
        if !path.is_dir() {
            return Err(FsError::NotADirectory(path.display().to_string()));
        }
        let entries: Vec<PathBuf> = fs::read_dir(path)
            .map_err(|e| Self::map_io_error(e, path))?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .collect();
        Ok(entries)
    }

    fn read_to_string(&self, path: &Path) -> Result<String, FsError> {
        fs::read_to_string(path).map_err(|e| Self::map_io_error(e, path))
    }

    fn write(&self, path: &Path, content: &str) -> Result<(), FsError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| Self::map_io_error(e, parent))?;
        }
        fs::write(path, content).map_err(|e| Self::map_io_error(e, path))
    }

    fn create_symlink(&self, original: &Path, link: &Path) -> Result<(), FsError> {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(original, link)
                .map_err(|e| Self::map_io_error(e, link))
        }
        #[cfg(not(unix))]
        {
            Err(FsError::IoError("Symlinks not supported on this platform".to_string()))
        }
    }

    fn read_link(&self, path: &Path) -> Result<PathBuf, FsError> {
        fs::read_link(path).map_err(|e| Self::map_io_error(e, path))
    }

    fn remove_symlink(&self, path: &Path) -> Result<(), FsError> {
        fs::remove_file(path).map_err(|e| Self::map_io_error(e, path))
    }

    fn available_space(&self, _path: &Path) -> Result<u64, FsError> {
        // A full implementation would use platform-specific APIs (statvfs on Unix)
        // For now, return a large default
        Ok(u64::MAX)
    }

    fn dir_size(&self, path: &Path) -> Result<u64, FsError> {
        if !path.is_dir() {
            return Err(FsError::NotADirectory(path.display().to_string()));
        }
        let mut total = 0u64;
        for entry in self.walk_dir(path)? {
            if entry.is_file() {
                total += fs::metadata(&entry)
                    .map(|m| m.len())
                    .unwrap_or(0);
            }
        }
        Ok(total)
    }

    fn copy_file(&self, from: &Path, to: &Path) -> Result<(), FsError> {
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).map_err(|e| Self::map_io_error(e, parent))?;
        }
        fs::copy(from, to)
            .map_err(|e| Self::map_io_error(e, from))?;
        Ok(())
    }

    fn hard_link(&self, original: &Path, link: &Path) -> Result<(), FsError> {
        if let Some(parent) = link.parent() {
            fs::create_dir_all(parent).map_err(|e| Self::map_io_error(e, parent))?;
        }
        fs::hard_link(original, link).map_err(|e| Self::map_io_error(e, original))
    }

    fn walk_dir(&self, path: &Path) -> Result<Vec<PathBuf>, FsError> {
        if !path.is_dir() {
            return Err(FsError::NotADirectory(path.display().to_string()));
        }

        let mut result = Vec::new();
        let mut stack = vec![path.to_path_buf()];

        while let Some(dir) = stack.pop() {
            for entry in fs::read_dir(&dir).map_err(|e| Self::map_io_error(e, &dir))? {
                let entry = entry.map_err(|e| FsError::IoError(e.to_string()))?;
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    stack.push(entry_path.clone());
                }
                result.push(entry_path);
            }
        }

        result.sort();
        Ok(result)
    }

    fn filesystem_type(&self, path: &Path) -> Option<String> {
        filesystem_type_impl(path)
    }
}

#[cfg(target_os = "macos")]
fn filesystem_type_impl(path: &Path) -> Option<String> {
    let c_path = CString::new(path.to_str()?).ok()?;
    unsafe {
        let mut stat: libc::statfs = std::mem::zeroed();
        if libc::statfs(c_path.as_ptr(), &mut stat) != 0 {
            return None;
        }
        let name_bytes: Vec<u8> = stat
            .f_fstypename
            .iter()
            .map(|&b| b as u8)
            .take_while(|&b| b != 0)
            .collect();
        String::from_utf8(name_bytes).ok()
    }
}

#[cfg(target_os = "linux")]
fn filesystem_type_impl(path: &Path) -> Option<String> {
    use std::io::BufRead;

    let canonical = path.canonicalize().ok()?;
    let canonical_str = canonical.to_str()?;

    let file = fs::File::open("/proc/mounts").ok()?;
    let reader = std::io::BufReader::new(file);

    let mut best_mount: Option<(usize, String)> = None;

    for line in reader.lines() {
        let line = line.ok()?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }
        let mount_point = parts[1];
        let fs_type = parts[2];

        if canonical_str.starts_with(mount_point) {
            let len = mount_point.len();
            if best_mount.as_ref().map_or(true, |(best_len, _)| len > *best_len) {
                best_mount = Some((len, fs_type.to_string()));
            }
        }
    }

    best_mount.map(|(_, fs_type)| fs_type)
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn filesystem_type_impl(_path: &Path) -> Option<String> {
    None
}
