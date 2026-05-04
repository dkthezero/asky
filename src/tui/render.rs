use crate::tui::widgets::{analytics, detail, list, mcp, status, tabs};
use crate::tui::{app::AppState, layout};
use ratatui::{
    style::{Color, Modifier, Style},
    text::Line,
    widgets::Paragraph,
    Frame,
};

pub fn draw(frame: &mut Frame, state: &AppState) {
    let layout = layout::compute(frame.area());

    // Header
    let search_hint = if state.search_query.is_empty() {
        String::new()
    } else {
        format!("  [ Search: {} ]", state.search_query)
    };
    let header_text = format!("agk v0.1.2{}", search_hint);
    frame.render_widget(
        Paragraph::new(Line::from(header_text)).style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        layout.header,
    );

    // Tab bar
    tabs::render(frame, layout.tabs, &state.tab_names, state.active_tab);

    // Content — dispatch by tab kind
    use crate::tui::app::TabKind;

    let is_live = state.is_active_tab_live();
    let active_kind = state
        .tab_kinds
        .get(state.active_tab)
        .cloned()
        .unwrap_or(TabKind::Asset);

    match active_kind {
        TabKind::Asset => {
            let filtered = state.filtered_packages();
            let selected_pkg = filtered.get(state.selected_index).copied();
            list::render(
                frame,
                layout.list,
                &filtered,
                state.selected_index,
                !is_live,
                state.active_config(),
                state.scroll_offset,
            );
            detail::render(
                frame,
                layout.detail,
                selected_pkg,
                !is_live,
                &state.vault_entries,
            );
        }
        TabKind::Vault => {
            list::render_vaults(
                frame,
                layout.list,
                &state.vault_entries,
                state.selected_index,
            );
            let selected_vault = state.vault_entries.get(state.selected_index);
            detail::render_vault_detail(frame, layout.detail, selected_vault);
        }
        TabKind::Provider => {
            list::render_providers(
                frame,
                layout.list,
                &state.provider_entries,
                state.selected_index,
            );
            let selected_provider = state.provider_entries.get(state.selected_index);
            detail::render_provider_detail(frame, layout.detail, selected_provider);
        }
        TabKind::Mcp => {
            let list_area = layout.list;
            let detail_area = layout.detail;
            // Build active provider list for checkbox rendering
            let active_providers: Vec<crate::domain::asset::ProviderEntry> = state
                .provider_entries
                .iter()
                .filter(|p| p.active)
                .cloned()
                .collect();
            mcp::render::render(
                frame,
                list_area,
                &state.mcp_state,
                state.selected_index,
                state.active_scope,
                &active_providers,
            );
            mcp::render::render_detail(frame, detail_area, &state.mcp_state, state.selected_index);
        }
        TabKind::Analytics => {
            let list_area = layout.list;
            let detail_area = layout.detail;
            analytics::render(
                frame,
                list_area,
                &state.analytics_config,
                state.selected_index,
            );
            analytics::render_detail(
                frame,
                detail_area,
                &state.analytics_config,
                state.selected_index,
            );
        }
    }
    let keybinds = match active_kind {
        TabKind::Asset => {
            "[↑/↓] Move  [Space] Toggle  [Enter] Update  [F5] Update All  [F4] Refresh  [type] Search  [Esc]x2 Quit"
        }
        TabKind::Provider => {
            "[↑/↓] Move  [Space] Toggle  [Enter] Update  [F4] Refresh  [Esc]x2 Quit"
        }
        TabKind::Mcp => {
            "[↑/↓] Move  [F2] Add MCP  [Space] Enable  [Enter] Test  [Esc]x2 Quit"
        }
        TabKind::Analytics => {
            "[↑/↓] Move  [F5] Refresh  [Esc]x2 Quit"
        }
        TabKind::Vault => {
            "[↑/↓] Move  [F2] Attach New  [Space] Toggle  [F4] Refresh  [Esc]x2 Quit"
        }
    };

    status::render(
        frame,
        layout.footer,
        &state.status_line,
        &state.search_query,
        keybinds,
        state.scope_label(),
        state.progress_summary().as_deref(),
    );
}
