use crate::domain::asset::{ProviderEntry, ScannedPackage, VaultEntry};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    package: Option<&ScannedPackage>,
    is_stub: bool,
    vault_entries: &[VaultEntry],
) {
    let block = Block::default().borders(Borders::ALL).title("Detail");

    if is_stub {
        frame.render_widget(
            Paragraph::new(Line::from("  [STUB] Not yet implemented")).block(block),
            area,
        );
        return;
    }

    let lines: Vec<Line> = match package {
        None => vec![Line::from("  No item selected")],
        Some(pkg) => {
            let label = |s: &str| Span::styled(s.to_string(), Style::default().fg(Color::Yellow));
            vec![
                Line::from(vec![
                    label("Name:     "),
                    Span::raw(pkg.identity.name.clone()),
                ]),
                Line::from(vec![
                    label("Kind:     "),
                    Span::raw(format!("{:?}", pkg.kind)),
                ]),
                Line::from(vec![
                    label("Vault:    "),
                    Span::raw(format!(
                        "{} ({})",
                        pkg.vault_id,
                        vault_entries
                            .iter()
                            .find(|v| v.id == pkg.vault_id)
                            .map(|v| v.kind.as_str())
                            .unwrap_or("unknown")
                    )),
                ]),
                Line::from(vec![
                    label("Path:     "),
                    Span::raw(pkg.path.display().to_string()),
                ]),
                Line::from(Span::raw("")),
                Line::from(vec![
                    label("Identity: "),
                    Span::raw(pkg.identity.to_string()),
                ]),
            ]
        }
    };

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

pub fn render_vault_detail(frame: &mut Frame, area: Rect, vault: Option<&VaultEntry>) {
    let block = Block::default().borders(Borders::ALL).title("Detail");
    let lines: Vec<Line> = match vault {
        None => vec![Line::from("  No vault selected")],
        Some(v) => {
            let label = |s: &str| Span::styled(s.to_string(), Style::default().fg(Color::Yellow));
            vec![
                Line::from(vec![label("Vault ID: "), Span::raw(v.id.clone())]),
                Line::from(vec![label("Type:     "), Span::raw(v.kind.clone())]),
                Line::from(vec![
                    label("Enabled:  "),
                    Span::raw(if v.enabled { "yes" } else { "no" }),
                ]),
                Line::from(Span::raw("")),
                Line::from(vec![
                    label("Skills:       "),
                    Span::raw(format!(
                        "{} installed / {} available",
                        v.installed_skills, v.available_skills
                    )),
                ]),
                Line::from(vec![
                    label("Instructions: "),
                    Span::raw(format!(
                        "{} installed / {} available",
                        v.installed_instructions, v.available_instructions
                    )),
                ]),
            ]
        }
    };
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

pub fn render_provider_detail(frame: &mut Frame, area: Rect, provider: Option<&ProviderEntry>) {
    let block = Block::default().borders(Borders::ALL).title("Detail");
    let lines: Vec<Line> = match provider {
        None => vec![Line::from("  No provider selected")],
        Some(p) => {
            let label = |s: &str| Span::styled(s.to_string(), Style::default().fg(Color::Yellow));
            vec![
                Line::from(vec![label("Provider:  "), Span::raw(p.name.clone())]),
                Line::from(vec![
                    label("Supported: "),
                    Span::raw("Agent Skills, Instructions"),
                ]),
            ]
        }
    };
    frame.render_widget(Paragraph::new(lines).block(block), area);
}
