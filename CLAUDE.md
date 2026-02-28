# CLAUDE.md

This file provides guidance for Claude Code when working on the Rsync Studio project.

## Project Overview

Rsync Studio is a cross-platform (macOS + Linux) rsync management tool with two frontends: a desktop GUI (Tauri v2 + React + TypeScript + shadcn/ui + Tailwind CSS) and a terminal UI (ratatui + crossterm). Both share a common Rust core library.

## Workspace Structure

```
rsync-studio/
├── Cargo.toml              # Workspace root (resolver = "2")
├── crates/
│   ├── rsync-core/         # Shared library: all domain logic, models, traits, services, tests
│   └── rsync-commander/    # Terminal UI: ratatui + crossterm + clap
├── src-tauri/              # GUI crate: Tauri shell, commands, state management
│   ├── src/commands.rs     # Tauri IPC command handlers
│   ├── src/execution.rs    # TauriEventHandler (ExecutionEventHandler impl)
│   ├── src/state.rs        # AppState (Database + Arc<JobExecutor>)
│   ├── src/lib.rs          # Tauri app builder + setup
│   └── src/main.rs         # Entry point
└── src/                    # React frontend
    ├── components/         # UI components (jobs/, ui/)
    ├── hooks/              # React hooks (use-jobs.ts, use-trailing-slash.ts, use-nas-auto-detect.ts)
    ├── lib/                # Utilities (tauri.ts, command-preview.ts, defaults.ts)
    ├── pages/              # Page components (jobs, history, settings)
    └── types/              # TypeScript type definitions mirroring Rust models
```

## Key Commands

```bash
# Frontend
npm install                    # Install JS dependencies
npm run dev                    # Start Vite dev server (port 1420)
npm run build                  # TypeScript check + Vite build
npx tsc --noEmit               # TypeScript type-check only

# Rust
cargo build --workspace        # Build all crates
cargo test -p rsync-core       # Run all tests (291 tests)
cargo test -p rsync-core -- <test_name>  # Run specific test

# Tauri (GUI)
npm run tauri dev              # Dev mode with hot reload
npm run tauri build            # Production build

# TUI
cargo run -p rsync-commander         # Run the terminal UI
cargo run -p rsync-commander -- list # List jobs (non-interactive)
cargo run -p rsync-commander -- run <job-id>  # Run single job (non-interactive)
cargo build -p rsync-commander --release      # Release build
```

## Architecture Patterns

### Trait-based Dependency Injection
All domain logic uses traits for testability:
- `RsyncClient` — rsync execution abstraction
- `FileSystem` — filesystem operations abstraction
- `JobRepository`, `InvocationRepository`, `SnapshotRepository` — data persistence

Production implementations use real processes/SQLite. Tests use in-memory fakes (`TestRsyncClient`, `TestFileSystem`).

### Settings System
App-level settings use a key-value `settings` table in SQLite, exposed through `SettingsService` (typed getters/setters), Tauri commands, and TypeScript wrappers. Settings consumed outside the Settings page use React hooks (e.g., `useTrailingSlash`, `useNasAutoDetect`). See the Settings section in `docs/FEATURES.md` for the full inventory and how-to-add guide.

### Per-Job vs App-Level Configuration
Some features have both layers: an app-level setting controlling the feature globally (in `SettingsService` / Settings page) and per-job fields on `RsyncOptions` (stored in the job's JSON). Example: NAS compatibility has `nas_auto_detect` (app-level, controls auto-detection) and `size_only` (per-job, on `RsyncOptions` — emits `--size-only` to compare files by size only).

### ExecutionEventHandler (Frontend Abstraction)
Job execution orchestration (log writing, statistics, snapshots) lives in `rsync-core`'s `JobExecutor`. The `ExecutionEventHandler` trait decouples it from any frontend: GUI implements with `AppHandle.emit()`, TUI implements with `mpsc::Sender`. If you modify execution behavior, change `job_executor.rs` — not the frontend wrappers.

### Shared Command Builder
`crates/rsync-core/src/services/command/command_builder.rs` builds rsync argument vectors. Both the Rust `ProcessRsyncClient` and the TypeScript `src/lib/command-preview.ts` mirror this logic. If you modify rsync flag handling, update both. Also update `command_parser.rs` (flag parsing) and `command_explainer.rs` (flag descriptions) if adding new recognized flags.

### Frontend-Backend Type Alignment
TypeScript types in `src/types/` must stay aligned with Rust models in `crates/rsync-core/src/models/`. The Tauri IPC layer serializes Rust structs as JSON, and the frontend deserializes into these TypeScript types.

### Form State Management
Job forms use `useReducer` with the full `JobDefinition` as state. The reducer auto-manages SSH config (initializes when an SSH location is selected, nulls when removed) and NAS compatibility (auto-enables `size_only` when a network filesystem is detected, if the app-level `nas_auto_detect` setting is enabled).

## Conventions

- All Rust models derive `Debug, Clone, Serialize, Deserialize, PartialEq`
- UUIDs are generated server-side in `JobService::create_job` (client sends a placeholder)
- Timestamps use `chrono::DateTime<Utc>` in Rust, ISO strings in TypeScript
- shadcn/ui components live in `src/components/ui/` (do not modify these)
- Path alias: `@/` maps to `src/` in both TypeScript and Vite config
- Error types use `thiserror` in Rust; Tauri commands convert errors to `String` for IPC
- `RsyncOptions` is defined in `crates/rsync-core/src/models/rsync_options.rs` and re-exported from `job.rs`. It contains 5 nested sub-structs: `CoreTransferOptions`, `FileHandlingOptions`, `MetadataOptions`, `OutputOptions`, `AdvancedOptions`. New fields require updates in **7 places**: the appropriate sub-struct + `Default` impl in `rsync_options.rs`, `command_builder.rs`, `command_parser.rs`, `command_explainer.rs`, the matching TS sub-interface in `src/types/job.ts`, `src/lib/defaults.ts`, `src/lib/command-preview.ts`
- Domain models use named sub-structs to stay under the ~10 field limit: `JobDefinition` contains `TransferConfig` (source + destination + backup_mode), `BackupInvocation` contains `TransferStats` (bytes/files transferred) and `ExecutionOutput` (command, exit_code, paths). The TUI `App` struct uses `AppServices`, `PageStates`, and `OverlayState` sub-structs.
- New `FileSystem` trait methods require stubs in `TestFileSystem` and `MockFs` (preflight.rs)
- Test files in `crates/rsync-core/src/tests/` are grouped by functional area into subdirectories (e.g., `command/`, `repository/`, `service/`, `fixtures/`). Each subdirectory has its own `mod.rs`. Standalone test files remain in the root.
- Services in `crates/rsync-core/src/services/` are grouped into subdirectories (`command/`, `execution/`, `retention/`, `scheduling/`) with `pub use` re-exports in the parent `mod.rs` for API stability.
- Models in `crates/rsync-core/src/models/` use an `execution/` subdirectory for runtime types (backup, progress, log, statistics, itemize).
- TUI ui modules in `crates/rsync-commander/src/ui/` are grouped into `pages/` and `widgets/` with re-exports.
- Frontend components in `src/components/jobs/` are grouped into `form/` (job form fields) and `execution/` (execution output views). Types in `src/types/` use `execution/` for runtime types.
- **No model types in the services layer.** All structs, enums, and type aliases that represent data (i.e., derive Serialize/Deserialize, hold domain data, or are used as IPC/API payloads) belong in `models/`. Services should only contain traits, trait implementations, service structs (holding `Arc` dependencies), and free functions. Service files import model types from `crate::models::*`. Runtime infrastructure types (e.g., `SchedulerHandle` wrapping an `mpsc::Sender`) that are tightly coupled to a trait may remain in the service file.

## Structural Limits

These thresholds are soft guidelines. When a limit is reached, refactor proactively rather than waiting for it to become painful.

- **Max ~8 files per directory.** When a directory exceeds this, group related files into subdirectories with their own `mod.rs` / `index.ts`. (Already enforced for `tests/`; applies equally to `models/`, `services/`, `src/components/`, `src/types/`, etc.)
- **Max ~10 fields per struct/interface.** When a struct or TypeScript interface exceeds this, group related fields into named sub-structs (e.g., `RsyncOptions` → `CoreTransferOptions` + `FileHandlingOptions` + …). Enum variants with data fields count toward the variant's own limit, not the parent enum.
- **File names must reflect the primary type or domain.** A Rust file exporting `FooService` should be named `foo_service.rs`; a TypeScript file exporting `BarOptions` should be `bar-options.ts`. Co-located helpers and secondary types are fine, but the file name should make the primary export obvious. Avoid generic names like `utils.rs` or `helpers.ts` for domain-specific code.

**Accepted exceptions:** `src/lib/utils.ts` uses the generic name because it is generated by shadcn/ui — this is an ecosystem convention.

## Testing

Tests live in `crates/rsync-core/src/tests/`. They use:
- `TestFileSystem`: in-memory filesystem with inode tracking for hard-link simulation
- `TestRsyncClient`: simulates rsync semantics (--delete, --link-dest, --exclude, --backup-dir, --dry-run)
- `tempfile` crate for SQLite tests (creates temp DB per test)

Run all tests with `cargo test -p rsync-core`.

## What Not to Do

- Do not modify files in `src/components/ui/` — these are generated by shadcn
- Do not add logic to `src-tauri/` or `crates/rsync-commander/` that belongs in `rsync-core` — keep both frontend crates as thin shells
- Do not use `unwrap()` in production Rust code — use `?` or proper error handling
- Do not add dependencies to `src-tauri/Cargo.toml` unless they are Tauri/GUI-specific
- Do not add dependencies to `crates/rsync-commander/Cargo.toml` unless they are TUI-specific (ratatui, crossterm, clap)
