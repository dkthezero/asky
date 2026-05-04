# Technical Design: MCP Vault Management

## Overview

MCP servers are introduced as a new `AssetKind`. They are registered in a global agk registry (`~/.config/agk/mcp.toml`) and activated per-provider per-scope. This requires a tab restructure and new domain models.

## Architecture Rules

1. **MCP is an AssetKind, not a Vault.** They are registered into agk's global config, not sourced from external repos.
2. **Registration is global; activation is scoped.** The registry lives in `~/.config/agk/`. Activation writes to provider-specific config in global or workspace scope.
3. **Two-phase install:** Register → Test → Activate. Testing performs an MCP `initialize` handshake.
4. **Tab restructure is breaking.** Vaults becomes Tab `0`. MCP Servers becomes Tab `4`. This is acceptable before v1.0.
5. **Security first.** Arbitrary code execution warning before test/activate. No sandboxing — warn the user.

## Data Schemas

### AssetKind (Extended)
```rust
pub enum AssetKind {
    Skill,
    Instruction,
    McpServer, // NEW
}
```

### McpServerRegistry
```toml
# ~/.config/agk/mcp.toml
[servers.fs]
name = "fs"
command = "npx"
args = ["@modelcontextprotocol/server-filesystem", "/Users/alice/docs"]
env = { NODE_ENV = "production" }
transport = "stdio"
description = "Filesystem access"
tested = true
tested_at = "2026-05-03T10:00:00Z"

[servers.fs.activation]
claude-code = { global = true, workspace = false }
opencode = { global = false, workspace = true }
```

### McpServer (Domain)
```rust
#[derive(Debug, Clone)]
pub struct McpServer {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub transport: McpTransport,
    pub description: Option<String>,
    pub tested: bool,
    pub tested_at: Option<DateTime<Utc>>,
}

pub enum McpTransport {
    Stdio,
    Sse { url: String },
}
```

### McpConfig (Per-Provider)
```rust
// Claude Desktop / Claude Code JSON
{
  "mcpServers": {
    "fs": {
      "command": "npx",
      "args": ["@modelcontextprotocol/server-filesystem", "/Users/alice/docs"],
      "env": { "NODE_ENV": "production" }
    }
  }
}

// OpenCode JSON (merged)
{
  "mcp": {
    "servers": {
      "fs": { ... }
    }
  }
}
```

## Internal Workflows

### Registration (Phase 1)
1. Collect metadata from TUI modal or CLI args.
2. Validate: name is unique, command exists on PATH.
3. **Test:** Spawn the process, send MCP `initialize` JSON-RPC request.
   ```json
   {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"agk","version":"0.1.2"}}}
   ```
4. Wait for `initialize` response (timeout: 10s).
5. If success: save to registry with `tested = true`.
6. If fail: show error, don't save (or save as `tested = false` if `--no-test`).

### Activation (Phase 2)
1. Determine active providers in current scope.
2. For each active provider that supports MCP:
   a. Load provider's MCP config file.
   b. Merge in the new server definition.
   c. Write back.
3. Update activation state in agk registry.

### Deactivation
1. For each provider where the server is currently enabled:
   a. Load MCP config.
   b. Remove the server entry.
   c. Write back.
2. Update activation state.

## Trait Contracts

### McpProvider (new trait, implemented by existing providers)
```rust
pub trait McpProvider {
    fn supports_mcp(&self) -> bool;
    fn mcp_config_path(&self, scope: Scope) -> Option<PathBuf>;
    fn write_mcp_config(&self, servers: &[McpServer], scope: Scope) -> Result<()>;
    fn remove_mcp_config(&self, name: &str, scope: Scope) -> Result<()>;
}
```

**Note:** Only Claude Desktop, Claude Code, and OpenCode support MCP initially. Other providers return `supports_mcp = false`.

## Module Structure

```
src/domain/
  asset.rs           # Add AssetKind::McpServer
  mcp.rs             # NEW: McpServer, McpTransport, McpRegistry
src/infra/
  mcp/
    mod.rs           # Re-export
    registry.rs      # Load/save mcp.toml
    tester.rs        # MCP initialize handshake
    provider_config.rs  # Write provider-specific MCP JSON
  provider/
    opencode.rs      # Implement McpProvider
    claude_code.rs   # Implement McpProvider
src/tui/
  widgets/
    mcp_list.rs      # Tab 4 rendering
    mcp_modal.rs     # Registration modal
```

## Testing Strategy

- **Unit tests:**
  - MCP JSON-RPC message serialization/deserialization.
  - Registry save/load round-trip.
  - Provider config merge (add/remove server without corrupting other keys).
- **Integration:**
  - Register a fake MCP server (mock the handshake).
  - Activate/deactivate and verify provider config files.
  - TUI tab switching works after restructure.

---

*End of Technical Design.*
