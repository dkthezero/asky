# ClawHub Vault Integration Design

**Date:** 2026-04-06
**Status:** Draft

---

## 1. Scope

Add ClawHub as a new vault type in agk, enabling users to discover, search, and install skills from the ClawHub public registry (clawhub.ai) via the `clawhub` CLI.

### Included

1. **`ClawHubVaultAdapter`** — new vault type (`type = "clawhub"`) alongside `local` and `github`
2. **Inactive by default** — listed in Vaults tab as unchecked; activation triggers CLI detection
3. **CLI-delegated operations** — all remote interactions go through `clawhub` CLI commands
4. **Parallel search** — when ClawHub is active and user types a search query, `clawhub search` runs in background alongside local filtering; results merge into Skills tab
5. **Two-job install pipeline** — remote ClawHub skills show two progress jobs in the progress stack
6. **Visual differentiation** — remote (uncached) ClawHub search results rendered in a distinct text color

### Out of scope

- ClawHub authentication (`clawhub login`) management within agk
- Publishing skills to ClawHub from agk
- ClawHub instructions support (skills only)
- Soul registry (onlycrabs.ai) integration

---

## 2. Architecture

### New domain types

```rust
/// Config variant for ClawHub vaults
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClawHubVaultSource {
    // No fields needed — ClawHub has a single global registry.
    // Reserved for future config (e.g., custom registry URL).
}
```

Add to existing `VaultConfig` enum:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum VaultConfig {
    Local(LocalVaultSource),
    Github(GithubVaultSource),
    Clawhub(ClawHubVaultSource),
}
```

### New infrastructure

**`src/infra/vault/clawhub.rs`** — `ClawHubVaultAdapter` implementing `VaultPort`:

- `id()` → `"clawhub"`
- `kind_name()` → `"clawhub"`
- `refresh()` → runs `clawhub update --all` if CLI is available
- `list_packages()` → scans ClawHub's default local install directory via `LocalVaultAdapter`

The ClawHub CLI installs skills to a local directory. By default, `clawhub install` places skills in `<cwd>/skills/<slug>/` with a lockfile at `<cwd>/.clawhub/lock.json`. The adapter uses a dedicated agk-managed directory as the workdir for ClawHub operations: `~/.config/agk/clawhub/` (i.e., `clawhub install --workdir ~/.config/agk/clawhub/ <slug>`). This keeps ClawHub's cache isolated from user workspaces. The adapter treats this directory as a local vault source for browsing.

### Config representation

```toml
# Global config.toml
vaults = []  # "clawhub" NOT included by default

[clawhub.vault]
type = "clawhub"
```

When activated by the user:

```toml
vaults = ["clawhub"]

[clawhub.vault]
type = "clawhub"
```

---

## 3. Activation Flow

When the user presses Space on the "ClawHub" entry in the Vaults tab:

```
User presses Space on "ClawHub"
        │
        ▼
  Is clawhub on $PATH?
  (which clawhub)
        │
   no ──┤──── yes
   │         │
   ▼         ▼
  Detect     Activate vault:
  package    add "clawhub" to
  manager    vaults list, persist
        │
        ▼
  Has Homebrew? (macOS)
        │
   no ──┤──── yes
   │         │
   ▼         ▼
  Alert:     Prompt:
  "Install   "Install clawhub
  clawhub    via Homebrew? (Y/n)"
  manually       │
  from           ▼
  clawhub.ai"  brew install clawhub
  (abort)        │
                 ▼
              Activate vault
```

**Fallback rule:** If no known package manager is available to install `clawhub`, show an alert directing the user to install it manually and abort activation. Do not attempt unknown install methods.

---

## 4. Search: Parallel Local + ClawHub

When ClawHub vault is active and the user types a search query in the Skills tab:

```
User types "web-tool"
        │
        ├──▶ Local filter (synchronous)
        │     Filter all active vaults' scanned packages
        │     by name substring match
        │     → immediate results
        │
        └──▶ clawhub search "web-tool" (async, background)
              Show spinner/loading indicator in Skills tab
              │
              ▼
        Parse CLI output into ScannedPackage entries
        with vault_id = "clawhub"
              │
              ▼
        Merge into Skills tab:
        - Deduplicate by skill name
        - If a skill exists both locally and in ClawHub,
          prefer the local entry
        - Remote-only results rendered in distinct color
        Remove spinner
```

### Remote result rendering

ClawHub search results that are **not locally cached** are rendered with a distinct text color (e.g., a dimmed or accent color) to visually separate them from locally available skills. Once a remote skill is installed, it appears in normal coloring on the next scan.

### Search result data

Remote search results are represented as `ScannedPackage` with:
- `vault_id`: `"clawhub"`
- `identity.name`: skill slug from ClawHub (e.g., `pskoett/self-improving-agent`)
- `identity.sha10`: empty or placeholder (not locally hashed yet)
- `identity.version`: version from ClawHub if available
- `path`: empty `PathBuf` (not on disk yet)
- `kind`: `AssetKind::Skill`

A boolean flag or marker distinguishes remote results from local ones (e.g., `is_remote: true` on the package or tracked in TUI state).

---

## 5. Install Flow: Two-Job Progress Stack

### Remote skill (not locally cached)

When the user presses Space to install a remote ClawHub search result, **two jobs** are registered and visible in the bottom-right progress stack:

```
┌─────────────────────────────────────────────┐
│ ⟳ Fetching pskoett/web-tool from ClawHub... │  ← Job 1
│ ◌ Installing pskoett/web-tool to global...  │  ← Job 2 (waiting)
└─────────────────────────────────────────────┘
```

**Job 1: Fetch from ClawHub**
- Runs `clawhub install <slug>`
- Downloads skill into ClawHub's local install directory
- Status: spinning → done/error

**Job 2: Copy to scope target (depends on Job 1)**
- Copies skill from ClawHub's local cache to the active scope's provider target
- Uses the same install logic as any other vault (existing `ProviderPort::install()`)
- Global scope → `~/.config/agk/providers/<provider>/skills/<name>/`
- Workspace scope → `.agk/providers/<provider>/skills/<name>/`
- Status: waiting → spinning → done/error

```
Job 1 completes
        │
        ▼
┌─────────────────────────────────────────────┐
│ ✓ Fetching pskoett/web-tool from ClawHub    │  ← Job 1 done
│ ⟳ Installing pskoett/web-tool to global...  │  ← Job 2 running
└─────────────────────────────────────────────┘
```

### Locally cached skill

If the skill is already in ClawHub's local directory (previously fetched), only Job 2 runs.

### Error handling

- If Job 1 fails (network error, CLI error), Job 2 is cancelled. Error shown in progress stack.
- If Job 2 fails (copy error, permission error), error shown. ClawHub local cache is not rolled back.

---

## 6. Bootstrap Changes

In `bootstrap.rs`, the ClawHub vault is **always registered** in the vault definitions but **not added to the active vaults list** by default:

```rust
// Always register ClawHub vault definition (inactive by default)
if !global_config.vault_defs.contains_key("clawhub") {
    // ClawHub is available but not active until user enables it
}
```

The `build_vaults()` function gains a match arm for `VaultConfig::Clawhub`:

```rust
crate::domain::config::VaultConfig::Clawhub(_) => {
    vaults.push(Box::new(
        crate::infra::vault::clawhub::ClawHubVaultAdapter::new(vault_id),
    ));
}
```

---

## 7. Files Changed

| File | Change |
|------|--------|
| `src/domain/config.rs` | Add `ClawHubVaultSource`, extend `VaultConfig` enum |
| `src/infra/vault/mod.rs` | Add `pub(crate) mod clawhub;` |
| `src/infra/vault/clawhub.rs` | New: `ClawHubVaultAdapter` implementing `VaultPort` |
| `src/app/bootstrap.rs` | Register ClawHub vault, handle `VaultConfig::Clawhub` in `build_vaults()` |
| `src/app/actions.rs` | Vault activation: CLI detection, install prompt, two-job install pipeline |
| `src/tui/render.rs` | Remote result color differentiation, progress stack rendering |
| `src/tui/app.rs` | Search state: parallel async search, merge logic, remote result tracking |
| `src/tui/event.rs` | Handle async search completion events |

---

## 8. Testing Strategy

- **Unit tests** for `ClawHubVaultAdapter`: scan local dir, handle missing dir, handle missing CLI
- **Unit tests** for config parsing: `VaultConfig::Clawhub` round-trip serialization
- **Unit tests** for search merge: deduplication, remote-vs-local preference, remote flagging
- **Integration test** for activation flow: mock `which clawhub` success/failure
- **Integration test** for two-job install: mock `clawhub install`, verify copy to scope
