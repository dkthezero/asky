use crate::domain::asset::ProviderEntry;
use crate::domain::mcp::McpTransport;
use crate::domain::scope::Scope;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use super::McpState;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &McpState,
    active_selected: usize,
    active_scope: Scope,
    active_providers: &[ProviderEntry],
) {
    let block = Block::default().borders(Borders::ALL).title("MCP Servers");

    let header = Row::new(vec![
        Cell::from(Span::raw("  ")).style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from(Span::raw("Server")).style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from(Span::raw("Command")).style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from(Span::raw("Transport")).style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from(Span::raw("Tested")).style(Style::default().add_modifier(Modifier::BOLD)),
    ]);

    let items = state.servers_list();
    let max_cmd_width = area.width as usize / 4;

    let mut rows: Vec<Row> = Vec::new();
    for (i, (id, server)) in items.iter().enumerate() {
        let is_selected = i == active_selected;
        let style = if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else if server.tested {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let cmd = truncate(&server.command, max_cmd_width);
        let transport = match &server.transport {
            McpTransport::Stdio => "stdio".to_string(),
            McpTransport::Sse { url } => format!("sse: {}", url),
        };
        let tested = if server.tested {
            "[✓]".to_string()
        } else {
            "[ ]".to_string()
        };
        // Checkbox: enabled if any ACTIVE provider has it enabled for this scope
        let enabled = active_providers.iter().any(|p| {
            server
                .activation
                .get(&p.id)
                .map(|a| match active_scope {
                    Scope::Global => a.global,
                    Scope::Workspace => a.workspace,
                })
                .unwrap_or(false)
        });
        let check = if enabled { "[x]" } else { "[ ]" };
        rows.push(
            Row::new(vec![
                Cell::from(Span::raw(check)).style(style),
                Cell::from(Span::raw(id.to_string())).style(style),
                Cell::from(Span::raw(cmd)).style(style),
                Cell::from(Span::raw(transport)).style(style),
                Cell::from(Span::raw(tested)).style(style),
            ])
            .style(style),
        );
    }

    let widths = [
        ratatui::layout::Constraint::Percentage(5),
        ratatui::layout::Constraint::Percentage(20),
        ratatui::layout::Constraint::Percentage(35),
        ratatui::layout::Constraint::Percentage(20),
        ratatui::layout::Constraint::Percentage(20),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    // Sync highlight to active_selected
    let mut table_state = ratatui::widgets::TableState::default();
    if !items.is_empty() {
        table_state.select(Some(active_selected));
    }
    frame.render_stateful_widget(table, area, &mut table_state);
}

pub fn render_detail(frame: &mut Frame, area: Rect, state: &McpState, active_selected: usize) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("MCP Server Detail");

    let items = state.servers_list();
    let Some((id, server)) = items.get(active_selected).copied() else {
        frame.render_widget(
            Paragraph::new("No server registered.\n\nUse `agk mcp add` to register a server.")
                .block(block),
            area,
        );
        return;
    };

    let transport = match &server.transport {
        McpTransport::Stdio => "stdio".to_string(),
        McpTransport::Sse { url } => format!("sse: {}", url),
    };

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(vec![
        Span::styled("ID: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(id.as_str()),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Command: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(server.command.as_str()),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Transport: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(transport.as_str()),
    ]));
    lines.push(Line::from(vec![
        Span::styled(
            "Description: ",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(server.description.as_deref().unwrap_or("—")),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Tested: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(if server.tested { "Yes" } else { "No" }),
    ]));
    if let Some(ref tested_at) = server.tested_at {
        lines.push(Line::from(vec![
            Span::styled("Tested at: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(tested_at.as_str()),
        ]));
    }

    let mut providers: Vec<&str> = server
        .activation
        .iter()
        .filter(|(_, a)| a.global || a.workspace)
        .map(|(id, _)| id.as_str())
        .collect();
    providers.sort();
    lines.push(Line::from(vec![
        Span::styled(
            "Active Providers: ",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(if providers.is_empty() { "none" } else { "—" }),
    ]));
    for p in providers {
        lines.push(Line::from(vec![Span::raw(format!("  • {}", p))]));
    }

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    if max == 0 {
        return String::new();
    }
    let mut end = max;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    if end == 0 {
        // max landed inside the first multi-byte char — count by chars instead
        return s.chars().take(max).collect();
    }
    format!("{}...", &s[..end])
}
