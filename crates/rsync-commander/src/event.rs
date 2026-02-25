use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};

use crate::handler::TuiEvent;

/// Unified event type for the TUI event loop.
#[derive(Debug)]
pub enum AppEvent {
    /// A key press from the terminal.
    Key(KeyEvent),
    /// A tick event (for periodic UI refresh).
    Tick,
    /// A job execution event from a background thread.
    Job(TuiEvent),
    /// Terminal resize event.
    Resize(u16, u16),
}

/// Polls for terminal events and multiplexes with job execution events.
pub struct EventLoop {
    job_rx: Receiver<TuiEvent>,
    job_tx: Sender<TuiEvent>,
    tick_rate: Duration,
}

impl EventLoop {
    pub fn new(tick_rate: Duration) -> Self {
        let (job_tx, job_rx) = mpsc::channel();
        Self {
            job_rx,
            job_tx,
            tick_rate,
        }
    }

    /// Get a sender for sending job execution events into the event loop.
    pub fn job_sender(&self) -> Sender<TuiEvent> {
        self.job_tx.clone()
    }

    /// Poll for the next event. Returns `None` on timeout (tick).
    pub fn next(&self) -> AppEvent {
        // First, drain all pending job events
        while let Ok(job_event) = self.job_rx.try_recv() {
            return AppEvent::Job(job_event);
        }

        // Poll for terminal events
        if event::poll(self.tick_rate).unwrap_or(false) {
            match event::read() {
                Ok(CrosstermEvent::Key(key)) => return AppEvent::Key(key),
                Ok(CrosstermEvent::Resize(w, h)) => return AppEvent::Resize(w, h),
                _ => {}
            }
        }

        AppEvent::Tick
    }
}
