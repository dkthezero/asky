# Phase 2 Design — `asky` TUI: Persistence, Actions & Live Tabs

**Date:** 2026-03-30
**Status:** Approved

---

## 1. Scope

Phase 2 completes the full working loop: scan → display → act → persist. It is delivered as feature slices, each independently reviewable and releasable.

### Included

1. **`config.toml` persistence layer** — load/save both global and workspace scope files using the full schema
2. **Instructions live** — `InstructionFeatureSet` scanning `instructions/` for `AGENTS.md` marker
3. **Vault attach/detach** — `Space` on the Vaults tab writes vault config to the active scope's `config.toml`
4. **Provider management** — `Space` on Providers tab installs (copies all checked assets) or removes a provider; auto-redirect to Providers tab if no provider is set
5. **Install/remove assets** — `Space` on Skills/Instructions tabs copies/removes files via the active provider for the active scope; writes asset identity to config
6. **Scope switching** — `s` toggles Global ↔ Workspace; all reads and writes use the active scope

### Out of scope

- GitHub vault adapter (remains a stub)
- Update-outdated bulk action (`U`)
- Version extraction from package metadata
- Multi-vault conflict resolution

---

## 2. Architecture

Phase 2 extends the existing hexagonal architecture. No existing layer boundaries change.

### New domain types

```rust
// Persisted record of an installed asset
struct InstalledAsset {
    kind: AssetKind,
    vault_id: String,
    identity: AssetIdentity,   // [name:version:sha10]
}

// Vault definition stored in config
struct VaultConfig {
    id: String,
    kind: VaultKind,           // Local | Github
    source: VaultSource,       // path or repo/ref/path
}

// Provider record stored in config
struct ProviderConfig {
    id: String,
}

// Full config file — one instance per scope
struct ConfigFile {
    version: u32,
    vaults: Vec<VaultConfig>,
    providers: Vec<ProviderConfig>,
    skills: HashMap<String, Vec<InstalledAsset>>,       // keyed by vault_id
    instructions: HashMap<String, Vec<InstalledAsset>>, // keyed by vault_id
}
```

### New port (app layer)

```rust
trait ConfigStorePort: Send + Sync {
    fn load(&self, scope: Scope) -> Result<ConfigFile>;
    fn save(&self, scope: Scope, config: &ConfigFile) -> Result<()>;
}
```

### New infra adapters

| Adapter | Responsibility |
|---|---|
| `TomlConfigStore` | Reads/writes `~/.config/asky/config.toml` (global) or `<cwd>/.asky/config.toml` (workspace) via the `toml` crate |
| `InstructionFeatureSet` | Scans `instructions/` for folders containing `AGENTS.md`; instruction name = folder name |
| `ClaudeCodeProvider` (filled in) | Global root: `~/.claude/`; workspace root: `<cwd>/.claude/`; copies files on install, removes on uninstall |

### New app use-case functions (`app/`)

```rust
fn install_asset(scope: Scope, pkg: &ScannedPackage, store: &dyn ConfigStorePort, provider: &dyn ProviderPort) -> Result<()>
fn remove_asset(scope: Scope, identity: &AssetIdentity, store: &dyn ConfigStorePort, provider: &dyn ProviderPort) -> Result<()>
fn attach_vault(scope: Scope, config: VaultConfig, store: &dyn ConfigStorePort) -> Result<()>
fn detach_vault(scope: Scope, vault_id: &str, store: &dyn ConfigStorePort) -> Result<()>
fn install_provider(scope: Scope, provider_id: &str, checked: &[ScannedPackage], store: &dyn ConfigStorePort, provider: &dyn ProviderPort) -> Result<()>
fn remove_provider(scope: Scope, provider_id: &str, store: &dyn ConfigStorePort, provider: &dyn ProviderPort) -> Result<()>
```

### AppState additions

```rust
active_scope: Scope                          // Global | Workspace, toggled by `s`
checked_items: HashSet<AssetKey>             // Space-toggled items (name + vault_id)
configs: HashMap<Scope, ConfigFile>          // Both scope configs loaded at boot
```

### Data flow — Space (install/remove asset)

```
KeyEvent(Space)
  → AppState: if no provider for active scope → set status_line warning, switch to Providers tab
  → AppState: toggles checked_items for selected row
  → app::install_asset / remove_asset
      → ClaudeCodeProvider: copies / removes files at scope path
      → TomlConfigStore: saves updated ConfigFile for active scope
  → AppState: refreshes row status from updated config
```

---

## 3. TUI Changes

### New keybindings

| Key | Context | Action |
|---|---|---|
| `s` | Any tab | Toggle active scope (Global ↔ Workspace) |
| `a` | Vaults tab | Open inline prompt to enter details for a **new** vault and attach it |
| `Space` | Skills / Instructions | Install ↔ remove asset via active provider |
| `Space` | Vaults | Enable ↔ disable an **existing** listed vault for the active scope |
| `Space` | Providers | Install ↔ remove provider for active scope |

### Status line

- Always shows active scope: `[global]` or `[workspace]`
- No-provider warning: `No provider set — press [4] to configure`

### Vaults tab (live)

```
[x] workspace   local   3/0s   0/0i
[ ] community   github  0/48s  0/12i
```

Columns: checked · vault id · type · skills count (`installed/available`) · instructions count

### Skills / Instructions tabs

Each row gains a status column:

```
[✓] web-browsing-tool    --    workspace   [✓]
[ ] concise-mode         --    workspace   [ ]
```

Status: `[✓]` installed · `[!]` outdated (sha10 mismatch) · `[ ]` available

Status is derived by comparing scanned `sha10` against the loaded `ConfigFile` for the active scope.

### Providers tab (live)

- Lists providers from the active scope's config
- Active provider marked distinctly
- `Space` installs provider (copies all checked assets into provider path) or removes it

### No-provider guard

On `Space` in Skills/Instructions tabs: if active scope's config has no provider, set `status_line` to the warning message and switch `active_tab` to Providers tab. The toggle action is not applied.

---

## 4. `config.toml` Schema & File Paths

### File locations

| Scope | Path |
|---|---|
| Global | `~/.config/asky/config.toml` |
| Workspace | `<cwd>/.asky/config.toml` |

Each file is independent with identical schema. Workspace does not inherit from global — they are read separately and the UI reflects whichever is active.

### Full schema

```toml
version = 1

vaults = ["workspace", "community"]
providers = ["claude-code"]

[workspace.vault]
type = "local"
path = "/Users/hung/dev/my-project"

[community.vault]
type = "github"
repo = "org/community-agent-vault"
ref = "main"
path = "vault"

# Asset buckets keyed by vault id — one section per vault that has installed assets
[workspace.skills]
items = [
  "[web-browsing-tool:--:a13c9ef042]",
  "[arxiv-researcher:--:66ad0110ab]",
]

[community.skills]
items = [
  "[another-tool:--:f9918bc0de]",
]

[workspace.instructions]
items = [
  "[concise-mode:--:d91ab3301f]",
]
```

### Schema rules

- `vaults` — ordered list of attached vault ids for this scope
- `providers` — list of installed provider ids for this scope
- `[<id>.vault]` — vault definition; `type` is `"local"` or `"github"`
  - local: `path = "/abs/path"`
  - github: `repo = "org/repo"`, `ref = "main"`, `path = "vault"`
- `[<vault-id>.skills]` and `[<vault-id>.instructions]` — asset buckets grouped by origin vault id
- Asset items encoded as `[<name>:<version>:<sha10>]`; version is `--` when unavailable
- If no assets installed from a vault, the bucket section is omitted
- Missing config file = empty `ConfigFile` (not an error)

### Bootstrap behaviour

1. Load both scope config files at startup (missing = empty config, no error)
2. Cache both `ConfigFile` instances in `AppState.configs`
3. Scan attached vaults from the active scope's config
4. Render TUI with status derived from loaded configs

---

## 5. Testing Strategy

TDD throughout: tests first, implement to pass, commit per slice.

### `TomlConfigStore`

- Round-trip: write a `ConfigFile`, read it back, assert equal
- Missing file returns empty `ConfigFile`
- Global and workspace paths resolve correctly from a temp dir
- Partial config (vaults only, no asset buckets) parses without error

### `InstructionFeatureSet`

- Detects `AGENTS.md` marker correctly
- Ignores folders without `AGENTS.md`
- Instruction name equals folder name
- `hash_files` includes all files under the instruction folder
- `is_stub()` returns `false`

### `ClaudeCodeProvider`

- `install` copies skill files to correct path for global scope (`~/.claude/skills/<name>/`)
- `install` copies to workspace scope (`<cwd>/.claude/skills/<name>/`)
- `remove` deletes the installed directory
- Instructions follow same pattern under `.claude/instructions/`

### App use-case functions

- `install_asset` writes identity to config and invokes provider copy
- `remove_asset` removes identity from config and invokes provider delete
- `attach_vault` / `detach_vault` updates vault list in config for the correct scope
- `install_provider` copies all checked assets and records provider in config
- `remove_provider` removes provider record and cleans up installed files
- `install_asset` returns a typed error when no provider is set for the scope

### AppState

- `s` key toggles `active_scope` between Global and Workspace
- Scope toggle re-derives row statuses from `configs[active_scope]`
- `checked_items` correctly reflects install/remove state per scope

### Integration (bootstrap)

- Bootstrap loads both scope configs from temp dirs and caches in AppState
- Vault scan runs only for vaults listed in the active scope's config
