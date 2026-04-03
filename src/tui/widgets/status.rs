use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    status: &str,
    search: &str,
    keybinds: &str,
    scope_label: &str,
    progress: Option<&str>,
) {
    let status_text = if !status.is_empty() {
        status.to_string()
    } else if !search.is_empty() {
        format!("Search: {}", search)
    } else {
        String::new()
    };

    let line1 = Line::from(vec![
        Span::styled(scope_label, Style::default().fg(Color::Green)),
        Span::raw("  "),
        Span::styled(keybinds, Style::default().fg(Color::DarkGray)),
    ]);

    let row_layout = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
        ])
        .split(area);

    frame.render_widget(Paragraph::new(line1), row_layout[0]);

    if let Some(prog) = progress {
        let bottom_layout = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                ratatui::layout::Constraint::Min(0),
                ratatui::layout::Constraint::Length(prog.len() as u16 + 2),
            ])
            .split(row_layout[1]);

        frame.render_widget(Paragraph::new(status_text), bottom_layout[0]);
        frame.render_widget(
            Paragraph::new(prog)
                .alignment(ratatui::layout::Alignment::Right)
                .style(Style::default().fg(Color::Yellow)),
            bottom_layout[1],
        );
    } else {
        frame.render_widget(Paragraph::new(status_text), row_layout[1]);
    }
}
