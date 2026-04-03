# PRD: Vaults Management (Tab 4)

## Overview
Vaults serve as the canonical upstream source to index raw assets (Skills and Instructions) into `asky`'s dependency tracking map before assigning those assets into target Provider ecosystems. A Vault is loosely defined as an external dictionary mapped by a discrete path (Local filesystem) or remote repository protocol (GitHub) containing tools.

## Functional Requirements
- **Simultaneous Tracking:** The `asky` interface must be able to support multiple configured vaults simultaneously, caching their file shapes locally and indexing the tools concurrently.
- **Support Sources:** Connect native compatibility backends for:
  - Local Directories (filesystem paths)
  - GitHub Remote Repositories (handling sparse cloning patterns strictly over HTTPS defaults)
- **Automatic Auditing:** Whenever a vault is "refreshed", `asky` should scan the vault contents immediately to parse metadata formats, and actively recompute deterministic `sha10` integrity arrays for all assets implicitly discovered. 
- **Persistence:** Configuration and vault caches are stored in:
    - **macOS/Linux**: `~/.config/asky/`
    - **Windows**: `%APPDATA%\asky\` (typically `C:\Users\<User>\AppData\Roaming\asky\`)
- **Tab 4 UI Details:** The TUI's 4th Tab must visually present:
  - Active enabling/disabling states (whether the vault is configured into `config.toml`)
  - Supported statistics / volume of cached metadata (e.g. `14 Skills / 3 Instructions`)
- **Interactive UI Shortcuts:**
  - `F2` (Attach Flow): Open an interactive buffer explicitly pausing normal keyboard hooks to allow user path dictation or **GitHub URL** (e.g. `https://github.com/user/repo`). For GitHub URLs, the process follows a multi-step confirmation:
    1. **Primary URL**: Enter the GitHub repository address.
    2. **Branch Confirmation**: Confirm or replace the target branch (defaults to `main`).
    3. **Subfolder Path**: Confirm or replace the relative path to assets (defaults to `skills/`).
  - Subfolder isolation: GitHub vaults utilize `git sparse-checkout` to pull only the specified subfolder, improving performance and reducing disk usage.
  - Interactive feedback: After confirmation, `asky` immediately triggers an async `git clone` or `pull`, updating the UI with real-time progress markers.
  - `F4` (Refresh Focus): Triggers a global background task iterating uniformly through `registry.vaults` to synchronize external repositories, pushing byte-loaded progress messages through MPSC channels into the active visual buffer status bar.
