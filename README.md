# Rsync Studio

A cross-platform desktop application for managing rsync backup jobs. Built with Tauri v2, React, TypeScript, and Rust.

## Features

- Create and manage rsync backup jobs with a visual interface
- Support for local, SSH, and rsync daemon storage locations
- Multiple backup modes: Mirror, Versioned, and Snapshot with retention policies
- Live rsync command preview as you configure jobs
- Full control over rsync flags, exclude/include patterns, and bandwidth limits
- SSH configuration management (port, identity files, host key checking)
- SQLite-based job persistence

## Screenshots

*Coming soon*

## Quick Start

### Prerequisites

- [Rsync](https://github.com/RsyncProject/rsync) (a relatively recent version)
- [Rust](https://rustup.rs) (stable)
- [Node.js](https://nodejs.org) (18+)
- Platform-specific Tauri v2 dependencies (see [setup guide](docs/setup.md))

### Install & Run

```bash
# Install frontend dependencies
npm install

# Run in development mode
npm run tauri dev
```

See [docs/setup.md](docs/setup.md) for detailed setup instructions.

## Architecture

Rsync Studio is a Cargo workspace with three crates:

| Crate | Role |
|-------|------|
| `rsync-core` | Shared library — models, traits, services, command builder, SQLite persistence |
| `rsync-gui` (`src-tauri/`) | Tauri application shell — IPC commands, state management |
| `rsync-tui` | Future terminal UI (stub) |

The React frontend communicates with the Rust backend through Tauri's IPC layer. All business logic lives in `rsync-core` so it can be shared between the GUI and future TUI.

See [docs/architecture.md](docs/architecture.md) for the full architectural overview.

## Development

```bash
# Build all Rust crates
cargo build --workspace

# Run tests (64 tests)
cargo test -p rsync-core

# TypeScript type check
npx tsc --noEmit

# Development mode with hot reload
npm run tauri dev

# Production build
npm run tauri build
```

## Tech Stack

- **Backend**: Rust, Tauri v2, rusqlite, serde, chrono, uuid, thiserror
- **Frontend**: React 18, TypeScript 5.6, Vite 6, Tailwind CSS 3, shadcn/ui, Radix UI, Lucide icons
- **Testing**: Rust unit tests with in-memory fakes (TestFileSystem, TestRsyncClient)

## Documentation

- [Setup Guide](docs/setup.md) — prerequisites, installation, running the app
- [Architecture](docs/architecture.md) — crate structure, data flow, design decisions
- [Testing](docs/testing.md) — test infrastructure, running tests, adding tests
- [Contributing](CONTRIBUTING.md) — how to contribute

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
