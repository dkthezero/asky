# Proposal: MCP Provider Expansion (P6)

**Status:** ✅ Research complete. 3 providers implementable; 3 unsupported.

## Goal
Expand MCP server support from 2 providers (Claude Code, OpenCode) to 5 providers by adding GitHub Copilot CLI, Gemini CLI, and AMP. Additionally, properly audit and document the 3 unsupported providers (Letta, Snowflake, Firebender).

## Research Summary

| Provider | MCP Support | Config Path | Schema | Notes |
|----------|-------------|-------------|--------|-------|
| Claude Code | ✅ Yes | `.claude/mcp.json` | `mcpServers: { name: { command, args, env } }` | Already implemented |
| OpenCode | ✅ Yes | `~/.config/opencode/opencode.json` | Flat `mcp.<name>: { type, enabled, command, args }` | Already implemented |
| **GitHub Copilot CLI** | ✅ **Yes** | `~/.copilot/mcp-config.json` | `mcpServers: { name: { type, command, args, env, tools } }` | CLI-specific; VS Code uses `.vscode/mcp.json` |
| **Gemini CLI** | ✅ **Yes** | `~/.gemini/settings.json` | `mcpServers: { name: { command, args, env, trust, includeTools } }` | Also supports `url` (SSE transport) |
| **AMP** | ✅ **Yes** | `.amp/settings.json` or `~/.config/amp/settings.json` | `amp.mcpServers` nested under settings | Standard schema: `command`/`args`/`env` for local, `url`/`headers` for remote |
| Letta | ❌ No | N/A | N/A | Proprietary `.skills/` directory system; no MCP client config documented |
| Snowflake Cortex | ❌ No (server-side) | N/A | `CREATE MCP SERVER ...` via SQL | Configured inside Snowflake platform; no local JSON file for end users |
| Firebender | ❌ Unknown | N/A | N/A | No discoverable MCP documentation or repo |

## Architecture

Each new provider follows the existing `McpProvider` trait pattern:

```rust
pub trait McpProvider: Send + Sync {
    fn provider_id(&self) -> &str;
    fn supports_mcp(&self) -> bool;
    fn mcp_config_path(&self, scope: Scope) -> Option<PathBuf>;
    fn write_mcp_server(&self, server: &McpServer, scope: Scope) -> Result<()>;
    fn remove_mcp_server(&self, name: &str, scope: Scope) -> Result<()>;
}
```

### GitHub Copilot CLI
- **Path:** `~/.copilot/mcp-config.json` (Global scope only — no workspace-level MCP config known)
- **Schema:** Top-level `mcpServers` object, same as Claude Code but with `type: "local"` and `tools: ["*"]`
- **Implementation:** Similar to `ClaudeCodeProvider::write_mcp_server` but targeting `~/.copilot/` directory.

### Gemini CLI
- **Path:** `~/.gemini/settings.json` (Global scope only)
- **Schema:** Top-level `mcpServers` object with `command`, `args`, `env`, `trust`, `includeTools`
- **Implementation:** Read → merge into `mcpServers` → write back.

### AMP
- **Path:** `.amp/settings.json` (Workspace) or `~/.config/amp/settings.json` (Global)
- **Schema:** Nested under `amp.mcpServers` inside a larger settings file.
- **Implementation:** Must preserve existing settings; only mutate the `amp.mcpServers` key.

### Unsupported Providers
For Letta, Snowflake, and Firebender: add `supports_mcp() -> false` and leave `write_mcp_server`/`remove_mcp_server` as `unimplemented!()` or `bail!()`. Do not write config files for platforms that don't support them.

## Files to Create/Modify

### New Tests
- `src/infra/provider/github.rs` — tests for `McpProvider` impl
- `src/infra/provider/gemini.rs` — tests for `McpProvider` impl
- `src/infra/provider/amp.rs` — tests for `McpProvider` impl

### Modified Files
- `src/infra/provider/github.rs` — add `McpProvider` impl
- `src/infra/provider/gemini.rs` — add `McpProvider` impl
- `src/infra/provider/amp.rs` — add `McpProvider` impl
- `src/infra/provider/letta.rs` — set `supports_mcp() -> false`
- `src/infra/provider/snowflake.rs` — set `supports_mcp() -> false`
- `src/infra/provider/firebender.rs` — set `supports_mcp() -> false`
- `src/infra/provider/mod.rs` — wire `build_mcp_providers()` to include new providers
- `src/app/bootstrap.rs` — add new providers to `build_providers()` and `build_mcp_providers()`
- `src/domain/telemetry.rs` — add log parser support for Gemini/AMP if applicable (future)

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-05-05 | GitHub Copilot targets CLI config (`~/.copilot/mcp-config.json`), not VS Code workspace | agk's provider model is per-user/per-tool, not per-workspace IDE. CLI config is the correct scope. |
| 2026-05-05 | Letta marked unsupported | No local MCP config file documented; proprietary `.skills/` system. Mark `supports_mcp: false`. |
| 2026-05-05 | Snowflake marked unsupported | MCP configured server-side via SQL; no local JSON to manage. |
| 2026-05-05 | Firebender marked unsupported | No discoverable MCP documentation. Revisit if format becomes known. |
| 2026-05-05 | Gemini and AMP support only Global scope for MCP | No documented workspace-level MCP config paths found. Can be expanded later. |

## Acceptance Criteria
- [ ] `agk mcp add` registers a server and writes it to Copilot CLI, Gemini CLI, and AMP configs.
- [ ] `agk mcp enable` toggles activation per provider per scope for all 5 MCP-capable providers.
- [ ] `agk mcp list` shows all registered servers with `[✓]` for tested and `[x]` for enabled across all active providers.
- [ ] Non-MCP-capable providers (Letta, Snowflake, Firebender) are excluded from MCP operations without errors.
- [ ] TUI Providers tab shows MCP checkbox `[✓]` only for providers that support MCP.
- [ ] All new provider configs preserve existing JSON content (no destructive overwrites).
- [ ] Tests cover write/read roundtrips for all 3 new providers.

---
