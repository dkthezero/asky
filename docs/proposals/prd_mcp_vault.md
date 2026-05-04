# PRD: MCP Vault Management

> **Product Mindset:** `agk` is the agent kit for teams to share the way they work with AI agents together. MCP (Model Context Protocol) is becoming the USB-C for AI tool integration. agk should act as the team MCP registry — centrally storing server definitions and then pushing the correct provider-specific config when enabled.

---

## Overview

MCP servers are runtime tools that AI agents invoke via the Model Context Protocol. This PRD introduces MCP servers as a new `AssetKind` in `agk`, managed through a dedicated TUI tab and headless CLI. The installation is two-phase:
1. **Registration:** Add an MCP server to agk's global registry, test it, and save its metadata.
2. **Activation:** Toggle the MCP server on/off for specific providers in the current scope (Global or Workspace), which writes the provider-specific MCP configuration.

This PRD also introduces a **breaking TUI restructure**: Vaults moves from Tab `4` to Tab `0`; MCP Servers becomes Tab `4`.

---

## Functional Requirements

### 1. MCP as a New AssetKind
- `AssetKind::McpServer` is added to the domain model.
- MCP servers are **not** sourced from external vaults (GitHub, ClawHub). They are registered directly into agk's global configuration, which acts as the **agk MCP Registry/Marketplace**.

### 2. Tab Restructure
- **Tab `0` — Vaults:** Sources of skills/instructions (Local, GitHub, ClawHub). Renumbered from current Tab `4`.
- **Tab `1` — Skills:** Unchanged.
- **Tab `2` — Instructions:** Unchanged.
- **Tab `3` — Providers:** Unchanged.
- **Tab `4` — MCP Servers:** New. Lists all registered MCP servers. Shows status: registered → tested → enabled-for-provider(s).

### 3. Phase 1 — Registration
#### TUI Flow
1. User presses `F2` in Tab `4` (or `a` for "add").
2. A modal/dialog appears collecting:
   - **Name** (required, unique in registry)
   - **Command** (required, e.g., `npx`, `python`, `docker`)
   - **Arguments** (optional, e.g., `@modelcontextprotocol/server-filesystem /path/to/dir`)
   - **Environment Variables** (optional key-value pairs)
   - **Transport** (required: `stdio` or `sse`)
   - **Description** (optional)
3. User presses `Enter` or clicks a **Test** button.
4. `agk` attempts to start the MCP server process, send an `initialize` request, and verify a valid `initialize` response.
5. **If test succeeds:** Server is saved to the agk MCP registry. It appears in Tab `4` with a green "Tested" badge.
6. **If test fails:** Error is shown in the modal (stderr, timeout, or invalid MCP handshake). User can edit and retry.

#### Headless CLI Flow
```bash
agk mcp add --name fs --command npx \
  --args "@modelcontextprotocol/server-filesystem /Users/alice/docs" \
  --env "NODE_ENV=production" \
  --transport stdio
```
- `--test` flag (default: true) immediately runs the test after saving.
- `--no-test` skips the test (useful for CI where the runtime isn't available).

### 4. Phase 2 — Activation
#### TUI Flow
1. In Tab `4`, user navigates to a tested MCP server.
2. Presses `Space`.
3. agk checks which providers are **active** in the current scope (Global or Workspace).
4. For each active provider, agk writes the provider-specific MCP configuration:
   - **Claude Desktop:** `claude_desktop_config.json` (or `~/.config/claude/claude_desktop_config.json`)
   - **Claude Code:** `.claude/mcp.json` or `~/.claude/mcp.json`
   - **OpenCode:** `opencode.json` under the `mcp.servers` key
   - **GitHub Copilot:** TBD (if/when Copilot supports MCP)
5. The MCP server row updates to show a green dot next to each enabled provider.

#### Headless CLI Flow
```bash
agk mcp enable fs --provider claude-code --scope workspace
agk mcp disable fs --provider opencode --scope global
```

### 5. agk MCP Registry Storage
- Stored in `~/.config/agk/mcp.toml` (or a dedicated `[mcp]` section in `~/.config/agk/config.toml`):
  ```toml
  [mcp.servers.fs]
  name = "fs"
  command = "npx"
  args = ["@modelcontextprotocol/server-filesystem", "/Users/alice/docs"]
  env = { NODE_ENV = "production" }
  transport = "stdio"
  description = "Filesystem access for project docs"
  tested = true
  tested_at = "2026-05-03T10:00:00Z"
  ```

### 6. Scope Behavior
- **Registration is always global.** The agk MCP registry is a global concept (stored in `~/.config/agk/`).
- **Activation is scoped.** Enabling an MCP server for a provider in Workspace scope writes to the workspace provider config. In Global scope, it writes to the global provider config.

---

## User Personas & Expected UX

### 👤 Human User

| Scenario | Expected UX |
|----------|-------------|
| Register a new MCP server | Tab `4` → `F2` → fill form → `Test` → `Save`. Server appears in list with "Tested ✓". |
| Enable an MCP server for Claude | Tab `4` → select `fs` → `Space`. Since Claude Code is active in Workspace scope, `.claude/mcp.json` is updated. Row shows "Claude ✓". |
| Disable an MCP server | Tab `4` → select `fs` → `Space` again. MCP entry is removed from provider configs. Row shows no provider dots. |
| Fix a broken MCP server | Tab `4` → select `fs` → `Enter` (edit). Change the path in args → `Test` → `Save`. Re-enable if needed. |
| View all registered servers | Tab `4` lists all servers. Untested ones are yellow. Tested but not enabled are white. Enabled are green. |

### 🤖 AI Agent User

| Scenario | Expected UX |
|----------|-------------|
| Agent registers a tool | `agk mcp add --name browser --command docker --args "run mcp-browser" --transport stdio --json` returns the saved registry entry. |
| Agent enables a tool | `agk mcp enable browser --provider claude-code --scope workspace --json` returns `{"enabled": true, "provider": "claude-code", "config_written": ".claude/mcp.json"}` |
| Agent lists available tools | `agk mcp list --json` returns all registered servers with their test/enable status. |
| Agent checks scope | `agk mcp list --scope workspace --json` shows which servers are enabled for the current workspace. |

### 🏭 CI/CD User

| Scenario | Expected UX |
|----------|-------------|
| Standardize team MCP tools | `agk mcp add --name fs ... --no-test` in a team setup script (skip test if runtime missing in CI). |
| Activate in project | `agk mcp enable fs --provider claude-code --scope workspace --quiet` in a repo's setup script. |
| Verify configuration | `agk mcp list --json` is parsed by the pipeline to ensure required MCP servers are registered. |

---

## Non-Goals
- Hosting an MCP server. agk registers and configures them, but the provider process owns execution.
- MCP server discovery from remote registries. agk's registry is local and user-curated. (Fast-follow: import from a URL or npm package name.)
- Real-time MCP server health monitoring after activation. agk tests at registration time; ongoing health is the provider's responsibility.
- Cross-machine MCP registry sync. (Fast-follow: export/import registry as JSON.)

---

## Security Considerations
- **Arbitrary code execution:** An MCP server command can run any binary. The TUI must show a **warning banner** before testing/activating: "This will execute: `npx @modelcontextprotocol/server-filesystem ...`. Only activate MCP servers you trust."
- **Environment variables:** Secrets in `env` are stored in plain text in `~/.config/agk/mcp.toml`. Document that this file should have `0600` permissions.
- **No sandboxing:** agk does not sandbox MCP processes. The test phase runs the command as the current user. Warn accordingly.

---

## Acceptance Criteria
- [ ] `AssetKind::McpServer` exists in the domain model.
- [ ] TUI tab restructure: Tab `0` = Vaults, Tab `4` = MCP Servers.
- [ ] `F2` in Tab `4` opens the registration modal.
- [ ] Registration collects name, command, args, env, transport, description.
- [ ] Test phase performs an MCP `initialize` handshake.
- [ ] Registry is stored in `~/.config/agk/mcp.toml`.
- [ ] `Space` in Tab `4` toggles activation for active providers in current scope.
- [ ] Provider-specific MCP config is written correctly (Claude Desktop, Claude Code, OpenCode).
- [ ] Headless CLI: `agk mcp add`, `agk mcp enable`, `agk mcp disable`, `agk mcp list`, `agk mcp test`.
- [ ] `--json` support for all MCP CLI commands.
- [ ] Security warning shown before executing unknown MCP commands.

---

*End of PRD.*
