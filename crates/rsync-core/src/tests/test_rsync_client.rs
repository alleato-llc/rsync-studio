use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use chrono::Utc;

use crate::tests::test_file_system::TestFileSystem;
use crate::traits::file_system::FileSystem;
use crate::traits::rsync_client::{RsyncClient, RsyncError, RsyncResult};

#[derive(Debug, Clone)]
pub struct RecordedCommand {
    pub args: Vec<String>,
    pub timestamp: chrono::DateTime<Utc>,
}

struct Inner {
    commands: Vec<RecordedCommand>,
    force_error: Option<RsyncError>,
}

pub struct TestRsyncClient {
    fs: Rc<TestFileSystem>,
    inner: RefCell<Inner>,
}

impl TestRsyncClient {
    pub fn new(fs: Rc<TestFileSystem>) -> Self {
        Self {
            fs,
            inner: RefCell::new(Inner {
                commands: Vec::new(),
                force_error: None,
            }),
        }
    }

    pub fn set_force_error(&self, error: Option<RsyncError>) {
        self.inner.borrow_mut().force_error = error;
    }

    pub fn recorded_commands(&self) -> Vec<RecordedCommand> {
        self.inner.borrow().commands.clone()
    }

    pub fn last_command(&self) -> Option<RecordedCommand> {
        self.inner.borrow().commands.last().cloned()
    }

    fn record_command(&self, args: &[String]) {
        self.inner.borrow_mut().commands.push(RecordedCommand {
            args: args.to_vec(),
            timestamp: Utc::now(),
        });
    }

    fn take_force_error(&self) -> Option<RsyncError> {
        self.inner.borrow_mut().force_error.take()
    }

    fn simulate_rsync(&self, args: &[String]) -> Result<RsyncResult, RsyncError> {
        let flags = ParsedFlags::from_args(args);

        if flags.dry_run {
            return Ok(RsyncResult {
                exit_code: 0,
                stdout: "DRY RUN: no changes made".to_string(),
                stderr: String::new(),
                command: format!("rsync {}", args.join(" ")),
            });
        }

        let source = &flags.source;
        let dest = &flags.destination;

        // Ensure destination directory exists
        self.fs
            .create_dir_all(Path::new(dest))
            .map_err(|e| RsyncError::IoError(e.to_string()))?;

        // Walk source files
        let source_path = Path::new(source);
        let source_files = if self.fs.is_dir(source_path) {
            self.fs
                .walk_dir(source_path)
                .map_err(|e| RsyncError::IoError(e.to_string()))?
                .into_iter()
                .filter(|p| self.fs.is_file(p))
                .collect::<Vec<_>>()
        } else if self.fs.is_file(source_path) {
            vec![source_path.to_path_buf()]
        } else {
            return Err(RsyncError::IoError(format!(
                "Source not found: {}",
                source
            )));
        };

        let mut files_transferred = 0u64;

        for src_file in &source_files {
            let relative = src_file
                .strip_prefix(source)
                .unwrap_or(src_file);

            // Check exclude patterns
            let rel_str = relative.to_string_lossy();
            if flags.exclude_patterns.iter().any(|pat| matches_glob(pat, &rel_str)) {
                continue;
            }

            let dest_file = PathBuf::from(dest).join(relative);

            let src_content = self
                .fs
                .read_to_string(src_file)
                .map_err(|e| RsyncError::IoError(e.to_string()))?;

            // Check if --backup and --backup-dir are set and dest file exists
            if flags.backup && flags.backup_dir.is_some() && self.fs.is_file(&dest_file) {
                let backup_dir = flags.backup_dir.as_ref().unwrap();
                let backup_path = PathBuf::from(backup_dir).join(relative);
                // Move existing file to backup dir
                let existing_content = self
                    .fs
                    .read_to_string(&dest_file)
                    .map_err(|e| RsyncError::IoError(e.to_string()))?;
                self.fs
                    .write(&backup_path, &existing_content)
                    .map_err(|e| RsyncError::IoError(e.to_string()))?;
            }

            // Check link-dest: if content matches, hard-link instead of copy
            if let Some(ref link_dest) = flags.link_dest {
                let link_file = PathBuf::from(link_dest).join(relative);
                if self.fs.is_file(&link_file) {
                    let link_content = self
                        .fs
                        .read_to_string(&link_file)
                        .map_err(|e| RsyncError::IoError(e.to_string()))?;
                    if link_content == src_content {
                        // Content unchanged — hard-link from link-dest
                        self.fs
                            .hard_link(&link_file, &dest_file)
                            .map_err(|e| RsyncError::IoError(e.to_string()))?;
                        files_transferred += 1;
                        continue;
                    }
                }
            }

            // Copy file
            self.fs
                .write(&dest_file, &src_content)
                .map_err(|e| RsyncError::IoError(e.to_string()))?;
            files_transferred += 1;
        }

        // Handle --delete: remove dest files not in source
        if flags.delete {
            let dest_path = Path::new(dest);
            if self.fs.is_dir(dest_path) {
                let dest_files = self
                    .fs
                    .walk_dir(dest_path)
                    .map_err(|e| RsyncError::IoError(e.to_string()))?
                    .into_iter()
                    .filter(|p| self.fs.is_file(p))
                    .collect::<Vec<_>>();

                for dest_file in dest_files {
                    let relative = dest_file
                        .strip_prefix(dest)
                        .unwrap_or(&dest_file);
                    let src_equivalent = PathBuf::from(source).join(relative);
                    if !self.fs.is_file(&src_equivalent) {
                        // Remove the file by writing to parent dir to "delete" —
                        // We need a remove operation. Use remove_dir_all on file won't work.
                        // We'll implement a simple removal by overwriting the node
                        // Actually, our FileSystem trait doesn't have remove_file.
                        // We can use remove_dir_all on the file's parent if it's the only file,
                        // but that's destructive. For now, let's handle this by noting
                        // that our TestFileSystem supports remove_dir_all which removes
                        // everything under a path. We need to remove just the file.
                        // Since we don't have remove_file in the trait, let's use
                        // a workaround - but actually for --delete simulation we should
                        // just note this limitation. However, the test expects deletion.
                        // Let's remove it by removing the parent dir and recreating other files.
                        // Actually, a simpler approach: since this is TestFileSystem,
                        // we have write access. We could overwrite with empty content.
                        // But the tests check for existence. Let's just remove_dir_all
                        // on the specific file path (TestFileSystem handles this).
                        let _ = self.fs.remove_dir_all(&dest_file);
                    }
                }
            }
        }

        Ok(RsyncResult {
            exit_code: 0,
            stdout: format!("Transferred {} files", files_transferred),
            stderr: String::new(),
            command: format!("rsync {}", args.join(" ")),
        })
    }
}

struct ParsedFlags {
    source: String,
    destination: String,
    delete: bool,
    dry_run: bool,
    link_dest: Option<String>,
    exclude_patterns: Vec<String>,
    backup: bool,
    backup_dir: Option<String>,
}

impl ParsedFlags {
    fn from_args(args: &[String]) -> Self {
        let mut delete = false;
        let mut dry_run = false;
        let mut link_dest = None;
        let mut exclude_patterns = Vec::new();
        let mut backup = false;
        let mut backup_dir = None;

        let mut positional = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            if arg == "--delete" {
                delete = true;
            } else if arg == "--dry-run" || arg == "-n" {
                dry_run = true;
            } else if arg == "--backup" || arg == "-b" {
                backup = true;
            } else if let Some(val) = arg.strip_prefix("--link-dest=") {
                link_dest = Some(val.to_string());
            } else if let Some(val) = arg.strip_prefix("--exclude=") {
                exclude_patterns.push(val.to_string());
            } else if let Some(val) = arg.strip_prefix("--backup-dir=") {
                backup_dir = Some(val.to_string());
                backup = true;
            } else if let Some(val) = arg.strip_prefix("--bwlimit=") {
                let _ = val; // ignore for simulation
            } else if arg == "-e" {
                // Skip the next arg (ssh command)
                i += 1;
            } else if arg.starts_with('-') {
                // Skip other flags
            } else {
                positional.push(arg.clone());
            }
            i += 1;
        }

        let (source, destination) = if positional.len() >= 2 {
            let dest = positional.pop().unwrap();
            let src = positional.pop().unwrap();
            (src, dest)
        } else {
            (String::new(), String::new())
        };

        Self {
            source,
            destination,
            delete,
            dry_run,
            link_dest,
            exclude_patterns,
            backup,
            backup_dir,
        }
    }
}

fn matches_glob(pattern: &str, path: &str) -> bool {
    // Simple glob matching: supports * wildcard
    if let Some(stripped) = pattern.strip_prefix("*.") {
        // Match file extension
        path.ends_with(&format!(".{}", stripped))
    } else if pattern.contains('*') {
        // Simple wildcard: split on * and check parts
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            path.starts_with(parts[0]) && path.ends_with(parts[1])
        } else {
            path.contains(pattern.trim_matches('*'))
        }
    } else {
        // Exact match
        path == pattern || path.ends_with(&format!("/{}", pattern))
    }
}

impl RsyncClient for TestRsyncClient {
    fn execute(&self, args: &[String]) -> Result<RsyncResult, RsyncError> {
        self.record_command(args);

        if let Some(err) = self.take_force_error() {
            return Err(err);
        }

        self.simulate_rsync(args)
    }

    fn dry_run(&self, args: &[String]) -> Result<RsyncResult, RsyncError> {
        let mut dry_args = args.to_vec();
        if !dry_args.contains(&"--dry-run".to_string()) {
            // Insert before the last two args (source and dest)
            let insert_pos = if dry_args.len() >= 2 {
                dry_args.len() - 2
            } else {
                0
            };
            dry_args.insert(insert_pos, "--dry-run".to_string());
        }
        self.execute(&dry_args)
    }

    fn version(&self) -> Result<String, RsyncError> {
        Ok("rsync version 3.2.7 protocol version 31 (test)".to_string())
    }
}
