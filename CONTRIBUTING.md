# Contributing

Thank you for your interest in contributing to Rsync Studio! This guide covers how to get started, the development workflow, and conventions to follow.

## Getting Started

1. Fork and clone the repository
2. Follow the [setup guide](docs/setup.md) to install prerequisites
3. Run `npm install` and `cargo build --workspace` to verify everything builds
4. Run `cargo test -p rsync-core` to confirm all tests pass

## Development Workflow

### Before Making Changes

1. Create a feature branch from `main`
2. Understand the [architecture](docs/architecture.md) — especially which crate your changes belong in

### Where to Put Code

| Type of change | Location |
|----------------|----------|
| Domain models, business logic, traits | `crates/rsync-core/` |
| Tauri IPC commands, app state | `src-tauri/` |
| React components, pages, hooks | `src/` |
| TypeScript types (mirroring Rust models) | `src/types/` |
| shadcn/ui components | `src/components/ui/` (auto-generated, do not edit) |

### Key Principle

Keep `src-tauri/` thin. Business logic belongs in `rsync-core` so it can be shared with the future TUI.

### Making Changes

1. Write your code following the conventions below
2. Add tests for new Rust logic in `crates/rsync-core/src/tests/`
3. Run the verification suite before submitting:

```bash
npx tsc --noEmit               # TypeScript compiles
cargo build --workspace        # Rust workspace builds
cargo test -p rsync-core       # All tests pass
```

### Submitting a Pull Request

1. Keep PRs focused — one feature or fix per PR
2. Write a clear description of what changed and why
3. Ensure all verification checks pass

## Code Conventions

### Rust

- All models derive `Debug, Clone, Serialize, Deserialize, PartialEq`
- Use `thiserror` for error types
- Use `?` for error propagation — avoid `unwrap()` in production code
- Traits go in `crates/rsync-core/src/traits/`
- Implementations go in `crates/rsync-core/src/implementations/`
- Repository traits use `Send + Sync` bounds

### TypeScript

- Types in `src/types/` must mirror Rust models exactly (field names, union shapes)
- Use `@/` path alias for imports
- Use `useReducer` for complex form state, `useState` for simple state
- Tauri invoke wrappers live in `src/lib/tauri.ts`

### General

- Keep functions focused and small
- Prefer explicit over clever
- No unnecessary dependencies — check if existing tools cover the need

## Adding a shadcn/ui Component

```bash
npx shadcn@latest add <component-name>
```

Do not manually edit files in `src/components/ui/`.

## Test Guidelines

- All domain logic should have tests
- Use `TestFileSystem` and `TestRsyncClient` for rsync-related tests
- SQLite tests use `tempfile` for isolated databases
- See [docs/testing.md](docs/testing.md) for details

## Keeping Types in Sync

When modifying a Rust model in `crates/rsync-core/src/models/`, update the corresponding TypeScript type in `src/types/`. Tauri serializes Rust structs as JSON — field names must match exactly (using snake_case).

If you change `command_builder.rs`, also update the TypeScript mirror in `src/lib/command-preview.ts`.

## Questions?

Open an issue for questions or to discuss proposed changes before starting work.
