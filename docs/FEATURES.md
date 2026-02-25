# Features

Brief reference for each feature — what it does, how it works, key files, and how to maintain it.

---

## Statistics

Run statistics track how much data rsync transfers across job executions. Five metrics are displayed on the Statistics page.

### How it works

```
rsync stdout → progress_parser → job_executor event loop → BackupInvocation → StatisticsService → SQLite
```

1. rsync runs with `--progress -v`, producing per-file progress lines and a transfer summary
2. `progress_parser.rs` extracts per-file progress (`parse_progress_line`) and the final summary (`parse_summary_line`)
3. `job_runner.rs` reads stdout line-by-line, emitting `ExecutionEvent::Progress` and `ExecutionEvent::StdoutLine`
4. `job_executor.rs` background thread tracks `last_files` (from `xfr#N`), `summary_sent_bytes` (from `sent X bytes`), and `last_speedup` (from `speedup is X.XX`)
5. On successful non-dry-run completion, a `RunStatistic` is recorded to SQLite

### Metrics

| Metric | Source | Notes |
|---|---|---|
| Files Transferred | `xfr#N` from progress lines | Cumulative — last value is the total |
| Data Transferred | `sent X bytes` from summary line | Total bytes rsync put on the wire |
| Total Time | `finished_at - started_at` | Wall-clock duration |
| Speedup | `speedup is X.XX` from summary line | rsync's delta-transfer efficiency ratio |
| Time Saved | `duration * (speedup - 1)` per run | Only counted when speedup > 1.0 |

Statistics are NOT recorded for dry runs, failed jobs, or cancelled jobs.

### rsync output formats

**Per-file progress** (`--progress`):
```
     32,768 100%   31.25kB/s    0:00:00 (xfr#1, to-chk=2/4)
  205.18M 100%    7.46M/s    0:00:26 (xfr#1, ir-chk=0/1)
```
- Bytes value is per-file, supports K/M/G suffixes
- Both `to-chk` (rsync 2.x) and `ir-chk` (rsync 3.1+) are supported
- `xfr#N` is the cumulative file transfer count

**Transfer summary** (always output with `-v`):
```
sent 123,456 bytes  received 789 bytes  41,415.00 bytes/sec
total size is 987,654  speedup is 8.00
```

### Key files

| File | Role |
|---|---|
| `crates/rsync-core/src/services/progress_parser.rs` | Regex parsing of progress lines and summary |
| `crates/rsync-core/src/services/job_executor.rs` | Event loop that tracks stats and records them |
| `crates/rsync-core/src/services/job_runner.rs` | Spawns rsync, reads stdout/stderr, emits events |
| `crates/rsync-core/src/services/statistics_service.rs` | Record, aggregate, export, reset |
| `crates/rsync-core/src/models/statistics.rs` | `RunStatistic` and `AggregatedStats` structs |
| `crates/rsync-core/src/repository/sqlite/statistics.rs` | SQLite persistence |
| `src/pages/statistics-page.tsx` | Frontend display |

### Tests

```bash
cargo test -p rsync-core -- progress_parser        # Parser unit tests (18)
cargo test -p rsync-core -- progress_statistics     # End-to-end pipeline tests (9)
cargo test -p rsync-core -- statistics_service      # Service + aggregation tests (7)
cargo test -p rsync-core -- sqlite_statistics       # Repository tests (4)
```

### Maintaining

**Adding a new metric:**
1. Add field to `RunStatistic` in `models/statistics.rs`
2. Add column to `run_statistics` table (migration in `database/sqlite/mod.rs`)
3. Update `SqliteStatisticsRepository` to read/write the column
4. Populate it in `job_executor.rs` (parse from rsync output or compute)
5. Add to `AggregatedStats` and update `aggregate()` in `statistics_service.rs`
6. Add to frontend type and display on Statistics page

**Modifying rsync output parsing** — the parser must handle raw integers with commas (`32,768`), human-readable suffixes (`205.18M`), both `to-chk` and `ir-chk`, and optional `(xfr#N, ...)` suffixes. `parse_human_bytes()` is the shared byte-parsing helper.

### Known limitations

- Speedup regex doesn't handle commas (e.g. `4,014.86`). In practice rsync doesn't comma-format speedup.
- No `--stats` parsing. The `xfr#` count and summary line are sufficient.
- If rsync is killed before the summary line, `bytes_transferred` falls back to the last per-file value. Only affects invocation records, not statistics.

---

## Settings

App-level settings stored in a SQLite `settings` table (key-value TEXT pairs, schema v3).

### How it works

```
SQLite DB → SettingsRepository trait → SettingsService (typed methods) → Tauri command → TS wrapper (tauri.ts) → React hook or Settings page
```

### Inventory

**Typed settings** (dedicated getter/setter in SettingsService):

| Setting | DB key | Default | React hook |
|---|---|---|---|
| Log directory | `log_directory` | App data dir `/logs` | — |
| Auto trailing slash | `auto_trailing_slash` | `true` | `useTrailingSlash` |
| NAS auto-detect | `nas_auto_detect` | `true` | `useNasAutoDetect` |

**Grouped settings** (struct-based):

| Group | DB keys | Defaults |
|---|---|---|
| Retention | `max_log_age_days`, `max_history_per_job` | 90 days, 15 per job |
| Dry mode | `dry_mode_itemize_changes`, `dry_mode_checksum` | both `false` |

**Raw key-value** (generic get/set from TS):

| Setting | DB key | Default |
|---|---|---|
| Max itemized changes | `max_itemized_changes` | 50,000 |

### Per-job vs app-level

- **App-level**: `settings` table, managed by `SettingsService`
- **Per-job**: fields on `RsyncOptions` or `JobDefinition` (stored in `jobs` table JSON)
- Some features span both (e.g., NAS: app-level `nas_auto_detect` + per-job `size_only`)

### Key files

| File | Role |
|---|---|
| `crates/rsync-core/src/services/settings_service.rs` | Typed getters/setters |
| `crates/rsync-core/src/repository/sqlite/settings.rs` | SQLite persistence |
| `src-tauri/src/commands.rs` | Tauri command handlers |
| `src/lib/tauri.ts` | TS invoke wrappers |
| `src/hooks/use-trailing-slash.ts`, `use-nas-auto-detect.ts` | React hooks |
| `src/pages/settings-page.tsx` | Settings UI |

### Maintaining

**Adding a simple setting:**
1. `settings_service.rs` — add key constant, default, getter/setter methods
2. `commands.rs` — add Tauri commands
3. `lib.rs` — register in `invoke_handler!`
4. `tauri.ts` — add TS wrappers
5. `src/hooks/` — (optional) create hook if consumed outside Settings page
6. `settings-page.tsx` — add state, load in `useEffect`, render Card

**Adding a grouped setting** — follow the `DryModeSettings` pattern: define a struct, read/write individual keys, mirror as TS interface, single pair of Tauri commands.

**Adding a new `RsyncOptions` field** — 7 places: Rust struct + `Default` impl, `command_builder.rs`, `command_parser.rs`, `command_explainer.rs`, TS interface, `defaults.ts`, `command-preview.ts`.

### UI patterns

- Each setting group gets a `<Card>` with `CardHeader` + `CardContent`
- Toggles use `<Switch>` with immediate save (`setState` + `api.set*()`)
- Text/number inputs use local state with a Save button

---

## NAS / Network Filesystem Detection

Auto-detects when a job source or destination is on a network mount (SMB, NFS, AFP) and enables `--size-only`.

### How it works

1. `FileSystem::filesystem_type()` calls `libc::statfs` (macOS) or parses `/proc/mounts` (Linux) to get the filesystem type string (e.g., `smbfs`, `cifs`, `nfs`)
2. The job form's `useEffect` calls `detectFilesystemType()` on local paths when they change
3. If a network FS is detected and app-level `nas_auto_detect` is enabled, the reducer dispatches `ENABLE_NAS_MODE` to auto-set `size_only: true`
4. An info banner explains why `--size-only` is needed
5. Users can manually toggle `size_only` regardless of detection

**Why `--size-only`**: Flags like `--no-times`, `--no-perms`, `--no-owner`, `--no-group` only prevent *setting* attributes — they don't stop rsync from *comparing* them. Only `--size-only` skips timestamp/permission comparison entirely.

### Key files

| File | Role |
|---|---|
| `crates/rsync-core/src/file_system/real_file_system.rs` | Platform-specific FS type detection |
| `src-tauri/src/commands.rs` | `detect_filesystem_type` command |
| `src/components/jobs/job-form.tsx` | Auto-detection effect + `ENABLE_NAS_MODE` reducer |
| `src/components/jobs/rsync-options-field.tsx` | NAS toggle + info banner |

---

## Job Execution

Orchestrates rsync process lifecycle: spawn, stream output, track progress, record results.

### How it works

1. `JobExecutor::execute()` builds rsync args, creates an invocation record, spawns rsync
2. `job_runner.rs` reads stdout/stderr in separate threads, parsing progress and itemized changes
3. A background thread in `job_executor.rs` processes all events, writes to log file, emits to frontend
4. On completion: updates invocation, records statistics (if successful), records snapshot (if snapshot mode), applies retention

### Key files

| File | Role |
|---|---|
| `crates/rsync-core/src/services/job_executor.rs` | Execution orchestration |
| `crates/rsync-core/src/services/job_runner.rs` | Process spawn + stdout/stderr streaming |
| `crates/rsync-core/src/services/command_builder.rs` | Builds rsync argument vector |
| `crates/rsync-core/src/services/execution_handler.rs` | `ExecutionEventHandler` trait |
| `crates/rsync-core/src/services/running_jobs.rs` | Thread-safe running process map |
| `src-tauri/src/execution.rs` | GUI event handler (Tauri emit) |

---

## Scheduling

Cron and interval-based job scheduling with pluggable backends.

### How it works

- `ScheduleConfig` on each job defines cron expression or interval
- `SchedulerBackend` trait with `InProcessScheduler` implementation (background check loop)
- `is_job_due()` + `next_run_time()` evaluate schedule against last run time
- Both GUI (system tray loop) and TUI use the same scheduler

### Key files

| File | Role |
|---|---|
| `crates/rsync-core/src/services/scheduler.rs` | `is_job_due()`, `next_run_time()` |
| `crates/rsync-core/src/services/scheduler_backend.rs` | `SchedulerBackend` trait + `InProcessScheduler` |
| `crates/rsync-core/src/models/schedule.rs` | `ScheduleConfig`, `ScheduleType` |
| `src/components/jobs/schedule-field.tsx` | Schedule form UI |

---

## Snapshot Backups

Timestamped backup directories with `--link-dest` for space-efficient incremental backups.

### How it works

- `BackupMode::Snapshot` creates dated subdirectories under the destination
- `--link-dest` points to the previous snapshot (hardlinks unchanged files)
- `retention.rs` groups snapshots by daily/weekly/monthly and prunes excess
- Snapshot records are stored in the `snapshots` table

### Key files

| File | Role |
|---|---|
| `crates/rsync-core/src/services/retention.rs` | `compute_snapshots_to_delete()` |
| `crates/rsync-core/src/services/retention_runner.rs` | `run_history_retention()` |
| `crates/rsync-core/src/models/backup.rs` | `SnapshotRecord` |

---

## Itemized Changes (Dry Mode)

Parses `--itemize-changes` output during dry runs to show a per-file change summary.

### How it works

- Dry mode settings (`DryModeSettings`) control whether `--itemize-changes` and `--checksum` are added
- `itemize_parser.rs` parses the 11-character rsync itemize format (e.g., `>f..T.......`)
- Changes are streamed to the frontend via `ExecutionEvent::ItemizedChange`
- `ItemizedChangesTable` displays results with filtering and virtualization, capped at `max_itemized_changes`

### Key files

| File | Role |
|---|---|
| `crates/rsync-core/src/services/itemize_parser.rs` | Parses itemize output |
| `crates/rsync-core/src/models/itemize.rs` | `ItemizedChange` struct |
| `src/components/jobs/itemized-changes-table.tsx` | Frontend table |

---

## Command Parser & Explainer

Parses rsync command strings and explains what each flag does.

### How it works

- `command_parser.rs` tokenizes an rsync command string into `ParsedCommand` (flags, source, destination)
- `command_explainer.rs` maps each flag to a human-readable description
- `ParsedCommand::to_job_definition()` converts to a `JobDefinition` for import-as-job
- Tools page exposes both parsing and import functionality

### Key files

| File | Role |
|---|---|
| `crates/rsync-core/src/services/command_parser.rs` | `parse_rsync_command()` |
| `crates/rsync-core/src/services/command_explainer.rs` | Flag registry + `explain_command()` |
| `src/pages/tools-page.tsx` | Tools page UI |

---

## Themes & Appearance

8-color theme system with light/dark/system appearance modes.

### How it works

- Themes swap CSS variables (stored in localStorage)
- Appearance mode (`light`/`dark`/`system`) toggles the `dark` class on `<html>`
- `useTheme` hook manages both theme and appearance state

### Key files

| File | Role |
|---|---|
| `src/lib/themes.ts` | Theme definitions (name, CSS variables, preview color) |
| `src/hooks/use-theme.ts` | `useTheme` hook |
| `src/index.css` | CSS variable declarations |
