# Architecture

## Overview

Rsync Desktop follows a layered architecture with clear separation between domain logic, persistence, and presentation. The Cargo workspace contains three crates, and the frontend is a React SPA that communicates with the Rust backend via Tauri's IPC mechanism.

```
┌──────────────────────────────────────────────────────┐
│  React Frontend (src/)                               │
│  TypeScript + shadcn/ui + Tailwind                   │
│  Pages, Components, Hooks, Typed Invoke Wrappers     │
├──────────────────────────────────────────────────────┤
│  Tauri IPC Layer (src-tauri/)                        │
│  Commands → State → Service calls                    │
├──────────────────────────────────────────────────────┤
│  rsync-core Library (crates/rsync-core/)             │
│  Models, Traits, Services, Implementations, Tests    │
└──────────────────────────────────────────────────────┘
```

## Crate Structure

### `rsync-core` (library crate)

The shared domain library. Contains all business logic, data models, and abstractions. Both the GUI and future TUI depend on this crate.

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
│   └── validation.rs       # PreflightResult, ValidationCheck
├── traits/                 # Abstractions for DI
│   ├── rsync_client.rs     # RsyncClient trait
│   ├── file_system.rs      # FileSystem trait
│   ├── job_repository.rs   # JobRepository trait
│   ├── invocation_repository.rs
│   └── snapshot_repository.rs
├── services/               # Business logic
│   ├── command_builder.rs  # build_rsync_args() — shared by Rust + TypeScript
│   └── job_service.rs      # JobService (CRUD, history, snapshots)
├── implementations/        # Production implementations
│   ├── database.rs         # SQLite connection management
│   ├── process_rsync_client.rs  # Real rsync subprocess
│   ├── real_file_system.rs      # Real filesystem operations
│   ├── sqlite_job_repository.rs
│   ├── sqlite_invocation_repository.rs
│   └── sqlite_snapshot_repository.rs
└── tests/                  # Test infrastructure + test suites
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
├── lib.rs        # Tauri Builder setup, database init, state wiring
├── commands.rs   # 6 Tauri IPC command handlers
└── state.rs      # AppState struct (Database + Arc<JobService>)
```

### `rsync-tui` (binary crate — stub)

Future terminal UI. Currently a placeholder that depends on `rsync-core`.

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

```
User Action → React Component → useJobs hook → tauri.ts invoke wrapper
    → Tauri IPC → commands.rs → JobService → Repository trait → SQLite
    → Result<T, AppError> → String error for IPC → TypeScript Promise
```

## Database

SQLite via `rusqlite` with bundled SQLite. Database file location: `{app_data_dir}/rsync-desktop.db`. Tables are created on first launch. Foreign key cascades handle cleanup (deleting a job removes its invocations and snapshots).
