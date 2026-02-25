use std::sync::mpsc::Sender;

use rsync_core::models::progress::{JobStatusEvent, LogLine, ProgressUpdate};
use rsync_core::services::execution_handler::ExecutionEventHandler;

/// Events sent from the job executor to the TUI event loop.
#[derive(Debug, Clone)]
pub enum TuiEvent {
    LogLine(LogLine),
    Progress(ProgressUpdate),
    StatusChange(JobStatusEvent),
}

/// TUI implementation of ExecutionEventHandler that sends events via mpsc channel.
pub struct TuiEventHandler {
    sender: Sender<TuiEvent>,
}

impl TuiEventHandler {
    pub fn new(sender: Sender<TuiEvent>) -> Self {
        Self { sender }
    }
}

impl ExecutionEventHandler for TuiEventHandler {
    fn on_log_line(&self, log_line: LogLine) {
        let _ = self.sender.send(TuiEvent::LogLine(log_line));
    }

    fn on_progress(&self, progress: &ProgressUpdate) {
        let _ = self.sender.send(TuiEvent::Progress(progress.clone()));
    }

    fn on_status_change(&self, status: JobStatusEvent) {
        let _ = self.sender.send(TuiEvent::StatusChange(status));
    }
}
