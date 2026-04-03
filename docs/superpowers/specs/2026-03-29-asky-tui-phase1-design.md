# asky — TUI Phase 1 Design Spec

**Date:** 2026-03-29
**Status:** Draft
**References:** [Technical Design Doc](../../plans/20260328_technical_design.md)

---

## 1. Scope

Phase 1 delivers a runnable `asky` binary with:

- Full `ratatui` TUI shell (5-zone layout, data-driven tabs, keybindings)
- **Skills feature set** wired to real data — scans workspace `skills/` on startup
- **Instructions, Providers, Vaults** render as stub tabs
- `sha10` hashing for discovered packages
- `docs/FEATURES.md` tracking all design doc features
- **Hexagonal architecture** — domain core is dependency-free; vaults, providers, and feature sets are pluggable adapters behind port traits

No `config.toml` persistence. No GitHub vault. No provider install logic. Read-only scan only.

---

## 2. Architecture: Hexagonal

```
         ┌──────────────────────────────────────┐
         │             Domain Core              │
         │  Asset, Identity, Scope, sha10       │
         │  (no external dependencies)          │
         └────────────────┬─────────────────────┘
                          │
         ┌────────────────▼─────────────────────┐
         │          Application Layer           │
         │  AppService orchestrates use-cases   │
         │  Ports (traits):                     │
         │    VaultPort                         │
         │    ProviderPort                      │
         │    FeatureSetPort                    │
         │  Registry holds all adapters         │
         └───────────┬──────────────┬───────────┘
                     │              │
         ┌───────────▼──┐  ┌────────▼────────────────┐
         │  TUI Adapter │  │     Infra Adapters       │
         │  (ratatui)   │  │  LocalVaultAdapter       │
         │  driven by   │  │  GithubVaultAdapter      │
         │  Registry    │  │  ClaudeCodeProvider      │
         └──────────────┘  │  SkillFeatureSet         │
                           │  InstructionFeatureSet   │
                           └─────────────────────────┘
```

**Rules:**
- `domain/` has zero external crate dependencies — pure Rust types and logic
- `app/` depends only on `domain/` and its own port traits — no infra imports
- `infra/` implements port traits — all I/O lives here
- `tui/` depends on `app/` and `domain/` only — never imports `infra/` directly
- Adding a new vault/provider/feature set = implement the trait in `infra/`, register in bootstrap — zero changes to core, app, or TUI

---

## 3. Port Traits

### 3.1 FeatureSetPort

Defines a managed asset category. Implementing this trait registers a new tab in the TUI.

```rust
trait FeatureSetPort: Send + Sync {
    fn kind_name(&self) -> &str;                          // "skill", "instruction"
    fn scan_root(&self) -> &str;                          // "skills/", "instructions/"
    fn is_package(&self, path: &Path) -> bool;            // e.g. SKILL.md present
    fn hash_files(&self, path: &Path) -> Vec<PathBuf>;    // files included in sha10
    fn display_name(&self) -> &str;                       // tab label: "Skills"
}
```

### 3.2 VaultPort

```rust
trait VaultPort: Send + Sync {
    fn id(&self) -> &str;
    fn kind_name(&self) -> &str;                          // "local", "github"
    fn list_packages(&self, feature: &dyn FeatureSetPort) -> Result<Vec<ScannedPackage>>;
}
```

### 3.3 ProviderPort

```rust
trait ProviderPort: Send + Sync {
    fn id(&self) -> &str;
    fn install(&self, pkg: &ScannedPackage, scope: Scope) -> Result<()>;
    fn remove(&self, identity: &AssetIdentity, scope: Scope) -> Result<()>;
}
```

---

## 4. Registry

Holds all registered adapters. Constructed in `app::bootstrap`, passed into `AppService` and TUI.

```rust
struct Registry {
    vaults: Vec<Box<dyn VaultPort>>,
    providers: Vec<Box<dyn ProviderPort>>,
    feature_sets: Vec<Box<dyn FeatureSetPort>>,
}
```

**TUI tabs are generated from `registry.feature_sets`** — order of registration = tab order. No hardcoded tab list.

Phase 1 registration in bootstrap:
```rust
registry.register_feature_set(Box::new(SkillFeatureSet));
// registry.register_feature_set(Box::new(InstructionFeatureSet)); // future
registry.register_vault(Box::new(LocalVaultAdapter::new(workspace_path)));
```

---

## 5. Module Layout

```
src/
├── main.rs
├── cli/
│   ├── mod.rs
│   └── entry.rs                — clap: no-arg default launches TUI
├── tui/
│   ├── mod.rs
│   ├── app.rs                  — AppState (data-driven tabs from Registry)
│   ├── event.rs                — crossterm input loop, AppEvent dispatch
│   ├── render.rs               — draw() dispatches to active feature set tab
│   ├── layout.rs               — terminal Rect splitting into 5 zones
│   └── widgets/
│       ├── tabs.rs             — tab bar built from Registry feature sets
│       ├── list.rs             — generic list pane widget
│       ├── detail.rs           — generic detail pane widget
│       └── status.rs           — footer/status line widget
├── app/
│   ├── mod.rs
│   ├── ports.rs                — VaultPort, ProviderPort, FeatureSetPort traits
│   ├── registry.rs             — Registry struct
│   ├── service.rs              — AppService: scan, future install/remove
│   └── bootstrap.rs            — wires adapters into Registry, runs initial scan
├── domain/
│   ├── mod.rs
│   ├── asset.rs                — ScannedPackage, AssetKind
│   ├── identity.rs             — AssetIdentity, [name:version:sha10] parsing
│   ├── scope.rs                — Scope enum
│   └── hashing.rs              — sha10 computation
├── infra/
│   ├── mod.rs
│   ├── vault/
│   │   ├── mod.rs
│   │   ├── local.rs            — LocalVaultAdapter: implements VaultPort
│   │   └── github.rs           — GithubVaultAdapter: stub
│   ├── provider/
│   │   ├── mod.rs
│   │   └── claude_code.rs      — ClaudeCodeProvider: stub
│   └── feature/
│       ├── mod.rs
│       ├── skill.rs            — SkillFeatureSet: implements FeatureSetPort
│       └── instruction.rs      — InstructionFeatureSet: stub
└── support/
    ├── error.rs                — AppError type
    └── types.rs                — shared primitives
```

---

## 6. Domain Types

### 6.1 AssetIdentity

```rust
struct AssetIdentity {
    name: String,
    version: Option<String>,   // None renders as "--"
    sha10: String,             // first 10 hex chars of sha256
}
```

### 6.2 ScannedPackage

```rust
struct ScannedPackage {
    identity: AssetIdentity,
    path: PathBuf,             // absolute path to package folder
    vault_id: String,          // which vault it came from
    kind: AssetKind,           // driven by FeatureSetPort
}
```

`name` is accessed via `identity.name` — no duplication.

### 6.3 AssetKind

```rust
enum AssetKind {
    Skill,
    Instruction,  // reserved
}
```

---

## 7. SkillFeatureSet (Phase 1 live adapter)

```rust
struct SkillFeatureSet;

impl FeatureSetPort for SkillFeatureSet {
    fn kind_name(&self) -> &str { "skill" }
    fn scan_root(&self) -> &str { "skills" }
    fn display_name(&self) -> &str { "Skills" }

    fn is_package(&self, path: &Path) -> bool {
        path.join("SKILL.md").exists()
    }

    fn hash_files(&self, path: &Path) -> Vec<PathBuf> {
        // all files under path, recursively, sorted by relative path
        walkdir::WalkDir::new(path)
            .sort_by_file_name()
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .collect()
    }
}
```

---

## 8. sha10 Computation

In `domain::hashing`:

1. Receive file list from `FeatureSetPort::hash_files()`
2. Sort by path (already sorted by walkdir, but enforce in domain)
3. For each file: read bytes, normalize `\r\n` → `\n`
4. Feed into a single `Sha256` hasher in order
5. Take first 10 hex characters of the digest

```rust
fn compute_sha10(files: &[PathBuf]) -> Result<String>
```

---

## 9. LocalVaultAdapter

```rust
struct LocalVaultAdapter {
    root: PathBuf,
    id: String,
}

impl VaultPort for LocalVaultAdapter {
    fn id(&self) -> &str { &self.id }
    fn kind_name(&self) -> &str { "local" }

    fn list_packages(&self, feature: &dyn FeatureSetPort) -> Result<Vec<ScannedPackage>> {
        let scan_root = self.root.join(feature.scan_root());
        // iterate direct subdirs of scan_root
        // for each: if feature.is_package() → compute sha10 → build ScannedPackage
    }
}
```

Phase 1: workspace root is detected from `std::env::current_dir()`.

---

## 10. AppState

```rust
struct AppState {
    active_tab: usize,                          // index into registry.feature_sets
    search_query: String,
    selected_index: usize,
    list_mode: ListMode,
    status_line: String,
    packages: HashMap<usize, Vec<ScannedPackage>>, // tab_index → packages
}
```

Tabs are not an enum — they are positions in the `Registry::feature_sets` vec. The TUI reads `registry.feature_sets[active_tab].display_name()` for the tab label.

---

## 11. TUI Layout

```
┌─ Header (1 line) ──────────────────────────────────────────┐
│ asky v0.1.0 ──────────────── [ Search: ___ ]               │
├─ Tab Bar (1 line) — generated from Registry ───────────────┤
│ [1] Skills  [2] Instructions  [3] Providers  [4] Vaults    │
├─ List Pane (60%) ──────────┬─ Detail Pane (40%) ───────────┤
│                            │                               │
│  (feature set list)        │  (package detail)             │
│                            │                               │
├────────────────────────────┴───────────────────────────────┘
│ Footer: keybindings + status line (2 lines)                │
└────────────────────────────────────────────────────────────┘
```

---

## 12. Keybindings

| Key | Action | Phase 1 behavior |
|-----|--------|-----------------|
| `1–9` | Switch to tab N | live (bound to registry index) |
| `↑` / `↓` | Move selection | live |
| `Space` | Toggle check | stub |
| `u` | Update selected | stub |
| `U` | Update all outdated | stub |
| `r` | Refresh scan | stub |
| `a` | Add vault | stub |
| `e` | Enable/disable vault | stub |
| `d` | Detach vault | stub |
| typing | Live search filter | live (active tab) |
| `Esc` | Clear search | live |
| `q` / `Ctrl+C` | Quit | live |

Stub actions display `[STUB] <action> not yet implemented` in `status_line`.

---

## 13. Error Handling

- `anyhow::Result` throughout
- Scan failure for a vault: logged to `status_line`, app continues with empty list
- Terminal init failure: stderr + exit code 1
- Terminal restore: `Drop` guard via `crossterm` ensures restore on panic

---

## 14. FEATURES.md

Stored at `docs/FEATURES.md`. Statuses: `[ ]` not started · `[~]` partial/stub · `[x]` complete

| # | Feature | Design Doc § | Status |
|---|---------|-------------|--------|
| 1 | TUI shell + event loop | §10 | `[ ]` |
| 2 | Data-driven tab switching | §10.2 | `[ ]` |
| 3 | List pane rendering | §11.3 | `[ ]` |
| 4 | Detail pane rendering | §11.3 | `[ ]` |
| 5 | Live search filter | §10.5 | `[ ]` |
| 6 | FeatureSetPort trait | — | `[ ]` |
| 7 | VaultPort trait | §9.1 | `[ ]` |
| 8 | ProviderPort trait | §8.1 | `[ ]` |
| 9 | Registry + bootstrap | — | `[ ]` |
| 10 | SkillFeatureSet adapter | §11.3 | `[ ]` |
| 11 | LocalVaultAdapter | §9 | `[ ]` |
| 12 | sha10 hashing | §6 | `[ ]` |
| 13 | Asset identity parsing | §4.1 | `[ ]` |
| 14 | Instructions tab | §11.3 | `[~]` stub |
| 15 | Providers tab | §11.4 | `[~]` stub |
| 16 | Vaults tab | §11.2 | `[~]` stub |
| 17 | config.toml read/write | §5 | `[ ]` |
| 18 | GithubVaultAdapter | §9 | `[ ]` |
| 19 | ClaudeCodeProvider adapter | §8 | `[ ]` |
| 20 | Install asset | §12.2 | `[ ]` |
| 21 | Update asset | §12.2 | `[ ]` |
| 22 | Remove asset | §3.1 | `[ ]` |
| 23 | Scope: global/workspace | §3.5 | `[ ]` |
| 24 | Vault attach/detach | §12.3 | `[ ]` |
| 25 | Space: toggle item check | §10.5 | `[ ]` |
| 26 | Version extraction from package | §4.2 | `[ ]` |
| 27 | InstructionFeatureSet adapter | §11.3 | `[ ]` |
