use std::io::BufRead;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};

use uuid::Uuid;

use crate::error::AppError;
use crate::models::itemize::ItemizedChange;
use crate::services::itemize_parser::parse_itemize_line;
use crate::services::progress_parser::parse_progress_line;
use crate::rsync_client::RsyncError;

#[derive(Debug, Clone)]
pub enum ExecutionEvent {
    StdoutLine(String),
    StderrLine(String),
    Progress(crate::models::progress::ProgressUpdate),
    ItemizedChange(ItemizedChange),
    Finished { exit_code: Option<i32> },
}

/// Spawns rsync as a child process with the given binary and args,
/// returning the child handle and a receiver for execution events.
///
/// Reader threads are spawned for stdout and stderr. Progress lines
/// from stdout are parsed and emitted as Progress events in addition
/// to the StdoutLine event. When both readers finish, a Finished event
/// is sent with the exit code.
pub fn run_job(
    binary: &str,
    args: &[String],
    invocation_id: Uuid,
) -> Result<(Child, Receiver<ExecutionEvent>), AppError> {
    let mut child = Command::new(binary)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AppError::RsyncError(RsyncError::IoError(format!("Failed to spawn rsync: {}", e))))?;

    let (tx, rx) = mpsc::channel();

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AppError::RsyncError(RsyncError::IoError("Failed to capture stdout".to_string())))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| AppError::RsyncError(RsyncError::IoError("Failed to capture stderr".to_string())))?;

    // Stdout reader thread
    let tx_out = tx.clone();
    let inv_id = invocation_id;
    std::thread::spawn(move || {
        let reader = std::io::BufReader::new(stdout);
        for line_result in reader.lines() {
            match line_result {
                Ok(text) => {
                    if let Some(progress) = parse_progress_line(&text, inv_id) {
                        let _ = tx_out.send(ExecutionEvent::Progress(progress));
                    }
                    if let Some(change) = parse_itemize_line(&text) {
                        let _ = tx_out.send(ExecutionEvent::ItemizedChange(change));
                    }
                    let _ = tx_out.send(ExecutionEvent::StdoutLine(text));
                }
                Err(_) => break,
            }
        }
    });

    // Stderr reader thread
    let tx_err = tx;
    std::thread::spawn(move || {
        let reader = std::io::BufReader::new(stderr);
        for line_result in reader.lines() {
            match line_result {
                Ok(text) => {
                    let _ = tx_err.send(ExecutionEvent::StderrLine(text));
                }
                Err(_) => break,
            }
        }
    });

    Ok((child, rx))
}
