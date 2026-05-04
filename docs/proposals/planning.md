# Proposal Implementation Plan — Product Owner Review

> **Status:** Reviewed and approved by Product Owner.  
> **Product North Star:** `agk` is the agent kit for teams to share the way they work with AI agents together. Every proposal must serve team collaboration, reproducible environments, and multi-provider consistency.

---

## 1. Executive Summary

| Rank | Proposal | Priority | Effort | Risk | Rationale (PO View) |
|------|----------|----------|--------|------|---------------------|
| 1 | **Headless CLI Operations** | **P1 — Must Have** | Medium | Low | The public API for CI/CD and AI agent automation. Without it, `agk` cannot be embedded in team onboarding scripts or agent toolchains. |
| 2 | **Skill Bundling & Meta-Skills** | **P2 — Should Have** | Medium | Medium | Enables tech leads to ship standardized "way of work" packs to juniors. Directly serves the team-sharing north star. |
| 3 | **OpenCode Provider Support** | **P3 — Should Have** | Small | Low | Another provider in the polyglot AI stack. File-based install (like Claude) makes it commodity adapter work. Good velocity win. |
| 4 | **Telemetry & Skill Usage Analytics** | **P4 — Could Have** | Large | High | Interesting for team leads, but provider log parsing is brittle. Defer until execution layer is stable and we have clear opt-in policy. |
| 5 | **MCP Vault Management** | **P5 — Strategic** | Large | Medium–High | High long-term value (MCP is becoming the USB-C for AI tools), but requires new AssetKind, tab restructure, and 2-phase install. Must follow P1 and needs formal design doc. |

> **Tab restructure note:** P5 introduces a breaking TUI change — Vaults moves to Tab `0`, MCP Servers becomes Tab `4`. This is accepted as a one-time reorganization before `agk` reaches v1.0.

---

## 2. Guiding Principles

1. **Team-first design.** Every feature must answer: "Does this make it easier for a team to standardize how they work with AI agents?"
2. **Three user personas, one interface.** Human (TUI), AI Agent (structured CLI), and CI/CD (deterministic, JSON, exit codes) must all be first-class citizens.
3. **Headless is the API.** The TUI is a discoverability layer; the CLI is the contract. P1 is not a nice-to-have — it is the foundation for all automation.
4. **Provider parity.** Adding a provider must be a 3–5 day task following the `ProviderPort` trait. If it takes longer, the trait is wrong.
5. **MCP is an AssetKind, not a Vault.** MCP servers are runtime tools that get registered in agk's global config (the agk MCP registry) and then enabled per-provider per-scope. They are not sourced from external vaults like GitHub repos.
6. **Documentation is a gate.** No proposal moves to engineering without both `prd.md` and `technical_design.md` under `docs/product/features/<feature-name>/`.

---

## 3. Persona Legend

| Icon | Persona | Context | Primary Interface |
|------|---------|---------|-------------------|
| 👤 | **Human** | Developer, tech lead, individual contributor | TUI + occasional CLI |
| 🤖 | **AI Agent** | Autonomous coding agent using `agk` as a tool | Headless CLI with `--json` |
| 🏭 | **CI/CD** | GitHub Actions, pre-commit hooks, team onboarding scripts | Headless CLI with exit codes |

---

## 4. Detailed Backlog

### P1 — Headless CLI Operations
**Source:** `docs/proposals/prd_headless_cli.md`  
**Status:** PRD refined. Ready for engineering.

**Product Rationale:** The TUI is great for discovery, but teams need reproducible, non-interactive workflows. A junior engineer's first day should run `agk sync` and get the exact same skill set as the tech lead.

**Key Persona Moments**
- 👤 `agk sync --dry-run` shows what would change before committing.
- 🤖 `agk install clawhub/web-browser --json` returns structured output an agent can parse.
- 🏭 `agk validate` runs in a Git pre-commit hook to ensure no broken skill configs are committed.

**Acceptance Criteria**
- [ ] `agk sync`, `agk install`, `agk validate`, `agk pack` subcommands exist.
- [ ] `--quiet`, `--verbose`, `--json` output flags work for all subcommands.
- [ ] Exit codes: `0` success, `1` general error, `2` validation error, `3` partial success.
- [ ] Headless mode never spawns a Ratatui buffer.
- [ ] All existing TUI flows continue to work (regression test).

**Estimated Effort:** 2–3 weeks

---

### P2 — Skill Bundling & Meta-Skills
**Source:** `docs/proposals/prd_skill_bundling.md`  
**Status:** PRD refined. Ready for engineering.

**Product Rationale:** The "Acme-Company-Pack" user story is the purest expression of agk's north star. One identifier, one command, and an entire team's standard AI workflow is deployed.

**Key Persona Moments**
- 👤 TUI shows a meta-skill as a single row with an expandable dependency tree in the detail pane.
- 🤖 `agk install acme-company-pack` recursively resolves and installs all deps.
- 🏭 A `SKILL.md` with `requires:` is validated in CI; circular dependencies fail the build.

**Acceptance Criteria**
- [ ] `SKILL.md` frontmatter parses `requires:` array of `[vault/]name` identifiers.
- [ ] Recursive installation uses the P1 pure async functions.
- [ ] Circular dependencies are detected and rejected with a clear error message.
- [ ] Diamond dependencies are deduplicated (same grandchild only installed once).
- [ ] TUI progress bar treats the pack as a macro-task with child increments.

**Estimated Effort:** 1–2 weeks

---

### P3 — OpenCode Provider Support
**Source:** `docs/proposals/prd_opencode_provider.md`  
**Status:** PRD newly authored. Ready for engineering.

**Product Rationale:** OpenCode (https://github.com/anomalyco/opencode) is a file-based, Claude-compatible agent runtime. Supporting it extends agk's polyglot coverage with minimal adapter complexity.

**Key Persona Moments**
- 👤 Tab 3 (Providers) shows "OpenCode" as a toggle; Space enables it.
- 🤖 `agk install opencode/react-component-builder` writes the skill to `.opencode/skills/`.
- 🏭 `agk sync` idempotently merges OpenCode config into `opencode.json` (never replaces, respecting OpenCode's merge semantics).

**Acceptance Criteria**
- [ ] `infra/provider/opencode.rs` implements `ProviderPort`.
- [ ] Global install path: `~/.config/opencode/skills/<name>/SKILL.md`.
- [ ] Workspace install path: `.opencode/skills/<name>/SKILL.md`.
- [ ] Config updates respect OpenCode's JSON/JSONC merge semantics (merge, don't replace).
- [ ] Registered in TUI Providers tab and headless CLI.

**Estimated Effort:** 3–5 days

---

### P4 — Telemetry & Skill Usage Analytics
**Source:** `docs/proposals/prd_telemetry.md`  
**Status:** PRD refined. Deferred pending P1 stability and privacy policy.

**Product Rationale:** Teams want to know which skills are actually being used so they can prune dead weight. But passive log parsing is fragile and privacy-sensitive. We ship this only with an explicit opt-in and graceful degradation.

**Key Persona Moments**
- 👤 Tab 5 ("Runs & Logs") shows a local-only dashboard: invocations count, last-used timestamp per skill.
- 🤖 Not a primary use case; agents don't need usage analytics.
- 🏭 Not applicable; telemetry is for human observability, not pipeline gates.

**Acceptance Criteria**
- [ ] Opt-in only; default is off.
- [ ] Passive local log tail for Claude, Copilot, and OpenCode (where applicable).
- [ ] Data stored only in `~/.config/agk/analytics.toml`; never transmitted.
- [ ] Missing log directories are silently skipped (no hard failures).
- [ ] Low-priority background tokio task; never blocks TUI render loop.

**Estimated Effort:** 3–4 weeks

---

### P5 — MCP Vault Management
**Source:** `docs/proposals/prd_mcp_vault.md`  
**Status:** PRD newly authored. Engineering design doc required before implementation.

**Product Rationale:** MCP (Model Context Protocol) is becoming the standard interface for AI tool integration. agk should act as the team MCP registry — centrally storing server definitions and then pushing the correct provider-specific config when enabled.

**Key Persona Moments**
- 👤 Tab 0 = Vaults, Tab 4 = MCP Servers. In Tab 4, press `F2` to register a new MCP server (name, command, args, env, transport). Test it. Save. It appears in the list. Press `Space` to enable it for active providers in the current scope.
- 🤖 `agk mcp add --name fs --command npx --args "@modelcontextprotocol/server-filesystem /path"` registers. `agk mcp enable fs --provider claude-code --scope workspace` activates.
- 🏭 `agk mcp add` and `agk mcp enable` in team onboarding scripts standardize the MCP toolchain across environments.

**Acceptance Criteria**
- [ ] MCP is a new `AssetKind`.
- [ ] Tab restructure: Tab `0` = Vaults, Tab `4` = MCP Servers.
- [ ] Phase 1 — Registration: Collect MCP metadata, test connection, save to agk global config (the agk MCP registry).
- [ ] Phase 2 — Activation: `[Space]` toggles enable per provider per scope; writes provider-specific MCP config.
- [ ] `agk mcp list`, `agk mcp add`, `agk mcp test`, `agk mcp enable`, `agk mcp disable` CLI subcommands.
- [ ] `--json` support for all MCP CLI commands.

**Estimated Effort:** 4–5 weeks (pending design doc)

---

## 5. Sequencing & Milestones

```
Month 1 ─┬─ Milestone 1: Foundation
         │   • P1 Headless CLI merged, tested, documented
         │   • P3 OpenCode Provider merged (parallel track; low risk)
         │   • `cargo test` passing; `cargo clippy -- -D warnings` clean
         │
Month 2 ─┬─ Milestone 2: Bundling & Ecosystem
         │   • P2 Skill Bundling merged
         │   • P3 OpenCode Provider stable in release
         │   • P5 MCP Vault design doc approved
         │
Month 3 ─┬─ Milestone 3: MCP & Observability
         │   • P5 MCP Vault merged (breaking tab restructure)
         │   • P4 Telemetry merged behind opt-in flag
         │
```

---

## 6. Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-05-03 | Headless CLI is P1 | It is the public API for all automation personas. All other proposals call installation logic. |
| 2026-05-03 | Skill Bundling is P2 | Directly enables the "team pack" user story — the core product value proposition. |
| 2026-05-03 | OpenCode is P3 | Small scoped adapter; expands polyglot coverage. Good velocity win between larger epics. |
| 2026-05-03 | Telemetry is P4 (deferred) | High maintenance risk (brittle log parsing). Needs explicit opt-in policy and P1 stability first. |
| 2026-05-03 | MCP is P5 (strategic) | High long-term value but requires new AssetKind, tab restructure, and 2-phase install. Must not block P1/P2. |
| 2026-05-03 | MCP is an AssetKind, not a Vault | MCP servers are runtime tools registered in agk's global registry, not sourced from external repos like GitHub or ClawHub. |
| 2026-05-03 | Tab restructure accepted | Tab `0` = Vaults, Tab `4` = MCP Servers. One-time breaking change before v1.0. |

---

## 7. Open Questions

1. **Telemetry privacy policy:** Should analytics be opt-in, opt-out, or always-on passive local-only? **→ PO decision required before P4 implementation.**
2. **`agk pack` MVP target:** Which provider format is the first pack target? **→ Suggest Claude Desktop zip (simplest serialization).**
3. **MCP provider config mapping:** Each provider (Claude, OpenCode, Copilot, etc.) uses different MCP config formats. Do we ship all at once or stagger? **→ Suggest Claude Desktop + OpenCode first; others fast-follow.**
4. **MCP security model:** Installing an MCP server can execute arbitrary code. Do we require explicit confirmation, sandbox warnings, or allowlisting? **→ PO + Security review required before P5 Phase 1.**

---

*End of plan.*
