use tauri::Emitter;

use rsync_core::models::progress::{JobStatusEvent, LogLine, ProgressUpdate};
use rsync_core::services::execution_handler::ExecutionEventHandler;

/// GUI implementation of ExecutionEventHandler that emits Tauri events.
pub struct TauriEventHandler {
    app_handle: tauri::AppHandle,
}

impl TauriEventHandler {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self { app_handle }
    }
}

impl ExecutionEventHandler for TauriEventHandler {
    fn on_log_line(&self, log_line: LogLine) {
        let _ = self.app_handle.emit("job-log", log_line);
    }

    fn on_progress(&self, progress: &ProgressUpdate) {
        let _ = self.app_handle.emit("job-progress", progress);
    }

    fn on_status_change(&self, status: JobStatusEvent) {
        let _ = self.app_handle.emit("job-status", status);
    }
}
