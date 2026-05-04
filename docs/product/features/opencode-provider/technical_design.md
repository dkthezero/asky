# Technical Design: OpenCode Provider Support

## Overview

OpenCode is a file-based agent runtime with Claude Code compatibility. The adapter implements `ProviderPort` to copy skills to `.opencode/skills/` and merge skill references into `opencode.json`.

## Architecture Rules

1. **ProviderPort trait only.** No new traits. Standard `install`/`remove` interface.
2. **JSON merge, not replace.** OpenCode's config system is hierarchical; we must preserve existing keys.
3. **Path isolation.** All OpenCode paths are resolved in one place (`OpenCodeProvider`) for testability.
4. **JSONC support.** Use a JSONC parser (or strip comments before parsing) to avoid corrupting user configs.

## Data Schemas

### OpenCodeConfig (Internal)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenCodeConfig {
    #[serde(default)]
    skills: Vec<OpenCodeSkillRef>,
    #[serde(flatten)]
    other: serde_json::Map<String, serde_json::Value>, // preserve unknown keys
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenCodeSkillRef {
    name: String,
    path: String,
}
```

## Internal Workflows

### Install Workflow
1. Determine scope (Global / Workspace).
2. Compute destination: `provider_root(scope)/skills/{name}/SKILL.md`.
3. Copy the skill directory to the destination using `infra::provider::common::copy_dir`.
4. Load existing `opencode.json` (global or workspace).
5. Parse as JSONC → `OpenCodeConfig`.
6. Add or update the skill reference in `config.skills`.
7. Serialize back to JSONC (preserve comments if possible; if not, write clean JSON).
8. Write to the config file.

### Remove Workflow
1. Determine scope.
2. Delete the skill directory.
3. Load `opencode.json`.
4. Remove the skill reference from `config.skills`.
5. Write back.

### JSONC Handling
- Use `jsonc-parser` crate (or implement a simple comment stripper).
- If we can't preserve comments, document that `opencode.json` may be rewritten as plain JSON.

## Trait Contracts

Uses existing `ProviderPort` and `McpProvider`:
```rust
fn install(&self, pkg: &ScannedPackage, scope: Scope) -> Result<()>;
fn remove(&self, identity: &AssetIdentity, kind: &AssetKind, scope: Scope) -> Result<()>;

// MCP support (McpProvider trait)
fn write_mcp_server(&self, server: &McpServer, scope: Scope) -> Result<()>;
fn remove_mcp_server(&self, name: &str, scope: Scope) -> Result<()>;
```

**MCP Schema (OpenCode):**
- Writes flat `mcp.<name>` entries to `opencode.json` (not nested `mcp.servers`).
- Required fields: `type` ("local" or "remote"), `enabled` (boolean).
- Local: `{ "type": "local", "command": "...", "args": [...], "env": {...}, "enabled": true }`
- Remote (SSE): `{ "type": "remote", "url": "...", "enabled": true }`
- On remove: drops the server entry. If `mcp` becomes empty, drops the entire `mcp` key to avoid schema validation errors.
- **Migration:** On write, drops any stale `mcp.servers` key (from earlier schema iteration).

## Module Structure

```
src/infra/provider/
  opencode.rs     # OpenCodeProvider implementation
  mod.rs          # Add `pub(crate) mod opencode;`
```

## Testing Strategy

- **Unit tests:**
  - Install copies files to correct path.
  - Remove deletes files and updates config.
  - JSON merge preserves unrelated keys.
  - JSONC parsing doesn't panic on comments.
- **Integration:**
  - Register in bootstrap, verify TUI shows OpenCode.

---

*End of Technical Design.*
