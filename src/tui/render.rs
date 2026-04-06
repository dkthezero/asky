use crate::tui::widgets::{detail, list, status, tabs};
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
    } else if state.clawhub_searching {
        format!("  [ Search: {} ] (searching ClawHub...)", state.search_query)
    } else {
        format!("  [ Search: {} ]", state.search_query)
    };
    let header_text = format!("agk v0.1.1{}", search_hint);
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
    }
    let keybinds = match active_kind {
        TabKind::Asset | TabKind::Provider => {
            "[↑/↓] Move  [Space] Toggle  [Enter] Update  [F5] Update All  [F4] Refresh  [type] Search  [Esc]x2 Quit"
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
