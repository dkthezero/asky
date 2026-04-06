# agk

A terminal-based manager for distributing AI agent skills and instructions across multiple providers.

Manage vaults of reusable skills and instructions, then install them to Claude Code, GitHub Copilot, Gemini, and other AI platforms — all from an interactive TUI.

![License](https://img.shields.io/github/license/dkthezero/agk)
![Crates.io](https://img.shields.io/crates/v/agk)
![GitHub release](https://img.shields.io/github/v/release/dkthezero/agk)

## Features

- **Multi-provider support** — Install to Claude Code, GitHub Copilot, Gemini, Letta, Snowflake, Firebender, and AMP
- **Local, GitHub & ClawHub vaults** — Source skills from local directories, GitHub repositories, or the [ClawHub](https://clawhub.ai) community marketplace
- **Interactive TUI** — Browse, search, install, and update assets with keyboard navigation
- **Change detection** — SHA-based hashing detects when vault assets have been updated
- **Scoped configuration** — Global settings for vaults, workspace-level settings for providers and installed assets
- **Batch operations** — Update all installed assets at once with F5

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

Launch the TUI:

```bash
agk
```

### Attach a vault

1. Press `4` to switch to the Vaults tab
2. Press `F2` to attach a new vault
3. Enter a local path (`./my-vault`) or GitHub URL (`owner/repo`)
4. Follow the prompts for branch and subfolder

### Browse & install from ClawHub

[ClawHub](https://clawhub.ai) is a community marketplace for agent skills. agk integrates with it out of the box:

1. Press `4` to switch to the Vaults tab — ClawHub appears as an inactive vault by default
2. Press `Space` on ClawHub to activate it (agk will help you install the `clawhub` CLI via Homebrew if needed)
3. Press `1` to switch to the Skills tab and start typing to search
4. agk searches your local vaults and ClawHub in parallel — remote results appear in gray with owner, downloads, and star counts
5. Press `Space` on a remote skill to install it

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

Skills and instructions support optional YAML frontmatter for metadata:

```markdown
---
name: my-skill
version: 1.0.0
---
# My Skill
...
```

## ClawHub integration

agk uses the [`clawhub` CLI](https://clawhub.ai) for all remote operations. When you activate the ClawHub vault, agk will:

1. Check if `clawhub` is on your `$PATH`
2. Offer to install it via Homebrew (`brew install clawhub`) if available
3. Display a manual install link if Homebrew is not available

Skills installed from ClawHub are cached in `~/.config/agk/clawhub/` and treated like any other vault source.

## Configuration

agk uses two configuration scopes:

| Scope | Path | Purpose |
|-------|------|---------|
| Global | `~/.config/agk/config.toml` | Vaults, enabled providers |
| Workspace | `.agk/config.toml` | Installed assets per workspace |

### Clean up

```bash
agk clean            # Remove workspace config
agk clean --global   # Remove global config
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

If you find agk useful, consider supporting its development:

[![Patreon](https://img.shields.io/badge/Patreon-Support-f96854?logo=patreon&logoColor=white)](https://www.patreon.com/dkthezero)

## License

[MIT](LICENSE)
