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
- `asky` delegates scanning packages entirely to the implementations wrapped beneath this structural hierarchy. 
- Network-bound vaults explicitly mandate a `refresh()` abstraction point to synchronize the remote filesystem dynamically. This utilizes lightweight `git` sparse-checkout commands directed efficiently into the `vaults/` subdirectory within the global configuration root (`~/.config/asky/` on macOS/Linux, `%APPDATA%\asky\` on Windows).
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
```

## Internal Workflows

### GitHub Vault Attachment (Multi-Step)
To provide a granular configuration experience for remote vaults, the TUI implements a 3-step state machine:
1. **URL Input**: Captures the repository URL. If valid, the system parses the repository name and transitions to branch selection.
2. **Branch Selection**: Prompts for the git reference (e.g., `main`, `develop`, or a specific tag). Defaults to `main`.
3. **Subfolder Path**: Prompts for the relative directory containing assets. Defaults to `skills/`.

Once finalized, the adapter immediately triggers an async `refresh()` to execute a sparse clone, followed by a global `TriggerReload` to populate the asset registry.

### Boot Execution Sequences
When `asky` starts, the bootstrap loader resolves `config.toml`:
1. Instantiates vault implementations natively registered through the configuration entries.
2. Iterates vault scanning routines.
3. Automatically computes `sha10` hashes (or fetches Git commit hashes for GitHub sources), identifying out-of-date records dynamically based purely on mathematical file tracking integrity constraints.
4. Renders Tab 4 list arrays natively encapsulating volume identifiers accurately.
