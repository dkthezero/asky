# PRD: Vaults Management (Tab 4)

## Overview
Vaults serve as the canonical upstream source to index raw assets (Skills and Instructions) into `agk`'s dependency tracking map before assigning those assets into target Provider ecosystems. A Vault is loosely defined as an external dictionary mapped by a discrete path (Local filesystem) or remote repository protocol (GitHub) containing tools.

## Functional Requirements
- **Simultaneous Tracking:** The `agk` interface must be able to support multiple configured vaults simultaneously, caching their file shapes locally and indexing the tools concurrently.
- **Support Sources:** Connect native compatibility backends for:
  - Local Directories (filesystem paths)
  - GitHub Remote Repositories (handling sparse cloning patterns strictly over HTTPS defaults)
  - ClawHub Registry (clawhub.ai — community skill marketplace with CLI-delegated operations)
- **Automatic Auditing:** Whenever a vault is "refreshed", `agk` should scan the vault contents immediately to parse metadata formats, and actively recompute deterministic `sha10` integrity arrays for all assets implicitly discovered. 
- **Persistence:** Configuration and vault caches are stored in:
    - **macOS/Linux**: `~/.config/agk/`
    - **Windows**: `%APPDATA%\agk\` (typically `C:\Users\<User>\AppData\Roaming\agk\`)
- **Tab 4 UI Details:** The TUI's 4th Tab must visually present:
  - Active enabling/disabling states (whether the vault is configured into `config.toml`)
  - Supported statistics / volume of cached metadata (e.g. `14 Skills / 3 Instructions`)
- **Interactive UI Shortcuts:**
  - `F2` (Attach Flow): Open an interactive buffer explicitly pausing normal keyboard hooks to allow user path dictation or **GitHub URL** (e.g. `https://github.com/user/repo`). For GitHub URLs, the process follows a multi-step confirmation:
    1. **Primary URL**: Enter the GitHub repository address.
    2. **Branch Confirmation**: Confirm or replace the target branch (defaults to `main`).
    3. **Subfolder Path**: Confirm or replace the relative path to assets (defaults to `skills/`).
  - Subfolder isolation: GitHub vaults utilize `git sparse-checkout` to pull only the specified subfolder, improving performance and reducing disk usage.
  - Interactive feedback: After confirmation, `agk` immediately triggers an async `git clone` or `pull`, updating the UI with real-time progress markers.
  - `F4` (Refresh Focus): Triggers a global background task iterating uniformly through `registry.vaults` to synchronize external repositories, pushing byte-loaded progress messages through MPSC channels into the active visual buffer status bar.

## ClawHub Vault Integration
ClawHub (clawhub.ai) is a community skill marketplace providing a curated registry of agent skills. The integration follows a CLI-delegated architecture — all remote operations are performed by shelling out to the `clawhub` CLI binary.

### Behavior
- **Default Presence:** A `clawhub` vault entry appears in the Vaults tab by default (injected in memory, not persisted to disk) with an inactive/disabled state.
- **Activation Flow:** When a user toggles the ClawHub vault via `Space`:
  1. Check if the `clawhub` CLI is installed on `$PATH`.
  2. If missing, check for Homebrew availability and offer automated install (`brew install clawhub`).
  3. If Homebrew is unavailable, display a manual install message directing the user to clawhub.ai.
  4. On successful CLI detection or install, activate the vault and persist to `config.toml`.
- **Parallel Search:** When ClawHub is active and the user types a search query on the Skills tab, `agk` dispatches `clawhub search <query>` in a background thread alongside local filtering. Search progress appears in the bottom-right task progress area.
- **Remote Results Display:** Search results from ClawHub are shown in `DarkGray` style with metadata columns: slug (left-aligned), owner (left-aligned), downloads (right-aligned), stars (right-aligned), version (right-aligned), vault (left-aligned). All columns use computed max widths for vertical alignment across rows.
- **Install Flow:** Installing a remote skill runs two sequential background jobs:
  1. `clawhub install <slug>` — fetches the skill into agk's ClawHub cache directory (`~/.config/agk/clawhub/`).
  2. Copy from cache to the active scope target, registering the identity in `config.toml`.
- **Cache Scanning:** The ClawHub adapter delegates to `LocalVaultAdapter` to scan the cache directory, reusing existing local vault scanning logic.
- **Identity Format:** ClawHub packages use `owner/slug` as the identity name (e.g., `be1human/self-evolve`). Version uses semver from the registry when available, falling back to `sha10` hash for local-only packages.
