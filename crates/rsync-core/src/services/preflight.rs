use std::path::Path;

use crate::models::job::{JobDefinition, StorageLocation};
use crate::models::validation::{CheckSeverity, CheckType, PreflightResult, ValidationCheck};
use crate::file_system::FileSystem;
use crate::rsync_client::RsyncClient;

/// Run preflight validation checks for a job.
///
/// Checks: rsync installed, source exists (local only), destination writable
/// (local only), disk space (local destination), SSH connectivity (dry-run test).
pub fn run_preflight(
    job: &JobDefinition,
    fs: &dyn FileSystem,
    rsync: &dyn RsyncClient,
) -> PreflightResult {
    let mut checks = Vec::new();

    checks.push(check_rsync_installed(rsync));
    checks.push(check_source_exists(&job.source, fs));
    checks.push(check_destination_writable(&job.destination, fs));
    checks.push(check_disk_space(&job.source, &job.destination, fs));

    if is_remote(&job.source) || is_remote(&job.destination) {
        checks.push(check_ssh_connectivity(job, rsync));
    }

    let overall_pass = checks
        .iter()
        .all(|c| c.passed || c.severity == CheckSeverity::Warning);

    PreflightResult {
        job_id: job.id,
        checks,
        overall_pass,
    }
}

fn is_remote(loc: &StorageLocation) -> bool {
    !matches!(loc, StorageLocation::Local { .. })
}

fn check_rsync_installed(rsync: &dyn RsyncClient) -> ValidationCheck {
    match rsync.version() {
        Ok(version) => ValidationCheck {
            check_type: CheckType::RsyncInstalled,
            passed: true,
            message: format!("rsync is installed ({})", version.trim()),
            severity: CheckSeverity::Error,
        },
        Err(_) => ValidationCheck {
            check_type: CheckType::RsyncInstalled,
            passed: false,
            message: "rsync is not installed or not found in PATH".to_string(),
            severity: CheckSeverity::Error,
        },
    }
}

fn check_source_exists(source: &StorageLocation, fs: &dyn FileSystem) -> ValidationCheck {
    match source {
        StorageLocation::Local { path } => {
            let exists = fs.exists(Path::new(path));
            ValidationCheck {
                check_type: CheckType::SourceExists,
                passed: exists,
                message: if exists {
                    format!("Source path exists: {}", path)
                } else {
                    format!("Source path does not exist: {}", path)
                },
                severity: CheckSeverity::Error,
            }
        }
        _ => ValidationCheck {
            check_type: CheckType::SourceExists,
            passed: true,
            message: "Remote source — cannot verify locally".to_string(),
            severity: CheckSeverity::Warning,
        },
    }
}

fn check_destination_writable(dest: &StorageLocation, fs: &dyn FileSystem) -> ValidationCheck {
    match dest {
        StorageLocation::Local { path } => {
            let p = Path::new(path);
            if fs.exists(p) && fs.is_dir(p) {
                ValidationCheck {
                    check_type: CheckType::DestinationWritable,
                    passed: true,
                    message: format!("Destination directory exists: {}", path),
                    severity: CheckSeverity::Error,
                }
            } else if fs.exists(p) {
                ValidationCheck {
                    check_type: CheckType::DestinationWritable,
                    passed: false,
                    message: format!("Destination exists but is not a directory: {}", path),
                    severity: CheckSeverity::Error,
                }
            } else {
                // Check if parent exists
                let parent = Path::new(path).parent();
                let parent_ok = parent.is_some_and(|pp| fs.exists(pp) && fs.is_dir(pp));
                ValidationCheck {
                    check_type: CheckType::DestinationWritable,
                    passed: parent_ok,
                    message: if parent_ok {
                        format!(
                            "Destination does not exist but parent directory is valid: {}",
                            path
                        )
                    } else {
                        format!(
                            "Destination and its parent directory do not exist: {}",
                            path
                        )
                    },
                    severity: CheckSeverity::Error,
                }
            }
        }
        _ => ValidationCheck {
            check_type: CheckType::DestinationWritable,
            passed: true,
            message: "Remote destination — cannot verify locally".to_string(),
            severity: CheckSeverity::Warning,
        },
    }
}

fn check_disk_space(
    source: &StorageLocation,
    dest: &StorageLocation,
    fs: &dyn FileSystem,
) -> ValidationCheck {
    let (src_local, dst_local) = match (source, dest) {
        (StorageLocation::Local { path: src }, StorageLocation::Local { path: dst }) => (src, dst),
        _ => {
            return ValidationCheck {
                check_type: CheckType::DiskSpace,
                passed: true,
                message: "Disk space check skipped for remote locations".to_string(),
                severity: CheckSeverity::Warning,
            };
        }
    };

    let src_size = fs.dir_size(Path::new(src_local)).unwrap_or(0);
    let dst_avail = fs.available_space(Path::new(dst_local)).unwrap_or(0);

    if src_size == 0 {
        return ValidationCheck {
            check_type: CheckType::DiskSpace,
            passed: true,
            message: "Source is empty or size could not be determined".to_string(),
            severity: CheckSeverity::Warning,
        };
    }

    let enough = dst_avail >= src_size;
    ValidationCheck {
        check_type: CheckType::DiskSpace,
        passed: enough,
        message: if enough {
            format!(
                "Sufficient disk space ({} available, {} needed)",
                format_bytes(dst_avail),
                format_bytes(src_size)
            )
        } else {
            format!(
                "Insufficient disk space ({} available, {} needed)",
                format_bytes(dst_avail),
                format_bytes(src_size)
            )
        },
        severity: if enough {
            CheckSeverity::Warning
        } else {
            CheckSeverity::Error
        },
    }
}

fn check_ssh_connectivity(job: &JobDefinition, rsync: &dyn RsyncClient) -> ValidationCheck {
    // Build a minimal dry-run command to test connectivity
    use crate::services::command_builder;

    let mut test_job = job.clone();
    test_job.options.dry_run = true;

    let args = command_builder::build_rsync_args(
        &test_job.source,
        &test_job.destination,
        &test_job.options,
        test_job.ssh_config.as_ref(),
        None,
        false,
    );
    match rsync.dry_run(&args) {
        Ok(result) if result.exit_code == 0 => ValidationCheck {
            check_type: CheckType::SshConnectivity,
            passed: true,
            message: "SSH connectivity test passed (dry-run succeeded)".to_string(),
            severity: CheckSeverity::Error,
        },
        Ok(result) => ValidationCheck {
            check_type: CheckType::SshConnectivity,
            passed: false,
            message: format!(
                "SSH connectivity test failed (exit code {}): {}",
                result.exit_code,
                result.stderr.lines().next().unwrap_or("unknown error")
            ),
            severity: CheckSeverity::Error,
        },
        Err(e) => ValidationCheck {
            check_type: CheckType::SshConnectivity,
            passed: false,
            message: format!("SSH connectivity test failed: {}", e),
            severity: CheckSeverity::Error,
        },
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::job::*;
    use crate::file_system::FsError;
    use crate::rsync_client::{RsyncError, RsyncResult};
    use std::path::{Path, PathBuf};
    use uuid::Uuid;

    // Minimal test filesystem
    struct MockFs {
        existing_dirs: Vec<String>,
        existing_files: Vec<String>,
        available_space_bytes: u64,
        dir_size_bytes: u64,
    }

    impl MockFs {
        fn new() -> Self {
            Self {
                existing_dirs: Vec::new(),
                existing_files: Vec::new(),
                available_space_bytes: 10 * 1024 * 1024 * 1024, // 10 GB
                dir_size_bytes: 1024 * 1024 * 1024,             // 1 GB
            }
        }

        fn with_dir(mut self, path: &str) -> Self {
            self.existing_dirs.push(path.to_string());
            self
        }

        fn with_space(mut self, available: u64, source_size: u64) -> Self {
            self.available_space_bytes = available;
            self.dir_size_bytes = source_size;
            self
        }
    }

    impl FileSystem for MockFs {
        fn exists(&self, path: &Path) -> bool {
            let s = path.to_string_lossy().to_string();
            self.existing_dirs.contains(&s) || self.existing_files.contains(&s)
        }
        fn is_dir(&self, path: &Path) -> bool {
            self.existing_dirs.contains(&path.to_string_lossy().to_string())
        }
        fn is_file(&self, path: &Path) -> bool {
            self.existing_files.contains(&path.to_string_lossy().to_string())
        }
        fn is_symlink(&self, _: &Path) -> bool {
            false
        }
        fn create_dir_all(&self, _: &Path) -> Result<(), FsError> {
            Ok(())
        }
        fn remove_dir_all(&self, _: &Path) -> Result<(), FsError> {
            Ok(())
        }
        fn read_dir(&self, _: &Path) -> Result<Vec<PathBuf>, FsError> {
            Ok(vec![])
        }
        fn read_to_string(&self, _: &Path) -> Result<String, FsError> {
            Ok(String::new())
        }
        fn write(&self, _: &Path, _: &str) -> Result<(), FsError> {
            Ok(())
        }
        fn create_symlink(&self, _: &Path, _: &Path) -> Result<(), FsError> {
            Ok(())
        }
        fn read_link(&self, _: &Path) -> Result<PathBuf, FsError> {
            Err(FsError::NotFound("not a link".to_string()))
        }
        fn remove_symlink(&self, _: &Path) -> Result<(), FsError> {
            Ok(())
        }
        fn available_space(&self, _: &Path) -> Result<u64, FsError> {
            Ok(self.available_space_bytes)
        }
        fn dir_size(&self, _: &Path) -> Result<u64, FsError> {
            Ok(self.dir_size_bytes)
        }
        fn copy_file(&self, _: &Path, _: &Path) -> Result<(), FsError> {
            Ok(())
        }
        fn hard_link(&self, _: &Path, _: &Path) -> Result<(), FsError> {
            Ok(())
        }
        fn walk_dir(&self, _: &Path) -> Result<Vec<PathBuf>, FsError> {
            Ok(vec![])
        }
    }

    struct MockRsync {
        installed: bool,
        dry_run_exit: i32,
    }

    impl MockRsync {
        fn installed() -> Self {
            Self {
                installed: true,
                dry_run_exit: 0,
            }
        }
        fn not_installed() -> Self {
            Self {
                installed: false,
                dry_run_exit: 1,
            }
        }
        fn with_dry_run_exit(mut self, code: i32) -> Self {
            self.dry_run_exit = code;
            self
        }
    }

    impl RsyncClient for MockRsync {
        fn execute(&self, _args: &[String]) -> Result<RsyncResult, RsyncError> {
            Ok(RsyncResult {
                exit_code: 0,
                stdout: String::new(),
                stderr: String::new(),
                command: "rsync".to_string(),
            })
        }
        fn dry_run(&self, _args: &[String]) -> Result<RsyncResult, RsyncError> {
            if !self.installed {
                return Err(RsyncError::RsyncNotFound);
            }
            Ok(RsyncResult {
                exit_code: self.dry_run_exit,
                stdout: String::new(),
                stderr: if self.dry_run_exit != 0 {
                    "ssh: connect to host server port 22: Connection refused".to_string()
                } else {
                    String::new()
                },
                command: "rsync --dry-run".to_string(),
            })
        }
        fn version(&self) -> Result<String, RsyncError> {
            if self.installed {
                Ok("rsync version 3.2.7".to_string())
            } else {
                Err(RsyncError::RsyncNotFound)
            }
        }
    }

    fn local_job() -> JobDefinition {
        JobDefinition {
            id: Uuid::new_v4(),
            name: "Test".to_string(),
            description: None,
            source: StorageLocation::Local {
                path: "/source".to_string(),
            },
            destination: StorageLocation::Local {
                path: "/dest".to_string(),
            },
            backup_mode: BackupMode::Mirror,
            options: RsyncOptions::default(),
            ssh_config: None,
            schedule: None,
            enabled: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn remote_job() -> JobDefinition {
        let mut job = local_job();
        job.destination = StorageLocation::RemoteSsh {
            user: "user".to_string(),
            host: "server".to_string(),
            port: 22,
            path: "/backup".to_string(),
            identity_file: None,
        };
        job
    }

    #[test]
    fn all_pass_for_local_job() {
        let fs = MockFs::new().with_dir("/source").with_dir("/dest");
        let rsync = MockRsync::installed();
        let result = run_preflight(&local_job(), &fs, &rsync);
        assert!(result.overall_pass);
        // Should have 4 checks (no SSH for local)
        assert_eq!(result.checks.len(), 4);
    }

    #[test]
    fn rsync_not_installed_fails() {
        let fs = MockFs::new().with_dir("/source").with_dir("/dest");
        let rsync = MockRsync::not_installed();
        let result = run_preflight(&local_job(), &fs, &rsync);
        assert!(!result.overall_pass);

        let rsync_check = result
            .checks
            .iter()
            .find(|c| c.check_type == CheckType::RsyncInstalled)
            .unwrap();
        assert!(!rsync_check.passed);
    }

    #[test]
    fn source_not_found_fails() {
        let fs = MockFs::new().with_dir("/dest"); // no /source
        let rsync = MockRsync::installed();
        let result = run_preflight(&local_job(), &fs, &rsync);
        assert!(!result.overall_pass);

        let src_check = result
            .checks
            .iter()
            .find(|c| c.check_type == CheckType::SourceExists)
            .unwrap();
        assert!(!src_check.passed);
    }

    #[test]
    fn destination_missing_but_parent_exists() {
        let fs = MockFs::new().with_dir("/source").with_dir("/"); // / exists but not /dest
        let rsync = MockRsync::installed();
        let result = run_preflight(&local_job(), &fs, &rsync);
        assert!(result.overall_pass);

        let dst_check = result
            .checks
            .iter()
            .find(|c| c.check_type == CheckType::DestinationWritable)
            .unwrap();
        assert!(dst_check.passed);
        assert!(dst_check.message.contains("parent directory is valid"));
    }

    #[test]
    fn destination_and_parent_missing_fails() {
        let fs = MockFs::new().with_dir("/source"); // neither /dest nor / marked as existing dir
        let rsync = MockRsync::installed();
        let result = run_preflight(&local_job(), &fs, &rsync);
        assert!(!result.overall_pass);

        let dst_check = result
            .checks
            .iter()
            .find(|c| c.check_type == CheckType::DestinationWritable)
            .unwrap();
        assert!(!dst_check.passed);
    }

    #[test]
    fn insufficient_disk_space_fails() {
        let fs = MockFs::new()
            .with_dir("/source")
            .with_dir("/dest")
            .with_space(500, 1000); // 500 available, 1000 needed
        let rsync = MockRsync::installed();
        let result = run_preflight(&local_job(), &fs, &rsync);
        assert!(!result.overall_pass);

        let space_check = result
            .checks
            .iter()
            .find(|c| c.check_type == CheckType::DiskSpace)
            .unwrap();
        assert!(!space_check.passed);
        assert!(space_check.message.contains("Insufficient"));
    }

    #[test]
    fn remote_job_includes_ssh_check() {
        let fs = MockFs::new().with_dir("/source");
        let rsync = MockRsync::installed();
        let result = run_preflight(&remote_job(), &fs, &rsync);

        assert_eq!(result.checks.len(), 5); // includes SSH
        let ssh_check = result
            .checks
            .iter()
            .find(|c| c.check_type == CheckType::SshConnectivity)
            .unwrap();
        assert!(ssh_check.passed);
    }

    #[test]
    fn ssh_connection_failure() {
        let fs = MockFs::new().with_dir("/source");
        let rsync = MockRsync::installed().with_dry_run_exit(255);
        let result = run_preflight(&remote_job(), &fs, &rsync);
        assert!(!result.overall_pass);

        let ssh_check = result
            .checks
            .iter()
            .find(|c| c.check_type == CheckType::SshConnectivity)
            .unwrap();
        assert!(!ssh_check.passed);
        assert!(ssh_check.message.contains("Connection refused"));
    }

    #[test]
    fn remote_source_skips_local_exists_check() {
        let mut job = local_job();
        job.source = StorageLocation::RemoteSsh {
            user: "user".to_string(),
            host: "server".to_string(),
            port: 22,
            path: "/data".to_string(),
            identity_file: None,
        };
        let fs = MockFs::new().with_dir("/dest");
        let rsync = MockRsync::installed();
        let result = run_preflight(&job, &fs, &rsync);

        let src_check = result
            .checks
            .iter()
            .find(|c| c.check_type == CheckType::SourceExists)
            .unwrap();
        assert!(src_check.passed);
        assert!(src_check.message.contains("Remote source"));
    }

    #[test]
    fn format_bytes_display() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GB");
        assert_eq!(format_bytes(2 * 1024 * 1024 * 1024), "2.0 GB");
    }
}
