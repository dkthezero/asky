# PRD: MCP Vault Management

> **Product Mindset:** `agk` is the agent kit for teams to share the way they work with AI agents together. MCP (Model Context Protocol) is becoming the USB-C for AI tool integration. agk should act as the team MCP registry — centrally storing server definitions and then pushing the correct provider-specific config when enabled.

---

## Overview

MCP servers are runtime tools that AI agents invoke via the Model Context Protocol. This PRD introduces MCP servers as a new `AssetKind` in `agk`, managed through a dedicated TUI tab and headless CLI. The installation is two-phase:
1. **Registration:** Add an MCP server to agk's global registry, test it, and save its metadata.
2. **Activation:** Toggle the MCP server on/off for active providers in the current scope (Global or Workspace), which writes the provider-specific MCP configuration.

This PRD introduces a **TUI restructure**: Vaults is right-aligned as Tab `[0]`; MCP Servers becomes Tab `[2]`, between Skills `[1]` and Instructions `[3]`.

---

## Functional Requirements

### 1. MCP as a New AssetKind
- `AssetKind::McpServer` is added to the domain model.
- MCP servers are **not** sourced from external vaults (GitHub, ClawHub). They are registered directly into agk's global configuration, which acts as the **agk MCP Registry/Marketplace** (`~/.config/agk/mcp.toml`).

### 2. Tab Restructure
- **Tab `[1]` — Skills:** Unchanged.
- **Tab `[2]` — MCP Servers:** New. Lists all registered MCP servers. Shows checkbox `[x]`/`[ ]` for enabled status in current scope, plus Tested column.
- **Tab `[3]` — Instructions:** Unchanged.
- **Tab `[4]` — Providers:** Unchanged.
- **Tab `[5]` — Telemetry:** Unchanged.
- **Tab `[0]` — Vaults:** Right-aligned. Sources of skills/instructions.

### 3. Phase 1 — Registration
#### TUI Flow
1. User presses `F2` in Tab `[2]`.
2. A multi-step modal collects:
   - **Name** (required, unique in registry)
   - **Command** (required, e.g., `npx`, `python`, `docker`)
   - **Arguments** (optional, e.g., `@modelcontextprotocol/server-filesystem /path/to/dir`)
   - **Transport** (required: `stdio` or `sse`)
   - **Description** (optional)
3. Final step shows a **security warning**: `WARNING: This will execute '{command} {args}' on your machine. Register? [y/N]`. User must confirm.
4. `agk` registers the server, auto-runs the test handshake, and saves.
5. **If test succeeds:** Server appears in Tab `[2]` with `[✓]` in Tested column.
6. **If test fails:** Task fails with error in status bar.

#### Headless CLI Flow
```bash
agk mcp add --name fs --command npx \
  --args "@modelcontextprotocol/server-filesystem /Users/alice/docs" \
  --transport stdio
```
- `--test` flag (default: true) immediately runs the test after saving.
- `--no-test` skips the test (useful for CI where the runtime isn't available).

### 4. Phase 2 — Activation
#### TUI Flow
1. In Tab `[2]`, user navigates to a tested MCP server.
2. Presses `Space`.
3. If **any active provider** already has this server enabled in the **current scope**, it disables for all active providers.
4. If **none** are enabled, it enables for all active MCP-capable providers (e.g., Claude Code, OpenCode).
5. Checkbox `[x]`/`[ ]` updates to reflect scope-specific status.

#### Headless CLI Flow
```bash
agk mcp enable fs --provider claude-code --scope workspace
agk mcp disable fs --provider opencode --scope global
```

### 5. Provider-Specific MCP Config
Each provider uses a different schema. agk normalizes to the correct format:
- **Claude Code:** `.claude/mcp.json` (or `~/.claude/mcp.json`) with nested `mcpServers`.
- **OpenCode:** Flat `mcp.<name>` in `opencode.json` with `type` ("local"/"remote") and `enabled` fields.
- **GitHub Copilot:** TBD (if/when Copilot supports MCP).

### 6. Scope Behavior
- **Registration is always global.** The agk MCP registry is a global concept (stored in `~/.config/agk/mcp.toml`).
- **Activation is scoped.** Enabling an MCP server for a provider in Workspace scope writes to the workspace provider config. In Global scope, it writes to the global provider config.

---

## User Personas & Expected UX

### 👤 Human User

| Scenario | Expected UX |
|----------|-------------|
| Register a new MCP server | Tab `[2]` → `F2` → fill form → confirm warning → server appears with Tested `[✓]`. |
| Enable an MCP server for Claude | Tab `[2]` → select `fs` → `Space`. Claude Code is active in Workspace scope → `.claude/mcp.json` updated. Checkbox shows `[x]`. |
| Disable an MCP server | Tab `[2]` → select `fs` → `Space` again. MCP entry is removed from provider configs. Checkbox shows `[ ]`. |
| View all registered servers | Tab `[2]` lists all servers. Untested are dimmed. Tested but not enabled show `[ ]`. Enabled show `[x]`. |

### 🤖 AI Agent User

| Scenario | Expected UX |
|----------|-------------|
| Agent registers a tool | `agk mcp add --name browser --command docker --args "run mcp-browser" --transport stdio --json` returns the saved registry entry. |
| Agent enables a tool | `agk mcp enable browser --provider claude-code --scope workspace --json` returns `{"enabled": true, "provider": "claude-code", "config_written": ".claude/mcp.json"}` |
| Agent lists available tools | `agk mcp list --json` returns all registered servers with test/enable status. |

### 🏭 CI/CD User

| Scenario | Expected UX |
|----------|-------------|
| Standardize team MCP tools | `agk mcp add --name fs ... --no-test` in a team setup script (skip test if runtime missing in CI). |
| Activate in project | `agk mcp enable fs --provider claude-code --scope workspace --quiet` in a repo's setup script. |
| Verify configuration | `agk mcp list --json` is parsed by the pipeline to ensure required MCP servers are registered. |

---

## Non-Goals
- Hosting an MCP server. agk registers and configures them, but the provider process owns execution.
- MCP server discovery from remote registries. agk's registry is local and user-curated.
- Real-time MCP server health monitoring after activation. agk tests at registration time; ongoing health is the provider's responsibility.
- Cross-machine MCP registry sync. (Fast-follow: export/import registry as JSON.)

---

## Security Considerations
- [x] **Arbitrary code execution warning:** The TUI shows a warning before registering: `WARNING: This will execute '{command} {args}' on your machine. Register? [y/N]`. User must explicitly confirm.
- [ ] **File permissions:** `~/.config/agk/mcp.toml` should be created with `0600` permissions (not yet enforced).
- [x] **No sandboxing:** agk does not sandbox MCP processes. The test phase runs the command as the current user. Warning covers this.

---

## Acceptance Criteria
- [x] `AssetKind::McpServer` exists in the domain model.
- [x] TUI tab: Tab `[2]` = MCP Servers (between Skills and Instructions).
- [x] `F2` in Tab `[2]` opens the registration modal with security confirmation.
- [x] Registration collects name, command, args, transport, description.
- [x] Test phase performs an MCP `initialize` handshake automatically after registration.
- [x] Registry is stored in `~/.config/agk/mcp.toml`.
- [x] `Space` in Tab `[2]` toggles activation for active MCP-capable providers in current scope.
- [x] Provider-specific MCP config is written correctly:
  - Claude Code: nested `mcpServers` in `.claude/mcp.json`
  - OpenCode: flat `mcp.<name>` in `opencode.json` with `type` + `enabled`
- [x] Headless CLI: `agk mcp add`, `agk mcp enable`, `agk mcp disable`, `agk mcp list`, `agk mcp test`.
- [x] `--json` support for `agk mcp list`.
- [x] Security warning shown before executing unknown MCP commands.
- [ ] `mcp.toml` file permissions `0600` (pending hardening).
- [ ] TUI edit existing MCP server (Enter to modify) — workaround: disable + re-register.

---

*End of PRD.*
