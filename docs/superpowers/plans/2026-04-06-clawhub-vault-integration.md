# ClawHub Vault Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add ClawHub as a new vault type that delegates to the `clawhub` CLI for remote skill discovery/install, with parallel search, two-job progress stack, and visual differentiation.

**Architecture:** New `ClawHubVaultAdapter` implementing `VaultPort`, new `VaultConfig::Clawhub` variant, new `AppEvent::ClawHubSearchResults` for async search merging, and a `is_remote` flag on `ScannedPackage` for rendering. The adapter scans ClawHub's local cache via `LocalVaultAdapter` for browse, and shells out to `clawhub search` for typed queries.

**Tech Stack:** Rust, ratatui, tokio, crossterm, std::process::Command

---

## File Structure

| File | Responsibility |
|------|---------------|
| `src/domain/config.rs` | Add `ClawHubVaultSource` struct, extend `VaultConfig` enum |
| `src/domain/asset.rs` | Add `is_remote` field to `ScannedPackage` |
| `src/domain/paths.rs` | Add `clawhub_cache_dir()` helper |
| `src/infra/vault/mod.rs` | Add `pub(crate) mod clawhub;` |
| `src/infra/vault/clawhub.rs` | New: `ClawHubVaultAdapter` implementing `VaultPort` + CLI helpers |
| `src/app/bootstrap.rs` | Register ClawHub vault def, handle `VaultConfig::Clawhub` in `build_vaults()` |
| `src/tui/app.rs` | Add `remote_packages` field, `ClawHubSearchResults` event handling |
| `src/tui/event.rs` | Parallel search dispatch, two-job install, vault activation with CLI check |
| `src/tui/widgets/list.rs` | Color remote packages differently |

---

### Task 1: Add `ClawHubVaultSource` to domain config

**Files:**
- Modify: `src/domain/config.rs:7-30` (VaultConfig enum area)

- [ ] **Step 1: Write the failing test for ClawHub config round-trip**

In `src/domain/config.rs`, add to the `#[cfg(test)] mod tests` block:

```rust
#[test]
fn clawhub_vault_config_round_trip() {
    let toml_str = r#"
version = 1
vaults = ["clawhub"]

[clawhub.vault]
type = "clawhub"
"#;
    let config: ConfigFile = toml::from_str(toml_str).unwrap();
    assert!(config.vaults.contains(&"clawhub".to_string()));
    let section = config.vault_defs.get("clawhub").unwrap();
    assert!(matches!(
        section.vault,
        Some(VaultConfig::Clawhub(ClawHubVaultSource {}))
    ));
    let serialized = toml::to_string(&config).unwrap();
    assert!(serialized.contains("type = \"clawhub\""));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test clawhub_vault_config_round_trip -- --nocapture`
Expected: FAIL — `ClawHubVaultSource` and `VaultConfig::Clawhub` don't exist yet.

- [ ] **Step 3: Add `ClawHubVaultSource` and extend `VaultConfig`**

In `src/domain/config.rs`, add after the `GithubVaultSource` struct:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClawHubVaultSource {}
```

Extend the `VaultConfig` enum:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum VaultConfig {
    Local(LocalVaultSource),
    Github(GithubVaultSource),
    Clawhub(ClawHubVaultSource),
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test clawhub_vault_config_round_trip -- --nocapture`
Expected: PASS

- [ ] **Step 5: Run full test suite to check for breakage**

Run: `cargo test`
Expected: All existing tests PASS. The new `Clawhub` variant may cause non-exhaustive match warnings in `bootstrap.rs` — that's expected and will be fixed in Task 4.

- [ ] **Step 6: Commit**

```bash
git add src/domain/config.rs
git commit -m "feat: add ClawHubVaultSource and VaultConfig::Clawhub variant"
```

---

### Task 2: Add `is_remote` flag to `ScannedPackage`

**Files:**
- Modify: `src/domain/asset.rs:11-16`

- [ ] **Step 1: Write the failing test**

In `src/domain/asset.rs`, add to `#[cfg(test)] mod tests`:

```rust
#[test]
fn scanned_package_default_not_remote() {
    let pkg = ScannedPackage {
        identity: AssetIdentity::new("my-skill", None, "abc1234567"),
        path: PathBuf::from("/skills/my-skill"),
        vault_id: "workspace".to_string(),
        kind: AssetKind::Skill,
        is_remote: false,
    };
    assert!(!pkg.is_remote);
}

#[test]
fn scanned_package_remote_flag() {
    let pkg = ScannedPackage {
        identity: AssetIdentity::new("remote-skill", None, "0000000000"),
        path: PathBuf::new(),
        vault_id: "clawhub".to_string(),
        kind: AssetKind::Skill,
        is_remote: true,
    };
    assert!(pkg.is_remote);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test scanned_package_default_not_remote scanned_package_remote_flag -- --nocapture`
Expected: FAIL — `is_remote` field does not exist.

- [ ] **Step 3: Add `is_remote` field**

In `src/domain/asset.rs`, modify `ScannedPackage`:

```rust
#[derive(Debug, Clone)]
pub struct ScannedPackage {
    pub identity: AssetIdentity,
    pub path: PathBuf,
    pub vault_id: String,
    pub kind: AssetKind,
    pub is_remote: bool,
}
```

- [ ] **Step 4: Fix all existing construction sites**

Every place that creates a `ScannedPackage` needs `is_remote: false`. Search for `ScannedPackage {` across the codebase and add the field. Key locations:

- `src/infra/vault/local.rs:56` — add `is_remote: false,` after `kind: feature.asset_kind(),`
- `src/tui/event.rs` — in `make_pkg` test helper and any inline construction
- `src/app/actions.rs` — in `make_pkg` test helper
- `src/tui/app.rs` — in `make_pkg` test helper
- `src/domain/asset.rs` — in existing test helpers

- [ ] **Step 5: Run full test suite**

Run: `cargo test`
Expected: All tests PASS.

- [ ] **Step 6: Commit**

```bash
git add src/domain/asset.rs src/infra/vault/local.rs src/tui/event.rs src/app/actions.rs src/tui/app.rs
git commit -m "feat: add is_remote flag to ScannedPackage"
```

---

### Task 3: Add `clawhub_cache_dir()` to domain paths

**Files:**
- Modify: `src/domain/paths.rs`

- [ ] **Step 1: Write the failing test**

In `src/domain/paths.rs`, add to `#[cfg(test)] mod tests`:

```rust
#[test]
fn test_clawhub_cache_dir() {
    let dir = clawhub_cache_dir();
    assert!(dir.to_string_lossy().contains("agk"));
    assert!(dir.to_string_lossy().ends_with("clawhub"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_clawhub_cache_dir -- --nocapture`
Expected: FAIL — function doesn't exist.

- [ ] **Step 3: Add `clawhub_cache_dir()`**

In `src/domain/paths.rs`, add after `global_vaults_dir()`:

```rust
/// Resolve the ClawHub cache directory: `<config_root>/clawhub`.
/// This is used as the `--workdir` for `clawhub install` commands,
/// keeping ClawHub's local state isolated from user workspaces.
pub fn clawhub_cache_dir() -> PathBuf {
    global_config_root().join("clawhub")
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test test_clawhub_cache_dir -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/domain/paths.rs
git commit -m "feat: add clawhub_cache_dir() path helper"
```

---

### Task 4: Create `ClawHubVaultAdapter`

**Files:**
- Create: `src/infra/vault/clawhub.rs`
- Modify: `src/infra/vault/mod.rs`

- [ ] **Step 1: Write the module declaration**

In `src/infra/vault/mod.rs`, add:

```rust
pub(crate) mod clawhub;
```

So the full file becomes:

```rust
pub(crate) mod clawhub;
pub(crate) mod github;
pub(crate) mod local;
```

- [ ] **Step 2: Write the failing tests for ClawHubVaultAdapter**

Create `src/infra/vault/clawhub.rs` with the test module first:

```rust
use crate::app::ports::{FeatureSetPort, VaultPort};
use crate::domain::asset::ScannedPackage;
use crate::infra::vault::local::LocalVaultAdapter;
use anyhow::Result;
use std::path::PathBuf;

pub struct ClawHubVaultAdapter {
    id: String,
}

impl ClawHubVaultAdapter {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

#[async_trait::async_trait]
impl VaultPort for ClawHubVaultAdapter {
    fn id(&self) -> &str {
        &self.id
    }

    fn kind_name(&self) -> &str {
        "clawhub"
    }

    fn list_packages(&self, feature: &dyn FeatureSetPort) -> Result<Vec<ScannedPackage>> {
        let cache_dir = crate::domain::paths::clawhub_cache_dir();
        if !cache_dir.exists() {
            return Ok(Vec::new());
        }
        let local = LocalVaultAdapter::new(&self.id, cache_dir);
        local.list_packages(feature)
    }
}

/// Check if the `clawhub` CLI is available on $PATH.
pub fn is_cli_available() -> bool {
    std::process::Command::new("which")
        .arg("clawhub")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if Homebrew is available (macOS).
pub fn is_homebrew_available() -> bool {
    std::process::Command::new("which")
        .arg("brew")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Install clawhub CLI via Homebrew. Returns Ok if successful.
pub fn install_cli_via_homebrew() -> Result<()> {
    let status = std::process::Command::new("brew")
        .args(["install", "clawhub"])
        .status()?;
    if !status.success() {
        anyhow::bail!("Failed to install clawhub via Homebrew");
    }
    Ok(())
}

/// Run `clawhub search <query>` and parse results into ScannedPackages.
/// Each line of output is expected to be a skill slug.
pub fn cli_search(query: &str) -> Result<Vec<ScannedPackage>> {
    let cache_dir = crate::domain::paths::clawhub_cache_dir();
    let output = std::process::Command::new("clawhub")
        .args(["search", query])
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("clawhub search failed: {}", stderr);
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let packages: Vec<ScannedPackage> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let name = line.trim().to_string();
            ScannedPackage {
                identity: crate::domain::identity::AssetIdentity::new(&name, None, "----------"),
                path: PathBuf::new(),
                vault_id: "clawhub".to_string(),
                kind: crate::domain::asset::AssetKind::Skill,
                is_remote: true,
            }
        })
        .collect();
    Ok(packages)
}

/// Run `clawhub install <slug>` with workdir set to agk's clawhub cache.
pub fn cli_install(slug: &str) -> Result<()> {
    let cache_dir = crate::domain::paths::clawhub_cache_dir();
    std::fs::create_dir_all(&cache_dir)?;
    let status = std::process::Command::new("clawhub")
        .args(["install", slug, "--workdir", &cache_dir.to_string_lossy()])
        .status()?;
    if !status.success() {
        anyhow::bail!("clawhub install '{}' failed", slug);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::feature::skill::SkillFeatureSet;

    #[test]
    fn clawhub_vault_id() {
        let adapter = ClawHubVaultAdapter::new("clawhub");
        assert_eq!(adapter.id(), "clawhub");
    }

    #[test]
    fn clawhub_vault_kind_name() {
        let adapter = ClawHubVaultAdapter::new("clawhub");
        assert_eq!(adapter.kind_name(), "clawhub");
    }

    #[test]
    fn list_packages_empty_when_no_cache_dir() {
        // clawhub_cache_dir won't exist in test env typically
        let adapter = ClawHubVaultAdapter::new("clawhub");
        let pkgs = adapter.list_packages(&SkillFeatureSet).unwrap();
        assert!(pkgs.is_empty());
    }

    #[test]
    fn list_packages_finds_cached_skills() {
        let dir = tempfile::tempdir().unwrap();
        let skill_dir = dir.path().join("skills").join("my-clawhub-skill");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), "# My Skill").unwrap();

        // Use LocalVaultAdapter directly since we can't override clawhub_cache_dir in test
        let local = LocalVaultAdapter::new("clawhub", dir.path().to_path_buf());
        let pkgs = local.list_packages(&SkillFeatureSet).unwrap();
        assert_eq!(pkgs.len(), 1);
        assert_eq!(pkgs[0].identity.name, "my-clawhub-skill");
        assert_eq!(pkgs[0].vault_id, "clawhub");
    }
}
```

- [ ] **Step 3: Run tests to verify they pass**

Run: `cargo test clawhub_vault -- --nocapture`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src/infra/vault/mod.rs src/infra/vault/clawhub.rs
git commit -m "feat: add ClawHubVaultAdapter with CLI helpers"
```

---

### Task 5: Register ClawHub vault in bootstrap

**Files:**
- Modify: `src/app/bootstrap.rs:83-117` (build_vaults function)
- Modify: `src/app/bootstrap.rs:196-208` (build_vault_entries — VaultConfig match)

- [ ] **Step 1: Write the failing test**

In `src/app/bootstrap.rs`, add to `#[cfg(test)] mod tests`:

```rust
#[test]
fn bootstrap_includes_clawhub_vault_entry() {
    let dir = tempfile::tempdir().unwrap();
    let workspace_root = dir.path().to_path_buf();
    let agk_dir = workspace_root.join(".agk");
    std::fs::create_dir_all(&agk_dir).unwrap();
    let global_dir = dir.path().join("global");
    std::fs::create_dir_all(&global_dir).unwrap();
    let config_content = r#"
version = 1
vaults = []

[clawhub.vault]
type = "clawhub"
"#;
    std::fs::write(global_dir.join("config.toml"), config_content).unwrap();
    let store = TomlConfigStore::new(
        global_dir.join("config.toml"),
        agk_dir.join("config.toml"),
    );
    let (registry, _scan, _store) = build_with_store(workspace_root, store).unwrap();
    // ClawHub vault should be registered in registry
    assert!(registry.vaults.iter().any(|v| v.id() == "clawhub"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test bootstrap_includes_clawhub_vault_entry -- --nocapture`
Expected: FAIL — `VaultConfig::Clawhub` not handled in `build_vaults()`.

- [ ] **Step 3: Add ClawHub handling to `build_vaults()`**

In `src/app/bootstrap.rs`, in the `build_vaults()` function, add a new match arm inside the `match vault_conf` block (after the `Github` arm):

```rust
crate::domain::config::VaultConfig::Clawhub(_) => {
    vaults.push(Box::new(
        crate::infra::vault::clawhub::ClawHubVaultAdapter::new(vault_id),
    ));
}
```

- [ ] **Step 4: Update `build_vault_entries()` to handle clawhub kind**

In `src/app/bootstrap.rs`, in `build_vault_entries()`, update the `kind` mapping (around line 203):

```rust
.map(|v| match v {
    crate::domain::config::VaultConfig::Local(_) => "local",
    crate::domain::config::VaultConfig::Github(_) => "github",
    crate::domain::config::VaultConfig::Clawhub(_) => "clawhub",
})
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test bootstrap_includes_clawhub_vault_entry -- --nocapture`
Expected: PASS

- [ ] **Step 6: Run full test suite**

Run: `cargo test`
Expected: All tests PASS.

- [ ] **Step 7: Commit**

```bash
git add src/app/bootstrap.rs
git commit -m "feat: register ClawHub vault in bootstrap"
```

---

### Task 6: Add `ClawHubSearchResults` event and async search dispatch

**Files:**
- Modify: `src/tui/event.rs` (AppEvent enum, search dispatch)
- Modify: `src/tui/app.rs` (remote_packages state)

- [ ] **Step 1: Add `remote_packages` field to `AppState`**

In `src/tui/app.rs`, add a new field to `AppState`:

```rust
pub remote_packages: Vec<ScannedPackage>,
pub clawhub_searching: bool,
```

Initialize both in `AppState::new()`:

```rust
remote_packages: Vec::new(),
clawhub_searching: false,
```

- [ ] **Step 2: Update `filtered_packages()` to merge remote results**

In `src/tui/app.rs`, modify `filtered_packages()`:

```rust
pub fn filtered_packages(&self) -> Vec<&ScannedPackage> {
    let pkgs = self.active_packages();
    let q = self.search_query.to_lowercase();

    let mut result: Vec<&ScannedPackage> = if q.is_empty() {
        pkgs.iter().collect()
    } else {
        pkgs.iter()
            .filter(|p| p.identity.name.to_lowercase().contains(&q))
            .collect()
    };

    // Merge remote ClawHub results, deduplicating by name
    if !self.search_query.is_empty() {
        let local_names: std::collections::HashSet<&str> =
            result.iter().map(|p| p.identity.name.as_str()).collect();
        for remote_pkg in &self.remote_packages {
            if !local_names.contains(remote_pkg.identity.name.as_str()) {
                result.push(remote_pkg);
            }
        }
    }

    result
}
```

- [ ] **Step 3: Add `ClawHubSearchResults` to `AppEvent`**

In `src/tui/event.rs`, add to the `AppEvent` enum:

```rust
ClawHubSearchResults {
    packages: Vec<crate::domain::asset::ScannedPackage>,
},
```

- [ ] **Step 4: Write the search dispatch in `apply_search_char`**

In `src/tui/event.rs`, modify `apply_search_char` to accept `EventContext` and dispatch ClawHub search. Change the signature and update the call site:

First, update the call in `handle()` (around line 103-108). Change:

```rust
KeyCode::Char(c) => {
    let active_kind = state.tab_kinds.get(state.active_tab).copied();
    if active_kind != Some(crate::tui::app::TabKind::Vault) {
        apply_search_char(state, *c);
    }
}
```

To:

```rust
KeyCode::Char(c) => {
    let active_kind = state.tab_kinds.get(state.active_tab).copied();
    if active_kind != Some(crate::tui::app::TabKind::Vault) {
        apply_search_char(state, *c);
        // Dispatch ClawHub search if vault is active and on Skills tab
        if active_kind == Some(crate::tui::app::TabKind::Asset)
            && is_clawhub_active(ctx)
            && !state.search_query.is_empty()
        {
            dispatch_clawhub_search(state, ctx);
        }
    }
}
```

Then add the helper functions:

```rust
fn is_clawhub_active(ctx: &EventContext) -> bool {
    ctx.store
        .load(crate::domain::scope::Scope::Global)
        .map(|c| c.vaults.contains(&"clawhub".to_string()))
        .unwrap_or(false)
}

fn dispatch_clawhub_search(state: &mut AppState, ctx: &EventContext) {
    state.clawhub_searching = true;
    let query = state.search_query.clone();
    let tx = ctx.tx.clone();
    tokio::task::spawn_blocking(move || {
        match crate::infra::vault::clawhub::cli_search(&query) {
            Ok(packages) => {
                let _ = tx.send(AppEvent::ClawHubSearchResults { packages });
            }
            Err(_) => {
                let _ = tx.send(AppEvent::ClawHubSearchResults {
                    packages: Vec::new(),
                });
            }
        }
    });
}
```

- [ ] **Step 5: Handle `ClawHubSearchResults` in the main event loop**

The main TUI loop (in `src/tui/mod.rs` or wherever `AppEvent` is consumed) needs to handle this event. Find where `AppEvent::TaskCompleted` etc. are handled (likely in the main `run` function) and add:

```rust
AppEvent::ClawHubSearchResults { packages } => {
    state.remote_packages = packages;
    state.clawhub_searching = false;
}
```

- [ ] **Step 6: Clear remote packages when search is cleared**

In `apply_esc()` in `src/tui/event.rs`, add:

```rust
state.remote_packages.clear();
state.clawhub_searching = false;
```

And in `handle_backspace()`, after `state.search_query.pop()`, when the search query becomes empty:

```rust
if state.search_query.is_empty() {
    state.list_mode = ListMode::Normal;
    state.remote_packages.clear();
    state.clawhub_searching = false;
}
```

- [ ] **Step 7: Write test for filtered_packages merging**

In `src/tui/app.rs` tests:

```rust
#[test]
fn filtered_packages_merges_remote_results() {
    let mut state = state_with_skills(vec![make_pkg("local-skill")]);
    state.search_query = "skill".to_string();

    let remote_pkg = ScannedPackage {
        identity: AssetIdentity::new("remote-skill", None, "----------"),
        path: PathBuf::new(),
        vault_id: "clawhub".to_string(),
        kind: AssetKind::Skill,
        is_remote: true,
    };
    state.remote_packages = vec![remote_pkg];

    let filtered = state.filtered_packages();
    assert_eq!(filtered.len(), 2);
}

#[test]
fn filtered_packages_deduplicates_remote() {
    let mut state = state_with_skills(vec![make_pkg("same-skill")]);
    state.search_query = "same".to_string();

    let remote_pkg = ScannedPackage {
        identity: AssetIdentity::new("same-skill", None, "----------"),
        path: PathBuf::new(),
        vault_id: "clawhub".to_string(),
        kind: AssetKind::Skill,
        is_remote: true,
    };
    state.remote_packages = vec![remote_pkg];

    let filtered = state.filtered_packages();
    assert_eq!(filtered.len(), 1); // local wins
    assert!(!filtered[0].is_remote);
}
```

- [ ] **Step 8: Run full test suite**

Run: `cargo test`
Expected: All tests PASS.

- [ ] **Step 9: Commit**

```bash
git add src/tui/app.rs src/tui/event.rs
git commit -m "feat: add parallel ClawHub search with result merging"
```

---

### Task 7: Two-job install pipeline for remote ClawHub skills

**Files:**
- Modify: `src/tui/event.rs` (handle_space_asset function)

- [ ] **Step 1: Modify `handle_space_asset` to detect remote ClawHub packages**

In `src/tui/event.rs`, in `handle_space_asset()`, after getting the `pkg` from `filtered_packages`, add a branch for remote packages. Replace the existing `handle_space_asset` function body with:

```rust
fn handle_space_asset(state: &mut AppState, ctx: &EventContext) -> Result<()> {
    let pkg_opt = {
        let filtered = state.filtered_packages();
        filtered.get(state.selected_index).copied().cloned()
    };
    if let Some(pkg) = pkg_opt {
        if pkg.is_remote {
            return handle_install_remote_clawhub(state, ctx, &pkg);
        }

        let is_installed = state.is_installed(&pkg.vault_id, &pkg.identity.name, &pkg.kind);
        let store = ctx.store.clone();
        let active_scope = state.active_scope;
        let tx = ctx.tx.clone();
        let registry = ctx.registry.clone();

        let id = crate::tui::app::NEXT_TASK_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        tokio::task::spawn_blocking(move || {
            let action = if is_installed {
                "Uninstalling"
            } else {
                "Installing"
            };
            let _ = tx.send(AppEvent::TaskStarted {
                id,
                name: format!("{} '{}'", action, pkg.identity.name),
            });

            let config = store.load(active_scope).unwrap_or_default();
            let providers = active_providers(&registry, &config);

            if providers.is_empty() {
                let _ = tx.send(AppEvent::TaskFailed {
                    id,
                    error: "No active providers to install to".into(),
                });
                return;
            }

            let mut success = true;
            for provider in providers {
                if is_installed {
                    if crate::app::actions::remove_asset(
                        active_scope,
                        &pkg.identity,
                        &pkg.kind,
                        &pkg.vault_id,
                        store.as_ref(),
                        provider,
                    )
                    .is_err()
                    {
                        success = false;
                    }
                } else if crate::app::actions::install_asset(
                    active_scope,
                    &pkg,
                    store.as_ref(),
                    provider,
                )
                .is_err()
                {
                    success = false;
                }
            }
            let _ = tx.send(AppEvent::TaskProgress { id, percent: 100 });
            let _ = tx.send(AppEvent::TriggerReload);
            if success {
                let done = if is_installed {
                    "Uninstalled"
                } else {
                    "Installed"
                };
                let _ = tx.send(AppEvent::TaskCompleted {
                    id,
                    message: format!("{} '{}'", done, pkg.identity.name),
                });
            } else {
                let _ = tx.send(AppEvent::TaskFailed {
                    id,
                    error: format!(
                        "Failed to {} '{}'",
                        action.to_lowercase(),
                        pkg.identity.name
                    ),
                });
            }
        });
    }
    Ok(())
}
```

- [ ] **Step 2: Add the two-job remote install handler**

In `src/tui/event.rs`, add:

```rust
fn handle_install_remote_clawhub(
    state: &mut AppState,
    ctx: &EventContext,
    pkg: &ScannedPackage,
) -> Result<()> {
    let slug = pkg.identity.name.clone();
    let store = ctx.store.clone();
    let tx = ctx.tx.clone();
    let registry = ctx.registry.clone();
    let active_scope = state.active_scope;

    // Job 1: Fetch from ClawHub
    let fetch_id =
        crate::tui::app::NEXT_TASK_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    // Job 2: Install to scope (registered upfront so both appear in progress stack)
    let install_id =
        crate::tui::app::NEXT_TASK_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    let _ = tx.send(AppEvent::TaskStarted {
        id: fetch_id,
        name: format!("Fetching '{}' from ClawHub", slug),
    });
    let _ = tx.send(AppEvent::TaskStarted {
        id: install_id,
        name: format!("Installing '{}' to {:?}", slug, active_scope),
    });

    tokio::task::spawn_blocking(move || {
        // Job 1: clawhub install
        match crate::infra::vault::clawhub::cli_install(&slug) {
            Ok(()) => {
                let _ = tx.send(AppEvent::TaskProgress {
                    id: fetch_id,
                    percent: 100,
                });
                let _ = tx.send(AppEvent::TaskCompleted {
                    id: fetch_id,
                    message: format!("Fetched '{}' from ClawHub", slug),
                });
            }
            Err(e) => {
                let _ = tx.send(AppEvent::TaskFailed {
                    id: fetch_id,
                    error: format!("Failed to fetch '{}': {}", slug, e),
                });
                let _ = tx.send(AppEvent::TaskFailed {
                    id: install_id,
                    error: "Cancelled — fetch failed".into(),
                });
                return;
            }
        }

        // Job 2: scan the now-cached package and install via providers
        let cache_dir = crate::domain::paths::clawhub_cache_dir();
        let local = crate::infra::vault::local::LocalVaultAdapter::new("clawhub", cache_dir);
        let feature = crate::infra::feature::skill::SkillFeatureSet;
        let cached_pkgs = match local.list_packages(&feature) {
            Ok(pkgs) => pkgs,
            Err(e) => {
                let _ = tx.send(AppEvent::TaskFailed {
                    id: install_id,
                    error: format!("Failed to scan cached package: {}", e),
                });
                return;
            }
        };

        let cached_pkg = cached_pkgs.iter().find(|p| p.identity.name == slug);
        if let Some(pkg) = cached_pkg {
            let config = store.load(active_scope).unwrap_or_default();
            let providers = active_providers(&registry, &config);

            if providers.is_empty() {
                let _ = tx.send(AppEvent::TaskFailed {
                    id: install_id,
                    error: "No active providers to install to".into(),
                });
                return;
            }

            let mut success = true;
            for provider in providers {
                if crate::app::actions::install_asset(active_scope, pkg, store.as_ref(), provider)
                    .is_err()
                {
                    success = false;
                }
            }

            let _ = tx.send(AppEvent::TaskProgress {
                id: install_id,
                percent: 100,
            });
            let _ = tx.send(AppEvent::TriggerReload);
            if success {
                let _ = tx.send(AppEvent::TaskCompleted {
                    id: install_id,
                    message: format!("Installed '{}' to {:?}", slug, active_scope),
                });
            } else {
                let _ = tx.send(AppEvent::TaskFailed {
                    id: install_id,
                    error: format!("Failed to install '{}'", slug),
                });
            }
        } else {
            let _ = tx.send(AppEvent::TaskFailed {
                id: install_id,
                error: format!("Skill '{}' not found in ClawHub cache after fetch", slug),
            });
        }
    });

    Ok(())
}
```

- [ ] **Step 3: Run full test suite**

Run: `cargo test`
Expected: All tests PASS.

- [ ] **Step 4: Commit**

```bash
git add src/tui/event.rs
git commit -m "feat: two-job progress pipeline for remote ClawHub skill install"
```

---

### Task 8: Vault activation with CLI detection

**Files:**
- Modify: `src/tui/event.rs` (handle_space_vault function)
- Modify: `src/tui/app.rs` (ListMode enum)

- [ ] **Step 1: Add `ConfirmClawHubInstall` to `ListMode`**

In `src/tui/app.rs`, extend the `ListMode` enum:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ListMode {
    Normal,
    Searching,
    AttachVault,
    AttachVaultBranch,
    AttachVaultPath,
    ConfirmDetachVault,
    ConfirmClawHubInstall,
}
```

- [ ] **Step 2: Add ClawHub activation logic to `handle_space_vault`**

In `src/tui/event.rs`, modify `handle_space_vault()`. After the existing `if is_attached { ... } else { ... }` block, add a special case for ClawHub. Replace the `else` branch (the attach branch):

In the `else` (not attached) branch, before the generic attach logic, add:

```rust
if vault.id == "clawhub" && vault.kind == "clawhub" {
    // Check if clawhub CLI is available
    if crate::infra::vault::clawhub::is_cli_available() {
        // CLI found — activate directly
        let store = ctx.store.clone();
        let tx = ctx.tx.clone();
        let id = crate::tui::app::NEXT_TASK_ID
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        tokio::task::spawn_blocking(move || {
            let _ = tx.send(AppEvent::TaskStarted {
                id,
                name: "Activating ClawHub vault".into(),
            });
            if let Ok(mut config) = store.load(crate::domain::scope::Scope::Global) {
                config.vaults.push("clawhub".to_string());
                let _ = store.save(crate::domain::scope::Scope::Global, &config);
            }
            let _ = tx.send(AppEvent::TaskProgress { id, percent: 100 });
            let _ = tx.send(AppEvent::TriggerReload);
            let _ = tx.send(AppEvent::TaskCompleted {
                id,
                message: "Activated ClawHub vault".into(),
            });
        });
    } else if crate::infra::vault::clawhub::is_homebrew_available() {
        // Offer to install via Homebrew
        state.list_mode = ListMode::ConfirmClawHubInstall;
        state.status_line =
            "ClawHub CLI not found. Install via Homebrew? [y/N]".to_string();
    } else {
        // No known install method
        state.status_line =
            "ClawHub CLI not found. Install manually from https://clawhub.ai"
                .to_string();
    }
    return Ok(());
}
```

- [ ] **Step 3: Handle `ConfirmClawHubInstall` mode input**

In `src/tui/event.rs`, in the `handle()` function, add a match arm before the existing `ConfirmDetachVault` handler:

```rust
KeyCode::Char('y') | KeyCode::Char('Y')
    if state.list_mode == ListMode::ConfirmClawHubInstall =>
{
    return handle_clawhub_install_confirm(state, ctx);
}
KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc
    if state.list_mode == ListMode::ConfirmClawHubInstall =>
{
    state.list_mode = ListMode::Normal;
    state.status_line = "Cancelled ClawHub CLI install".to_string();
    return Ok(ControlFlow::Continue);
}
```

Add the confirm handler:

```rust
fn handle_clawhub_install_confirm(
    state: &mut AppState,
    ctx: &EventContext,
) -> Result<ControlFlow> {
    state.list_mode = ListMode::Normal;
    let store = ctx.store.clone();
    let tx = ctx.tx.clone();
    let id = crate::tui::app::NEXT_TASK_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    tokio::task::spawn_blocking(move || {
        let _ = tx.send(AppEvent::TaskStarted {
            id,
            name: "Installing ClawHub CLI via Homebrew".into(),
        });
        match crate::infra::vault::clawhub::install_cli_via_homebrew() {
            Ok(()) => {
                let _ = tx.send(AppEvent::TaskProgress { id, percent: 50 });
                // Now activate the vault
                if let Ok(mut config) = store.load(crate::domain::scope::Scope::Global) {
                    config.vaults.push("clawhub".to_string());
                    let _ = store.save(crate::domain::scope::Scope::Global, &config);
                }
                let _ = tx.send(AppEvent::TaskProgress { id, percent: 100 });
                let _ = tx.send(AppEvent::TriggerReload);
                let _ = tx.send(AppEvent::TaskCompleted {
                    id,
                    message: "ClawHub CLI installed and vault activated".into(),
                });
            }
            Err(e) => {
                let _ = tx.send(AppEvent::TaskFailed {
                    id,
                    error: format!("Failed to install ClawHub CLI: {}", e),
                });
            }
        }
    });

    Ok(ControlFlow::Continue)
}
```

- [ ] **Step 4: Run full test suite**

Run: `cargo test`
Expected: All tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/tui/app.rs src/tui/event.rs
git commit -m "feat: ClawHub vault activation with CLI detection and install prompt"
```

---

### Task 9: Visual differentiation for remote packages

**Files:**
- Modify: `src/tui/widgets/list.rs`

- [ ] **Step 1: Update the `render` function to color remote packages**

In `src/tui/widgets/list.rs`, modify the `render()` function's item mapping. Replace the items iterator:

```rust
let items: Vec<ListItem> = packages
    .iter()
    .map(|pkg| {
        let version = pkg.identity.version.as_deref().unwrap_or("--");
        let status = install_status(
            config,
            &pkg.vault_id,
            &pkg.identity.name,
            &pkg.kind,
            &pkg.identity.sha10,
        );
        let text = format!(
            "{} {:<32} {:<8} {}",
            status, pkg.identity.name, version, pkg.vault_id
        );
        let style = if pkg.is_remote {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
        };
        ListItem::new(Line::from(text)).style(style)
    })
    .collect();
```

- [ ] **Step 2: Run full test suite**

Run: `cargo test`
Expected: All tests PASS.

- [ ] **Step 3: Commit**

```bash
git add src/tui/widgets/list.rs
git commit -m "feat: render remote ClawHub skills in dimmed color"
```

---

### Task 10: Handle `ClawHubSearchResults` event in TUI main loop

**Files:**
- Modify: `src/tui/mod.rs` (or wherever the main event loop processes `AppEvent`)

- [ ] **Step 1: Find the main event loop**

Search for where `AppEvent::TaskCompleted` is matched — this is where all event processing happens. It's likely in `src/tui/mod.rs` in the main `run()` function.

- [ ] **Step 2: Add the `ClawHubSearchResults` handler**

Add alongside the other `AppEvent` match arms:

```rust
AppEvent::ClawHubSearchResults { packages } => {
    state.remote_packages = packages;
    state.clawhub_searching = false;
}
```

- [ ] **Step 3: Also handle backspace search dispatch**

In `src/tui/event.rs`, in `handle_backspace()`, after `state.search_query.pop()`, when the query is still non-empty, also dispatch a new ClawHub search:

```rust
// In the else branch (not vault tab, not attach mode):
} else if active_kind != Some(crate::tui::app::TabKind::Vault) {
    state.search_query.pop();
    if state.search_query.is_empty() {
        state.list_mode = ListMode::Normal;
        state.remote_packages.clear();
        state.clawhub_searching = false;
    }
    state.selected_index = 0;
}
```

Note: we intentionally don't re-dispatch ClawHub search on every backspace to avoid excessive CLI calls. The user can type more to trigger a new search.

- [ ] **Step 4: Run full test suite**

Run: `cargo test`
Expected: All tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/tui/mod.rs src/tui/event.rs
git commit -m "feat: wire ClawHubSearchResults event into TUI main loop"
```

---

### Task 11: Register default ClawHub vault definition at bootstrap

**Files:**
- Modify: `src/app/bootstrap.rs`

- [ ] **Step 1: Write the failing test**

In `src/app/bootstrap.rs` tests:

```rust
#[test]
fn bootstrap_clawhub_vault_inactive_by_default() {
    let dir = tempfile::tempdir().unwrap();
    let workspace_root = dir.path().to_path_buf();
    let agk_dir = workspace_root.join(".agk");
    std::fs::create_dir_all(&agk_dir).unwrap();
    let global_dir = dir.path().join("global");
    std::fs::create_dir_all(&global_dir).unwrap();
    // No clawhub in config at all
    let config_content = "version = 1\nvaults = []\n";
    std::fs::write(global_dir.join("config.toml"), config_content).unwrap();
    let store = TomlConfigStore::new(
        global_dir.join("config.toml"),
        agk_dir.join("config.toml"),
    );
    let (registry, _scan, store) = build_with_store(workspace_root, store).unwrap();

    // ClawHub vault should appear in vault entries (for Vaults tab) but NOT be enabled
    let global_config = crate::app::ports::ConfigStorePort::load(
        &store,
        crate::domain::scope::Scope::Global,
    )
    .unwrap();
    let entries = build_vault_entries(&global_config, &global_config, &_scan, &registry);
    let clawhub_entry = entries.iter().find(|e| e.id == "clawhub");
    assert!(clawhub_entry.is_some(), "ClawHub should appear in vault entries");
    assert!(!clawhub_entry.unwrap().enabled, "ClawHub should be inactive by default");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test bootstrap_clawhub_vault_inactive_by_default -- --nocapture`
Expected: FAIL — no ClawHub entry appears.

- [ ] **Step 3: Ensure ClawHub vault def always exists in global config**

In `src/app/bootstrap.rs`, in `build_with_store()`, after loading `global_config` and before `build_vaults()`, inject the ClawHub vault definition if not present:

```rust
// Ensure ClawHub vault definition always exists (inactive by default)
if !global_config.vault_defs.contains_key("clawhub") {
    global_config.vault_defs.insert(
        "clawhub".to_string(),
        crate::domain::config::VaultSection {
            vault: Some(crate::domain::config::VaultConfig::Clawhub(
                crate::domain::config::ClawHubVaultSource {},
            )),
            skills: None,
            instructions: None,
        },
    );
}
```

Note: This is injected in memory only — not persisted to disk — so the default config file stays clean. The definition is persisted when the user actually activates it.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test bootstrap_clawhub_vault_inactive_by_default -- --nocapture`
Expected: PASS

- [ ] **Step 5: Run full test suite**

Run: `cargo test`
Expected: All tests PASS.

- [ ] **Step 6: Commit**

```bash
git add src/app/bootstrap.rs
git commit -m "feat: register ClawHub vault definition as inactive by default"
```

---

### Task 12: Show search spinner for ClawHub

**Files:**
- Modify: `src/tui/render.rs`

- [ ] **Step 1: Pass `clawhub_searching` to the status bar**

In `src/tui/render.rs`, in the `draw()` function, update the `status::render()` call to include the ClawHub searching state. The simplest approach is to append a search indicator to the `search_hint` in the header:

```rust
let search_hint = if state.search_query.is_empty() {
    String::new()
} else {
    let searching = if state.clawhub_searching {
        " (searching ClawHub...)"
    } else {
        ""
    };
    format!("  [ Search: {}{} ]", state.search_query, searching)
};
```

- [ ] **Step 2: Run full test suite**

Run: `cargo test`
Expected: All tests PASS.

- [ ] **Step 3: Manual verification**

Run `cargo run` and verify:
1. Vaults tab shows "clawhub" entry, unchecked
2. Pressing Space on it checks for CLI / offers install
3. When active, typing in Skills tab shows "(searching ClawHub...)" in header
4. Remote results appear dimmed
5. Space on a remote result shows two progress jobs

- [ ] **Step 4: Commit**

```bash
git add src/tui/render.rs
git commit -m "feat: show ClawHub search indicator in header"
```

---

### Task 13: Final integration test and cleanup

**Files:**
- All modified files

- [ ] **Step 1: Run full test suite**

Run: `cargo test`
Expected: All tests PASS.

- [ ] **Step 2: Run clippy**

Run: `cargo clippy -- -D warnings`
Expected: No warnings.

- [ ] **Step 3: Run rustfmt**

Run: `cargo fmt`

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "chore: clippy and fmt cleanup for ClawHub integration"
```
