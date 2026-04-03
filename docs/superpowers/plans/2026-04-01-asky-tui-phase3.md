# Phase 3 Implementation Plan — Live Vaults & Providers Tabs, Update, Version Extraction

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete all remaining features from the technical design doc: live Vaults tab, live Providers tab, update asset action, version extraction, and wired vault attach/detach.

**Architecture:** The Vaults and Providers tabs don't fit the FeatureSetPort scan model — they display config-level data, not scanned packages. We introduce a `TabKind` enum so the TUI can dispatch rendering to specialized widget functions for these config-driven tabs, while keeping the existing FeatureSetPort pattern for Skills/Instructions. The update action reuses existing install logic with sha10 comparison. Version extraction reads frontmatter from SKILL.md/AGENTS.md during scan.

**Tech Stack:** Rust, ratatui, crossterm, serde, toml, sha2

---

## Scope Check

This plan covers 5 remaining features (15, 16, 21, 24, 26) from FEATURES.md. Feature 18 (GithubVaultAdapter) is excluded — it requires an HTTP client dependency and GitHub API integration, warranting its own plan.

---

## File Structure

| File | Action | Responsibility |
|---|---|---|
| `src/tui/app.rs` | Modify | Add `TabKind` enum, vault/provider display data to AppState |
| `src/tui/render.rs` | Modify | Dispatch to vault/provider rendering for config tabs |
| `src/tui/widgets/list.rs` | Modify | Add `render_vaults()` and `render_providers()` functions |
| `src/tui/widgets/detail.rs` | Modify | Add `render_vault_detail()` and `render_provider_detail()` functions |
| `src/tui/event.rs` | Modify | Wire attach/detach/update actions, pass registry+store |
| `src/app/bootstrap.rs` | Modify | Register ClaudeCodeProvider, expose store+registry to TUI event loop |
| `src/app/actions.rs` | Modify | Add `update_asset()` function |
| `src/app/ports.rs` | Modify | Add `version()` method to FeatureSetPort |
| `src/infra/feature/skill.rs` | Modify | Implement version extraction from SKILL.md frontmatter |
| `src/infra/feature/instruction.rs` | Modify | Implement version extraction from AGENTS.md frontmatter |
| `src/infra/vault/local.rs` | Modify | Pass extracted version into AssetIdentity |
| `src/domain/asset.rs` | Modify | Add `VaultEntry` and `ProviderEntry` display structs |
| `src/main.rs` | Modify | Pass registry+store into TUI event loop |

---

## Task 1: Add VaultEntry and ProviderEntry display structs

These are lightweight structs for the TUI to display vault and provider rows without depending on infra types.

**Files:**
- Modify: `src/domain/asset.rs`

- [ ] **Step 1: Write the failing test**

```rust
// At bottom of src/domain/asset.rs, in #[cfg(test)] mod tests
#[test]
fn vault_entry_display_counts() {
    let entry = VaultEntry {
        id: "community".to_string(),
        kind: "github".to_string(),
        enabled: true,
        installed_skills: 30,
        available_skills: 48,
        installed_instructions: 8,
        available_instructions: 12,
    };
    assert_eq!(entry.id, "community");
    assert_eq!(entry.counts_label(), "30/48s  8/12i");
}

#[test]
fn provider_entry_active_marker() {
    let entry = ProviderEntry {
        id: "claude-code".to_string(),
        active: true,
    };
    assert!(entry.active);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test vault_entry_display_counts provider_entry_active_marker -- --nocapture 2>&1 | tail -20`
Expected: FAIL — structs don't exist

- [ ] **Step 3: Write minimal implementation**

Add to `src/domain/asset.rs`:

```rust
/// Display-only struct for the Vaults tab.
#[derive(Debug, Clone)]
pub struct VaultEntry {
    pub id: String,
    pub kind: String,
    pub enabled: bool,
    pub installed_skills: usize,
    pub available_skills: usize,
    pub installed_instructions: usize,
    pub available_instructions: usize,
}

impl VaultEntry {
    pub fn counts_label(&self) -> String {
        format!(
            "{}/{}s  {}/{}i",
            self.installed_skills, self.available_skills,
            self.installed_instructions, self.available_instructions,
        )
    }
}

/// Display-only struct for the Providers tab.
#[derive(Debug, Clone)]
pub struct ProviderEntry {
    pub id: String,
    pub active: bool,
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test vault_entry_display_counts provider_entry_active_marker -- --nocapture 2>&1 | tail -20`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/domain/asset.rs
git commit -m "feat(domain): add VaultEntry and ProviderEntry display structs"
```

---

## Task 2: Add TabKind to AppState for config-driven tabs

The TUI needs to know which tabs show scanned packages vs config-level data. Add `TabKind` and vault/provider entry lists to AppState.

**Files:**
- Modify: `src/tui/app.rs`

- [ ] **Step 1: Write the failing test**

Add to `src/tui/app.rs` tests:

```rust
#[test]
fn tab_kind_vaults_and_providers() {
    use crate::domain::asset::{VaultEntry, ProviderEntry};
    let mut state = AppState::new(
        vec!["Skills".into(), "Instructions".into(), "Providers".into(), "Vaults".into()],
        vec![true, true, true, true],
        HashMap::new(),
    );
    state.tab_kinds = vec![TabKind::Asset, TabKind::Asset, TabKind::Provider, TabKind::Vault];
    assert_eq!(state.tab_kinds[2], TabKind::Provider);
    assert_eq!(state.tab_kinds[3], TabKind::Vault);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test tab_kind_vaults_and_providers -- --nocapture 2>&1 | tail -20`
Expected: FAIL — `TabKind` doesn't exist

- [ ] **Step 3: Write minimal implementation**

Add to `src/tui/app.rs`:

```rust
use crate::domain::asset::{VaultEntry, ProviderEntry};

#[derive(Debug, Clone, PartialEq)]
pub enum TabKind {
    Asset,      // Skills, Instructions — shows ScannedPackages
    Vault,      // Shows VaultEntry list from config
    Provider,   // Shows ProviderEntry list from config
}
```

Add fields to `AppState`:

```rust
pub tab_kinds: Vec<TabKind>,
pub vault_entries: Vec<VaultEntry>,
pub provider_entries: Vec<ProviderEntry>,
```

Initialize in `AppState::new()`:

```rust
tab_kinds: Vec::new(),
vault_entries: Vec::new(),
provider_entries: Vec::new(),
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test tab_kind_vaults_and_providers -- --nocapture 2>&1 | tail -20`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/tui/app.rs
git commit -m "feat(tui): add TabKind enum and vault/provider entries to AppState"
```

---

## Task 3: Render Vaults tab list

Replace the stub rendering for the Vaults tab with real vault list columns: checkbox, vault id, type, skills count, instructions count.

**Files:**
- Modify: `src/tui/widgets/list.rs`

- [ ] **Step 1: Add render_vaults function**

Add to `src/tui/widgets/list.rs`:

```rust
use crate::domain::asset::VaultEntry;

pub fn render_vaults(
    frame: &mut Frame,
    area: Rect,
    vaults: &[VaultEntry],
    selected: usize,
) {
    let block = Block::default().borders(Borders::ALL).title("Vaults");

    if vaults.is_empty() {
        let items = vec![ListItem::new(Line::from("  No vaults attached. Press 'a' to add one."))];
        frame.render_widget(List::new(items).block(block), area);
        return;
    }

    let items: Vec<ListItem> = vaults
        .iter()
        .map(|v| {
            let check = if v.enabled { "[x]" } else { "[ ]" };
            ListItem::new(Line::from(format!(
                "{} {:<20} {:<8} {}",
                check, v.id, v.kind, v.counts_label()
            )))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut state = ListState::default();
    if !vaults.is_empty() {
        state.select(Some(selected));
    }

    frame.render_stateful_widget(list, area, &mut state);
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check 2>&1 | tail -20`
Expected: compiles (function not yet called)

- [ ] **Step 3: Commit**

```bash
git add src/tui/widgets/list.rs
git commit -m "feat(tui): add render_vaults list widget"
```

---

## Task 4: Render Providers tab list

**Files:**
- Modify: `src/tui/widgets/list.rs`

- [ ] **Step 1: Add render_providers function**

Add to `src/tui/widgets/list.rs`:

```rust
use crate::domain::asset::ProviderEntry;

pub fn render_providers(
    frame: &mut Frame,
    area: Rect,
    providers: &[ProviderEntry],
    selected: usize,
) {
    let block = Block::default().borders(Borders::ALL).title("Providers");

    if providers.is_empty() {
        let items = vec![ListItem::new(Line::from("  No providers installed."))];
        frame.render_widget(List::new(items).block(block), area);
        return;
    }

    let items: Vec<ListItem> = providers
        .iter()
        .map(|p| {
            let marker = if p.active { " ✓" } else { "  " };
            ListItem::new(Line::from(format!("  {}{}", p.id, marker)))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut state = ListState::default();
    if !providers.is_empty() {
        state.select(Some(selected));
    }

    frame.render_stateful_widget(list, area, &mut state);
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check 2>&1 | tail -20`
Expected: compiles

- [ ] **Step 3: Commit**

```bash
git add src/tui/widgets/list.rs
git commit -m "feat(tui): add render_providers list widget"
```

---

## Task 5: Render vault detail and provider detail panels

**Files:**
- Modify: `src/tui/widgets/detail.rs`

- [ ] **Step 1: Add render_vault_detail function**

Add to `src/tui/widgets/detail.rs`:

```rust
use crate::domain::asset::{VaultEntry, ProviderEntry};

pub fn render_vault_detail(frame: &mut Frame, area: Rect, vault: Option<&VaultEntry>) {
    let block = Block::default().borders(Borders::ALL).title("Detail");

    let lines: Vec<Line> = match vault {
        None => vec![Line::from("  No vault selected")],
        Some(v) => {
            let label = |s: &str| Span::styled(s.to_string(), Style::default().fg(Color::Yellow));
            vec![
                Line::from(vec![label("Vault ID: "), Span::raw(v.id.clone())]),
                Line::from(vec![label("Type:     "), Span::raw(v.kind.clone())]),
                Line::from(vec![
                    label("Enabled:  "),
                    Span::raw(if v.enabled { "yes" } else { "no" }),
                ]),
                Line::from(Span::raw("")),
                Line::from(vec![
                    label("Skills:       "),
                    Span::raw(format!("{} installed / {} available", v.installed_skills, v.available_skills)),
                ]),
                Line::from(vec![
                    label("Instructions: "),
                    Span::raw(format!("{} installed / {} available", v.installed_instructions, v.available_instructions)),
                ]),
            ]
        }
    };

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

pub fn render_provider_detail(frame: &mut Frame, area: Rect, provider: Option<&ProviderEntry>) {
    let block = Block::default().borders(Borders::ALL).title("Detail");

    let lines: Vec<Line> = match provider {
        None => vec![Line::from("  No provider selected")],
        Some(p) => {
            let label = |s: &str| Span::styled(s.to_string(), Style::default().fg(Color::Yellow));
            vec![
                Line::from(vec![label("Provider: "), Span::raw(p.id.clone())]),
                Line::from(vec![
                    label("Status:   "),
                    Span::raw(if p.active { "active" } else { "installed" }),
                ]),
            ]
        }
    };

    frame.render_widget(Paragraph::new(lines).block(block), area);
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check 2>&1 | tail -20`
Expected: compiles

- [ ] **Step 3: Commit**

```bash
git add src/tui/widgets/detail.rs
git commit -m "feat(tui): add vault and provider detail panel widgets"
```

---

## Task 6: Dispatch rendering by TabKind in render.rs

Wire the new list/detail functions into the main draw loop, dispatching based on `TabKind`.

**Files:**
- Modify: `src/tui/render.rs`

- [ ] **Step 1: Update draw() to dispatch by TabKind**

Replace the content area rendering in `src/tui/render.rs`:

```rust
use crate::tui::app::TabKind;

// Replace the existing content rendering block with:
let active_kind = state.tab_kinds.get(state.active_tab).cloned().unwrap_or(TabKind::Asset);

match active_kind {
    TabKind::Asset => {
        let is_live = state.is_active_tab_live();
        let filtered = state.filtered_packages();
        let selected_pkg = filtered.get(state.selected_index).copied();

        list::render(
            frame,
            layout.list,
            &filtered,
            state.selected_index,
            !is_live,
            state.active_config(),
        );
        detail::render(frame, layout.detail, selected_pkg, !is_live);
    }
    TabKind::Vault => {
        list::render_vaults(frame, layout.list, &state.vault_entries, state.selected_index);
        let selected_vault = state.vault_entries.get(state.selected_index);
        detail::render_vault_detail(frame, layout.detail, selected_vault);
    }
    TabKind::Provider => {
        list::render_providers(frame, layout.list, &state.provider_entries, state.selected_index);
        let selected_provider = state.provider_entries.get(state.selected_index);
        detail::render_provider_detail(frame, layout.detail, selected_provider);
    }
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check 2>&1 | tail -20`
Expected: compiles

- [ ] **Step 3: Commit**

```bash
git add src/tui/render.rs
git commit -m "feat(tui): dispatch rendering by TabKind for vault/provider tabs"
```

---

## Task 7: Build vault/provider entries from config in bootstrap

Populate `vault_entries` and `provider_entries` in AppState from the loaded configs and scanned data.

**Files:**
- Modify: `src/app/bootstrap.rs`
- Modify: `src/tui/app.rs`

- [ ] **Step 1: Add helper to build vault entries**

Add to `src/app/bootstrap.rs`:

```rust
use crate::domain::asset::{VaultEntry, ProviderEntry};
use crate::domain::config::ConfigFile;
use crate::tui::app::TabKind;

pub fn build_vault_entries(
    config: &ConfigFile,
    scan: &ScanResult,
    registry: &Registry,
) -> Vec<VaultEntry> {
    let mut entries = Vec::new();
    for vault_id in &config.vaults {
        let kind = config.vault_defs.get(vault_id)
            .and_then(|s| s.vault.as_ref())
            .map(|v| match v {
                crate::domain::config::VaultConfig::Local(_) => "local",
                crate::domain::config::VaultConfig::Github(_) => "github",
            })
            .unwrap_or("local")
            .to_string();

        let installed_skills = config.installed_skills(vault_id).len();
        let installed_instructions = config.installed_instructions(vault_id).len();

        // Count available from scan (all tabs that are skill/instruction)
        let mut available_skills = 0usize;
        let mut available_instructions = 0usize;
        for (tab_idx, pkgs) in scan.packages_by_tab.iter().enumerate() {
            let is_skill = registry.feature_sets.get(tab_idx)
                .map(|f| f.kind_name() == "skill")
                .unwrap_or(false);
            let is_instruction = registry.feature_sets.get(tab_idx)
                .map(|f| f.kind_name() == "instruction")
                .unwrap_or(false);
            for pkg in pkgs {
                if pkg.vault_id == *vault_id {
                    if is_skill { available_skills += 1; }
                    if is_instruction { available_instructions += 1; }
                }
            }
        }

        entries.push(VaultEntry {
            id: vault_id.clone(),
            kind,
            enabled: true,
            installed_skills,
            available_skills,
            installed_instructions,
            available_instructions,
        });
    }
    entries
}

pub fn build_provider_entries(config: &ConfigFile) -> Vec<ProviderEntry> {
    config.providers.iter().map(|id| ProviderEntry {
        id: id.clone(),
        active: config.providers.first() == Some(id),
    }).collect()
}
```

- [ ] **Step 2: Wire into build() function**

Update the `build()` function to set `tab_kinds`, `vault_entries`, and `provider_entries` on AppState after construction. Since `AppState` is constructed in `main.rs`, we need to return these from `build()`. Add to the `ScanResult` struct:

```rust
pub struct ScanResult {
    pub packages_by_tab: Vec<Vec<ScannedPackage>>,
    pub tab_kinds: Vec<TabKind>,
    pub vault_entries: Vec<VaultEntry>,
    pub provider_entries: Vec<ProviderEntry>,
}
```

Update `build()` to populate these:

```rust
pub fn build(workspace_root: PathBuf) -> Result<(Registry, ScanResult, TomlConfigStore)> {
    // ... existing code ...

    let store = TomlConfigStore::standard(&workspace_root);

    // Load active scope config for display data
    let config = store.load(crate::domain::scope::Scope::Global).unwrap_or_default();

    let scan_result = scan(&registry)?;

    let vault_entries = build_vault_entries(&config, &scan_result, &registry);
    let provider_entries = build_provider_entries(&config);

    let tab_kinds = registry.feature_sets.iter().map(|f| {
        match f.kind_name() {
            "vault" => TabKind::Vault,
            "provider" => TabKind::Provider,
            _ => TabKind::Asset,
        }
    }).collect();

    let result = ScanResult {
        packages_by_tab: scan_result.packages_by_tab,
        tab_kinds,
        vault_entries,
        provider_entries,
    };

    Ok((registry, result, store))
}
```

- [ ] **Step 3: Update main.rs to set AppState fields**

In `main.rs` (or wherever AppState is constructed from ScanResult), set:

```rust
state.tab_kinds = scan.tab_kinds;
state.vault_entries = scan.vault_entries;
state.provider_entries = scan.provider_entries;
```

- [ ] **Step 4: Verify it compiles and runs**

Run: `cargo check 2>&1 | tail -20`
Expected: compiles

- [ ] **Step 5: Commit**

```bash
git add src/app/bootstrap.rs src/tui/app.rs src/main.rs
git commit -m "feat(bootstrap): build vault/provider entries from config for live tabs"
```

---

## Task 8: Wire vault attach action to prompt UI

When the user presses Enter in AttachVault mode, call `attach_vault()` and rescan.

**Files:**
- Modify: `src/tui/event.rs`
- Modify: `src/main.rs` (if needed to pass store/registry)

- [ ] **Step 1: Design the approach**

The event handler currently only has `&mut AppState`. To call `attach_vault()`, it needs access to `ConfigStorePort`. Two options:
1. Pass `&dyn ConfigStorePort` into `handle()`
2. Store a reference/Arc in AppState

The cleanest approach: add a `context` parameter to `handle()` that carries references to store and registry.

Add a struct to `src/tui/event.rs`:

```rust
pub struct EventContext<'a> {
    pub store: &'a dyn crate::app::ports::ConfigStorePort,
    pub registry: &'a crate::app::registry::Registry,
}
```

Update `handle()` signature:

```rust
pub fn handle(state: &mut AppState, ctx: &EventContext) -> Result<ControlFlow> {
```

- [ ] **Step 2: Wire attach on Enter in AttachVault mode**

Replace the Enter handler for AttachVault:

```rust
KeyCode::Enter if state.list_mode == ListMode::AttachVault => {
    state.list_mode = ListMode::Normal;
    let path = std::mem::take(&mut state.prompt_buffer);
    if path.is_empty() {
        state.status_line = "Cancelled — empty path".to_string();
    } else {
        let vault_id = std::path::Path::new(&path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let vault_config = crate::domain::config::VaultConfig::Local(
            crate::domain::config::LocalVaultSource { path: path.clone() },
        );
        match crate::app::actions::attach_vault(
            state.active_scope.clone(),
            vault_id.clone(),
            vault_config,
            ctx.store,
        ) {
            Ok(()) => {
                // Reload config and rebuild entries
                if let Ok(config) = ctx.store.load(state.active_scope.clone()) {
                    state.configs.insert(state.active_scope.clone(), config);
                }
                state.status_line = format!("Attached vault '{}'", vault_id);
            }
            Err(e) => {
                state.status_line = format!("Failed to attach: {}", e);
            }
        }
    }
}
```

- [ ] **Step 3: Update all callers of handle() to pass EventContext**

In `main.rs`, construct `EventContext` and pass it:

```rust
let ctx = event::EventContext { store: &store, registry: &registry };
// In the loop:
match event::handle(&mut state, &ctx)? { ... }
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo check 2>&1 | tail -20`
Expected: compiles

- [ ] **Step 5: Commit**

```bash
git add src/tui/event.rs src/main.rs
git commit -m "feat(tui): wire vault attach action to prompt UI with EventContext"
```

---

## Task 9: Wire vault detach action to 'd' key

**Files:**
- Modify: `src/tui/event.rs`

- [ ] **Step 1: Replace detach stub with real action**

Replace the `'d'` key handler:

```rust
KeyCode::Char('d') if state.list_mode == ListMode::Normal => {
    let vaults_idx = state.tab_names.iter().position(|n| n == "Vaults").unwrap_or(3);
    if state.active_tab == vaults_idx {
        if let Some(vault) = state.vault_entries.get(state.selected_index) {
            let vault_id = vault.id.clone();
            match crate::app::actions::detach_vault(
                state.active_scope.clone(),
                &vault_id,
                ctx.store,
            ) {
                Ok(()) => {
                    if let Ok(config) = ctx.store.load(state.active_scope.clone()) {
                        state.configs.insert(state.active_scope.clone(), config);
                    }
                    state.vault_entries.retain(|v| v.id != vault_id);
                    if state.selected_index > 0 && state.selected_index >= state.vault_entries.len() {
                        state.selected_index = state.vault_entries.len().saturating_sub(1);
                    }
                    state.status_line = format!("Detached vault '{}'", vault_id);
                }
                Err(e) => {
                    state.status_line = format!("Failed to detach: {}", e);
                }
            }
        }
    } else {
        state.status_line = "Press 'd' on the Vaults tab to detach".to_string();
    }
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check 2>&1 | tail -20`
Expected: compiles

- [ ] **Step 3: Commit**

```bash
git add src/tui/event.rs
git commit -m "feat(tui): wire vault detach to 'd' key on Vaults tab"
```

---

## Task 10: Add update_asset action

**Files:**
- Modify: `src/app/actions.rs`

- [ ] **Step 1: Write the failing test**

Add to `src/app/actions.rs` tests:

```rust
#[test]
fn update_asset_replaces_identity_in_config() {
    let store = FakeStore::default();
    let provider = FakeProvider::new();

    // Pre-install with old sha
    let mut config = ConfigFile::default();
    config.providers = vec!["fake".to_string()];
    config.vault_defs.insert("workspace".to_string(), VaultSection {
        vault: None,
        skills: Some(AssetBucket { items: vec!["[my-skill:--:old_sha_old]".to_string()] }),
        instructions: None,
    });
    store.save(Scope::Workspace, &config).unwrap();

    // Update with new sha
    let pkg = ScannedPackage {
        identity: AssetIdentity::new("my-skill", None, "new_sha_new"),
        path: std::path::PathBuf::from("/fake"),
        vault_id: "workspace".to_string(),
        kind: AssetKind::Skill,
    };
    update_asset(Scope::Workspace, &pkg, &store, &provider).unwrap();

    let loaded = store.load(Scope::Workspace).unwrap();
    let skills = loaded.installed_skills("workspace");
    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].sha10, "new_sha_new");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test update_asset_replaces -- --nocapture 2>&1 | tail -20`
Expected: FAIL — function doesn't exist

- [ ] **Step 3: Write minimal implementation**

Add to `src/app/actions.rs`:

```rust
/// Update an installed asset: remove old identity, reinstall from scanned package.
pub fn update_asset(
    scope: Scope,
    pkg: &ScannedPackage,
    store: &dyn ConfigStorePort,
    provider: &dyn ProviderPort,
) -> Result<()> {
    // Remove old identity for this name from config
    let mut config = store.load(scope.clone())?;
    if let Some(section) = config.vault_defs.get_mut(&pkg.vault_id) {
        let name = &pkg.identity.name;
        match pkg.kind {
            AssetKind::Skill => {
                if let Some(bucket) = section.skills.as_mut() {
                    bucket.items.retain(|s| {
                        crate::domain::config::parse_identity(s)
                            .map(|id| id.name != *name)
                            .unwrap_or(true)
                    });
                }
            }
            AssetKind::Instruction => {
                if let Some(bucket) = section.instructions.as_mut() {
                    bucket.items.retain(|s| {
                        crate::domain::config::parse_identity(s)
                            .map(|id| id.name != *name)
                            .unwrap_or(true)
                    });
                }
            }
        }
    }
    store.save(scope.clone(), &config)?;

    // Re-install with new identity
    install_asset(scope, pkg, store, provider)
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test update_asset_replaces -- --nocapture 2>&1 | tail -20`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/app/actions.rs
git commit -m "feat(app): add update_asset action that replaces identity in config"
```

---

## Task 11: Wire 'u' key to update action

**Files:**
- Modify: `src/tui/event.rs`

- [ ] **Step 1: Replace update stub with real action**

Replace the `'u'` handler:

```rust
KeyCode::Char('u') if state.list_mode == ListMode::Normal => {
    let active_kind = state.tab_kinds.get(state.active_tab).cloned().unwrap_or(TabKind::Asset);
    if active_kind != TabKind::Asset {
        state.status_line = "Update only applies to Skills/Instructions tabs".to_string();
    } else if !state.active_scope_has_provider() {
        let providers_idx = state.tab_names.iter().position(|n| n == "Providers").unwrap_or(2);
        apply_space_no_provider(state, providers_idx);
    } else if let Some(pkg) = state.filtered_packages().get(state.selected_index).cloned() {
        // Check if installed — only update installed items
        let is_installed = state.is_installed(&pkg.vault_id, &pkg.identity.name, &pkg.kind);
        if !is_installed {
            state.status_line = "Item not installed — use Space to install first".to_string();
        } else if let Some(provider) = ctx.registry.providers.first() {
            match crate::app::actions::update_asset(
                state.active_scope.clone(),
                pkg,
                ctx.store,
                provider.as_ref(),
            ) {
                Ok(()) => {
                    if let Ok(config) = ctx.store.load(state.active_scope.clone()) {
                        state.configs.insert(state.active_scope.clone(), config);
                    }
                    state.status_line = format!("Updated '{}'", pkg.identity.name);
                }
                Err(e) => {
                    state.status_line = format!("Update failed: {}", e);
                }
            }
        }
    }
}
```

Note: The `filtered_packages()` returns `&ScannedPackage` references. To call `update_asset()` we need to clone. Adjust:

```rust
} else {
    let pkg_clone = {
        let filtered = state.filtered_packages();
        filtered.get(state.selected_index).map(|p| (*p).clone())
    };
    if let Some(pkg) = pkg_clone {
        let is_installed = state.is_installed(&pkg.vault_id, &pkg.identity.name, &pkg.kind);
        if !is_installed {
            state.status_line = "Item not installed — use Space to install first".to_string();
        } else if let Some(provider) = ctx.registry.providers.first() {
            match crate::app::actions::update_asset(
                state.active_scope.clone(),
                &pkg,
                ctx.store,
                provider.as_ref(),
            ) {
                Ok(()) => {
                    if let Ok(config) = ctx.store.load(state.active_scope.clone()) {
                        state.configs.insert(state.active_scope.clone(), config);
                    }
                    state.status_line = format!("Updated '{}'", pkg.identity.name);
                }
                Err(e) => {
                    state.status_line = format!("Update failed: {}", e);
                }
            }
        }
    }
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check 2>&1 | tail -20`
Expected: compiles

- [ ] **Step 3: Commit**

```bash
git add src/tui/event.rs
git commit -m "feat(tui): wire 'u' key to update asset action"
```

---

## Task 12: Add version extraction to FeatureSetPort

Extract version from SKILL.md / AGENTS.md frontmatter during scan.

**Files:**
- Modify: `src/app/ports.rs`
- Modify: `src/infra/feature/skill.rs`
- Modify: `src/infra/feature/instruction.rs`
- Modify: `src/infra/vault/local.rs`

- [x] **Step 1: Write the failing test for version extraction**

Add to `src/infra/feature/skill.rs` tests:

```rust
#[test]
fn extract_version_from_frontmatter() {
    let dir = tempfile::tempdir().unwrap();
    let skill_dir = dir.path().join("my-skill");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: my-skill\nversion: 2.1.0\n---\n# My Skill\n",
    ).unwrap();
    let version = SkillFeatureSet.extract_version(&skill_dir);
    assert_eq!(version, Some("2.1.0".to_string()));
}

#[test]
fn extract_version_none_when_no_frontmatter() {
    let dir = tempfile::tempdir().unwrap();
    let skill_dir = dir.path().join("my-skill");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(skill_dir.join("SKILL.md"), "# My Skill\n").unwrap();
    let version = SkillFeatureSet.extract_version(&skill_dir);
    assert!(version.is_none());
}
```

- [x] **Step 2: Run test to verify it fails**

Run: `cargo test extract_version -- --nocapture 2>&1 | tail -20`
Expected: FAIL — method doesn't exist

- [x] **Step 3: Add extract_version to FeatureSetPort trait**

In `src/app/ports.rs`, add to the trait:

```rust
/// Extract version from package metadata (e.g., frontmatter). Returns None if unavailable.
fn extract_version(&self, _path: &Path) -> Option<String> {
    None
}
```

- [x] **Step 4: Implement in SkillFeatureSet**

Add to `src/infra/feature/skill.rs`:

```rust
fn extract_version(&self, path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path.join("SKILL.md")).ok()?;
    extract_frontmatter_version(&content)
}
```

Add helper function in the same file:

```rust
fn extract_frontmatter_version(content: &str) -> Option<String> {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return None;
    }
    let rest = &content[3..];
    let end = rest.find("---")?;
    let frontmatter = &rest[..end];
    for line in frontmatter.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("version:") {
            let version = value.trim().trim_matches('"').trim_matches('\'');
            if !version.is_empty() {
                return Some(version.to_string());
            }
        }
    }
    None
}
```

- [x] **Step 5: Implement in InstructionFeatureSet**

Add to `src/infra/feature/instruction.rs`:

```rust
fn extract_version(&self, path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path.join("AGENTS.md")).ok()?;
    extract_frontmatter_version(&content)
}
```

Add the same `extract_frontmatter_version` helper. To avoid duplication, move it to a shared location. Add to `src/infra/feature/mod.rs`:

```rust
pub fn extract_frontmatter_version(content: &str) -> Option<String> {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return None;
    }
    let rest = &content[3..];
    let end = rest.find("---")?;
    let frontmatter = &rest[..end];
    for line in frontmatter.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("version:") {
            let version = value.trim().trim_matches('"').trim_matches('\'');
            if !version.is_empty() {
                return Some(version.to_string());
            }
        }
    }
    None
}
```

Then in both `skill.rs` and `instruction.rs`, call `super::extract_frontmatter_version(...)`.

- [x] **Step 6: Run tests to verify they pass**

Run: `cargo test extract_version -- --nocapture 2>&1 | tail -20`
Expected: PASS

- [x] **Step 7: Commit**

```bash
git add src/app/ports.rs src/infra/feature/mod.rs src/infra/feature/skill.rs src/infra/feature/instruction.rs
git commit -m "feat(infra): extract version from SKILL.md/AGENTS.md frontmatter"
```

---

## Task 13: Use extracted version in LocalVaultAdapter scan

**Files:**
- Modify: `src/infra/vault/local.rs`

- [x] **Step 1: Write the failing test**

Add to `src/infra/vault/local.rs` tests:

```rust
#[test]
fn list_packages_extracts_version_from_frontmatter() {
    let dir = tempfile::tempdir().unwrap();
    let skill_dir = dir.path().join("skills").join("my-skill");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: my-skill\nversion: 1.5.0\n---\n# My Skill\n",
    ).unwrap();
    let vault = LocalVaultAdapter::new("workspace", dir.path().to_path_buf());
    let pkgs = vault.list_packages(&SkillFeatureSet).unwrap();
    assert_eq!(pkgs[0].identity.version, Some("1.5.0".to_string()));
}
```

- [x] **Step 2: Run test to verify it fails**

Run: `cargo test list_packages_extracts_version -- --nocapture 2>&1 | tail -20`
Expected: FAIL — version is None

- [x] **Step 3: Update list_packages to extract version**

In `src/infra/vault/local.rs`, in the `list_packages` method, replace:

```rust
let identity = AssetIdentity::new(name, None, sha10);
```

with:

```rust
let version = feature.extract_version(&path);
let identity = AssetIdentity::new(name, version, sha10);
```

- [x] **Step 4: Run test to verify it passes**

Run: `cargo test list_packages_extracts_version -- --nocapture 2>&1 | tail -20`
Expected: PASS

- [x] **Step 5: Commit**

```bash
git add src/infra/vault/local.rs
git commit -m "feat(vault): extract version from package frontmatter during scan"
```

---

## Task 14: Mark Vaults and Providers tabs as live in bootstrap

Remove StubFeatureSet for Vaults/Providers — they're now rendered via TabKind dispatch.

**Files:**
- Modify: `src/app/bootstrap.rs`

- [x] **Step 1: Keep StubFeatureSet but mark tab_live as true**

The StubFeatureSet is still needed to hold the tab name in the registry. But now the rendering dispatches by TabKind, so `is_stub()` is no longer checked for Vault/Provider tabs. We need `tab_live` to be `true` for these tabs so the status bar shows correct state.

Actually, with TabKind dispatch in render.rs, the `is_stub` / `tab_live` check is only used for Asset tabs. For Vault/Provider tabs, rendering goes through the new path regardless. So we can simply set `tab_live[2] = true` and `tab_live[3] = true`.

The simplest change: in `AppState::is_active_tab_live()`, also return true for Vault and Provider TabKinds:

```rust
pub fn is_active_tab_live(&self) -> bool {
    match self.tab_kinds.get(self.active_tab) {
        Some(TabKind::Vault) | Some(TabKind::Provider) => true,
        _ => self.tab_live.get(self.active_tab).copied().unwrap_or(false),
    }
}
```

- [x] **Step 2: Verify it compiles**

Run: `cargo check 2>&1 | tail -20`
Expected: compiles

- [x] **Step 3: Commit**

```bash
git add src/tui/app.rs
git commit -m "feat(tui): mark Vaults and Providers tabs as live via TabKind"
```

---

## Task 15: Update FEATURES.md

**Files:**
- Modify: `docs/FEATURES.md`

- [x] **Step 1: Update feature statuses**

```
| 15 | Providers tab | §11.4 | `[x]` |
| 16 | Vaults tab | §11.2 | `[x]` |
| 21 | Update asset | §12.2 | `[x]` |
| 24 | Vault attach/detach | §12.3 | `[x]` |
| 26 | Version extraction from package | §4.2 | `[x]` |
```

- [x] **Step 2: Commit**

```bash
git add docs/FEATURES.md
git commit -m "docs: update FEATURES.md for Phase 3 completion"
```

---

## Summary

After Phase 3, the only remaining feature from the technical design doc is:
- **Feature 18: GithubVaultAdapter** — requires HTTP client (reqwest), GitHub API integration, and its own plan

All TUI tabs will be fully live, all CRUD actions wired, and version extraction operational.
