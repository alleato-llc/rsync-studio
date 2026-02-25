# Release Process

## Overview

Rsync Studio uses [release-please](https://github.com/googleapis/release-please) for automated versioning and releases. When commits following [Conventional Commits](https://www.conventionalcommits.org/) are pushed to `main`, release-please creates a pull request that bumps versions across the workspace and updates `CHANGELOG.md`. Merging that PR triggers a GitHub Release with pre-built TUI binaries and Tauri desktop app bundles for all supported platforms.

## Repository Setup

In your GitHub repository settings:

1. Go to **Settings → Actions → General**
2. Under "Workflow permissions", enable **"Allow GitHub Actions to create and approve pull requests"**

This is required for release-please to create and update release PRs.

## How the Pipeline Works

```
Push to main (conventional commits)
        │
        ▼
   CI workflow runs
   (test + build-tui + build-desktop)
        │
        ▼ (on success)
   Release workflow triggers
        │
        ▼
   release-please creates/updates PR
   (bumps package.json + Cargo.toml versions + CHANGELOG.md)
        │
        ▼ (PR merged)
   release-please creates GitHub Release
        │
        ▼
   upload-assets job builds TUI binaries
   + Tauri desktop bundles and attaches
   them to the release
```

## Configuration Files

| File | Purpose |
|------|---------|
| `release-please-config.json` | Release-please behavior: release type, version bump rules, extra files to update |
| `.release-please-manifest.json` | Current version tracker (updated automatically by release-please) |
| `.github/workflows/ci.yml` | CI pipeline: test + multi-platform TUI + desktop builds |
| `.github/workflows/release.yml` | Release automation: release-please + asset upload |

## CI Jobs

The CI workflow has three jobs:

| Job | Runner | What it does |
|-----|--------|-------------|
| `test` | `ubuntu-latest` | Rust tests (`cargo test -p rsync-core`), TypeScript type check (`npx tsc --noEmit`), workspace build |
| `build-tui` | matrix (macOS + Linux) | Builds `rsync-commander` release binary |
| `build-desktop` | matrix (macOS + Linux) | Builds Tauri app bundles (`.dmg`, `.deb`, `.AppImage`) |

## Platform Matrix

### TUI Binary (`rsync-commander`)

| Platform | Runner | Artifact Name | Architecture |
|----------|--------|---------------|--------------|
| macOS | `macos-latest` | `rsync-commander-darwin-arm64` | Apple Silicon (arm64) |
| Linux | `ubuntu-latest` | `rsync-commander-linux-amd64` | x86_64 |

### Desktop App (Rsync Studio)

| Platform | Runner | Bundle Formats | Architecture |
|----------|--------|---------------|--------------|
| macOS | `macos-latest` | `.dmg` | Apple Silicon (arm64) |
| Linux | `ubuntu-latest` | `.deb`, `.AppImage` | x86_64 |

## Artifact Retention

| Branch Type | Retention |
|-------------|-----------|
| `feature/*` | 1 day |
| Pull request | 7 days |
| `main` | 90 days |

## Version Management

- `release-type: node` in `release-please-config.json` uses `package.json` as the canonical version source
- The `extra-files` list tells release-please to also bump versions in:
  - `src-tauri/tauri.conf.json`
  - `crates/rsync-core/Cargo.toml`
  - `crates/rsync-commander/Cargo.toml`
  - `src-tauri/Cargo.toml`
- This keeps all crate versions and the Tauri app version in sync with `package.json`

## Conventional Commits

Release-please determines version bumps from commit messages:

| Commit Type | Version Bump | Example |
|-------------|-------------|---------|
| `feat:` | Minor (0.x.0) | `feat: add snapshot retention policy` |
| `fix:` | Patch (0.0.x) | `fix: handle empty directory in backup` |
| `feat!:` or `BREAKING CHANGE:` | Major (x.0.0) | `feat!: change job definition schema` |
| `docs:`, `chore:`, `ci:`, `test:`, `refactor:` | No bump | `docs: update README` |

## Local Hook Setup

A `commit-msg` git hook validates that commit messages follow Conventional Commits format before they reach the remote. Run the setup script once after cloning:

```bash
./scripts/setup-hooks.sh
```

This sets `core.hooksPath` to the `hooks/` directory in the repository. The hook rejects messages that don't match `<type>[scope][!]: <description>` and allows merge/revert commits through unchanged.

## Troubleshooting

### release-please PR not created

- Verify conventional commit messages on `main`
- Check that the CI workflow completed successfully (release workflow triggers on CI success)
- Ensure workflow permissions are configured (see Repository Setup)

### Binary upload fails

- Check that the release tag exists (created by release-please)
- Verify `gh` CLI authentication in the workflow (`GITHUB_TOKEN` is provided automatically)
- Use `--clobber` flag on `gh release upload` to overwrite existing assets on retry

### Tauri build fails on Linux

The CI installs these system dependencies automatically, but if building locally on a fresh machine you'll need:

```bash
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  librsvg2-dev \
  patchelf \
  libssl-dev \
  libgtk-3-dev
```
