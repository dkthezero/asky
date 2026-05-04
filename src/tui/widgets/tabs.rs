use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Render tabs: [1]-[5] left-aligned for the first five tabs,
/// [0] right-aligned for the last tab (Vault).
/// Data order: [0]=Skills, [1]=MCP, [2]=Instructions, [3]=Providers, [4]=Runs&Logs, [5]=Vault.
pub fn render(frame: &mut Frame, area: Rect, tab_names: &[String], active: usize) {
    let left_names = &tab_names[..tab_names.len().saturating_sub(1)];
    let spacing = "   "; // three spaces between tabs

    // Build left spans with spacing inserted between them
    let mut left_spans: Vec<Span> = Vec::new();
    let mut left_width: u16 = 0;
    for (i, name) in left_names.iter().enumerate() {
        let label = format!("[{}] {}", i + 1, name);
        let span = if i == active {
            Span::styled(
                label.clone(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(label.clone(), Style::default().fg(Color::White))
        };
        left_spans.push(span);
        left_width += label.len() as u16;
        if i + 1 < left_names.len() {
            left_spans.push(Span::raw(spacing));
            left_width += spacing.len() as u16;
        }
    }

    // Compute right label and width
    let right_active = active == tab_names.len() - 1;
    let right_name = tab_names.last().map(|s| s.as_str()).unwrap_or("Vault");
    let right_label = format!("[0] {}", right_name);
    let right_width = right_label.len() as u16;

    let right_span = if right_active {
        Span::styled(
            right_label,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(right_label, Style::default().fg(Color::White))
    };

    let left_line = Line::from(left_spans);

    let left_area = Rect {
        width: left_width.min(area.width),
        ..area
    };
    let right_area = Rect {
        x: area.x + area.width.saturating_sub(right_width),
        width: right_width,
        ..area
    };

    frame.render_widget(
        Paragraph::new(left_line).alignment(Alignment::Left),
        left_area,
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![right_span])).alignment(Alignment::Right),
        right_area,
    );
}
