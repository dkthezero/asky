# Technical Design: Vaults

## Overview
Vaults act as immutable origins encapsulating logical packages (Skills and Instructions) without being strictly coupled to how that data behaves on the execution layer. Vault backends map paths to remote servers or local file nodes identically through Trait abstractions. 

## Trait Contracts

### VaultSource Trait
```rust
trait VaultSource {
    fn id(&self) -> &str;
    fn kind(&self) -> VaultKind;

    fn list_skills(&self) -> Result<Vec<RemoteAsset>>;
    fn list_instructions(&self) -> Result<Vec<RemoteAsset>>;

    fn fetch_skill(&self, name: &str) -> Result<SkillPackage>;
    fn fetch_instruction(&self, name: &str) -> Result<InstructionPackage>;
}
```
**Architecture Rules:**
- `agk` delegates scanning packages entirely to the implementations wrapped beneath this structural hierarchy. 
- Network-bound vaults explicitly mandate a `refresh()` abstraction point to synchronize the remote filesystem dynamically. This utilizes lightweight `git` sparse-checkout commands directed efficiently into the `vaults/` subdirectory within the global configuration root (`~/.config/agk/` on macOS/Linux, `%APPDATA%\agk\` on Windows).
- Replaceable Backend: Local filesystem abstractions resolve cleanly, while GitHub nodes translate network layers into pure package outputs implicitly without bleeding standard protocol errors upwards into the presentation layers natively.

## Schema Implementations

A Vault's configuration inside the monolithic `config.toml` requires definitions on what it tracks structurally.

### TOML Shape Configuration
```toml
vaults = ["community", "internal-team", "local-dev"]

[vault.community]
type = "github"
repo = "org/community-agent-vault"
ref = "main"
path = "vault"

[vault.internal-team]
type = "github"
repo = "org/internal-agent-vault"
ref = "main"
path = "vault"

[vault.local-dev]
type = "local"
path = "/Users/jane/dev/agent-vault"

[vault.clawhub]
type = "clawhub"
```

## Internal Workflows

### GitHub Vault Attachment (Multi-Step)
To provide a granular configuration experience for remote vaults, the TUI implements a 3-step state machine:
1. **URL Input**: Captures the repository URL. If valid, the system parses the repository name and transitions to branch selection.
2. **Branch Selection**: Prompts for the git reference (e.g., `main`, `develop`, or a specific tag). Defaults to `main`.
3. **Subfolder Path**: Prompts for the relative directory containing assets. Defaults to `skills/`.

Once finalized, the adapter immediately triggers an async `refresh()` to execute a sparse clone, followed by a global `TriggerReload` to populate the asset registry.

### ClawHub Vault Adapter
The `ClawHubVaultAdapter` implements `VaultPort` using a CLI-delegation pattern. All remote operations shell out to the `clawhub` binary rather than implementing HTTP/API calls directly.

```rust
pub struct ClawHubVaultAdapter {
    id: String,
}

impl VaultPort for ClawHubVaultAdapter {
    fn id(&self) -> &str { &self.id }
    fn kind_name(&self) -> &str { "clawhub" }
    fn list_packages(&self, feature: &dyn FeatureSetPort) -> Result<Vec<ScannedPackage>> {
        // Delegates to LocalVaultAdapter scanning ~/.config/agk/clawhub/
    }
}
```

**CLI Commands Used:**
- `clawhub search <query>` — returns matching slugs (stdout), one per line, format: `slug  DisplayName  (score)`
- `clawhub inspect <slug> --json` — returns JSON with `owner.handle`, `skill.summary`, `skill.stats.downloads`, `skill.stats.stars`, `latestVersion.version`
- `clawhub install <slug> --workdir <path>` — installs skill files into the specified directory

**Helper Functions:**
- `is_cli_available()` — checks `which clawhub` on `$PATH`
- `is_homebrew_available()` — checks `which brew`
- `install_cli_via_homebrew()` — runs `brew install clawhub`
- `cli_search(query)` — parses search output, enriches each result (up to 10) via `inspect_slug()`, returns `Vec<ScannedPackage>` with `is_remote: true` and populated `RemoteMetadata`
- `cli_install(name)` — strips `owner/` prefix, runs install into `clawhub_cache_dir()`

**Search Integration:**
Search dispatches are registered as tasks via `TaskStarted`/`TaskCompleted` events, so progress renders in the bottom-right task progress area alongside other background operations. Results arrive via `AppEvent::ClawHubSearchResults` and are merged with local results (local wins on deduplication).

### Boot Execution Sequences
When `agk` starts, the bootstrap loader resolves `config.toml`:
1. Instantiates vault implementations natively registered through the configuration entries.
2. Iterates vault scanning routines.
3. Automatically computes `sha10` hashes (or fetches Git commit hashes for GitHub sources), identifying out-of-date records dynamically based purely on mathematical file tracking integrity constraints.
4. Renders Tab 4 list arrays natively encapsulating volume identifiers accurately.
