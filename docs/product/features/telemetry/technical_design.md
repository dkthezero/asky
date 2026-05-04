# Technical Design: Telemetry & Skill Usage Analytics

## Overview

A passive, local-only analytics system that scans provider log directories to infer skill invocations. All data stays on the machine; no network transmission.

## Architecture Rules

1. **Opt-in only.** Default is disabled. First visit to the analytics tab prompts for consent.
2. **Best-effort parsing.** If a log directory is missing or a format changes, silently skip. Never hard-fail.
3. **Background only.** The scanner runs in a low-priority tokio task. Never blocks the render loop.
4. **Privacy by design.** `analytics.toml` is local-only. No telemetry endpoints, no aggregation.

## Data Schemas

### AnalyticsConfig
```toml
# ~/.config/agk/analytics.toml
[settings]
enabled = true
last_scan = "2026-05-01T14:32:00Z"

[skills."web-browsing-tool"]
total_invocations = 42
last_used = "2026-05-01T14:32:00Z"
providers = ["claude-code"]

[skills."arxiv-researcher"]
total_invocations = 0
```

### SkillInvocation (Runtime)
```rust
#[derive(Debug, Clone)]
struct SkillInvocation {
    skill_name: String,
    provider_id: String,
    timestamp: DateTime<Utc>,
}
```

## Internal Workflows

### Log Parser Trait
```rust
trait LogParser: Send + Sync {
    fn provider_id(&self) -> &str;
    fn log_directories(&self) -> Vec<PathBuf>;
    fn parse_line(&self, line: &str) -> Option<SkillInvocation>;
}
```

### ClaudeCodeLogParser
- Directories: `~/Library/Logs/Claude/` (macOS), `~/.local/share/Claude/logs/` (Linux), `%APPDATA%/Claude/logs/` (Windows)
- Patterns: `"executed tool `{name}'"`, `"skill `{name}' invoked"`, `"running skill: {name}"`

### CopilotLogParser
- Directories: `~/Library/Logs/GitHub Copilot/`, `%APPDATA%/GitHub Copilot/logs/`
- Patterns: TBD based on actual Copilot log format research.

### Scanner Workflow
1. Check if analytics is enabled.
2. For each registered `LogParser`:
   a. Check if log directories exist.
   b. If not, skip silently.
   c. Read files, parse lines, collect invocations.
3. Aggregate into `AnalyticsConfig`.
4. Write to `~/.config/agk/analytics.toml`.
5. Sleep 60 seconds, repeat.

### TUI Tab 5 Workflow
1. On first visit: show consent prompt if `analytics.toml` doesn't exist.
2. Load `analytics.toml`.
3. Render sortable table: name, invocations, last used, providers.
4. Dim rows with 0 invocations in last 30 days.

## Module Structure

```
src/infra/telemetry/
  mod.rs            # Re-export
  parser.rs         # LogParser trait, ClaudeCodeLogParser, CopilotLogParser
  scanner.rs        # Background scanning loop
  store.rs          # AnalyticsConfig load/save
src/tui/
  widgets/
    analytics.rs    # Tab 5 rendering (NEW)
```

## Testing Strategy

- **Unit tests:**
  - Mock log lines → correct `SkillInvocation` extraction.
  - Missing directory → no panic.
  - Config round-trip (save/load).
- **Integration:**
  - `agk telemetry status --json` returns correct structure.

---

*End of Technical Design.*
