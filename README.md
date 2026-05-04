# agk

**The agent kit for teams to share the way they work with AI agents together.**

A terminal-based manager for distributing AI agent skills, instructions, and MCP tools across multiple providers and team members.

![License](https://img.shields.io/github/license/dkthezero/agk)
![Crates.io](https://img.shields.io/crates/v/agk)
![GitHub release](https://img.shields.io/github/v/release/dkthezero/agk)

## What's New in 0.2.0

agk 0.2.0 is a major feature release with five new capabilities:

- **🔌 MCP Vault Management** — Register, test, and enable MCP servers for Claude Code, OpenCode, and other providers
- **📊 Telemetry & Analytics** — Track which skills your team actually uses with local, privacy-first analytics
- **🤖 Headless CLI** — Automate skill distribution with `agk sync`, `install`, `validate`, and `pack`
- **📦 Skill Bundling** — Meta-skills with `requires:` in SKILL.md frontmatter install entire dependency trees
- **🎯 OpenCode Provider** — Full support for the file-based, Claude-compatible agent runtime

---

## Features

- **Multi-provider support** — Install to Claude Code, GitHub Copilot, Gemini, Letta, Snowflake, Firebender, AMP, and **OpenCode**
- **MCP Server Registry** — Centrally register MCP servers, test them with JSON-RPC handshake, and enable per-provider per-scope
- **Skill Bundling** — Write `requires:` in SKILL.md frontmatter to create meta-skills that auto-install dependency trees
- **Local, GitHub & ClawHub vaults** — Source skills from local directories, GitHub repositories, or the [ClawHub](https://clawhub.ai) community marketplace
- **Interactive TUI** — Browse, search, install, and update assets with keyboard navigation
- **Headless CLI** — All operations available as non-interactive commands with `--json`, `--quiet`, and deterministic exit codes
- **Telemetry (local-only)** — Track skill invocations across providers. Data stays on your machine; enabled by default, opt-out anytime
- **Change detection** — SHA-based hashing detects when vault assets have been updated
- **Scoped configuration** — Global settings for vaults, workspace-level settings for providers and installed assets

## Installation

### Homebrew (macOS & Linux)

```bash
brew tap dkthezero/agk
brew install agk
```

### Cargo (from source)

```bash
cargo install agk
```

### From releases (macOS, Linux, Windows)

Download pre-built binaries from the [Releases](https://github.com/dkthezero/agk/releases) page.

## Quick start

### Launch the TUI

```bash
agk
```

### Tab layout

```
[1] Skills    [2] MCP Servers    [3] Instructions    [4] Providers    [5] Telemetry    [0] Vaults
```

| Tab | What you see |
|-----|--------------|
| `[1] Skills` | Installed and available skills from all vaults |
| `[2] MCP Servers` | Registered MCP servers with `[x]`/`[ ]` enable status and `[✓]` test status |
| `[3] Instructions` | Installed and available agent instructions |
| `[4] Providers` | Claude Code, Copilot, Gemini, OpenCode, etc. Toggle with `Space` |
| `[5] Telemetry` | Skill usage stats — invocations, last used, stale-skill dimming |
| `[0] Vaults` | Attached vaults (Local, GitHub, ClawHub). Right-aligned for quick access |

### Attach a vault

1. Press `0` to switch to the Vaults tab
2. Press `F2` to attach a new vault
3. Enter a local path (`./my-vault`) or GitHub URL (`owner/repo`)
4. Follow the prompts for branch and subfolder

### Browse & install from ClawHub

[ClawHub](https://clawhub.ai) is a community marketplace for agent skills:

1. Press `0` to switch to the Vaults tab — ClawHub appears as an inactive vault by default
2. Press `Space` on ClawHub to activate it
3. Press `1` to switch to the Skills tab and start typing to search
4. agk searches your local vaults and ClawHub in parallel — remote results appear in gray with owner, downloads, and star counts
5. Press `Space` on a remote skill to install it

### Register an MCP server

1. Press `2` to switch to the MCP Servers tab
2. Press `F2` to register a new MCP server
3. Fill in: Name, Command, Arguments, Transport (`stdio` or `sse`), Description
4. Confirm the security warning: `This will execute 'npx @modelcontextprotocol/server-filesystem ...' on your machine`
5. agk auto-runs the MCP `initialize` handshake test. If it passes, the server appears in the list with `[✓]`

### Enable/disable MCP for providers

1. In the MCP Servers tab, navigate to a tested server
2. Press `Space` to toggle enable/disable for **all active MCP-capable providers** in the current scope
3. Checkbox `[x]` means enabled; `[ ]` means disabled
4. Claude Code writes `.claude/mcp.json`, OpenCode writes flat `mcp.<name>` in `opencode.json`

### Headless CLI

agk works in CI/CD and automation without a terminal:

```bash
# Sync all configured assets (team onboarding)
agk sync

# Install a specific skill
agk install clawhub/web-browser

# Validate installed assets against source vaults
agk validate

# Pack a skill for distribution
agk pack my-skill --target claude-desktop

# Register an MCP server
agk mcp add --name fs --command npx \
  --args "@modelcontextprotocol/server-filesystem /tmp" \
  --transport stdio

# Enable MCP for a provider
agk mcp enable fs --provider claude-code --scope workspace

# Check telemetry status
agk telemetry status
```

All commands support `--quiet`, `--verbose`, and `--json`.

### Create a meta-skill (skill pack)

Add `requires:` to your SKILL.md frontmatter:

```yaml
---
name: acme-company-pack
version: 1.0.0
description: "Acme's standard AI workflow"
requires:
  - clawhub/react-parser
  - clawhub/css-linter
  - internal-vault/component-generator
requires_optional:
  - clawhub/storybook-scaffold
---
```

When someone installs `acme-company-pack`, agk recursively resolves and installs all dependencies. Circular dependencies are detected and rejected. Diamond dependencies are deduplicated.

## Keybindings

| Key | Action |
|-----|--------|
| `0` | Vaults tab |
| `1`–`5` | Skills, MCP, Instructions, Providers, Telemetry |
| `Up/Down` | Navigate list |
| `Space` | Install/uninstall asset, toggle provider/vault/MCP |
| `Enter` | Update selected asset |
| `F2` | Add new vault (Vaults tab) or register MCP server (MCP tab) |
| `F4` | Refresh all vaults from source |
| `F5` | Update all installed assets |
| `Tab` | Toggle between Global and Workspace scope |
| `Type` | Search/filter by name (searches ClawHub in parallel when active) |
| `Esc` | Clear search / cancel / quit |

## Vault structure

agk expects vaults to follow this layout:

```
my-vault/
├── skills/
│   ├── my-skill/
│   │   ├── SKILL.md        # Required
│   │   └── ...              # Supporting files
│   └── another-skill/
│       └── SKILL.md
└── instructions/
    └── my-instruction/
        ├── AGENTS.md        # Required
        └── ...
```

Skills support optional YAML frontmatter for metadata and dependencies:

```markdown
---
name: my-skill
version: 1.0.0
requires:
  - clawhub/dep-skill
requires_optional:
  - clawhub/optional-skill
---
# My Skill
...
```

## Configuration

agk uses two configuration scopes:

| Scope | Path | Purpose |
|-------|------|---------|
| Global | `~/.config/agk/config.toml` | Vaults, enabled providers, telemetry settings |
| Workspace | `.agk/config.toml` | Installed assets per workspace |

MCP servers are stored in a separate global registry:

| File | Purpose |
|------|---------|
| `~/.config/agk/mcp.toml` | Registered MCP servers with activation state per provider |
| `~/.config/agk/analytics.toml` | Telemetry data (local-only, never transmitted) |

### Clean up

```bash
agk clean            # Remove workspace config
agk clean --global   # Remove global config
```

## Supported providers

| Provider | Skills | Instructions | MCP |
|----------|--------|-------------|-----|
| [Claude Code](https://docs.anthropic.com/en/docs/claude-code/overview) | `~/.claude/skills/` | `.claude/instructions/` | `.claude/mcp.json` |
| [OpenCode](https://github.com/anomalyco/opencode) | `~/.config/opencode/skills/` | (Claude-compatible paths) | Flat `mcp.<name>` in `opencode.json` |
| [GitHub Copilot](https://docs.github.com/en/copilot/how-tos/configure-custom-instructions) | `~/.copilot/` | `.github/` | TBD |
| [Gemini CLI](https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/gemini-md.md) | Provider-specific | Provider-specific | — |
| [Letta](https://docs.letta.com/introduction) | Provider-specific | Provider-specific | — |
| [Snowflake](https://docs.snowflake.com/en/user-guide/snowflake-cortex/cortex-agents) | Provider-specific | Provider-specific | — |
| [Firebender](https://docs.firebender.com/get-started/agent) | Provider-specific | Provider-specific | — |
| [AMP](https://ampcode.com/manual) | Provider-specific | Provider-specific | — |

## ClawHub integration

agk uses the [`clawhub` CLI](https://clawhub.ai) for all remote operations. When you activate the ClawHub vault, agk will:

1. Check if `clawhub` is on your `$PATH`
2. Offer to install it via Homebrew (`brew install clawhub`) if available
3. Display a manual install link if Homebrew is not available

Skills installed from ClawHub are cached in `~/.config/agk/clawhub/` and treated like any other vault source.

## Development

```bash
# Build
cargo build

# Run TUI
cargo run

# Test
cargo test

# Format (CI-enforced)
cargo fmt

# Lint
cargo clippy -- -D warnings
```

## Support

If you find agk useful, consider supporting its development:

[![Patreon](https://img.shields.io/badge/Patreon-Support-f96854?logo=patreon&logoColor=white)](https://www.patreon.com/dkthezero)

## License

[MIT](LICENSE)
