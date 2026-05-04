# Technical Design: MCP Vault Management

## Overview

MCP servers are introduced as a new `AssetKind::McpServer`. They are registered in a global agk registry (`~/.config/agk/mcp.toml`) and activated per-provider per-scope. This requires domain model extensions, a dedicated TUI tab, and provider-specific config serialization.

---

## Architecture Rules

1. **MCP is an AssetKind, not a Vault.** They are registered into agk's global config, not sourced from external repos.
2. **Registration is global; activation is scoped.** The registry lives in `~/.config/agk/mcp.toml`. Activation writes to provider-specific config in global or workspace scope.
3. **Two-phase install:** Register → Test → Activate. Testing performs an MCP `initialize` handshake automatically after registration.
4. **Tab placement:** Tab `[2]` = MCP Servers (between Skills `[1]` and Instructions `[3]`). Vaults remains right-aligned as `[0]`.
5. **Security first.** Arbitrary code execution: the TUI shows a warning before registering ("This will execute '{command} {args}' on your machine"). No sandboxing — explicit user confirmation is required.

---

## Data Schemas

### AssetKind (Extended)
```rust
pub enum AssetKind {
    Skill,
    Instruction,
    #[allow(dead_code)]
    McpServer, // NEW
}
```

### McpRegistry
Stored at `~/.config/agk/mcp.toml`:
```toml
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub transport: McpTransport,
    pub description: Option<String>,
    pub tested: bool,
    pub tested_at: Option<String>,
    // Provider activation state: provider_id → { global, workspace }
    #[serde(default)]
    pub activation: HashMap<String, McpActivation>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct McpActivation {
    pub global: bool,
    pub workspace: bool,
}

pub enum McpTransport {
    Stdio,
    Sse { url: String },
}
```

### Provider-Specific MCP Schemas

#### Claude Code
`.claude/mcp.json` (or `~/.claude/mcp.json`):
```json
{
  "mcpServers": {
    "fs": {
      "command": "npx",
      "args": ["@modelcontextprotocol/server-filesystem", "/Users/alice/docs"],
      "env": { "NODE_ENV": "production" }
    }
  }
}
```

#### OpenCode
Flat `mcp.<name>` in `opencode.json` (NOT nested `mcp.servers`):
```json
{
  "mcp": {
    "fs": {
      "type": "local",
      "command": "npx",
      "args": ["@modelcontextprotocol/server-filesystem", "/Users/alice/docs"],
      "env": { "NODE_ENV": "production" },
      "enabled": true
    }
  }
}
```
On remove, if `mcp` becomes empty, the entire `mcp` key is dropped to avoid schema validation errors.

---

## Internal Workflows

### Registration (Phase 1)

#### TUI Flow
1. User presses `F2` in Tab `[2]`.
2. Modal collects step-by-step:
   - **Name** (required)
   - **Command** (required)
   - **Arguments** (optional)
   - **Transport** (`stdio` or `sse`)
   - **Description** (optional)
3. Final confirmation: show `"WARNING: This will execute 'cmd args' on your machine. Register? [y/N]"`.
4. User presses `y` to confirm, `n` or `Esc` to cancel.
5. Background task registers, auto-runs test, saves to `mcp.toml`, sends `TriggerReload`.

#### CLI Flow
```bash
agk mcp add --name fs --command npx --args "arg1 arg2" --transport stdio
```
- Default `--test` (true): registers + test in one shot.
- `--no-test`: skips handshake.

#### Test Handshake
1. Spawn process (`command` + `args`).
2. Send JSON-RPC `initialize` request:
   ```json
   {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"agk","version":"0.1.2"}}}
   ```
3. Wait for response (timeout: 10s).
4. Kill the process.
5. If response contains `"jsonrpc"`: mark `tested = true`, save.
6. If timeout/no response: mark `tested = false`, report error.

### Activation (Phase 2)

#### TUI Flow
1. Navigate to a server in Tab `[2]`.
2. Press `Space`.
3. Check if **any active provider** has this server enabled in the **current scope**.
4. If enabled: **disable** for all active MCP-capable providers in current scope.
5. If disabled: **enable** for all active MCP-capable providers in current scope.
6. Checkbox `[x]`/`[ ]` updates in real time after `TriggerReload`.

#### CLI Flow
```bash
agk mcp enable fs --provider claude-code --scope workspace
agk mcp disable fs --provider opencode --scope global
```

#### Provider Activation Steps
1. `enable(name, provider_id, scope, providers)`:
   a. Load `mcp.toml`, get server.
   b. Find `McpProvider` matching `provider_id`.
   c. Call `provider.write_mcp_server(server, scope)`.
   d. Update `activation[provider_id].{global,workspace}` in registry.
   e. Save `mcp.toml`.
2. `disable(name, provider_id, scope, providers)`:
   a. Find provider, call `provider.remove_mcp_server(name, scope)`.
   b. Update `activation[provider_id]` flags in registry.
   c. Save `mcp.toml`.

---

## Trait Contracts

### McpProvider
```rust
pub trait McpProvider: Send + Sync {
    fn provider_id(&self) -> &str;
    fn supports_mcp(&self) -> bool;
    fn mcp_config_path(&self, scope: Scope) -> Option<PathBuf>;
    fn write_mcp_server(&self, server: &McpServer, scope: Scope) -> Result<()>;
    fn remove_mcp_server(&self, name: &str, scope: Scope) -> Result<()>;
}
```

**Implementations:**
- **Claude Code** (`infra/provider/claude_code.rs`): Writes `.claude/mcp.json` with nested `mcpServers`.
- **OpenCode** (`infra/provider/opencode.rs`): Writes flat `mcp.<name>` to `opencode.json` with `type` ("local"/"remote") and `enabled` fields. Migrates away from stale `mcp.servers` key.

---

## Module Structure

```
src/domain/
  asset.rs           # AssetKind::McpServer
  mcp.rs             # McpServer, McpTransport, McpRegistry, McpActivation
  paths.rs           # mcp_path()
src/infra/
  mcp/mod.rs         # register(), enable(), disable(), test_server(), build_mcp_providers()
src/infra/provider/
  claude_code.rs     # McpProvider impl
  opencode.rs        # McpProvider impl with flat schema
src/tui/
  widgets/mcp/mod.rs       # McpState
  widgets/mcp/render.rs    # Table with checkbox + detail panel
  event.rs                 # handle_register_mcp_input(), handle_mcp_register_confirm(), handle_space_mcp()
  app.rs                   # McpState, pending_mcp_* fields
```

---

## TUI Integration

### Tab `[2]` MCP Servers
- Columns: Checkbox `[x]`/`[ ]` (enabled in current scope), Server name, Command, Transport, Tested `[✓]`/`[ ]`.
- Checkbox logic: `[x]` if any **active** provider has activation for the **active scope**.
- **F2:** Open registration modal (5-step + confirm).
- **Space:** Toggle enable/disable for all active MCP-capable providers in active scope.
- **Up/Down:** Navigate. Selection synced to `AppState.selected_index`.
- **Refresh:** `TriggerReload` refreshes `McpState` from `mcp.toml`.

### Keybinds
```
Tab [2]: [↑/↓] Move  [F2] Add MCP  [Space] Enable  [Enter] Test  [Esc]x2 Quit
```

---

## Testing Strategy

- **Unit tests:**
  - MCP registry save/load round-trip (`domain/mcp.rs` tests).
  - Provider config merge: add server without corrupting existing keys; remove server drops empty object chain.
  - OpenCode: `mcp.servers` migration drops stale key.
- **Integration:**
  - Register a test MCP server (mock handshake).
  - Activate/deactivate and verify provider config files exist with correct content.
  - TUI task events: `TaskStarted`, `TaskProgress`, `TriggerReload`, `TaskCompleted`.
- **Security:**
  - Confirm registration without warning is not possible in TUI flow.
  - `agk mcp add` via CLI does not bypass the test phase (unless `--no-test`).

---

*End of Technical Design.*
