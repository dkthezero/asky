# PRD: OpenCode Provider Support

> **Product Mindset:** `agk` is the agent kit for teams to share the way they work with AI agents together. OpenCode is a file-based, Claude-compatible agent runtime; supporting it extends agk's polyglot coverage with minimal adapter complexity.

---

## Overview

Add [OpenCode](https://github.com/anomalyco/opencode) as a first-class deployment target. OpenCode uses a hierarchical, merge-based configuration system and stores skills as markdown files with YAML frontmatter — making it structurally similar to Claude Code. This adapter follows the existing `ProviderPort` trait pattern and should be implementable in 3–5 days.

---

## Research Summary

**Configuration Storage:**
- Global config: `~/.config/opencode/opencode.json`
- Project config: `opencode.json` in project root
- Skills directory: `.opencode/skills/<name>/SKILL.md`
- Format: JSON / JSONC (JSON with Comments)
- Merge semantics: Configuration files are **merged** (not replaced), allowing layered settings.
- Claude Code compatibility: OpenCode supports Claude Code paths for migration.

**Skills Support:**
- Location: `.opencode/skills/<name>/SKILL.md`
- Format: Markdown with YAML frontmatter (fields: `name`, `description`)
- Search locations: project, global, and Claude-compatible paths

**Installation Mechanism:**
- File-based (not API-based).
- Skills loaded from filesystem directories.
- Supports `package.json` in `.opencode/` directory that triggers `bun install` for dependencies.

---

## Functional Requirements

### Provider Adapter
- Create `infra/provider/opencode.rs` implementing `ProviderPort`.
- **Global install path:** `~/.config/opencode/skills/<name>/SKILL.md`
- **Workspace install path:** `.opencode/skills/<name>/SKILL.md`

### Config Merge Semantics
- When updating `opencode.json`, **merge** the new skill reference into existing config.
- Never replace the entire file — respect OpenCode's layered configuration philosophy.
- Support both JSON and JSONC formats (preserve comments where possible, or at minimum don't corrupt them).

### Scope Targeting
- **Global scope:** Installs skills to `~/.config/opencode/skills/` and updates `~/.config/opencode/opencode.json`.
- **Workspace scope:** Installs skills to `.opencode/skills/` and updates `opencode.json` in the project root.

### TUI Integration
- Add "OpenCode" to the Providers tab (Tab 3).
- Space toggles active/inactive.
- Active marker shows when synchronizing.

### Headless CLI Integration
- `agk sync` installs to OpenCode if it is an active provider.
- `agk install opencode/<skill>` or `agk install <skill> --provider opencode` targets OpenCode explicitly.

---

## User Personas & Expected UX

### 👤 Human User

| Scenario | Expected UX |
|----------|-------------|
| Enable OpenCode as a target | Tab 3 (Providers) → navigate to "OpenCode" → press `Space`. A checkmark appears. |
| Install a skill to OpenCode | Tab 1 (Skills) → select skill → press `Space`. Skill is copied to `.opencode/skills/<name>/SKILL.md` (workspace) or `~/.config/opencode/skills/<name>/SKILL.md` (global). `opencode.json` is updated with the skill reference. |
| Verify install | OpenCode CLI can now use the skill natively because it scans the same directories. |

### 🤖 AI Agent User

| Scenario | Expected UX |
|----------|-------------|
| Agent targets OpenCode explicitly | `agk install clawhub/react-builder --provider opencode --json` returns `{"installed": true, "path": ".opencode/skills/react-builder/SKILL.md", "config_updated": "opencode.json"}` |
| Agent checks provider status | `agk sync --dry-run --json` shows OpenCode as an active target and lists which skills would be installed there. |

### 🏭 CI/CD User

| Scenario | Expected UX |
|----------|-------------|
| Standardize OpenCode skills in CI | `agk sync --global --quiet` ensures `~/.config/opencode/skills/` contains exactly the skills declared in `~/.config/agk/config.toml`. |
| Project-specific OpenCode setup | `agk sync` (workspace scope) in a repo ensures `.opencode/skills/` and `opencode.json` are correct. |
| Config merge safety | If `opencode.json` already contains unrelated settings (other plugins, custom commands), `agk` only adds/removes the skill array; all other keys are preserved. |

---

## Non-Goals
- OpenCode plugin management (`bun install` for `.opencode/package.json`). agk manages skills, not runtime dependencies.
- Migration from Claude Code to OpenCode. OpenCode already supports Claude-compatible paths; agk simply writes to the native OpenCode paths.
- Remote OpenCode API configuration. This adapter is file-system only.

---

## Acceptance Criteria
- [ ] `infra/provider/opencode.rs` implements `ProviderPort` (`id`, `name`, `install`, `remove`).
- [ ] Global path: `~/.config/opencode/skills/<name>/SKILL.md`.
- [ ] Workspace path: `.opencode/skills/<name>/SKILL.md`.
- [ ] `opencode.json` is merged (not replaced) when skills are added or removed.
- [ ] TUI Providers tab shows OpenCode with toggle support.
- [ ] Headless `agk sync` and `agk install` support `--provider opencode`.
- [ ] Unit tests for JSON/JSONC merge logic.
- [ ] `cargo test` and `cargo clippy -- -D warnings` pass.

---

*End of PRD.*
