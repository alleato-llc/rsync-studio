# Development Setup

## Prerequisites

- **Rust** (stable, latest recommended) — [rustup.rs](https://rustup.rs)
- **Node.js** (18+ recommended) + npm
- **System dependencies for Tauri v2:**
  - **macOS**: Xcode Command Line Tools (`xcode-select --install`)
  - **Linux (Debian/Ubuntu)**:
    ```bash
    sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
      libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
    ```
  - **Linux (Fedora)**:
    ```bash
    sudo dnf install webkit2gtk4.1-devel openssl-devel curl wget file \
      libxdo-devel libappindicator-gtk3-devel librsvg2-devel
    ```

For the full list of platform requirements, see the [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/).

## Getting Started

```bash
# Clone the repository
git clone <repository-url>
cd rsync-studio

# Install frontend dependencies
npm install

# Build all Rust crates (validates workspace compiles)
cargo build --workspace

# Run tests
cargo test -p rsync-core

# TypeScript type check
npx tsc --noEmit
```

## Running the App

### Desktop GUI (Development Mode)

```bash
# Start Tauri dev mode (Vite frontend + Rust backend with hot reload)
npm run tauri dev
```

This starts:
- Vite dev server on `http://localhost:1420`
- Tauri window loading from the dev server
- Rust backend with automatic rebuild on changes to `src-tauri/`

### Desktop GUI (Production Build)

```bash
npm run tauri build
```

Produces platform-specific installers in `src-tauri/target/release/bundle/`.

### Terminal UI

The TUI requires only Rust and rsync — no Node.js, no Tauri, no display server.

```bash
# Build
cargo build -p rsync-commander --release

# Launch interactive TUI
./target/release/rsync-commander

# List configured jobs
./target/release/rsync-commander list

# Run a single job non-interactively (for cron/systemd)
./target/release/rsync-commander run <job-id>

# Use a custom database location
./target/release/rsync-commander --db-path /path/to/rsync-studio.db

# Use a custom log directory
./target/release/rsync-commander --log-dir /var/log/rsync-studio
```

The TUI shares the same SQLite database as the GUI, so jobs created in one are visible in the other.

**Headless server deployment**: Copy just the `rsync-commander` binary to the server. No other files are needed — the database and log directory are created automatically on first run at `~/.local/share/rsync-studio/`.

## Project Structure Quick Reference

```
rsync-studio/
├── crates/rsync-core/     # Shared Rust library (all domain logic)
├── crates/rsync-commander/ # Terminal UI (ratatui + crossterm)
├── src-tauri/             # Tauri GUI shell (thin wrapper over rsync-core)
├── src/                   # React + TypeScript frontend
├── Cargo.toml             # Workspace configuration
├── package.json           # Node.js dependencies and scripts
├── vite.config.ts         # Vite bundler configuration
├── tsconfig.json          # TypeScript configuration
├── tailwind.config.js     # Tailwind CSS configuration
└── components.json        # shadcn/ui configuration
```

## Common Tasks

| Task | Command |
|------|---------|
| Install JS dependencies | `npm install` |
| Build Rust workspace | `cargo build --workspace` |
| Run Rust tests | `cargo test -p rsync-core` |
| TypeScript check | `npx tsc --noEmit` |
| Dev mode (GUI) | `npm run tauri dev` |
| Production build (GUI) | `npm run tauri build` |
| Build TUI | `cargo build -p rsync-commander --release` |
| Run TUI | `cargo run -p rsync-commander` |
| Add shadcn component | `npx shadcn@latest add <component>` |
| Frontend only (no Rust) | `npm run dev` |

## IDE Setup

### VS Code

Recommended extensions:
- rust-analyzer
- Tailwind CSS IntelliSense
- ES7+ React/Redux/React-Native snippets

The project uses `@/` path aliases — `tsconfig.json` and `vite.config.ts` are already configured for this.

## Troubleshooting

**Rust build fails with SQLite errors:**
`rsync-core` uses `rusqlite` with the `bundled` feature, which compiles SQLite from source. Ensure you have a C compiler installed (`gcc`, `clang`, or MSVC).

**Tauri dev mode shows blank window:**
Make sure the Vite dev server is running on port 1420. Check that `src-tauri/tauri.conf.json` has `"devUrl": "http://localhost:1420"`.

**TypeScript errors after modifying Rust models:**
TypeScript types in `src/types/` must be manually kept in sync with Rust models in `crates/rsync-core/src/models/`. Update both when changing data structures.
