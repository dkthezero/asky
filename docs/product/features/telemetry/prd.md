# PRD: Telemetry & Skill Usage Analytics

> **Product Mindset:** `agk` is the agent kit for teams to share the way they work with AI agents together. Telemetry helps teams prune dead weight and invest in high-value skills — but only if it is strictly local, opt-out-capable, and resilient.

---

## Overview

Users and team leads lack visibility into which installed skills are actually being leveraged by their AI providers. This PRD proposes a passive, local-only analytics layer that scans provider log directories to infer skill invocation patterns. The data never leaves the machine.

> **Privacy-first policy:** All analytics are stored locally in `~/.config/agk/analytics.toml`. No network transmission. No cloud aggregation. Enabled by default but can be disabled via `agk telemetry disable`.

---

## Functional Requirements

### Passive Log Tailing
- `agk` does not hook network requests or invade provider processes.
- It implements lightweight string-matching parsers for known provider log directories:
  - **Claude Code:** `~/Library/Logs/Claude/` (macOS), `%APPDATA%/Claude/logs/` (Windows), `~/.local/share/Claude/logs/` (Linux)
  - **GitHub Copilot:** `%APPDATA%/GitHub Copilot/logs/`, `~/Library/Logs/GitHub Copilot/`
  - **OpenCode:** `~/.config/opencode/logs/` (if applicable; TBD based on OpenCode log conventions)
- Parsers look for skill-name execution patterns (e.g., `"executed tool `web-browsing-tool'`", `"skill `react-parser' invoked"`).

### Data Structure
- `~/.config/agk/analytics.toml` stores:
  ```toml
  [analytics.skills."web-browsing-tool"]
  total_invocations = 42
  last_used = "2026-05-01T14:32:00Z"
  providers = ["claude-code"]
  ```

### TUI Dashboard (Tab 5 — "Telemetry")
- Displays a table with columns: Skill name, Total invocations, Last used timestamp, Providers.
- Stale skills (no invocations in last 30 days) are dimmed.
- Toggle to enable/disable analytics collection from the TUI not yet implemented — use CLI for now.
- **Tab spacing design:** At least 3 spaces between adjacent tab labels for readability.

### Background Processing
- Log scanning runs in a low-priority background `tokio` task.
- It wakes every 60 seconds or on explicit user request (`F5` in Telemetry tab).
- Never blocks the TUI render loop.

---

## User Personas & Expected UX

### 👤 Human User

| Scenario | Expected UX |
|----------|-------------|
| Team lead audits skill value | Opens Tab 5 (Telemetry). Sees `web-browsing-tool` with 150 invocations and `arxiv-researcher` with 0. Decides to remove the dead skill from the team pack. |
| User disables analytics | `agk telemetry disable` stops all scanning. `agk telemetry enable` resumes. |
| User checks analytics status | `agk telemetry status` shows whether collection is active and how many skills are tracked. `--json` returns structured output. |
| Missing log directory | If a provider log directory doesn't exist (e.g., Copilot not installed), the row for that provider shows "No logs found" in gray. No error modal. |

### 🤖 AI Agent User

| Scenario | Expected UX |
|----------|-------------|
| Agent checks skill popularity | Not a primary use case. Agents do not need usage analytics; they need installation and execution. |
| Agent queries telemetry status | `agk telemetry status --json` returns `{"enabled": true, "skills_tracked": 12, "last_scan": "2026-05-01T14:32:00Z"}` |

### 🏭 CI/CD User

| Scenario | Expected UX |
|----------|-------------|
| Not applicable | Telemetry is an observability feature for human decision-making, not a pipeline gate. CI/CD does not consume analytics data. |

---

## Non-Goals
- Network transmission of any data. This is strictly local.
- Real-time provider hooking or monkey-patching.
- Cross-machine aggregation or dashboards.
- Analytics for Instructions (only Skills, since Instructions are passive context, not invoked tools).
- Provider log parsing for providers that do not write structured logs (e.g., AMP, Firebender) unless their log format is formally documented.

---

## Acceptance Criteria
- [x] Enabled by default (opt-out via `agk telemetry disable`).
- [x] Passive log parsers for Claude Code and GitHub Copilot (minimum viable set).
- [x] Data stored only in `~/.config/agk/analytics.toml`.
- [x] TUI Tab 5 displays usage stats with sortable columns.
- [x] Background task scans logs every 60s without blocking the render loop.
- [x] Missing log directories are silently skipped (no panics, no modals).
- [x] `agk telemetry enable|disable|status` CLI subcommands.
- [x] `--json` support for `agk telemetry status`.
- [ ] Toggle to enable/disable from TUI Tab 5 (future enhancement).

---

*End of PRD.*
