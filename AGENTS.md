# AGENTS.md

This file provides guidance to AI coding agents (Claude Code, GitHub Copilot, Gemini CLI, etc.) when working with code in this repository.

## Build & Development Commands

```bash
cargo build              # Build
cargo run                # Run TUI
cargo test               # Run all tests

cargo fmt --check        # Check formatting (CI enforced - MUST PASS)
cargo fmt                # Auto-format
cargo clippy -- -D warnings  # Lint (treat warnings as errors)
```

> **Formatting is enforced.** Run `cargo fmt` before every commit. CI will reject unformatted code.

CI (.github/workflows/ci.yml) runs `cargo check`, `cargo fmt --check`, and `cargo test --verbose` on push to master and PRs.

## Architecture

Hexagonal (Ports & Adapters) architecture with four layers:

```
TUI (tui/)  →  App (app/)  →  Domain (domain/)
                   ↓
              Infra (infra/)
```

- **domain/**: Pure data models — no I/O. AssetIdentity, ConfigFile, Scope, ScannedPackage, hashing.
- **app/**: Business logic orchestration. `ports.rs` defines the four core traits. `bootstrap.rs` is the composition root (only place infra is wired). `actions.rs` has reusable operations.
- **infra/**: I/O adapters implementing port traits. Vault backends (local, github, clawhub), provider installers (Claude Code, Copilot, Gemini, etc.), TOML config store.
- **tui/**: Ratatui-based UI. `app.rs` holds reactive AppState. `event.rs` maps keycodes to actions. Background tasks use `tokio::sync::mpsc::UnboundedSender<AppEvent>`.

### Core Port Traits (app/ports.rs)

- `FeatureSetPort` — defines how to scan a package type (skills vs instructions)
- `VaultPort` — vault source abstraction (id, list_packages, refresh)
- `ProviderPort` — target AI platform installer (install, remove)
- `ConfigStorePort` — scoped config persistence (Global vs Workspace)

### Key Patterns

- **SHA10 hashing** for asset change detection, not semantic versions. Version is display metadata; sha10 is the source of truth for freshness.
- **Scoped config**: Global (`~/.config/agk/config.toml`) for vaults/providers, Workspace (`.agk/config.toml`) for installed assets.
- **Async I/O**: All network/git operations run on tokio tasks via `AppEvent` channel to keep TUI responsive. Never block the render loop.
- **Bootstrap is the only DI point**: `app/bootstrap.rs` wires infra adapters. No infra imports outside this file and main.rs.
- **ClawHub vault**: CLI-delegated — shells out to `clawhub` binary for search/install/inspect. Uses LocalVaultAdapter to scan its cache at `~/.config/agk/clawhub/`.

### Vault Structure Convention

Skills require `SKILL.md`, instructions require `AGENTS.md` as the marker file within their directory under `skills/` or `instructions/`.

## Documentation Requirements

When implementing a new feature or modifying an existing one, always update the corresponding documentation under `docs/product/features/`. Each feature area must have both files:

- `prd.md` — Product requirements: what the feature does, user-facing behavior, functional requirements
- `technical_design.md` — Technical design: trait contracts, data schemas, internal workflows, architecture rules

If adding a new feature area, create a new directory under `docs/product/features/<feature-name>/` with both files.

## Working with Worktrees

Feature branches often use git worktrees at `.worktrees/<branch-name>/`. Code changes in a worktree are isolated from the main working directory — remember to `cd` into the worktree or use its path when building/testing.
