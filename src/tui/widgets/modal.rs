use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

/// Estimate how many display lines `text` will occupy when wrapped to `width`.
fn estimate_wrapped_lines(text: &str, width: u16) -> u16 {
    let w = width.max(1);
    text.lines()
        .map(|line| {
            let len = line.chars().count() as u16;
            len.div_ceil(w)
        })
        .sum::<u16>()
        .max(1)
}

/// Render a centered selection modal with a title and list of options.
pub fn render_select_modal(
    frame: &mut Frame,
    title: &str,
    options: &[(String, String)],
    selected: usize,
) {
    let area = frame.area();
    let width = (area.width as f32 * 0.6).clamp(30.0, 60.0) as u16;
    let height = (options.len() as u16 + 4).min(area.height.saturating_sub(4));
    let popup = centered_rect(width, height, area);

    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, (folder, desc))| {
            let style = if i == selected {
                Style::default()
                    .bg(Color::Cyan)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let text = format!("{} — {}", folder, desc);
            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}

/// Render a centered text-input modal with a title, field label, and current value.
pub fn render_input_modal(frame: &mut Frame, title: &str, label: &str, value: &str) {
    let area = frame.area();
    let width = (area.width as f32 * 0.6).clamp(30.0, 70.0) as u16;
    let height = 7u16.min(area.height.saturating_sub(4));
    let popup = centered_rect(width, height, area);

    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(vec![Span::styled(
        label,
        Style::default().fg(Color::White),
    )]));
    lines.push(Line::from(""));
    // Highlight the cursor area by rendering value in Cyan, mimicking an active input
    let value_span = if value.is_empty() {
        Span::styled("_", Style::default().fg(Color::DarkGray))
    } else {
        Span::styled(value, Style::default().fg(Color::Cyan))
    };
    lines.push(Line::from(vec![value_span]));

    let paragraph = Paragraph::new(Text::from(lines)).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);
}

/// Parse a keybinds string like "[Enter] Confirm  [Esc] Cancel" into colored spans.
/// Keys (inside `[]`) are shown in Cyan; everything else is DarkGray.
fn color_keys(input: &str) -> Vec<Span<'_>> {
    let mut spans = Vec::new();
    let mut current = String::new();
    let mut in_bracket = false;

    for ch in input.chars() {
        if ch == '[' && !in_bracket {
            if !current.is_empty() {
                spans.push(Span::styled(
                    current.clone(),
                    Style::default().fg(Color::DarkGray),
                ));
                current.clear();
            }
            in_bracket = true;
            current.push(ch);
        } else if ch == ']' && in_bracket {
            current.push(ch);
            spans.push(Span::styled(
                current.clone(),
                Style::default().fg(Color::Cyan),
            ));
            current.clear();
            in_bracket = false;
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        let color = if in_bracket {
            Color::Cyan
        } else {
            Color::DarkGray
        };
        spans.push(Span::styled(current, Style::default().fg(color)));
    }
    spans
}

/// Render a centered confirmation modal with a title, message, and action hints.
/// Actions are pinned to the bottom and keys inside `[]` are colored Cyan.
pub fn render_confirm_modal(frame: &mut Frame, title: &str, message: &str, actions: &str) {
    let area = frame.area();
    let width = (area.width as f32 * 0.6).clamp(30.0, 70.0) as u16;
    let inner_width = width.saturating_sub(4); // borders + margin
    let msg_lines = estimate_wrapped_lines(message, inner_width.max(1));
    let action_lines = estimate_wrapped_lines(actions, inner_width.max(1));
    let height = (msg_lines + action_lines + 5).min(area.height.saturating_sub(4));
    let popup = centered_rect(width, height, area);

    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    // Compute inner area before rendering the block so we can use it for layout
    let inner = block.inner(popup).inner(Margin::new(1, 1));

    frame.render_widget(block, popup);

    // Split vertically: message on top (flexible), actions at bottom
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(action_lines + 1)])
        .split(inner);

    let msg_paragraph = Paragraph::new(message)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    frame.render_widget(msg_paragraph, vertical[0]);

    let action_line = Line::from(color_keys(actions));
    let action_paragraph = Paragraph::new(action_line).alignment(Alignment::Left);
    frame.render_widget(action_paragraph, vertical[1]);
}

fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((r.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Length((r.height.saturating_sub(height)) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((r.width.saturating_sub(width)) / 2),
            Constraint::Length(width),
            Constraint::Length((r.width.saturating_sub(width)) / 2),
        ])
        .split(popup_layout[1])[1]
}
