# Architecture

## Overview

Rsync Studio follows a layered architecture with clear separation between domain logic, persistence, and presentation. The Cargo workspace contains three crates, and the frontend is a React SPA that communicates with the Rust backend via Tauri's IPC mechanism.

```
┌──────────────────────────────┐  ┌──────────────────────────────┐
│  React Frontend (src/)       │  │  Terminal UI (rsync-commander/) │
│  TypeScript + shadcn/ui      │  │  ratatui + crossterm + clap  │
│  Pages, Components, Hooks    │  │  Pages, Keybindings, Themes  │
├──────────────────────────────┤  ├──────────────────────────────┤
│  Tauri IPC (src-tauri/)      │  │  TuiEventHandler (mpsc)      │
│  TauriEventHandler (emit)    │  │  Direct service calls        │
├──────────────────────────────┴──┴──────────────────────────────┤
│  rsync-core Library (crates/rsync-core/)                       │
│  Models, Traits, Services, JobExecutor, Scheduler, Tests       │
└────────────────────────────────────────────────────────────────┘
```

Both frontends depend on `rsync-core` and share the same database. The `ExecutionEventHandler` trait is the key abstraction: the GUI implements it with Tauri's `AppHandle.emit()`, and the TUI implements it with `mpsc::Sender`.

## Crate Structure

### `rsync-core` (library crate)

The shared domain library. Contains all business logic, data models, and abstractions. Both the GUI and TUI depend on this crate.

```
crates/rsync-core/src/
├── lib.rs                  # Module exports
├── error.rs                # AppError enum (thiserror)
├── models/                 # Data structures
│   ├── job.rs              # JobDefinition, StorageLocation, BackupMode, RsyncOptions, SshConfig
│   ├── backup.rs           # BackupInvocation, SnapshotRecord
│   ├── log.rs              # LogEntry, LogLevel
│   ├── progress.rs         # ProgressUpdate, LogLine, JobStatusEvent
│   ├── schedule.rs         # ScheduleConfig, ScheduleType
│   ├── statistics.rs       # RunStatistic, AggregatedStats
│   └── validation.rs       # PreflightResult, ValidationCheck
├── traits/                 # Abstractions for DI
│   ├── rsync_client.rs     # RsyncClient trait
│   ├── file_system.rs      # FileSystem trait
│   ├── job_repository.rs   # JobRepository trait
│   ├── invocation_repository.rs
│   └── snapshot_repository.rs
├── services/               # Business logic
│   ├── command_builder.rs       # build_rsync_args()
│   ├── command_parser.rs        # parse_rsync_command()
│   ├── command_explainer.rs     # explain_command()
│   ├── execution_handler.rs     # ExecutionEventHandler trait
│   ├── job_executor.rs          # JobExecutor (execution orchestration)
│   ├── job_runner.rs            # Spawns rsync subprocess, streams events
│   ├── job_service.rs           # JobService (CRUD, history, snapshots)
│   ├── running_jobs.rs          # RunningJobs (thread-safe child process map)
│   ├── scheduler.rs             # is_job_due(), next_run_time()
│   ├── scheduler_backend.rs     # SchedulerBackend trait + InProcessScheduler
│   ├── retention.rs             # compute_snapshots_to_delete()
│   ├── retention_runner.rs      # run_history_retention()
│   ├── history_retention.rs     # compute_invocations_to_prune()
│   ├── statistics_service.rs    # Record/aggregate/export run statistics
│   ├── settings_service.rs      # Typed get/set for app settings
│   ├── export_import.rs         # Job export/import as JSON
│   ├── log_scrubber.rs          # Log file search/redact
│   ├── preflight.rs             # Pre-execution validation checks
│   └── progress_parser.rs       # Parse rsync progress output
├── implementations/        # Production implementations
│   ├── database.rs              # SQLite connection management
│   ├── process_rsync_client.rs  # Real rsync subprocess
│   ├── real_file_system.rs      # Real filesystem operations
│   └── sqlite_*.rs              # SQLite repository implementations
└── tests/                  # Test infrastructure + test suites (191 tests)
    ├── test_file_system.rs      # In-memory FS with inode tracking
    ├── test_rsync_client.rs     # Simulated rsync behavior
    ├── test_helpers.rs          # Shared test utilities
    └── *_tests.rs               # Test modules
```

### `rsync-gui` (binary + cdylib crate)

The Tauri application shell. Intentionally thin — delegates all logic to `rsync-core`.

```
src-tauri/src/
├── main.rs       # Entry point
├── lib.rs        # Tauri Builder setup, database init, scheduler, tray
├── commands.rs   # Tauri IPC command handlers
├── execution.rs  # TauriEventHandler (implements ExecutionEventHandler)
└── state.rs      # AppState struct (Database + Arc<JobExecutor>)
```

### `rsync-commander` (binary crate)

Terminal UI for headless servers. Uses ratatui for rendering and crossterm for terminal I/O. Shares the same database and services as the GUI.

```
crates/rsync-commander/src/
├── main.rs       # Entry point, CLI parsing (clap), terminal setup/teardown
├── app.rs        # App state machine, keybinding dispatch, page state
├── event.rs      # Event loop: multiplexes terminal events + job events
├── handler.rs    # TuiEventHandler (implements ExecutionEventHandler via mpsc)
├── theme.rs      # 4 color themes (Default, Dark, Solarized, Nord)
└── ui/
    ├── mod.rs         # Top-level draw(), layout helpers
    ├── tabs.rs        # Tab bar widget (1:Jobs 2:History ...)
    ├── status_bar.rs  # Bottom bar (running count, help hints)
    ├── popup.rs       # Modal dialogs (help, confirm, error)
    ├── text_input.rs  # Reusable text input with cursor + Ctrl shortcuts
    ├── jobs.rs        # Jobs table with search
    ├── job_form.rs    # Job create/edit form
    ├── job_output.rs  # Live output viewer with follow mode
    ├── history.rs     # Invocation history + log viewer
    ├── statistics.rs  # Aggregated + per-job stats dashboard
    ├── tools.rs       # Command explainer + log scrubber
    ├── settings.rs    # Settings editor
    └── about.rs       # Version info
```

## Frontend Architecture

```
src/
├── main.tsx                    # React entry point
├── App.tsx                     # App shell with sidebar nav + page routing
├── index.css                   # Tailwind + CSS variables (shadcn theme)
├── pages/                      # Top-level page components
│   ├── jobs-page.tsx           # Sub-view routing (list/create/edit)
│   ├── history-page.tsx        # Placeholder
│   └── settings-page.tsx       # Placeholder
├── components/
│   ├── sidebar.tsx             # Navigation sidebar
│   ├── ui/                     # shadcn/ui primitives (auto-generated)
│   └── jobs/                   # Job CRUD components
│       ├── job-list.tsx        # Grid of job cards + empty state
│       ├── job-card.tsx        # Individual job summary card
│       ├── job-form.tsx        # useReducer form with tabs + command preview
│       ├── job-form-general.tsx
│       ├── storage-location-field.tsx
│       ├── backup-mode-field.tsx
│       ├── rsync-options-field.tsx
│       ├── ssh-config-field.tsx
│       ├── pattern-list-field.tsx
│       ├── command-preview.tsx
│       └── delete-job-dialog.tsx
├── hooks/
│   └── use-jobs.ts             # Job CRUD hook wrapping Tauri invocations
├── lib/
│   ├── tauri.ts                # Typed Tauri invoke wrappers
│   ├── command-preview.ts      # Client-side rsync command builder (mirrors Rust)
│   ├── defaults.ts             # Default JobDefinition factory
│   └── utils.ts                # cn() classname utility
└── types/                      # TypeScript types mirroring Rust models
    ├── index.ts                # Barrel exports
    ├── job.ts                  # JobDefinition, StorageLocation, BackupMode, etc.
    ├── backup.ts               # BackupInvocation, SnapshotRecord
    ├── schedule.ts             # ScheduleConfig, ScheduleType
    ├── log.ts                  # LogEntry
    ├── progress.ts             # ProgressUpdate, LogLine, JobStatusEvent
    └── validation.ts           # PreflightResult, ValidationCheck
```

## Key Design Decisions

### ExecutionEventHandler — Frontend Abstraction

Job execution orchestration (~300 lines covering log writing, statistics recording, snapshot management, and retention) lives in `rsync-core`'s `JobExecutor`. The `ExecutionEventHandler` trait decouples this logic from any specific frontend:

```rust
pub trait ExecutionEventHandler: Send + Sync {
    fn on_log_line(&self, log_line: LogLine);
    fn on_progress(&self, progress: &ProgressUpdate);
    fn on_status_change(&self, status: JobStatusEvent);
}
```

| Frontend | Implementation | Event Delivery |
|----------|---------------|----------------|
| GUI | `TauriEventHandler` | `AppHandle.emit()` → JavaScript event listeners |
| TUI | `TuiEventHandler` | `mpsc::Sender` → event loop `try_recv()` |
| CLI (`run`) | `TuiEventHandler` | `mpsc::Sender` → blocking `recv()` loop |

### Pluggable Scheduler

The `SchedulerBackend` trait allows different scheduling strategies:

- **`InProcessScheduler`** — background thread with configurable check interval (used by both GUI and TUI)
- **External schedulers** — the `rsync-commander run <job-id>` subcommand enables crontab or systemd timer integration without an in-process scheduler

### Trait-based Dependency Injection

All external dependencies (rsync process, filesystem, database) are abstracted behind traits. This enables:
- Unit testing with in-memory fakes (no real rsync/filesystem/DB needed)
- Future alternative implementations (e.g., different DB backends)
- Clear interface boundaries between layers

### Command Builder Duplication

`build_rsync_args()` exists in both Rust (`command_builder.rs`) and TypeScript (`command-preview.ts`). The TypeScript version provides live command preview in the UI without IPC round-trips on every keystroke. The Rust version is the source of truth used during actual execution.

### Form State with useReducer

The job creation/edit form uses `useReducer` instead of individual `useState` calls because `JobDefinition` is deeply nested (StorageLocation union, BackupMode union, RsyncOptions with 12+ fields, SSH config). The reducer also handles cross-field logic like auto-initializing SSH config when an SSH location is selected.

### Sub-view Routing

Page-level navigation uses React state (`useState<View>`) inside `JobsPage` rather than a router library. This keeps dependencies minimal and avoids changes to `App.tsx` when adding views within a page.

## Data Flow

### GUI

```
User Action → React Component → useJobs hook → tauri.ts invoke wrapper
    → Tauri IPC → commands.rs → JobExecutor/JobService → Repository → SQLite
    → Result<T, AppError> → String error for IPC → TypeScript Promise
```

### TUI

```
Key Event → App::handle_key() → JobExecutor/JobService → Repository → SQLite
    → ExecutionEventHandler → mpsc channel → EventLoop → App::handle_job_event()
    → terminal redraw
```

### Headless (`rsync-commander run`)

```
CLI args → JobService::get_job() → JobExecutor::execute() → mpsc channel
    → blocking recv() loop → stdout/stderr → exit code
```

## Database

SQLite via `rusqlite` with bundled SQLite. Database file location: `{app_data_dir}/rsync-studio.db`. Tables are created on first launch. Foreign key cascades handle cleanup (deleting a job removes its invocations and snapshots).
