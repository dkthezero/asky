# asky TUI Phase 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a runnable `asky` binary with a full ratatui TUI that scans the workspace `skills/` folder and displays skill packages using hexagonal architecture with data-driven, pluggable tabs.

**Architecture:** Hexagonal — `domain/` has zero external deps (pure types/logic); `app/` defines port traits and orchestrates use-cases; `infra/` implements port traits; `tui/` renders AppState driven by a Registry of adapters. Adding a new feature set, vault, or provider = implement one trait in `infra/`, register in bootstrap — no core changes.

**Tech Stack:** Rust 2021, ratatui 0.29, crossterm 0.28, clap 4 (derive), sha2 0.10, hex 0.4, walkdir 2, anyhow 1, tempfile 3 (dev)

---

## File Map

| File | Responsibility |
|------|---------------|
| `Cargo.toml` | Dependencies and binary config |
| `src/main.rs` | Entry: bootstrap → terminal init → event loop |
| `src/cli/mod.rs` | CLI module declaration |
| `src/cli/entry.rs` | clap `Cli` struct, `parse()` |
| `src/domain/mod.rs` | Domain module declarations |
| `src/domain/asset.rs` | `AssetKind`, `ScannedPackage` |
| `src/domain/identity.rs` | `AssetIdentity`, Display impl |
| `src/domain/scope.rs` | `Scope` enum |
| `src/domain/hashing.rs` | `compute_sha10()` |
| `src/app/mod.rs` | App module declarations |
| `src/app/ports.rs` | `FeatureSetPort`, `VaultPort`, `ProviderPort` trait definitions |
| `src/app/registry.rs` | `Registry` struct — holds all adapters |
| `src/app/bootstrap.rs` | Wires adapters, runs initial scan, produces `AppState` |
| `src/infra/mod.rs` | Infra module declarations |
| `src/infra/feature/mod.rs` | Feature adapters module |
| `src/infra/feature/skill.rs` | `SkillFeatureSet` — live FeatureSetPort impl |
| `src/infra/feature/stub.rs` | `StubFeatureSet` — placeholder tabs |
| `src/infra/vault/mod.rs` | Vault adapters module |
| `src/infra/vault/local.rs` | `LocalVaultAdapter` — scans local filesystem |
| `src/infra/vault/github.rs` | `GithubVaultAdapter` — stub |
| `src/infra/provider/mod.rs` | Provider adapters module |
| `src/infra/provider/claude_code.rs` | `ClaudeCodeProvider` — stub |
| `src/tui/mod.rs` | TUI module declarations |
| `src/tui/app.rs` | `AppState`, `ListMode` |
| `src/tui/event.rs` | `handle()` — crossterm key dispatch, returns `ControlFlow` |
| `src/tui/render.rs` | `draw()` — top-level frame painter |
| `src/tui/layout.rs` | `AppLayout`, `compute()` — splits terminal rect into 5 zones |
| `src/tui/widgets/mod.rs` | Widgets module declarations |
| `src/tui/widgets/tabs.rs` | Tab bar widget |
| `src/tui/widgets/list.rs` | List pane widget |
| `src/tui/widgets/detail.rs` | Detail pane widget |
| `src/tui/widgets/status.rs` | Footer/status widget |
| `src/support/mod.rs` | Support module declarations |
| `src/support/error.rs` | `AppError` type |
| `docs/FEATURES.md` | Feature tracker — all design doc features with status |

---

### Task 1: Project Scaffold

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/cli/mod.rs`, `src/cli/entry.rs`
- Create: `src/domain/mod.rs`, `src/app/mod.rs`, `src/infra/mod.rs`
- Create: `src/tui/mod.rs`, `src/tui/widgets/mod.rs`
- Create: `src/support/mod.rs`

- [ ] **Step 1: Create `Cargo.toml`**

```toml
[package]
name = "asky"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "asky"
path = "src/main.rs"

[dependencies]
ratatui = "0.29"
crossterm = "0.28"
clap = { version = "4", features = ["derive"] }
sha2 = "0.10"
hex = "0.4"
walkdir = "2"
anyhow = "1"

[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 2: Create all module stub files**

`src/main.rs`:
```rust
mod cli;
mod domain;
mod app;
mod infra;
mod tui;
mod support;

fn main() {
    println!("asky");
}
```

`src/cli/mod.rs`:
```rust
pub mod entry;
```

`src/cli/entry.rs`:
```rust
pub fn parse() {}
```

`src/domain/mod.rs`:
```rust
pub mod asset;
pub mod identity;
pub mod scope;
pub mod hashing;
```

`src/domain/asset.rs`, `src/domain/identity.rs`, `src/domain/scope.rs`, `src/domain/hashing.rs` — each contains just:
```rust
// placeholder
```

`src/app/mod.rs`:
```rust
pub mod ports;
pub mod registry;
pub mod bootstrap;
```

`src/app/ports.rs`, `src/app/registry.rs`, `src/app/bootstrap.rs` — placeholder.

`src/infra/mod.rs`:
```rust
pub mod feature;
pub mod vault;
pub mod provider;
```

`src/infra/feature/mod.rs`:
```rust
pub mod skill;
pub mod stub;
```

`src/infra/feature/skill.rs`, `src/infra/feature/stub.rs` — placeholder.

`src/infra/vault/mod.rs`:
```rust
pub mod local;
pub mod github;
```

`src/infra/vault/local.rs`, `src/infra/vault/github.rs` — placeholder.

`src/infra/provider/mod.rs`:
```rust
pub mod claude_code;
```

`src/infra/provider/claude_code.rs` — placeholder.

`src/tui/mod.rs`:
```rust
pub mod app;
pub mod event;
pub mod render;
pub mod layout;
pub mod widgets;
```

`src/tui/app.rs`, `src/tui/event.rs`, `src/tui/render.rs`, `src/tui/layout.rs` — placeholder.

`src/tui/widgets/mod.rs`:
```rust
pub mod tabs;
pub mod list;
pub mod detail;
pub mod status;
```

`src/tui/widgets/tabs.rs`, `src/tui/widgets/list.rs`, `src/tui/widgets/detail.rs`, `src/tui/widgets/status.rs` — placeholder.

`src/support/mod.rs`:
```rust
pub mod error;
```

`src/support/error.rs` — placeholder.

- [ ] **Step 3: Verify it compiles**

```bash
cargo build 2>&1 | head -20
```

Expected: compiles with warnings, binary produced.

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml src/
git commit -m "chore: scaffold asky project structure"
```

---

### Task 2: Domain — `AssetKind` and `Scope`

**Files:**
- Modify: `src/domain/asset.rs`
- Modify: `src/domain/scope.rs`

- [ ] **Step 1: Write failing tests in `src/domain/asset.rs`**

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum AssetKind {
    Skill,
    Instruction,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_kind_clone() {
        let k = AssetKind::Skill;
        assert_eq!(k.clone(), AssetKind::Skill);
    }

    #[test]
    fn asset_kind_eq() {
        assert_ne!(AssetKind::Skill, AssetKind::Instruction);
    }
}
```

- [ ] **Step 2: Write `src/domain/scope.rs`**

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Scope {
    Global,
    Workspace,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_clone() {
        assert_eq!(Scope::Global.clone(), Scope::Global);
    }
}
```

- [ ] **Step 3: Run tests**

```bash
cargo test domain::asset::tests cargo test domain::scope::tests
```

Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src/domain/asset.rs src/domain/scope.rs
git commit -m "feat(domain): add AssetKind and Scope types"
```

---

### Task 3: Domain — `AssetIdentity`

**Files:**
- Modify: `src/domain/identity.rs`

- [ ] **Step 1: Write failing tests**

```rust
// src/domain/identity.rs — tests only first, impl in step 2

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_with_version() {
        let id = AssetIdentity::new("web-tool", Some("1.2.0".to_string()), "a13c9ef042");
        assert_eq!(id.to_string(), "[web-tool:1.2.0:a13c9ef042]");
    }

    #[test]
    fn display_without_version() {
        let id = AssetIdentity::new("local-script", None, "9ac00ff113");
        assert_eq!(id.to_string(), "[local-script:--:9ac00ff113]");
    }

    #[test]
    fn name_accessor() {
        let id = AssetIdentity::new("my-skill", None, "0000000000");
        assert_eq!(id.name, "my-skill");
    }
}
```

- [ ] **Step 2: Run to verify it fails**

```bash
cargo test domain::identity::tests 2>&1 | head -10
```

Expected: compile error — `AssetIdentity` not defined.

- [ ] **Step 3: Implement `AssetIdentity`**

```rust
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct AssetIdentity {
    pub name: String,
    pub version: Option<String>,
    pub sha10: String,
}

impl AssetIdentity {
    pub fn new(
        name: impl Into<String>,
        version: Option<String>,
        sha10: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            version,
            sha10: sha10.into(),
        }
    }
}

impl fmt::Display for AssetIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let version = self.version.as_deref().unwrap_or("--");
        write!(f, "[{}:{}:{}]", self.name, version, self.sha10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_with_version() {
        let id = AssetIdentity::new("web-tool", Some("1.2.0".to_string()), "a13c9ef042");
        assert_eq!(id.to_string(), "[web-tool:1.2.0:a13c9ef042]");
    }

    #[test]
    fn display_without_version() {
        let id = AssetIdentity::new("local-script", None, "9ac00ff113");
        assert_eq!(id.to_string(), "[local-script:--:9ac00ff113]");
    }

    #[test]
    fn name_accessor() {
        let id = AssetIdentity::new("my-skill", None, "0000000000");
        assert_eq!(id.name, "my-skill");
    }
}
```

- [ ] **Step 4: Run tests**

```bash
cargo test domain::identity::tests
```

Expected: 3 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/domain/identity.rs
git commit -m "feat(domain): add AssetIdentity with Display impl"
```

---

### Task 4: Domain — `ScannedPackage`

**Files:**
- Modify: `src/domain/asset.rs`

- [ ] **Step 1: Write failing test**

Add to `src/domain/asset.rs`:

```rust
// Add to existing tests module
#[test]
fn scanned_package_name_via_identity() {
    use std::path::PathBuf;
    use crate::domain::identity::AssetIdentity;
    let pkg = ScannedPackage {
        identity: AssetIdentity::new("my-skill", None, "abc1234567"),
        path: PathBuf::from("/skills/my-skill"),
        vault_id: "workspace".to_string(),
        kind: AssetKind::Skill,
    };
    assert_eq!(pkg.identity.name, "my-skill");
    assert_eq!(pkg.vault_id, "workspace");
}
```

- [ ] **Step 2: Run to verify it fails**

```bash
cargo test domain::asset::tests::scanned_package 2>&1 | head -10
```

Expected: compile error — `ScannedPackage` not defined.

- [ ] **Step 3: Add `ScannedPackage` to `src/domain/asset.rs`**

```rust
use std::path::PathBuf;
use crate::domain::identity::AssetIdentity;

#[derive(Debug, Clone, PartialEq)]
pub enum AssetKind {
    Skill,
    Instruction,
}

#[derive(Debug, Clone)]
pub struct ScannedPackage {
    pub identity: AssetIdentity,
    pub path: PathBuf,
    pub vault_id: String,
    pub kind: AssetKind,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_kind_clone() {
        let k = AssetKind::Skill;
        assert_eq!(k.clone(), AssetKind::Skill);
    }

    #[test]
    fn asset_kind_eq() {
        assert_ne!(AssetKind::Skill, AssetKind::Instruction);
    }

    #[test]
    fn scanned_package_name_via_identity() {
        let pkg = ScannedPackage {
            identity: AssetIdentity::new("my-skill", None, "abc1234567"),
            path: PathBuf::from("/skills/my-skill"),
            vault_id: "workspace".to_string(),
            kind: AssetKind::Skill,
        };
        assert_eq!(pkg.identity.name, "my-skill");
        assert_eq!(pkg.vault_id, "workspace");
    }
}
```

- [ ] **Step 4: Run tests**

```bash
cargo test domain::asset::tests
```

Expected: 3 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/domain/asset.rs
git commit -m "feat(domain): add ScannedPackage"
```

---

### Task 5: Domain — `sha10` Hashing

**Files:**
- Modify: `src/domain/hashing.rs`

- [ ] **Step 1: Write failing tests**

```rust
// src/domain/hashing.rs — tests first

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn write_temp_file(dir: &std::path::Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn sha10_is_ten_chars() {
        let dir = tempfile::tempdir().unwrap();
        let f = write_temp_file(dir.path(), "SKILL.md", "hello world");
        let result = compute_sha10(&[f]).unwrap();
        assert_eq!(result.len(), 10);
    }

    #[test]
    fn sha10_normalizes_crlf() {
        let dir = tempfile::tempdir().unwrap();
        let unix = write_temp_file(dir.path(), "unix.md", "hello\nworld");
        let windows = write_temp_file(dir.path(), "windows.md", "hello\r\nworld");
        let sha_unix = compute_sha10(&[unix]).unwrap();
        let sha_windows = compute_sha10(&[windows]).unwrap();
        assert_eq!(sha_unix, sha_windows);
    }

    #[test]
    fn sha10_empty_files_returns_fixed_value() {
        let result = compute_sha10(&[]).unwrap();
        assert_eq!(result, "0000000000");
    }

    #[test]
    fn sha10_is_deterministic() {
        let dir = tempfile::tempdir().unwrap();
        let f = write_temp_file(dir.path(), "test.md", "deterministic content");
        let a = compute_sha10(&[f.clone()]).unwrap();
        let b = compute_sha10(&[f]).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn sha10_different_content_differs() {
        let dir = tempfile::tempdir().unwrap();
        let f1 = write_temp_file(dir.path(), "a.md", "content alpha");
        let f2 = write_temp_file(dir.path(), "b.md", "content beta");
        let sha_a = compute_sha10(&[f1]).unwrap();
        let sha_b = compute_sha10(&[f2]).unwrap();
        assert_ne!(sha_a, sha_b);
    }
}
```

- [ ] **Step 2: Run to verify it fails**

```bash
cargo test domain::hashing::tests 2>&1 | head -10
```

Expected: compile error — `compute_sha10` not defined.

- [ ] **Step 3: Implement `compute_sha10`**

```rust
use std::path::PathBuf;
use anyhow::Result;
use sha2::{Digest, Sha256};

pub fn compute_sha10(files: &[PathBuf]) -> Result<String> {
    if files.is_empty() {
        return Ok("0000000000".to_string());
    }

    let mut sorted = files.to_vec();
    sorted.sort();

    let mut hasher = Sha256::new();
    for path in &sorted {
        let bytes = std::fs::read(path)?;
        let content = String::from_utf8_lossy(&bytes);
        let normalized = content.replace("\r\n", "\n");
        hasher.update(normalized.as_bytes());
    }

    let digest = hasher.finalize();
    let hex_str = hex::encode(digest);
    Ok(hex_str[..10].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_temp_file(dir: &std::path::Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn sha10_is_ten_chars() {
        let dir = tempfile::tempdir().unwrap();
        let f = write_temp_file(dir.path(), "SKILL.md", "hello world");
        let result = compute_sha10(&[f]).unwrap();
        assert_eq!(result.len(), 10);
    }

    #[test]
    fn sha10_normalizes_crlf() {
        let dir = tempfile::tempdir().unwrap();
        let unix = write_temp_file(dir.path(), "unix.md", "hello\nworld");
        let windows = write_temp_file(dir.path(), "windows.md", "hello\r\nworld");
        let sha_unix = compute_sha10(&[unix]).unwrap();
        let sha_windows = compute_sha10(&[windows]).unwrap();
        assert_eq!(sha_unix, sha_windows);
    }

    #[test]
    fn sha10_empty_files_returns_fixed_value() {
        let result = compute_sha10(&[]).unwrap();
        assert_eq!(result, "0000000000");
    }

    #[test]
    fn sha10_is_deterministic() {
        let dir = tempfile::tempdir().unwrap();
        let f = write_temp_file(dir.path(), "test.md", "deterministic content");
        let a = compute_sha10(&[f.clone()]).unwrap();
        let b = compute_sha10(&[f]).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn sha10_different_content_differs() {
        let dir = tempfile::tempdir().unwrap();
        let f1 = write_temp_file(dir.path(), "a.md", "content alpha");
        let f2 = write_temp_file(dir.path(), "b.md", "content beta");
        let sha_a = compute_sha10(&[f1]).unwrap();
        let sha_b = compute_sha10(&[f2]).unwrap();
        assert_ne!(sha_a, sha_b);
    }
}
```

- [ ] **Step 4: Run tests**

```bash
cargo test domain::hashing::tests
```

Expected: 5 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/domain/hashing.rs
git commit -m "feat(domain): implement sha10 hashing with CRLF normalization"
```

---

### Task 6: App — Port Traits

**Files:**
- Modify: `src/app/ports.rs`

- [ ] **Step 1: Write failing test**

```rust
// src/app/ports.rs — test that a concrete impl satisfies the trait bounds

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use crate::domain::asset::{AssetKind, ScannedPackage};

    struct TestFeatureSet;
    impl FeatureSetPort for TestFeatureSet {
        fn kind_name(&self) -> &str { "test" }
        fn display_name(&self) -> &str { "Test" }
        fn scan_root(&self) -> &str { "test_root" }
        fn asset_kind(&self) -> AssetKind { AssetKind::Skill }
        fn is_package(&self, _: &Path) -> bool { false }
        fn hash_files(&self, _: &Path) -> Vec<PathBuf> { vec![] }
    }

    #[test]
    fn feature_set_port_default_not_stub() {
        let f = TestFeatureSet;
        assert!(!f.is_stub());
    }

    #[test]
    fn feature_set_port_kind_name() {
        let f = TestFeatureSet;
        assert_eq!(f.kind_name(), "test");
    }
}
```

- [ ] **Step 2: Run to verify it fails**

```bash
cargo test app::ports::tests 2>&1 | head -10
```

Expected: compile error.

- [ ] **Step 3: Implement port traits**

```rust
use std::path::{Path, PathBuf};
use anyhow::Result;
use crate::domain::asset::{AssetKind, ScannedPackage};
use crate::domain::identity::AssetIdentity;
use crate::domain::scope::Scope;

pub trait FeatureSetPort: Send + Sync {
    fn kind_name(&self) -> &str;
    fn display_name(&self) -> &str;
    fn scan_root(&self) -> &str;
    fn asset_kind(&self) -> AssetKind;
    fn is_package(&self, path: &Path) -> bool;
    fn hash_files(&self, path: &Path) -> Vec<PathBuf>;

    /// Override to return `true` for placeholder tabs not yet implemented.
    fn is_stub(&self) -> bool {
        false
    }
}

pub trait VaultPort: Send + Sync {
    fn id(&self) -> &str;
    fn kind_name(&self) -> &str;
    fn list_packages(&self, feature: &dyn FeatureSetPort) -> Result<Vec<ScannedPackage>>;
}

pub trait ProviderPort: Send + Sync {
    fn id(&self) -> &str;
    fn install(&self, pkg: &ScannedPackage, scope: Scope) -> Result<()>;
    fn remove(&self, identity: &AssetIdentity, scope: Scope) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestFeatureSet;
    impl FeatureSetPort for TestFeatureSet {
        fn kind_name(&self) -> &str { "test" }
        fn display_name(&self) -> &str { "Test" }
        fn scan_root(&self) -> &str { "test_root" }
        fn asset_kind(&self) -> AssetKind { AssetKind::Skill }
        fn is_package(&self, _: &Path) -> bool { false }
        fn hash_files(&self, _: &Path) -> Vec<PathBuf> { vec![] }
    }

    #[test]
    fn feature_set_port_default_not_stub() {
        let f = TestFeatureSet;
        assert!(!f.is_stub());
    }

    #[test]
    fn feature_set_port_kind_name() {
        let f = TestFeatureSet;
        assert_eq!(f.kind_name(), "test");
    }
}
```

- [ ] **Step 4: Run tests**

```bash
cargo test app::ports::tests
```

Expected: 2 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/app/ports.rs
git commit -m "feat(app): define FeatureSetPort, VaultPort, ProviderPort traits"
```

---

### Task 7: App — Registry

**Files:**
- Modify: `src/app/registry.rs`

- [ ] **Step 1: Write failing test**

```rust
// Tests go at the bottom of registry.rs after impl

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use crate::app::ports::FeatureSetPort;
    use crate::domain::asset::AssetKind;

    struct FakeFeature(&'static str);
    impl FeatureSetPort for FakeFeature {
        fn kind_name(&self) -> &str { self.0 }
        fn display_name(&self) -> &str { self.0 }
        fn scan_root(&self) -> &str { "" }
        fn asset_kind(&self) -> AssetKind { AssetKind::Skill }
        fn is_package(&self, _: &Path) -> bool { false }
        fn hash_files(&self, _: &Path) -> Vec<PathBuf> { vec![] }
    }

    #[test]
    fn registry_starts_empty() {
        let r = Registry::new();
        assert!(r.feature_sets.is_empty());
        assert!(r.vaults.is_empty());
        assert!(r.providers.is_empty());
    }

    #[test]
    fn registry_register_feature_set() {
        let mut r = Registry::new();
        r.register_feature_set(Box::new(FakeFeature("skill")));
        r.register_feature_set(Box::new(FakeFeature("instruction")));
        assert_eq!(r.feature_sets.len(), 2);
        assert_eq!(r.feature_sets[0].kind_name(), "skill");
    }
}
```

- [ ] **Step 2: Run to verify it fails**

```bash
cargo test app::registry::tests 2>&1 | head -10
```

Expected: compile error.

- [ ] **Step 3: Implement `Registry`**

```rust
use crate::app::ports::{FeatureSetPort, ProviderPort, VaultPort};

pub struct Registry {
    pub feature_sets: Vec<Box<dyn FeatureSetPort>>,
    pub vaults: Vec<Box<dyn VaultPort>>,
    pub providers: Vec<Box<dyn ProviderPort>>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            feature_sets: Vec::new(),
            vaults: Vec::new(),
            providers: Vec::new(),
        }
    }

    pub fn register_feature_set(&mut self, fs: Box<dyn FeatureSetPort>) {
        self.feature_sets.push(fs);
    }

    pub fn register_vault(&mut self, vault: Box<dyn VaultPort>) {
        self.vaults.push(vault);
    }

    pub fn register_provider(&mut self, provider: Box<dyn ProviderPort>) {
        self.providers.push(provider);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use crate::app::ports::FeatureSetPort;
    use crate::domain::asset::AssetKind;

    struct FakeFeature(&'static str);
    impl FeatureSetPort for FakeFeature {
        fn kind_name(&self) -> &str { self.0 }
        fn display_name(&self) -> &str { self.0 }
        fn scan_root(&self) -> &str { "" }
        fn asset_kind(&self) -> AssetKind { AssetKind::Skill }
        fn is_package(&self, _: &Path) -> bool { false }
        fn hash_files(&self, _: &Path) -> Vec<PathBuf> { vec![] }
    }

    #[test]
    fn registry_starts_empty() {
        let r = Registry::new();
        assert!(r.feature_sets.is_empty());
        assert!(r.vaults.is_empty());
        assert!(r.providers.is_empty());
    }

    #[test]
    fn registry_register_feature_set() {
        let mut r = Registry::new();
        r.register_feature_set(Box::new(FakeFeature("skill")));
        r.register_feature_set(Box::new(FakeFeature("instruction")));
        assert_eq!(r.feature_sets.len(), 2);
        assert_eq!(r.feature_sets[0].kind_name(), "skill");
    }
}
```

- [ ] **Step 4: Run tests**

```bash
cargo test app::registry::tests
```

Expected: 2 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/app/registry.rs
git commit -m "feat(app): add Registry for pluggable adapters"
```

---

### Task 8: Infra — `SkillFeatureSet`

**Files:**
- Modify: `src/infra/feature/skill.rs`

- [ ] **Step 1: Write failing test**

```rust
// src/infra/feature/skill.rs — tests only first

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_feature_set_kind_name() {
        assert_eq!(SkillFeatureSet.kind_name(), "skill");
    }

    #[test]
    fn skill_feature_set_detects_package() {
        let dir = tempfile::tempdir().unwrap();
        let skill_dir = dir.path().join("my-skill");
        std::fs::create_dir(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), "# My Skill").unwrap();
        assert!(SkillFeatureSet.is_package(&skill_dir));
    }

    #[test]
    fn skill_feature_set_rejects_non_package() {
        let dir = tempfile::tempdir().unwrap();
        let other_dir = dir.path().join("not-a-skill");
        std::fs::create_dir(&other_dir).unwrap();
        std::fs::write(other_dir.join("README.md"), "nothing").unwrap();
        assert!(!SkillFeatureSet.is_package(&other_dir));
    }

    #[test]
    fn skill_feature_set_hash_files_includes_all_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("SKILL.md"), "skill").unwrap();
        std::fs::write(dir.path().join("notes.md"), "notes").unwrap();
        let files = SkillFeatureSet.hash_files(dir.path());
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn skill_feature_set_is_not_stub() {
        use crate::app::ports::FeatureSetPort;
        assert!(!SkillFeatureSet.is_stub());
    }
}
```

- [ ] **Step 2: Run to verify it fails**

```bash
cargo test infra::feature::skill::tests 2>&1 | head -10
```

Expected: compile error.

- [ ] **Step 3: Implement `SkillFeatureSet`**

```rust
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crate::app::ports::FeatureSetPort;
use crate::domain::asset::AssetKind;

pub struct SkillFeatureSet;

impl FeatureSetPort for SkillFeatureSet {
    fn kind_name(&self) -> &str { "skill" }
    fn display_name(&self) -> &str { "Skills" }
    fn scan_root(&self) -> &str { "skills" }
    fn asset_kind(&self) -> AssetKind { AssetKind::Skill }

    fn is_package(&self, path: &Path) -> bool {
        path.join("SKILL.md").exists()
    }

    fn hash_files(&self, path: &Path) -> Vec<PathBuf> {
        WalkDir::new(path)
            .sort_by_file_name()
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::ports::FeatureSetPort;

    #[test]
    fn skill_feature_set_kind_name() {
        assert_eq!(SkillFeatureSet.kind_name(), "skill");
    }

    #[test]
    fn skill_feature_set_detects_package() {
        let dir = tempfile::tempdir().unwrap();
        let skill_dir = dir.path().join("my-skill");
        std::fs::create_dir(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), "# My Skill").unwrap();
        assert!(SkillFeatureSet.is_package(&skill_dir));
    }

    #[test]
    fn skill_feature_set_rejects_non_package() {
        let dir = tempfile::tempdir().unwrap();
        let other_dir = dir.path().join("not-a-skill");
        std::fs::create_dir(&other_dir).unwrap();
        std::fs::write(other_dir.join("README.md"), "nothing").unwrap();
        assert!(!SkillFeatureSet.is_package(&other_dir));
    }

    #[test]
    fn skill_feature_set_hash_files_includes_all_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("SKILL.md"), "skill").unwrap();
        std::fs::write(dir.path().join("notes.md"), "notes").unwrap();
        let files = SkillFeatureSet.hash_files(dir.path());
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn skill_feature_set_is_not_stub() {
        assert!(!SkillFeatureSet.is_stub());
    }
}
```

- [ ] **Step 4: Run tests**

```bash
cargo test infra::feature::skill::tests
```

Expected: 5 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/infra/feature/skill.rs
git commit -m "feat(infra): implement SkillFeatureSet adapter"
```

---

### Task 9: Infra — `StubFeatureSet`

**Files:**
- Modify: `src/infra/feature/stub.rs`

- [ ] **Step 1: Write failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::ports::FeatureSetPort;

    #[test]
    fn stub_is_stub() {
        let s = StubFeatureSet::new("instruction", "Instructions", "instructions");
        assert!(s.is_stub());
    }

    #[test]
    fn stub_display_name() {
        let s = StubFeatureSet::new("provider", "Providers", "");
        assert_eq!(s.display_name(), "Providers");
    }

    #[test]
    fn stub_is_package_always_false() {
        let s = StubFeatureSet::new("vault", "Vaults", "");
        assert!(!s.is_package(std::path::Path::new("/any/path")));
    }
}
```

- [ ] **Step 2: Run to verify it fails**

```bash
cargo test infra::feature::stub::tests 2>&1 | head -10
```

Expected: compile error.

- [ ] **Step 3: Implement `StubFeatureSet`**

```rust
use std::path::{Path, PathBuf};
use crate::app::ports::FeatureSetPort;
use crate::domain::asset::AssetKind;

pub struct StubFeatureSet {
    kind: &'static str,
    display: &'static str,
    root: &'static str,
}

impl StubFeatureSet {
    pub fn new(kind: &'static str, display: &'static str, root: &'static str) -> Self {
        Self { kind, display, root }
    }
}

impl FeatureSetPort for StubFeatureSet {
    fn kind_name(&self) -> &str { self.kind }
    fn display_name(&self) -> &str { self.display }
    fn scan_root(&self) -> &str { self.root }
    fn asset_kind(&self) -> AssetKind { AssetKind::Instruction }
    fn is_package(&self, _: &Path) -> bool { false }
    fn hash_files(&self, _: &Path) -> Vec<PathBuf> { vec![] }
    fn is_stub(&self) -> bool { true }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::ports::FeatureSetPort;

    #[test]
    fn stub_is_stub() {
        let s = StubFeatureSet::new("instruction", "Instructions", "instructions");
        assert!(s.is_stub());
    }

    #[test]
    fn stub_display_name() {
        let s = StubFeatureSet::new("provider", "Providers", "");
        assert_eq!(s.display_name(), "Providers");
    }

    #[test]
    fn stub_is_package_always_false() {
        let s = StubFeatureSet::new("vault", "Vaults", "");
        assert!(!s.is_package(std::path::Path::new("/any/path")));
    }
}
```

- [ ] **Step 4: Run tests**

```bash
cargo test infra::feature::stub::tests
```

Expected: 3 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/infra/feature/stub.rs
git commit -m "feat(infra): add StubFeatureSet for placeholder tabs"
```

---

### Task 10: Infra — `LocalVaultAdapter`

**Files:**
- Modify: `src/infra/vault/local.rs`

- [ ] **Step 1: Write failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::feature::skill::SkillFeatureSet;

    fn make_skill(root: &std::path::Path, name: &str) {
        let skill_dir = root.join("skills").join(name);
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), format!("# {}", name)).unwrap();
    }

    #[test]
    fn list_packages_finds_skills() {
        let dir = tempfile::tempdir().unwrap();
        make_skill(dir.path(), "alpha-skill");
        make_skill(dir.path(), "beta-skill");
        let vault = LocalVaultAdapter::new("workspace", dir.path().to_path_buf());
        let pkgs = vault.list_packages(&SkillFeatureSet).unwrap();
        assert_eq!(pkgs.len(), 2);
        // sorted alphabetically
        assert_eq!(pkgs[0].identity.name, "alpha-skill");
        assert_eq!(pkgs[1].identity.name, "beta-skill");
    }

    #[test]
    fn list_packages_empty_when_no_skills_dir() {
        let dir = tempfile::tempdir().unwrap();
        let vault = LocalVaultAdapter::new("workspace", dir.path().to_path_buf());
        let pkgs = vault.list_packages(&SkillFeatureSet).unwrap();
        assert!(pkgs.is_empty());
    }

    #[test]
    fn list_packages_skips_dirs_without_skill_md() {
        let dir = tempfile::tempdir().unwrap();
        let not_a_skill = dir.path().join("skills").join("not-a-skill");
        std::fs::create_dir_all(&not_a_skill).unwrap();
        std::fs::write(not_a_skill.join("README.md"), "nope").unwrap();
        let vault = LocalVaultAdapter::new("workspace", dir.path().to_path_buf());
        let pkgs = vault.list_packages(&SkillFeatureSet).unwrap();
        assert!(pkgs.is_empty());
    }

    #[test]
    fn list_packages_sets_vault_id() {
        let dir = tempfile::tempdir().unwrap();
        make_skill(dir.path(), "my-skill");
        let vault = LocalVaultAdapter::new("my-vault", dir.path().to_path_buf());
        let pkgs = vault.list_packages(&SkillFeatureSet).unwrap();
        assert_eq!(pkgs[0].vault_id, "my-vault");
    }

    #[test]
    fn list_packages_sha10_is_ten_chars() {
        let dir = tempfile::tempdir().unwrap();
        make_skill(dir.path(), "some-skill");
        let vault = LocalVaultAdapter::new("workspace", dir.path().to_path_buf());
        let pkgs = vault.list_packages(&SkillFeatureSet).unwrap();
        assert_eq!(pkgs[0].identity.sha10.len(), 10);
    }
}
```

- [ ] **Step 2: Run to verify it fails**

```bash
cargo test infra::vault::local::tests 2>&1 | head -10
```

Expected: compile error.

- [ ] **Step 3: Implement `LocalVaultAdapter`**

```rust
use std::path::PathBuf;
use anyhow::Result;
use crate::app::ports::{FeatureSetPort, VaultPort};
use crate::domain::asset::ScannedPackage;
use crate::domain::hashing::compute_sha10;
use crate::domain::identity::AssetIdentity;

pub struct LocalVaultAdapter {
    id: String,
    root: PathBuf,
}

impl LocalVaultAdapter {
    pub fn new(id: impl Into<String>, root: PathBuf) -> Self {
        Self { id: id.into(), root }
    }
}

impl VaultPort for LocalVaultAdapter {
    fn id(&self) -> &str { &self.id }
    fn kind_name(&self) -> &str { "local" }

    fn list_packages(&self, feature: &dyn FeatureSetPort) -> Result<Vec<ScannedPackage>> {
        let scan_root = self.root.join(feature.scan_root());
        if !scan_root.exists() {
            return Ok(Vec::new());
        }

        let mut packages = Vec::new();
        for entry in std::fs::read_dir(&scan_root)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            if !feature.is_package(&path) {
                continue;
            }
            let files = feature.hash_files(&path);
            let sha10 = compute_sha10(&files)?;
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            let identity = AssetIdentity::new(name, None, sha10);
            packages.push(ScannedPackage {
                identity,
                path,
                vault_id: self.id.clone(),
                kind: feature.asset_kind(),
            });
        }

        packages.sort_by(|a, b| a.identity.name.cmp(&b.identity.name));
        Ok(packages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::feature::skill::SkillFeatureSet;

    fn make_skill(root: &std::path::Path, name: &str) {
        let skill_dir = root.join("skills").join(name);
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), format!("# {}", name)).unwrap();
    }

    #[test]
    fn list_packages_finds_skills() {
        let dir = tempfile::tempdir().unwrap();
        make_skill(dir.path(), "alpha-skill");
        make_skill(dir.path(), "beta-skill");
        let vault = LocalVaultAdapter::new("workspace", dir.path().to_path_buf());
        let pkgs = vault.list_packages(&SkillFeatureSet).unwrap();
        assert_eq!(pkgs.len(), 2);
        assert_eq!(pkgs[0].identity.name, "alpha-skill");
        assert_eq!(pkgs[1].identity.name, "beta-skill");
    }

    #[test]
    fn list_packages_empty_when_no_skills_dir() {
        let dir = tempfile::tempdir().unwrap();
        let vault = LocalVaultAdapter::new("workspace", dir.path().to_path_buf());
        let pkgs = vault.list_packages(&SkillFeatureSet).unwrap();
        assert!(pkgs.is_empty());
    }

    #[test]
    fn list_packages_skips_dirs_without_skill_md() {
        let dir = tempfile::tempdir().unwrap();
        let not_a_skill = dir.path().join("skills").join("not-a-skill");
        std::fs::create_dir_all(&not_a_skill).unwrap();
        std::fs::write(not_a_skill.join("README.md"), "nope").unwrap();
        let vault = LocalVaultAdapter::new("workspace", dir.path().to_path_buf());
        let pkgs = vault.list_packages(&SkillFeatureSet).unwrap();
        assert!(pkgs.is_empty());
    }

    #[test]
    fn list_packages_sets_vault_id() {
        let dir = tempfile::tempdir().unwrap();
        make_skill(dir.path(), "my-skill");
        let vault = LocalVaultAdapter::new("my-vault", dir.path().to_path_buf());
        let pkgs = vault.list_packages(&SkillFeatureSet).unwrap();
        assert_eq!(pkgs[0].vault_id, "my-vault");
    }

    #[test]
    fn list_packages_sha10_is_ten_chars() {
        let dir = tempfile::tempdir().unwrap();
        make_skill(dir.path(), "some-skill");
        let vault = LocalVaultAdapter::new("workspace", dir.path().to_path_buf());
        let pkgs = vault.list_packages(&SkillFeatureSet).unwrap();
        assert_eq!(pkgs[0].identity.sha10.len(), 10);
    }
}
```

- [ ] **Step 4: Run tests**

```bash
cargo test infra::vault::local::tests
```

Expected: 5 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/infra/vault/local.rs
git commit -m "feat(infra): implement LocalVaultAdapter"
```

---

### Task 11: Infra — Stub Vault and Provider Adapters

**Files:**
- Modify: `src/infra/vault/github.rs`
- Modify: `src/infra/provider/claude_code.rs`

- [ ] **Step 1: Implement `GithubVaultAdapter` stub**

```rust
// src/infra/vault/github.rs
use anyhow::{bail, Result};
use crate::app::ports::{FeatureSetPort, VaultPort};
use crate::domain::asset::ScannedPackage;

pub struct GithubVaultAdapter {
    pub id: String,
}

impl GithubVaultAdapter {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

impl VaultPort for GithubVaultAdapter {
    fn id(&self) -> &str { &self.id }
    fn kind_name(&self) -> &str { "github" }
    fn list_packages(&self, _feature: &dyn FeatureSetPort) -> Result<Vec<ScannedPackage>> {
        bail!("[STUB] GithubVaultAdapter not yet implemented")
    }
}
```

- [ ] **Step 2: Implement `ClaudeCodeProvider` stub**

```rust
// src/infra/provider/claude_code.rs
use anyhow::{bail, Result};
use crate::app::ports::ProviderPort;
use crate::domain::asset::ScannedPackage;
use crate::domain::identity::AssetIdentity;
use crate::domain::scope::Scope;

pub struct ClaudeCodeProvider;

impl ProviderPort for ClaudeCodeProvider {
    fn id(&self) -> &str { "claude-code" }
    fn install(&self, _pkg: &ScannedPackage, _scope: Scope) -> Result<()> {
        bail!("[STUB] ClaudeCodeProvider::install not yet implemented")
    }
    fn remove(&self, _identity: &AssetIdentity, _scope: Scope) -> Result<()> {
        bail!("[STUB] ClaudeCodeProvider::remove not yet implemented")
    }
}
```

- [ ] **Step 3: Verify it compiles**

```bash
cargo build 2>&1 | grep -E "^error"
```

Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add src/infra/vault/github.rs src/infra/provider/claude_code.rs
git commit -m "feat(infra): add stub GithubVaultAdapter and ClaudeCodeProvider"
```

---

### Task 12: App — Bootstrap

**Files:**
- Modify: `src/app/bootstrap.rs`

- [ ] **Step 1: Write failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn make_skill(root: &std::path::Path, name: &str) {
        let skill_dir = root.join("skills").join(name);
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), format!("# {}", name)).unwrap();
    }

    #[test]
    fn bootstrap_produces_four_tabs() {
        let dir = tempfile::tempdir().unwrap();
        let (registry, _) = build(dir.path().to_path_buf()).unwrap();
        // Skills + Instructions + Providers + Vaults
        assert_eq!(registry.feature_sets.len(), 4);
    }

    #[test]
    fn bootstrap_scans_skills() {
        let dir = tempfile::tempdir().unwrap();
        make_skill(dir.path(), "alpha");
        make_skill(dir.path(), "beta");
        let (registry, scan) = build(dir.path().to_path_buf()).unwrap();
        // Skills is tab index 0
        assert_eq!(scan.packages_by_tab[0].len(), 2);
    }

    #[test]
    fn bootstrap_skill_tab_is_live() {
        let dir = tempfile::tempdir().unwrap();
        let (registry, _) = build(dir.path().to_path_buf()).unwrap();
        assert!(!registry.feature_sets[0].is_stub());
        assert!(registry.feature_sets[1].is_stub()); // Instructions
    }
}
```

- [ ] **Step 2: Run to verify it fails**

```bash
cargo test app::bootstrap::tests 2>&1 | head -10
```

Expected: compile error.

- [ ] **Step 3: Implement bootstrap**

```rust
use std::path::PathBuf;
use anyhow::Result;
use crate::app::registry::Registry;
use crate::domain::asset::ScannedPackage;
use crate::infra::feature::skill::SkillFeatureSet;
use crate::infra::feature::stub::StubFeatureSet;
use crate::infra::vault::local::LocalVaultAdapter;

pub struct ScanResult {
    /// Index matches `registry.feature_sets` index.
    pub packages_by_tab: Vec<Vec<ScannedPackage>>,
}

/// Build a Registry wired for the given workspace root and run an initial scan.
pub fn build(workspace_root: PathBuf) -> Result<(Registry, ScanResult)> {
    let mut registry = Registry::new();

    // Feature sets — order defines tab order
    registry.register_feature_set(Box::new(SkillFeatureSet));
    registry.register_feature_set(Box::new(StubFeatureSet::new(
        "instruction", "Instructions", "instructions",
    )));
    registry.register_feature_set(Box::new(StubFeatureSet::new(
        "provider", "Providers", "",
    )));
    registry.register_feature_set(Box::new(StubFeatureSet::new(
        "vault", "Vaults", "",
    )));

    // Vaults
    registry.register_vault(Box::new(LocalVaultAdapter::new(
        "workspace",
        workspace_root,
    )));

    let scan = scan(&registry)?;
    Ok((registry, scan))
}

/// Scan all vaults for all feature sets and return packages grouped by tab index.
pub fn scan(registry: &Registry) -> Result<ScanResult> {
    let mut packages_by_tab = Vec::new();
    for feature in &registry.feature_sets {
        let mut tab_packages = Vec::new();
        if !feature.is_stub() {
            for vault in &registry.vaults {
                match vault.list_packages(feature.as_ref()) {
                    Ok(mut pkgs) => tab_packages.append(&mut pkgs),
                    Err(e) => eprintln!("vault '{}' scan error: {}", vault.id(), e),
                }
            }
        }
        packages_by_tab.push(tab_packages);
    }
    Ok(ScanResult { packages_by_tab })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_skill(root: &std::path::Path, name: &str) {
        let skill_dir = root.join("skills").join(name);
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), format!("# {}", name)).unwrap();
    }

    #[test]
    fn bootstrap_produces_four_tabs() {
        let dir = tempfile::tempdir().unwrap();
        let (registry, _) = build(dir.path().to_path_buf()).unwrap();
        assert_eq!(registry.feature_sets.len(), 4);
    }

    #[test]
    fn bootstrap_scans_skills() {
        let dir = tempfile::tempdir().unwrap();
        make_skill(dir.path(), "alpha");
        make_skill(dir.path(), "beta");
        let (_, scan) = build(dir.path().to_path_buf()).unwrap();
        assert_eq!(scan.packages_by_tab[0].len(), 2);
    }

    #[test]
    fn bootstrap_skill_tab_is_live() {
        let dir = tempfile::tempdir().unwrap();
        let (registry, _) = build(dir.path().to_path_buf()).unwrap();
        assert!(!registry.feature_sets[0].is_stub());
        assert!(registry.feature_sets[1].is_stub());
    }
}
```

- [ ] **Step 4: Run tests**

```bash
cargo test app::bootstrap::tests
```

Expected: 3 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/app/bootstrap.rs
git commit -m "feat(app): implement bootstrap — wires registry and runs initial scan"
```

---

### Task 13: Support — Error Types

**Files:**
- Modify: `src/support/error.rs`

- [ ] **Step 1: Implement `AppError`**

```rust
// src/support/error.rs
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),
    Scan(String),
    Terminal(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "IO error: {}", e),
            AppError::Scan(msg) => write!(f, "Scan error: {}", msg),
            AppError::Terminal(msg) => write!(f, "Terminal error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e)
    }
}
```

- [ ] **Step 2: Verify it compiles**

```bash
cargo build 2>&1 | grep "^error"
```

Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add src/support/error.rs
git commit -m "feat(support): add AppError type"
```

---

### Task 14: TUI — `AppState`

**Files:**
- Modify: `src/tui/app.rs`

- [ ] **Step 1: Write failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::domain::asset::{AssetKind, ScannedPackage};
    use crate::domain::identity::AssetIdentity;
    use std::path::PathBuf;

    fn make_pkg(name: &str) -> ScannedPackage {
        ScannedPackage {
            identity: AssetIdentity::new(name, None, "0000000000"),
            path: PathBuf::from("/skills").join(name),
            vault_id: "workspace".to_string(),
            kind: AssetKind::Skill,
        }
    }

    fn state_with_skills(pkgs: Vec<ScannedPackage>) -> AppState {
        let mut packages = HashMap::new();
        packages.insert(0usize, pkgs);
        let tab_names = vec!["Skills".to_string(), "Instructions".to_string()];
        let tab_live = vec![true, false];
        AppState::new(tab_names, tab_live, packages)
    }

    #[test]
    fn active_packages_returns_current_tab() {
        let state = state_with_skills(vec![make_pkg("alpha"), make_pkg("beta")]);
        assert_eq!(state.active_packages().len(), 2);
    }

    #[test]
    fn filtered_packages_empty_query_returns_all() {
        let state = state_with_skills(vec![make_pkg("alpha"), make_pkg("beta")]);
        assert_eq!(state.filtered_packages().len(), 2);
    }

    #[test]
    fn filtered_packages_filters_by_name() {
        let state = state_with_skills(vec![make_pkg("alpha-skill"), make_pkg("beta-tool")]);
        let mut s = state;
        s.search_query = "alpha".to_string();
        let filtered = s.filtered_packages();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].identity.name, "alpha-skill");
    }

    #[test]
    fn filtered_packages_case_insensitive() {
        let state = state_with_skills(vec![make_pkg("MySkill")]);
        let mut s = state;
        s.search_query = "myskill".to_string();
        assert_eq!(s.filtered_packages().len(), 1);
    }

    #[test]
    fn default_active_tab_is_zero() {
        let state = state_with_skills(vec![]);
        assert_eq!(state.active_tab, 0);
    }
}
```

- [ ] **Step 2: Run to verify it fails**

```bash
cargo test tui::app::tests 2>&1 | head -10
```

Expected: compile error.

- [ ] **Step 3: Implement `AppState`**

```rust
use std::collections::HashMap;
use crate::domain::asset::ScannedPackage;

#[derive(Debug, Clone, PartialEq)]
pub enum ListMode {
    Normal,
    Searching,
}

pub struct AppState {
    pub active_tab: usize,
    pub search_query: String,
    pub selected_index: usize,
    pub list_mode: ListMode,
    pub status_line: String,
    pub tab_names: Vec<String>,
    pub tab_live: Vec<bool>,
    pub packages: HashMap<usize, Vec<ScannedPackage>>,
}

impl AppState {
    pub fn new(
        tab_names: Vec<String>,
        tab_live: Vec<bool>,
        packages: HashMap<usize, Vec<ScannedPackage>>,
    ) -> Self {
        Self {
            active_tab: 0,
            search_query: String::new(),
            selected_index: 0,
            list_mode: ListMode::Normal,
            status_line: String::new(),
            tab_names,
            tab_live,
            packages,
        }
    }

    pub fn active_packages(&self) -> &[ScannedPackage] {
        self.packages
            .get(&self.active_tab)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn filtered_packages(&self) -> Vec<&ScannedPackage> {
        let pkgs = self.active_packages();
        if self.search_query.is_empty() {
            return pkgs.iter().collect();
        }
        let q = self.search_query.to_lowercase();
        pkgs.iter()
            .filter(|p| p.identity.name.to_lowercase().contains(&q))
            .collect()
    }

    pub fn is_active_tab_live(&self) -> bool {
        self.tab_live.get(self.active_tab).copied().unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::domain::asset::{AssetKind, ScannedPackage};
    use crate::domain::identity::AssetIdentity;
    use std::path::PathBuf;

    fn make_pkg(name: &str) -> ScannedPackage {
        ScannedPackage {
            identity: AssetIdentity::new(name, None, "0000000000"),
            path: PathBuf::from("/skills").join(name),
            vault_id: "workspace".to_string(),
            kind: AssetKind::Skill,
        }
    }

    fn state_with_skills(pkgs: Vec<ScannedPackage>) -> AppState {
        let mut packages = HashMap::new();
        packages.insert(0usize, pkgs);
        AppState::new(
            vec!["Skills".to_string(), "Instructions".to_string()],
            vec![true, false],
            packages,
        )
    }

    #[test]
    fn active_packages_returns_current_tab() {
        let state = state_with_skills(vec![make_pkg("alpha"), make_pkg("beta")]);
        assert_eq!(state.active_packages().len(), 2);
    }

    #[test]
    fn filtered_packages_empty_query_returns_all() {
        let state = state_with_skills(vec![make_pkg("alpha"), make_pkg("beta")]);
        assert_eq!(state.filtered_packages().len(), 2);
    }

    #[test]
    fn filtered_packages_filters_by_name() {
        let state = state_with_skills(vec![make_pkg("alpha-skill"), make_pkg("beta-tool")]);
        let mut s = state;
        s.search_query = "alpha".to_string();
        let filtered = s.filtered_packages();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].identity.name, "alpha-skill");
    }

    #[test]
    fn filtered_packages_case_insensitive() {
        let state = state_with_skills(vec![make_pkg("MySkill")]);
        let mut s = state;
        s.search_query = "myskill".to_string();
        assert_eq!(s.filtered_packages().len(), 1);
    }

    #[test]
    fn default_active_tab_is_zero() {
        let state = state_with_skills(vec![]);
        assert_eq!(state.active_tab, 0);
    }
}
```

- [ ] **Step 4: Run tests**

```bash
cargo test tui::app::tests
```

Expected: 5 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/tui/app.rs
git commit -m "feat(tui): implement AppState with filtering"
```

---

### Task 15: TUI — Layout

**Files:**
- Modify: `src/tui/layout.rs`

- [ ] **Step 1: Implement layout splitting**

```rust
use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct AppLayout {
    pub header: Rect,
    pub tabs: Rect,
    pub list: Rect,
    pub detail: Rect,
    pub footer: Rect,
}

pub fn compute(area: Rect) -> AppLayout {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Length(1), // tab bar
            Constraint::Min(1),    // list + detail
            Constraint::Length(2), // footer
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ])
        .split(vertical[2]);

    AppLayout {
        header: vertical[0],
        tabs: vertical[1],
        list: horizontal[0],
        detail: horizontal[1],
        footer: vertical[3],
    }
}
```

- [ ] **Step 2: Verify it compiles**

```bash
cargo build 2>&1 | grep "^error"
```

Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add src/tui/layout.rs
git commit -m "feat(tui): implement 5-zone terminal layout"
```

---

### Task 16: TUI — Tab Bar and Status Widgets

**Files:**
- Modify: `src/tui/widgets/tabs.rs`
- Modify: `src/tui/widgets/status.rs`

- [ ] **Step 1: Implement tab bar widget**

```rust
// src/tui/widgets/tabs.rs
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Tabs,
    Frame,
};

pub fn render(frame: &mut Frame, area: Rect, tab_names: &[String], active: usize) {
    let titles: Vec<String> = tab_names
        .iter()
        .enumerate()
        .map(|(i, name)| format!("[{}] {}", i + 1, name))
        .collect();

    let tabs = Tabs::new(titles)
        .select(active)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(tabs, area);
}
```

- [ ] **Step 2: Implement status widget**

```rust
// src/tui/widgets/status.rs
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render(frame: &mut Frame, area: Rect, status: &str, search: &str, is_live: bool) {
    let keybinds = if is_live {
        "[↑/↓] Move  [Space] Toggle  [u] Update  [1-4] Tabs  [r] Refresh  [q] Quit  [type] Search  [Esc] Clear"
    } else {
        "[1-4] Switch tabs  [q] Quit"
    };

    let status_text = if !status.is_empty() {
        status.to_string()
    } else if !search.is_empty() {
        format!("Search: {}", search)
    } else {
        String::new()
    };

    let lines = vec![
        Line::from(Span::styled(keybinds, Style::default().fg(Color::DarkGray))),
        Line::from(Span::raw(status_text)),
    ];

    frame.render_widget(Paragraph::new(lines), area);
}
```

- [ ] **Step 3: Verify it compiles**

```bash
cargo build 2>&1 | grep "^error"
```

Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add src/tui/widgets/tabs.rs src/tui/widgets/status.rs
git commit -m "feat(tui): implement tab bar and status widgets"
```

---

### Task 17: TUI — List and Detail Widgets

**Files:**
- Modify: `src/tui/widgets/list.rs`
- Modify: `src/tui/widgets/detail.rs`

- [ ] **Step 1: Implement list widget**

```rust
// src/tui/widgets/list.rs
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};
use crate::domain::asset::ScannedPackage;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    packages: &[&ScannedPackage],
    selected: usize,
    is_stub: bool,
) {
    let block = Block::default().borders(Borders::ALL).title("Packages");

    if is_stub {
        let items = vec![ListItem::new(Line::from(
            "  [STUB] Not yet implemented",
        ))];
        frame.render_widget(List::new(items).block(block), area);
        return;
    }

    let items: Vec<ListItem> = packages
        .iter()
        .map(|pkg| {
            let version = pkg.identity.version.as_deref().unwrap_or("--");
            ListItem::new(Line::from(format!(
                "[ ] {:<32} {:<8} {}",
                pkg.identity.name, version, pkg.vault_id
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
    if !packages.is_empty() {
        state.select(Some(selected));
    }

    frame.render_stateful_widget(list, area, &mut state);
}
```

- [ ] **Step 2: Implement detail widget**

```rust
// src/tui/widgets/detail.rs
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::domain::asset::ScannedPackage;

pub fn render(frame: &mut Frame, area: Rect, package: Option<&ScannedPackage>, is_stub: bool) {
    let block = Block::default().borders(Borders::ALL).title("Detail");

    if is_stub {
        frame.render_widget(
            Paragraph::new(Line::from("  [STUB] Not yet implemented")).block(block),
            area,
        );
        return;
    }

    let lines: Vec<Line> = match package {
        None => vec![Line::from("  No item selected")],
        Some(pkg) => {
            let label = |s: &str| Span::styled(s.to_string(), Style::default().fg(Color::Yellow));
            vec![
                Line::from(vec![label("Name:     "), Span::raw(pkg.identity.name.clone())]),
                Line::from(vec![
                    label("Kind:     "),
                    Span::raw(format!("{:?}", pkg.kind)),
                ]),
                Line::from(vec![
                    label("Vault:    "),
                    Span::raw(format!("{} (local)", pkg.vault_id)),
                ]),
                Line::from(vec![
                    label("Path:     "),
                    Span::raw(pkg.path.display().to_string()),
                ]),
                Line::from(""),
                Line::from(vec![
                    label("Identity: "),
                    Span::raw(pkg.identity.to_string()),
                ]),
            ]
        }
    };

    frame.render_widget(Paragraph::new(lines).block(block), area);
}
```

- [ ] **Step 3: Verify it compiles**

```bash
cargo build 2>&1 | grep "^error"
```

Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add src/tui/widgets/list.rs src/tui/widgets/detail.rs
git commit -m "feat(tui): implement list and detail widgets"
```

---

### Task 18: TUI — Render

**Files:**
- Modify: `src/tui/render.rs`

- [ ] **Step 1: Implement `draw()`**

```rust
use ratatui::{
    style::{Color, Modifier, Style},
    text::Line,
    widgets::Paragraph,
    Frame,
};
use crate::tui::{app::AppState, layout};
use crate::tui::widgets::{detail, list, status, tabs};

pub fn draw(frame: &mut Frame, state: &AppState) {
    let layout = layout::compute(frame.area());

    // Header
    let search_hint = if state.search_query.is_empty() {
        String::new()
    } else {
        format!("  [ Search: {} ]", state.search_query)
    };
    let header_text = format!("asky v0.1.0{}", search_hint);
    frame.render_widget(
        Paragraph::new(Line::from(header_text)).style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        layout.header,
    );

    // Tab bar
    tabs::render(frame, layout.tabs, &state.tab_names, state.active_tab);

    // Content — live vs stub
    let is_live = state.is_active_tab_live();
    let filtered = state.filtered_packages();
    let selected_pkg = filtered.get(state.selected_index).copied();

    list::render(frame, layout.list, &filtered, state.selected_index, !is_live);
    detail::render(frame, layout.detail, selected_pkg, !is_live);
    status::render(frame, layout.footer, &state.status_line, &state.search_query, is_live);
}
```

- [ ] **Step 2: Verify it compiles**

```bash
cargo build 2>&1 | grep "^error"
```

Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add src/tui/render.rs
git commit -m "feat(tui): implement top-level draw() dispatcher"
```

---

### Task 19: TUI — Event Loop

**Files:**
- Modify: `src/tui/event.rs`

- [ ] **Step 1: Write failing test for key dispatch logic**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::tui::app::{AppState, ListMode};

    fn empty_state(tab_count: usize) -> AppState {
        AppState::new(
            (0..tab_count).map(|i| format!("Tab{}", i)).collect(),
            vec![true; tab_count],
            HashMap::new(),
        )
    }

    #[test]
    fn switch_tab_updates_active_tab() {
        let mut state = empty_state(4);
        apply_tab_switch(&mut state, 2, 4);
        assert_eq!(state.active_tab, 2);
    }

    #[test]
    fn switch_tab_resets_selection_and_search() {
        let mut state = empty_state(4);
        state.selected_index = 3;
        state.search_query = "foo".to_string();
        apply_tab_switch(&mut state, 1, 4);
        assert_eq!(state.selected_index, 0);
        assert!(state.search_query.is_empty());
    }

    #[test]
    fn switch_tab_ignores_out_of_range() {
        let mut state = empty_state(4);
        apply_tab_switch(&mut state, 9, 4);
        assert_eq!(state.active_tab, 0); // unchanged
    }

    #[test]
    fn search_query_appends_char() {
        let mut state = empty_state(1);
        apply_search_char(&mut state, 'a');
        apply_search_char(&mut state, 'b');
        assert_eq!(state.search_query, "ab");
        assert_eq!(state.list_mode, ListMode::Searching);
    }

    #[test]
    fn esc_clears_search() {
        let mut state = empty_state(1);
        state.search_query = "hello".to_string();
        state.list_mode = ListMode::Searching;
        apply_esc(&mut state);
        assert!(state.search_query.is_empty());
        assert_eq!(state.list_mode, ListMode::Normal);
    }
}
```

- [ ] **Step 2: Run to verify it fails**

```bash
cargo test tui::event::tests 2>&1 | head -10
```

Expected: compile error.

- [ ] **Step 3: Implement event handling**

```rust
use std::time::Duration;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crate::tui::app::{AppState, ListMode};

pub enum ControlFlow {
    Continue,
    Quit,
}

pub fn handle(state: &mut AppState) -> Result<ControlFlow> {
    if !event::poll(Duration::from_millis(16))? {
        return Ok(ControlFlow::Continue);
    }

    if let Event::Key(key) = event::read()? {
        // Ctrl+C always quits
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Ok(ControlFlow::Quit);
        }

        match &key.code {
            KeyCode::Char('q') if state.list_mode == ListMode::Normal => {
                return Ok(ControlFlow::Quit);
            }
            KeyCode::Char(c @ '1'..='9') if state.list_mode == ListMode::Normal => {
                let idx = (*c as usize) - ('1' as usize);
                apply_tab_switch(state, idx, state.tab_names.len());
            }
            KeyCode::Up => {
                if state.selected_index > 0 {
                    state.selected_index -= 1;
                }
            }
            KeyCode::Down => {
                let count = state.filtered_packages().len();
                if state.selected_index + 1 < count {
                    state.selected_index += 1;
                }
            }
            KeyCode::Esc => apply_esc(state),
            KeyCode::Backspace => {
                state.search_query.pop();
                if state.search_query.is_empty() {
                    state.list_mode = ListMode::Normal;
                }
                state.selected_index = 0;
            }
            KeyCode::Char(' ') if state.list_mode == ListMode::Normal => {
                state.status_line = "[STUB] toggle not yet implemented".to_string();
            }
            KeyCode::Char('u') if state.list_mode == ListMode::Normal => {
                state.status_line = "[STUB] update not yet implemented".to_string();
            }
            KeyCode::Char('U') if state.list_mode == ListMode::Normal => {
                state.status_line = "[STUB] update-all not yet implemented".to_string();
            }
            KeyCode::Char('r') if state.list_mode == ListMode::Normal => {
                state.status_line = "[STUB] refresh not yet implemented".to_string();
            }
            KeyCode::Char('a') if state.list_mode == ListMode::Normal => {
                state.status_line = "[STUB] add vault not yet implemented".to_string();
            }
            KeyCode::Char('e') if state.list_mode == ListMode::Normal => {
                state.status_line = "[STUB] enable/disable vault not yet implemented".to_string();
            }
            KeyCode::Char('d') if state.list_mode == ListMode::Normal => {
                state.status_line = "[STUB] detach vault not yet implemented".to_string();
            }
            KeyCode::Char(c) => apply_search_char(state, *c),
            _ => {}
        }
    }

    Ok(ControlFlow::Continue)
}

pub fn apply_tab_switch(state: &mut AppState, idx: usize, tab_count: usize) {
    if idx < tab_count {
        state.active_tab = idx;
        state.selected_index = 0;
        state.search_query.clear();
        state.status_line.clear();
        state.list_mode = ListMode::Normal;
    }
}

pub fn apply_search_char(state: &mut AppState, c: char) {
    state.search_query.push(c);
    state.list_mode = ListMode::Searching;
    state.selected_index = 0;
    state.status_line.clear();
}

pub fn apply_esc(state: &mut AppState) {
    state.search_query.clear();
    state.list_mode = ListMode::Normal;
    state.selected_index = 0;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::tui::app::{AppState, ListMode};

    fn empty_state(tab_count: usize) -> AppState {
        AppState::new(
            (0..tab_count).map(|i| format!("Tab{}", i)).collect(),
            vec![true; tab_count],
            HashMap::new(),
        )
    }

    #[test]
    fn switch_tab_updates_active_tab() {
        let mut state = empty_state(4);
        apply_tab_switch(&mut state, 2, 4);
        assert_eq!(state.active_tab, 2);
    }

    #[test]
    fn switch_tab_resets_selection_and_search() {
        let mut state = empty_state(4);
        state.selected_index = 3;
        state.search_query = "foo".to_string();
        apply_tab_switch(&mut state, 1, 4);
        assert_eq!(state.selected_index, 0);
        assert!(state.search_query.is_empty());
    }

    #[test]
    fn switch_tab_ignores_out_of_range() {
        let mut state = empty_state(4);
        apply_tab_switch(&mut state, 9, 4);
        assert_eq!(state.active_tab, 0);
    }

    #[test]
    fn search_query_appends_char() {
        let mut state = empty_state(1);
        apply_search_char(&mut state, 'a');
        apply_search_char(&mut state, 'b');
        assert_eq!(state.search_query, "ab");
        assert_eq!(state.list_mode, ListMode::Searching);
    }

    #[test]
    fn esc_clears_search() {
        let mut state = empty_state(1);
        state.search_query = "hello".to_string();
        state.list_mode = ListMode::Searching;
        apply_esc(&mut state);
        assert!(state.search_query.is_empty());
        assert_eq!(state.list_mode, ListMode::Normal);
    }
}
```

- [ ] **Step 4: Run tests**

```bash
cargo test tui::event::tests
```

Expected: 5 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/tui/event.rs
git commit -m "feat(tui): implement event loop with keybinding dispatch"
```

---

### Task 20: CLI Entry and Main

**Files:**
- Modify: `src/cli/entry.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Implement CLI entry**

```rust
// src/cli/entry.rs
use clap::Parser;

#[derive(Parser)]
#[command(name = "asky", about = "Agent skill and instruction manager TUI", version)]
pub struct Cli {}

pub fn parse() -> Cli {
    Cli::parse()
}
```

- [ ] **Step 2: Implement `main.rs`**

```rust
mod app;
mod cli;
mod domain;
mod infra;
mod support;
mod tui;

use std::collections::HashMap;
use std::io;
use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

fn main() -> Result<()> {
    cli::entry::parse();

    let workspace = std::env::current_dir()?;
    let (registry, scan) = app::bootstrap::build(workspace)?;

    let tab_names: Vec<String> = registry
        .feature_sets
        .iter()
        .map(|f| f.display_name().to_string())
        .collect();

    let tab_live: Vec<bool> = registry
        .feature_sets
        .iter()
        .map(|f| !f.is_stub())
        .collect();

    let packages: HashMap<usize, Vec<_>> = scan
        .packages_by_tab
        .into_iter()
        .enumerate()
        .collect();

    let mut state = tui::app::AppState::new(tab_names, tab_live, packages);

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, &mut state);

    // Always restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    state: &mut tui::app::AppState,
) -> Result<()> {
    loop {
        terminal.draw(|frame| tui::render::draw(frame, state))?;

        match tui::event::handle(state)? {
            tui::event::ControlFlow::Quit => break,
            tui::event::ControlFlow::Continue => {}
        }
    }
    Ok(())
}
```

- [ ] **Step 3: Build and run**

```bash
cargo build 2>&1 | grep "^error"
```

Expected: no errors.

```bash
cargo run
```

Expected: TUI launches, Skills tab shows packages from `skills/`, `q` exits cleanly.

- [ ] **Step 4: Run all tests**

```bash
cargo test
```

Expected: all tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/cli/entry.rs src/main.rs
git commit -m "feat: wire CLI entry and main event loop"
```

---

### Task 21: `docs/FEATURES.md`

**Files:**
- Create: `docs/FEATURES.md`

- [ ] **Step 1: Create feature tracker**

```markdown
# Feature Tracker

Tracks all features from the [Technical Design Doc](plans/20260328_technical_design.md) against implementation status.

**Status key:** `[ ]` not started · `[~]` partial/stub · `[x]` complete

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
| 13 | Asset identity parsing / display | §4.1 | `[ ]` |
| 14 | Instructions tab | §11.3 | `[~]` stub |
| 15 | Providers tab | §11.4 | `[~]` stub |
| 16 | Vaults tab | §11.2 | `[~]` stub |
| 17 | config.toml read/write | §5 | `[ ]` |
| 18 | GithubVaultAdapter | §9 | `[~]` stub |
| 19 | ClaudeCodeProvider adapter | §8 | `[~]` stub |
| 20 | Install asset | §12.2 | `[ ]` |
| 21 | Update asset | §12.2 | `[ ]` |
| 22 | Remove asset | §3.1 | `[ ]` |
| 23 | Scope: global/workspace | §3.5 | `[ ]` |
| 24 | Vault attach/detach | §12.3 | `[ ]` |
| 25 | Space: toggle item check | §10.5 | `[ ]` |
| 26 | Version extraction from package | §4.2 | `[ ]` |
| 27 | InstructionFeatureSet adapter | §11.3 | `[ ]` |
```

After phase 1 implementation is complete, update features 1–13 from `[ ]` to `[x]`.

- [ ] **Step 2: Commit**

```bash
git add docs/FEATURES.md
git commit -m "docs: add FEATURES.md feature tracker"
```

---

## Self-Review

**Spec coverage check:**

| Spec section | Covered by task |
|---|---|
| §2 Architecture (hexagonal) | Tasks 6, 7, 8, 9, 10, 11 |
| §3 Port traits | Task 6 |
| §4 Registry | Task 7 |
| §5 Module layout | Task 1 |
| §6 Domain types | Tasks 2–5 |
| §7 SkillFeatureSet | Task 8 |
| §8 sha10 | Task 5 |
| §9 LocalVaultAdapter | Task 10 |
| §10 AppState | Task 14 |
| §11 TUI Layout | Task 15 |
| §12 Per-tab rendering | Tasks 16, 17, 18 |
| §12 Keybindings | Task 19 |
| §13 Error handling | Tasks 13, 20 |
| §14 FEATURES.md | Task 21 |

**Type consistency check:** `ScannedPackage.identity: AssetIdentity` used consistently across tasks 4, 10, 12, 14, 17, 18. `Registry` fields match port trait boxes throughout. `AppState.tab_live: Vec<bool>` set in bootstrap (task 12) and read in render (task 18) via `is_active_tab_live()`.

**No placeholders found.**
