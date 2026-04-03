# asky

A terminal-based manager for distributing AI agent skills and instructions across multiple providers.

Manage vaults of reusable skills and instructions, then install them to Claude Code, GitHub Copilot, Gemini, and other AI platforms — all from an interactive TUI.

![License](https://img.shields.io/github/license/dkthezero/asky)
![Crates.io](https://img.shields.io/crates/v/asky)
![GitHub release](https://img.shields.io/github/v/release/dkthezero/asky)

## Features

- **Multi-provider support** — Install to Claude Code, GitHub Copilot, Gemini, Letta, Snowflake, Firebender, and AMP
- **Local & GitHub vaults** — Source skills from local directories or any GitHub repository
- **Interactive TUI** — Browse, search, install, and update assets with keyboard navigation
- **Change detection** — SHA-based hashing detects when vault assets have been updated
- **Scoped configuration** — Global settings for vaults, workspace-level settings for providers and installed assets
- **Batch operations** — Update all installed assets at once with F5

## Installation

### Homebrew (macOS & Linux)

```bash
brew tap dkthezero/asky
brew install asky
```

### Cargo (from source)

```bash
cargo install asky
```

### From releases

Download pre-built binaries from the [Releases](https://github.com/dkthezero/asky/releases) page.

## Quick start

Launch the TUI:

```bash
asky
```

### Attach a vault

1. Press `4` to switch to the Vaults tab
2. Press `F2` to attach a new vault
3. Enter a local path (`./my-vault`) or GitHub URL (`owner/repo`)
4. Follow the prompts for branch and subfolder

### Install a skill

1. Press `1` to switch to the Skills tab
2. Navigate with arrow keys, press `Space` to install
3. The skill is copied to all active providers

### Configure providers

1. Press `3` to switch to the Providers tab
2. Press `Space` to toggle providers on/off

## Keybindings

| Key | Action |
|-----|--------|
| `1`-`4` | Switch tabs (Skills, Instructions, Providers, Vaults) |
| `Up/Down` | Navigate list |
| `Space` | Install/uninstall asset, toggle provider/vault |
| `Enter` | Update selected asset |
| `F2` | Attach new vault (Vaults tab) |
| `F4` | Refresh all vaults from source |
| `F5` | Update all installed assets |
| `Tab` | Toggle between Global and Workspace scope |
| `Type` | Search/filter by name |
| `Esc` | Clear search / cancel / quit |

## Vault structure

Asky expects vaults to follow this layout:

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

Skills and instructions support optional YAML frontmatter for metadata:

```markdown
---
name: my-skill
version: 1.0.0
---
# My Skill
...
```

## Configuration

Asky uses two configuration scopes:

| Scope | Path | Purpose |
|-------|------|---------|
| Global | `~/.config/asky/config.toml` | Vaults, enabled providers |
| Workspace | `.asky/config.toml` | Installed assets per workspace |

### Clean up

```bash
asky clean            # Remove workspace config
asky clean --global   # Remove global config
```

## Supported providers

| Provider | Global install path | Workspace install path |
|----------|-------------------|----------------------|
| [Claude Code](https://docs.anthropic.com/en/docs/claude-code/overview) | `~/.claude/` | `.claude/` |
| [GitHub Copilot](https://docs.github.com/en/copilot/how-tos/configure-custom-instructions) | `~/.copilot/` | `.github/` |
| [Gemini CLI](https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/gemini-md.md) | Provider-specific | Provider-specific |
| [Letta](https://docs.letta.com/introduction) | Provider-specific | Provider-specific |
| [Snowflake](https://docs.snowflake.com/en/user-guide/snowflake-cortex/cortex-agents) | Provider-specific | Provider-specific |
| [Firebender](https://docs.firebender.com/get-started/agent) | Provider-specific | Provider-specific |
| [AMP](https://ampcode.com/manual) | Provider-specific | Provider-specific |

## Development

```bash
# Build
cargo build

# Run
cargo run

# Test
cargo test
```

## Support

If you find asky useful, consider supporting its development:

[![Patreon](https://img.shields.io/badge/Patreon-Support-f96854?logo=patreon&logoColor=white)](https://www.patreon.com/dkthezero)

## License

[MIT](LICENSE)
