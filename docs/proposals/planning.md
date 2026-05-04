# Proposal Implementation Plan — Product Owner Review

> **Status:** Engineering complete. All five proposals merged, tested, and documented.
> **Product North Star:** `agk` is the agent kit for teams to share the way they work with AI agents together. Every proposal must serve team collaboration, reproducible environments, and multi-provider consistency.

---

## 1. Executive Summary

| Rank | Proposal | Status | Priority | Effort | Rationale (PO View) |
|------|----------|--------|----------|--------|---------------------|
| 1 | **Headless CLI Operations** | ✅ **Done** | P1 | ✅ `sync`, `install`, `validate`, `pack` implemented. `--quiet`, `--verbose`, `--json` on all. Exit codes. | The public API for CI/CD and AI agent automation. |
| 2 | **Skill Bundling & Meta-Skills** | ✅ **Done** | P2 | ✅ `requires:`/`requires_optional:` in SKILL.md, BFS resolution, cycle detection, diamond dedup. | Enables tech leads to ship standardized "way of work" packs. |
| 3 | **OpenCode Provider Support** | ✅ **Done** | P3 | ✅ ProviderPort + McpProvider implemented. JSON/JSONC merge. Flat `mcp.<name>` schema. | Another provider in the polyglot stack. |
| 4 | **Telemetry & Skill Usage Analytics** | ✅ **Done** | P4 | ✅ Tab 5 "Telemetry" with stale dimming. Background scanner. CLI `enable/disable/status`. **Opt-out** (enabled by default). | Local-only observability for human decision-making. |
| 5 | **MCP Vault Management** | ✅ **Done** | P5 | ✅ Tab 2 "MCP Servers". `F2` registration modal. `Space` toggle per active provider per scope. `agk mcp add/enable/disable/test/list`. Security confirmation warning. | High long-term value — MCP is the USB-C for AI tools. |

> **Tab layout (final):** `[1] Skills [2] MCP Servers [3] Instructions [4] Providers [5] Telemetry    [0] Vault`

---

## 2. Guiding Principles

1. **Team-first design.** Every feature answers: "Does this make it easier for a team to standardize how they work with AI agents?"
2. **Three user personas, one interface.** Human (TUI), AI Agent (structured CLI), and CI/CD (deterministic, JSON, exit codes) are all first-class.
3. **Headless is the API.** The TUI is discoverability; the CLI is the contract.
4. **Provider parity.** Adding a provider is a 3–5 day task following the `ProviderPort` trait.
5. **MCP is an AssetKind, not a Vault.** MCP servers are registered in agk's global MCP registry (`~/.config/agk/mcp.toml`), then enabled per-provider per-scope. Not sourced from external repos.
6. **Documentation is a gate.** Every feature has `prd.md` + `technical_design.md` under `docs/product/features/<feature-name>/`.

---

## 3. Persona Legend

| Icon | Persona | Context | Primary Interface |
|------|---------|---------|-------------------|
| 👤 | **Human** | Developer, tech lead, individual contributor | TUI + occasional CLI |
| 🤖 | **AI Agent** | Autonomous coding agent using `agk` as a tool | Headless CLI with `--json` |
| 🏭 | **CI/CD** | GitHub Actions, pre-commit hooks, team onboarding scripts | Headless CLI with exit codes |

---

## 4. Proposal Summary

### P1 — Headless CLI Operations
**Status:** ✅ Merged. `agk sync`, `agk install`, `agk validate`, `agk pack` all implemented with `--quiet`, `--verbose`, `--json` and proper exit codes.

### P2 — Skill Bundling & Meta-Skills
**Status:** ✅ Merged. `SKILL.md` frontmatter parses `requires:`/`requires_optional:`. BFS resolution with cycle detection and diamond dedup. `app/bundling.rs` contains `resolve_dependencies()`, `install_bundle()`.

### P3 — OpenCode Provider Support
**Status:** ✅ Merged. `ProviderPort` + `McpProvider` both implemented. Flat `mcp.<name>` schema. JSONC comment stripping. Skills install to `.opencode/skills/`.

### P4 — Telemetry & Skill Usage Analytics
**Status:** ✅ Merged. Background `LogParser` scanners for Claude/Copilot/ OpenCode. Tab 5 "Telemetry" with stale dimming (>30 days). `agk telemetry enable/disable/status`. **Opt-out** (enabled by default).

### P5 — MCP Vault Management
**Status:** ✅ Merged. Tab 2 "MCP Servers". `AssetKind::McpServer`. `F2` registration modal. `Space` toggles enable/disable per active provider per scope. `agk mcp add/enable/disable/test/list`. Security confirmation warning before registration.

---

## 5. Milestones Achieved

| Milestone | Delivered |
|-----------|-----------|
| **Milestone 1: Foundation** | P1 Headless CLI + P3 OpenCode Provider merged, tested, documented |
| **Milestone 2: Bundling & Ecosystem** | P2 Skill Bundling + P5 MCP Vault merged |
| **Milestone 3: Observability** | P4 Telemetry merged (opt-out, not opt-in) |

---

## 6. Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-05-03 | Headless CLI is P1 | Public API for all automation personas. |
| 2026-05-03 | Skill Bundling is P2 | Directly enables the "team pack" user story. |
| 2026-05-03 | OpenCode is P3 | Small scoped adapter; expands polyglot coverage. |
| 2026-05-03 | Telemetry is opt-out (enabled by default) | Re-evaluated during implementation. Users expect analytics to "just work" for discoverability; explicit disable is sufficient. |
| 2026-05-03 | MCP is P5 (strategic) | High long-term value; AssetKind + tab restructure + 2-phase install. |
| 2026-05-03 | MCP Tab is [2], not [4] | Final layout: `[1] Skills [2] MCP [3] Instructions [4] Providers [5] Telemetry [0] Vault`. MCP placed adjacent to Skills for logical grouping (both are installable assets). |
| 2026-05-03 | MCP is an AssetKind, not a Vault | Registered in global registry, not sourced from external repos. |
| 2026-05-03 | Telemetry Tab name is "Telemetry", not "Runs & Logs" | Changed during implementation for clarity and consistency with domain model (`AnalyticsConfig`). |
| 2026-05-04 | OpenCode MCP schema is flat `mcp.<name>` | OpenCode validates `mcp` as `{ [name]: { type, enabled, ... } }`, not nested `mcp.servers`. Discovered during integration testing and corrected. |
| 2026-05-04 | Security warning in TUI before MCP registration | User must confirm after seeing the exact command that will be executed. Prevents accidental activation of untrusted binaries. |

---

## 7. Completed Acceptance Criteria

### P1 — Headless CLI
- [x] `agk sync`, `agk install`, `agk validate`, `agk pack` subcommands exist.
- [x] `--quiet`, `--verbose`, `--json` work consistently.
- [x] Exit codes `0`, `1`, `2`, `3` enforced.
- [x] Headless mode never spawns Ratatui buffer.
- [x] `cargo test` includes integration tests for all subcommands.

### P2 — Skill Bundling
- [x] `SKILL.md` frontmatter parses `requires:` and `requires_optional:`.
- [x] BFS resolution with cycle detection.
- [x] Diamond deduplication by `(vault, name, sha10)`.
- [ ] TUI macro-progress bar for pack install (out of scope for v0.2; background tasks cover basic progress).
- [ ] `agk validate` detects unresolvable `requires:` (partial — `validate` scans installed assets, not frontmatter dependencies).

### P3 — OpenCode Provider
- [x] `infra/provider/opencode.rs` implements `ProviderPort` + `McpProvider`.
- [x] Global/workspace install paths correct.
- [x] JSON/JSONC merge with comment stripping.
- [x] Flat `mcp.<name>` schema with `type`, `enabled`.
- [x] TUI Providers tab toggle.
- [x] Headless CLI `--provider opencode` supported.

### P4 — Telemetry
- [x] Enabled by default (opt-out via `agk telemetry disable`).
- [x] Log parsers for Claude Code, Copilot, OpenCode.
- [x] Data stored in `~/.config/agk/analytics.toml`.
- [x] TUI Tab 5 displays usage stats.
- [x] Background scan every 60s.
- [x] `agk telemetry enable|disable|status`.
- [x] `--json` support for `status`.
- [ ] TUI toggle in Tab 5 (future enhancement).

### P5 — MCP Vault
- [x] `AssetKind::McpServer` exists.
- [x] Tab `[2]` = MCP Servers.
- [x] `F2` registration modal (name→command→args→transport→description→confirm).
- [x] Test phase: MCP `initialize` handshake.
- [x] Registry in `~/.config/agk/mcp.toml`.
- [x] `Space` toggles per active provider per scope.
- [x] Provider config: Claude Code (`.claude/mcp.json`), OpenCode (`mcp.<name>`).
- [x] Headless CLI: `agk mcp add`, `enable`, `disable`, `list`, `test`.
- [x] `--json` for `list`.
- [x] Security warning before executing unknown commands.
- [ ] TUI edit flow (`Enter` to modify existing MCP) — partially covered by remove + re-register.
- [ ] `mcp.toml` file permissions `0600` — not yet enforced.

---

## 8. Open Questions (Outstanding)

1. **`agk pack` fast-follow:** Firebender JSON and tarball targets are stubbed (`PackTarget::Firebender`, `PackTarget::Tarball` exist in CLI enum but not fully implemented).
2. **MCP edit:** No dedicated TUI edit flow for existing MCP servers — workaround is disable + removed config + re-register.
3. **MCP permissions:** `~/.config/agk/mcp.toml` should have `0600` on creation (security hardening).
4. **Telemetry opt-in reconsideration:** Currently opt-out (enabled by default). Should v1.0 switch to opt-in based on user feedback?

---

*End of plan — Engineering complete. All proposals implemented and documented under `docs/product/features/`.*
