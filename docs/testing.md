# Testing

## Overview

All tests live in `crates/rsync-core/src/tests/`. The test suite validates domain logic, data persistence, and rsync behavior simulation without requiring a real rsync binary or filesystem.

## Running Tests

```bash
# Run all tests
cargo test -p rsync-core

# Run a specific test
cargo test -p rsync-core -- test_basic_sync_copies_files

# Run tests with output
cargo test -p rsync-core -- --nocapture

# Run tests matching a pattern
cargo test -p rsync-core -- test_rsync

# TypeScript type checking (no runtime tests yet)
npx tsc --noEmit
```

## Test Count

64 tests across 7 test modules:

| Module | Tests | What it covers |
|--------|-------|----------------|
| `command_builder::tests` | 10 | Flag ordering, SSH args, path formatting, excludes, bandwidth |
| `test_file_system_tests` | 14 | In-memory FS: CRUD, symlinks, hard links, inodes, walk_dir |
| `test_rsync_client_tests` | 16 | Simulated rsync: sync, delete, dry-run, link-dest, excludes, backup-dir |
| `sqlite_job_repository_tests` | 4 | Job CRUD in SQLite |
| `sqlite_invocation_repository_tests` | 4 | Invocation CRUD + cascade deletes |
| `sqlite_snapshot_repository_tests` | 4 | Snapshot CRUD + latest lookup + cascades |
| `job_service_integration_tests` | 12 | Full service lifecycle, validation, history limits, cascading deletes |

## Test Infrastructure

### TestFileSystem

An in-memory filesystem implementation (`tests/test_file_system.rs`) that implements the `FileSystem` trait.

Features:
- `HashMap<PathBuf, FsNode>` storage (files, directories, symlinks)
- Inode tracking for hard-link simulation
- Thread-safe via `Mutex<Inner>`
- `walk_dir`, `create_dir_all`, `remove_dir_all`, `hard_link`, `symlink`
- `available_space()` returns a configurable value

Usage:
```rust
let fs = TestFileSystem::new();
fs.create_dir_all(Path::new("/src"))?;
fs.write(Path::new("/src/file.txt"), b"hello")?;
```

### TestRsyncClient

A simulated rsync client (`tests/test_rsync_client.rs`) that implements the `RsyncClient` trait using `TestFileSystem`.

Simulated behaviors:
- Basic file synchronization (copies files from source to dest)
- `--delete` flag (removes extraneous files in destination)
- `--exclude` patterns (skips matching files)
- `--link-dest` (hard-links unchanged files, copies changed ones)
- `--backup-dir` (moves replaced files to backup directory)
- `--dry-run` (no filesystem modifications, reports what would change)
- Command recording (tracks all executed commands)
- Forced errors (inject errors for testing error handling)

Usage:
```rust
let fs = TestFileSystem::new();
let client = TestRsyncClient::new(fs.clone());
let args = build_rsync_args(&source, &dest, &options, None, None);
let result = client.execute(&args)?;
```

### SQLite Test Helpers

SQLite repository tests use `tempfile::NamedTempFile` to create isolated databases per test. Each test gets a fresh database with schema auto-created.

```rust
fn setup() -> (Database, /* repos */) {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let db = Database::open(tmp.path().to_str().unwrap()).unwrap();
    // ...
}
```

## Adding Tests

1. Add test functions in the appropriate `*_tests.rs` module
2. Use existing test helpers from `tests/test_helpers.rs`
3. For new test categories, create a new module in `tests/` and register it in `tests/mod.rs`
4. Run `cargo test -p rsync-core` to verify

## What's Not Tested Yet

- Frontend components (no React test framework configured)
- Tauri IPC commands (integration testing with Tauri is complex)
- `ProcessRsyncClient` (requires real rsync binary)
- `RealFileSystem` (requires real filesystem access)
