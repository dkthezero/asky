# Interactive Modal Inputs — Technical Design

## Architecture
The change is **TUI-only** (render + minor keybind strings). No domain or application-layer logic is modified.

## New Widgets
Located in `src/tui/widgets/modal.rs`:

- `render_input_modal(frame, title, label, value)`
  - Renders a centered block with a title, a field label, and the current user input value.
  - Height is fixed at **7** (title + padding + label + value + padding). Width is `60%` of frame width clamped between `30` and `70`.
  - Uses `Clear` to wipe underlying UI, and `Wrap { trim: false }` so long values (e.g. GitHub URLs) are soft-wrapped instead of clipped.

- `render_confirm_modal(frame, title, message)`
  - Renders a centered block with a title and a message.
  - Height is estimated from wrapped line count (using popup inner width) plus 5. Same width heuristic as input modal.
  - Keybinds shown in the footer: `[y] Yes  [n] No  [Enter] Confirm  [Esc] Cancel`.

## Changes to `src/tui/render.rs`

After drawing the full layout (header, tabs, content, footer), inspect `state.list_mode` and dispatch:
- `ListMode::AttachVault`, `AttachVaultBranch`, `AttachVaultPath` → `render_input_modal`
- `ListMode::RegisterMcpStep{Name,Command,Args,Transport,Description}` → `render_input_modal`
- `ListMode::ConfirmMcpTest`, `ConfirmClawHubInstall`, `ConfirmDetachVault` → `render_confirm_modal`
- `ListMode::SelectProviderRoot` → existing `render_select_modal`

This keeps the popup rendering in a single post-layout pass, identical to how `SelectProviderRoot` already works.

## Footer / Keybind Adaptation
`render.rs` already branches footer keybinds for `SelectProviderRoot`. We extend that branch to cover all modal modes:
- **Input modals & select modal**: `[Enter] Confirm  [Esc] Cancel`
- **Confirmation modals**: `[y] Yes  [n] No  [Enter] Confirm  [Esc] Cancel`

Because the footer is rendered *before* the popup overlay, the user sees the correct keybind hint regardless of which layer is on top.

## Event Handling
`src/tui/event.rs` required only minimal changes:
- `handle_attach_vault_input` and `handle_register_mcp_input` already write into `state.prompt_buffer`; no logic changed.
- `handle_esc` and confirmation handlers already reset `list_mode` to `Normal`.
- Added Enter alias for confirmation modals (maps Enter to the same confirm action as y/Y).
- Restored empty-command validation error in `RegisterMcpStepCommand` so users get feedback when required fields are empty.
- Removed the now-unused `update_attach_status` and `update_register_mcp_status` helpers; the modal renders the value directly.

## State Model
No new fields on `AppState`. The existing `list_mode` enum already encodes which modal is open. `prompt_buffer` continues to be the shared input buffer.

## Testing Plan
- `cargo test` — unit tests in `event.rs` rely on `list_mode` transitions, not rendering, so they continue to pass.
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- Visual smoke-test via `cargo run` triggers:
  1. Vault tab → `F2` attach vault (3-step modal)
  2. MCP tab → `F2` add server (5-step modal)
  3. Vault tab → `Space` on attached vault (confirm modal)
