use crate::models::progress::{JobStatusEvent, LogLine, ProgressUpdate};

/// Trait for receiving execution events from a running job.
///
/// The GUI implements this with Tauri's `AppHandle.emit()`.
/// The TUI implements this with `mpsc::Sender`.
pub trait ExecutionEventHandler: Send + Sync {
    fn on_log_line(&self, log_line: LogLine);
    fn on_progress(&self, progress: &ProgressUpdate);
    fn on_status_change(&self, status: JobStatusEvent);
}
