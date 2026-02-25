# Rsync Studio

[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Built with Claude](https://img.shields.io/badge/Built%20with-Claude-blueviolet)](https://claude.ai)

A cross-platform desktop application for managing rsync backup jobs. Includes both a graphical desktop app (Tauri v2 + React) and a terminal UI for headless servers. Built with Rust, TypeScript, and a shared core library.

## Features

- Create and manage rsync backup jobs with a visual interface
- **Two frontends**: Desktop GUI (Tauri + React) and Terminal UI (ratatui)
- Support for local, SSH, and rsync daemon storage locations
- Multiple backup modes: Mirror, Versioned, and Snapshot with retention policies
- Live rsync command preview as you configure jobs
- Full control over rsync flags, exclude/include patterns, and bandwidth limits
- SSH configuration management (port, identity files, host key checking)
- Job scheduling (cron expressions and interval-based)
- Run statistics tracking and export
- rsync command explainer and log scrubber tools
- SQLite-based job persistence (shared between GUI and TUI)

## Screenshots

*Coming soon*

## Quick Start

### Prerequisites

- [Rsync](https://github.com/RsyncProject/rsync) (a relatively recent version)
- [Rust](https://rustup.rs) (stable)
- [Node.js](https://nodejs.org) (18+) — only needed for the GUI
- Platform-specific Tauri v2 dependencies (see [setup guide](docs/setup.md)) — only needed for the GUI

### Desktop GUI

```bash
npm install
npm run tauri dev
```

### Terminal UI

The TUI has no Node.js or Tauri dependencies — just Rust and rsync.

```bash
# Build
cargo build -p rsync-tui --release

# Launch interactive TUI
./target/release/rsync-tui

# Or run directly with cargo
cargo run -p rsync-tui
```

See [Terminal UI](#terminal-ui-1) below for full usage, or [docs/setup.md](docs/setup.md) for detailed setup instructions.

## Architecture

Rsync Studio is a Cargo workspace with three crates:

| Crate | Role |
|-------|------|
| `rsync-core` | Shared library — models, traits, services, job execution, SQLite persistence |
| `rsync-gui` (`src-tauri/`) | Tauri desktop app — IPC commands, state management, system tray |
| `rsync-tui` | Terminal UI — ratatui + crossterm, runs on headless servers |

Both frontends share the same `rsync-core` library, database, and job executor. The `ExecutionEventHandler` trait allows the GUI to emit Tauri events while the TUI uses mpsc channels — same execution logic, different event delivery.

See [docs/architecture.md](docs/architecture.md) for the full architectural overview.

## Terminal UI

The TUI provides the same feature set as the desktop GUI but runs entirely in the terminal — ideal for headless servers, SSH sessions, and environments without a display server.

### Usage

```bash
rsync-tui                          # Launch interactive TUI
rsync-tui list                     # List all jobs (non-interactive)
rsync-tui run <job-id>             # Run a single job (non-interactive, for cron/systemd)
rsync-tui --db-path <path>         # Custom database location
rsync-tui --log-dir <path>         # Custom log directory
```

### Pages

| Key | Page | Description |
|-----|------|-------------|
| `1` | Jobs | List, create, edit, run, dry-run, cancel, delete jobs |
| `2` | History | Browse invocation history, view log files |
| `3` | Stats | Aggregated and per-job run statistics |
| `4` | Tools | rsync command explainer and log scrubber |
| `5` | Settings | Log directory, retention, theme, export/import |
| `6` | About | Version and build information |

### Keybindings

| Key | Action |
|-----|--------|
| `1`-`6` | Switch pages |
| `Tab` / `Shift+Tab` | Cycle pages |
| `j` / `k` | Navigate up/down |
| `q` / `Ctrl+C` | Quit |
| `?` | Help popup |

**Jobs page**: `n` new, `Enter` edit, `r` run, `d` dry-run, `c` cancel, `x` delete, `o` view output, `/` search

**Output viewer**: `j`/`k` scroll, `g`/`G` top/bottom, `f` toggle follow, `PgUp`/`PgDn` page scroll, `c` cancel, `Esc` close

**History**: `Enter` view log, `d` delete invocation

**Statistics**: `r` reset, `e` export

### Themes

Four built-in color schemes, changeable from the Settings page: **Default**, **Dark**, **Solarized**, **Nord**.

### Headless Execution

The `run` subcommand executes a single job and exits, making it suitable for cron or systemd timers:

```bash
# In a crontab:
0 2 * * * /usr/local/bin/rsync-tui run 550e8400-e29b-41d4-a716-446655440000

# Or with a systemd timer
[Service]
Type=oneshot
ExecStart=/usr/local/bin/rsync-tui run 550e8400-e29b-41d4-a716-446655440000
```

### Shared Database

The GUI and TUI share the same SQLite database. Jobs created in one frontend are visible in the other. The TUI automatically detects the GUI's database if it exists:

| Platform | GUI path | TUI standalone path |
|----------|----------|---------------------|
| macOS | `~/Library/Application Support/com.rsync-studio.app/rsync-studio.db` | `~/Library/Application Support/rsync-studio/rsync-studio.db` |
| Linux | `~/.local/share/com.rsync-studio.app/rsync-studio.db` | `~/.local/share/rsync-studio/rsync-studio.db` |

If the GUI database exists, the TUI uses it automatically. Use `--db-path` to override.

## Development

```bash
# Build all Rust crates
cargo build --workspace

# Run tests (191 tests)
cargo test -p rsync-core

# TypeScript type check
npx tsc --noEmit

# Development mode with hot reload (GUI)
npm run tauri dev

# Production build (GUI)
npm run tauri build

# Run the TUI
cargo run -p rsync-tui
```

## Tech Stack

- **Core**: Rust, rusqlite, serde, chrono, uuid, thiserror, croner
- **GUI**: Tauri v2, React 18, TypeScript 5.6, Vite 6, Tailwind CSS 3, shadcn/ui, Radix UI, Lucide icons
- **TUI**: ratatui 0.29, crossterm 0.28, clap 4
- **Testing**: Rust unit tests with in-memory fakes (TestFileSystem, TestRsyncClient)

## Documentation

- [Setup Guide](docs/setup.md) — prerequisites, installation, running the app
- [Architecture](docs/architecture.md) — crate structure, data flow, design decisions
- [Testing](docs/testing.md) — test infrastructure, running tests, adding tests
- [Contributing](CONTRIBUTING.md) — how to contribute

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
