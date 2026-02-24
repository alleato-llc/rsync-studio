use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

use crate::traits::rsync_client::{RsyncClient, RsyncError, RsyncResult};

pub struct ProcessRsyncClient {
    rsync_binary: String,
}

impl ProcessRsyncClient {
    pub fn new() -> Self {
        Self {
            rsync_binary: "rsync".to_string(),
        }
    }

    pub fn with_binary(binary: String) -> Self {
        Self {
            rsync_binary: binary,
        }
    }
}

impl Default for ProcessRsyncClient {
    fn default() -> Self {
        Self::new()
    }
}

impl RsyncClient for ProcessRsyncClient {
    fn execute(&self, args: &[String]) -> Result<RsyncResult, RsyncError> {
        let command_str = format!("{} {}", self.rsync_binary, args.join(" "));

        let mut child = Command::new(&self.rsync_binary)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    RsyncError::RsyncNotFound
                } else {
                    RsyncError::IoError(e.to_string())
                }
            })?;

        let mut stdout_lines = Vec::new();

        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                let line = line.map_err(|e| RsyncError::IoError(e.to_string()))?;
                stdout_lines.push(line);
            }
        }

        let stderr_output = if let Some(stderr) = child.stderr.take() {
            let reader = BufReader::new(stderr);
            reader
                .lines()
                .filter_map(|l| l.ok())
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            String::new()
        };

        let status = child
            .wait()
            .map_err(|e| RsyncError::IoError(e.to_string()))?;

        let exit_code = status.code().unwrap_or(-1);

        if exit_code != 0 {
            return Err(RsyncError::ProcessError {
                message: stderr_output.clone(),
                exit_code: Some(exit_code),
            });
        }

        Ok(RsyncResult {
            exit_code,
            stdout: stdout_lines.join("\n"),
            stderr: stderr_output,
            command: command_str,
        })
    }

    fn dry_run(&self, args: &[String]) -> Result<RsyncResult, RsyncError> {
        let mut dry_args = args.to_vec();
        if !dry_args.contains(&"--dry-run".to_string()) {
            dry_args.insert(0, "--dry-run".to_string());
        }
        self.execute(&dry_args)
    }

    fn version(&self) -> Result<String, RsyncError> {
        let output = Command::new(&self.rsync_binary)
            .arg("--version")
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    RsyncError::RsyncNotFound
                } else {
                    RsyncError::IoError(e.to_string())
                }
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let first_line = stdout.lines().next().unwrap_or("unknown").to_string();
        Ok(first_line)
    }
}
