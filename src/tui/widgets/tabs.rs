use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Tabs,
    Frame,
};

pub fn render(frame: &mut Frame, area: Rect, tab_names: &[String], active: usize) {
    let titles: Vec<String> = tab_names
        .iter()
        .enumerate()
        .map(|(i, name)| format!("[{}] {}", i + 1, name))
        .collect();

    let tabs = Tabs::new(titles)
        .select(active)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(tabs, area);
}
