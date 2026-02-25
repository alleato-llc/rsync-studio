# Database Documentation

## Overview

Rsync Studio uses **SQLite** for all persistent storage. The database is created automatically on first launch in the application data directory.

- **macOS**: `~/Library/Application Support/com.rsync-studio.app/rsync-studio.db`
- **Linux**: `~/.local/share/com.rsync-studio.app/rsync-studio.db`

## Connection Details

- **Journal mode**: WAL (Write-Ahead Logging) for concurrent read/write performance
- **Foreign keys**: Enabled (`PRAGMA foreign_keys=ON`)
- **Access pattern**: `Arc<Mutex<Connection>>` shared across all repository implementations
- **Library**: `rusqlite` (Rust bindings for SQLite)

## External Access

To inspect the database while the application is running:

```bash
# Read-only access (safe while app is running due to WAL mode)
sqlite3 -readonly ~/Library/Application\ Support/com.rsync-studio.app/rsync-studio.db

# Check schema version
SELECT * FROM schema_version;

# List all jobs
SELECT id, name, enabled FROM jobs;

# Recent invocations
SELECT id, job_id, started_at, status FROM invocations ORDER BY started_at DESC LIMIT 10;
```

> **Warning**: Do not modify the database while the application is running. WAL mode supports concurrent readers but only one writer.

## Migration System

Migrations are embedded SQL files applied sequentially on startup. The `schema_version` table tracks which migrations have been applied.

| Version | File | Description |
|---------|------|-------------|
| 1 | `v001_initial.sql` | Core tables: jobs, invocations, snapshots |
| 2 | `v002_run_statistics.sql` | Run statistics tracking |
| 3 | `v003_settings.sql` | Key-value settings store |

Migration logic in `crates/rsync-core/src/implementations/database.rs`:
1. Create `schema_version` table if it doesn't exist
2. Read `MAX(version)` from `schema_version`
3. Apply each migration where version > current, recording each in `schema_version`

## Entity-Relationship Diagram

```
+------------------+       +--------------------+       +-------------------+
|     jobs         |       |   invocations      |       |    snapshots      |
|------------------|       |--------------------|       |-------------------|
| id          PK   |<──┐   | id            PK   |<──┐   | id           PK   |
| name             |   │   | job_id        FK───|───┘   | job_id       FK───|──> jobs
| description      |   │   | started_at         |   ┌───| invocation_id FK  |
| source      JSON |   │   | finished_at        |   │   | snapshot_path     |
| destination JSON |   │   | status        JSON |   │   | link_dest_path    |
| backup_mode JSON |   │   | bytes_transferred  |   │   | created_at        |
| options     JSON |   │   | files_transferred  |   │   | size_bytes        |
| ssh_config  JSON |   │   | total_files        |   │   | file_count        |
| schedule    JSON |   │   | snapshot_path      |   │   | is_latest         |
| enabled          |   │   | command_executed   |   │   +-------------------+
| created_at       |   │   | exit_code          |   │
| updated_at       |   │   | trigger       JSON |   │   +-------------------+
+------------------+   │   | log_file_path      |   │   | run_statistics    |
                       │   +--------------------+   │   |-------------------|
                       │                            │   | id           PK   |
                       │                            │   | job_id       FK───|──> jobs
                       └────────────────────────────│───| invocation_id FK  |
                                                    │   | recorded_at       |
                                                    │   | files_transferred |
                                                    │   | bytes_transferred |
                                                    │   | duration_secs     |
                                                    │   | speedup           |
                                                    │   +-------------------+
                                                    │
+------------------+   +-------------------+        │
| schema_version   |   |    settings       |        │
|------------------|   |-------------------|        │
| version     PK   |   | key          PK   |        │
| applied_at       |   | value             |        │
+------------------+   +-------------------+        │
```

## Table Descriptions

### `schema_version`

Tracks applied database migrations.

| Column | Type | Description |
|--------|------|-------------|
| `version` | INTEGER PK | Migration version number |
| `applied_at` | TEXT | ISO 8601 timestamp of when the migration was applied |

### `jobs`

Stores job definitions. Complex fields are stored as JSON.

| Column | Type | Nullable | Description |
|--------|------|----------|-------------|
| `id` | TEXT PK | No | UUID v4 |
| `name` | TEXT | No | Human-readable job name |
| `description` | TEXT | Yes | Optional description |
| `source` | TEXT | No | JSON `StorageLocation` (Local, RemoteSsh, RemoteRsync) |
| `destination` | TEXT | No | JSON `StorageLocation` |
| `backup_mode` | TEXT | No | JSON `BackupMode` (Mirror or Snapshot with retention policy) |
| `options` | TEXT | No | JSON `RsyncOptions` (flags, excludes, etc.) |
| `ssh_config` | TEXT | Yes | JSON `SshConfig` (port, identity file) |
| `schedule` | TEXT | Yes | JSON `Schedule` (cron/interval, enabled flag) |
| `enabled` | INTEGER | No | 1 = enabled, 0 = disabled |
| `created_at` | TEXT | No | ISO 8601 timestamp |
| `updated_at` | TEXT | No | ISO 8601 timestamp |

### `invocations`

Records each execution of a job (manual or scheduled).

| Column | Type | Nullable | Description |
|--------|------|----------|-------------|
| `id` | TEXT PK | No | UUID v4 |
| `job_id` | TEXT FK | No | References `jobs(id)` ON DELETE CASCADE |
| `started_at` | TEXT | No | ISO 8601 timestamp |
| `finished_at` | TEXT | Yes | ISO 8601 timestamp (null while running) |
| `status` | TEXT | No | JSON enum: "Running", "Succeeded", "Failed", "Cancelled" |
| `bytes_transferred` | INTEGER | No | Bytes transferred by rsync |
| `files_transferred` | INTEGER | No | Number of files transferred |
| `total_files` | INTEGER | No | Total files considered |
| `snapshot_path` | TEXT | Yes | Path to snapshot directory (snapshot mode only) |
| `command_executed` | TEXT | No | Full rsync command string |
| `exit_code` | INTEGER | Yes | rsync exit code (null while running or if killed) |
| `trigger` | TEXT | No | JSON enum: "Manual", "Scheduled" |
| `log_file_path` | TEXT | Yes | Path to the log file on disk |

**Index**: `idx_invocations_job_id` on `job_id`

### `snapshots`

Records snapshot backups created by snapshot-mode jobs.

| Column | Type | Nullable | Description |
|--------|------|----------|-------------|
| `id` | TEXT PK | No | UUID v4 |
| `job_id` | TEXT FK | No | References `jobs(id)` ON DELETE CASCADE |
| `invocation_id` | TEXT FK | No | References `invocations(id)` ON DELETE CASCADE |
| `snapshot_path` | TEXT | No | Full path to the snapshot directory |
| `link_dest_path` | TEXT | Yes | Path used for --link-dest (previous snapshot) |
| `created_at` | TEXT | No | ISO 8601 timestamp |
| `size_bytes` | INTEGER | No | Total bytes in the snapshot |
| `file_count` | INTEGER | No | Number of files in the snapshot |
| `is_latest` | INTEGER | No | 1 = this is the most recent snapshot for the job |

**Index**: `idx_snapshots_job_id` on `job_id`

### `run_statistics`

Per-invocation performance metrics for successful runs.

| Column | Type | Nullable | Description |
|--------|------|----------|-------------|
| `id` | TEXT PK | No | UUID v4 |
| `job_id` | TEXT FK | No | References `jobs(id)` ON DELETE CASCADE |
| `invocation_id` | TEXT FK | No | References `invocations(id)` ON DELETE CASCADE |
| `recorded_at` | TEXT | No | ISO 8601 timestamp |
| `files_transferred` | INTEGER | No | Files transferred in this run |
| `bytes_transferred` | INTEGER | No | Bytes transferred in this run |
| `duration_secs` | REAL | No | Wall-clock duration in seconds |
| `speedup` | REAL | Yes | rsync speedup factor (null if not reported) |

**Index**: `idx_run_statistics_job_id` on `job_id`

### `settings`

Key-value store for application settings.

| Column | Type | Nullable | Description |
|--------|------|----------|-------------|
| `key` | TEXT PK | No | Setting key (e.g., "log_directory", "max_log_age_days") |
| `value` | TEXT | No | Setting value as a string |

**Known keys**:
- `log_directory` — Custom path for log file storage
- `max_log_age_days` — Maximum age (days) before invocations are auto-pruned (default: 90)
- `max_history_per_job` — Maximum invocations kept per job (default: 15)

## Cascade Behavior

All foreign keys use `ON DELETE CASCADE`:

- Deleting a **job** automatically deletes all its invocations, snapshots, and run statistics
- Deleting an **invocation** automatically deletes its associated snapshot record and run statistic
- The application also cleans up log files on disk when deleting invocations through the UI or retention system
