use crate::domain::asset::{AssetKind, ProviderEntry, ScannedPackage, VaultEntry};
use crate::domain::config::ConfigFile;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

fn truncate_name(name: &str, max_width: usize) -> String {
    if name.len() <= max_width {
        name.to_string()
    } else if max_width > 1 {
        format!("{}…", &name[..max_width - 1])
    } else {
        "…".to_string()
    }
}

fn scroll_name(name: &str, max_width: usize, offset: usize) -> String {
    if name.len() <= max_width {
        return name.to_string();
    }
    let padded = format!("{}   {}", name, name);
    let start = offset % (name.len() + 3);
    let end = (start + max_width).min(padded.len());
    padded[start..end].to_string()
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    packages: &[&ScannedPackage],
    selected: usize,
    is_stub: bool,
    config: &ConfigFile,
    scroll_offset: usize,
) {
    let block = Block::default().borders(Borders::ALL).title("Packages");

    if is_stub {
        let items = vec![ListItem::new(Line::from("  [STUB] Not yet implemented"))];
        frame.render_widget(List::new(items).block(block), area);
        return;
    }

    let inner_width = area.width.saturating_sub(2) as usize;

    let items: Vec<ListItem> = packages
        .iter()
        .enumerate()
        .map(|(idx, pkg)| {
            let version_str = pkg.identity.version.as_deref().unwrap_or("--");
            let vault_str = &pkg.vault_id;
            let status = install_status(
                config,
                vault_str,
                &pkg.identity.name,
                &pkg.kind,
                &pkg.identity.sha10,
            );
            let is_selected = idx == selected;

            let mut spans: Vec<Span> = vec![Span::raw(format!("{} ", status))];

            if pkg.is_remote {
                let meta_str = if let Some(meta) = &pkg.remote_meta {
                    format!(
                        "  {} \u{2193}{} \u{2605}{}",
                        meta.owner, meta.downloads, meta.stars
                    )
                } else {
                    String::new()
                };
                let fixed = 4 + version_str.len() + 2 + vault_str.len() + 2 + meta_str.len();
                let name_budget = inner_width.saturating_sub(fixed);
                let display_name = if is_selected {
                    scroll_name(&pkg.identity.name, name_budget, scroll_offset)
                } else {
                    truncate_name(&pkg.identity.name, name_budget)
                };
                let name_len = display_name.len();
                let remote_style = Style::default().fg(Color::DarkGray);
                spans.push(Span::styled(display_name, remote_style));
                if !meta_str.is_empty() {
                    spans.push(Span::styled(meta_str.clone(), remote_style));
                }
                let used =
                    4 + name_len + meta_str.len() + version_str.len() + 2 + vault_str.len() + 2;
                let pad = inner_width.saturating_sub(used);
                if pad > 0 {
                    spans.push(Span::raw(" ".repeat(pad)));
                }
                spans.push(Span::styled(format!("  {}", version_str), remote_style));
                spans.push(Span::styled(format!("  {}", vault_str), remote_style));
            } else {
                let fixed = 4 + version_str.len() + 2 + vault_str.len() + 2;
                let name_budget = inner_width.saturating_sub(fixed);
                let display_name = if is_selected {
                    scroll_name(&pkg.identity.name, name_budget, scroll_offset)
                } else {
                    truncate_name(&pkg.identity.name, name_budget)
                };
                let name_len = display_name.len();
                spans.push(Span::raw(display_name));
                let used = 4 + name_len + version_str.len() + 2 + vault_str.len() + 2;
                let pad = inner_width.saturating_sub(used);
                if pad > 0 {
                    spans.push(Span::raw(" ".repeat(pad)));
                }
                spans.push(Span::raw(format!("  {}", version_str)));
                spans.push(Span::raw(format!("  {}", vault_str)));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut state = ListState::default();
    if !packages.is_empty() {
        state.select(Some(selected));
    }

    frame.render_stateful_widget(list, area, &mut state);
}

pub fn render_vaults(frame: &mut Frame, area: Rect, vaults: &[VaultEntry], selected: usize) {
    let block = Block::default().borders(Borders::ALL).title("Vaults");
    if vaults.is_empty() {
        let items = vec![ListItem::new(Line::from(
            "  No vaults attached. Press 'a' to add one.",
        ))];
        frame.render_widget(List::new(items).block(block), area);
        return;
    }
    let items: Vec<ListItem> = vaults
        .iter()
        .map(|v| {
            let check = if v.enabled { "[x]" } else { "[ ]" };
            ListItem::new(Line::from(format!(
                "{} {:<20} {:<8} {}",
                check,
                v.id,
                v.kind,
                v.counts_label()
            )))
        })
        .collect();
    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");
    let mut state = ListState::default();
    if !vaults.is_empty() {
        state.select(Some(selected));
    }
    frame.render_stateful_widget(list, area, &mut state);
}

pub fn render_providers(
    frame: &mut Frame,
    area: Rect,
    providers: &[ProviderEntry],
    selected: usize,
) {
    let block = Block::default().borders(Borders::ALL).title("Providers");
    if providers.is_empty() {
        let items = vec![ListItem::new(Line::from("  No providers installed."))];
        frame.render_widget(List::new(items).block(block), area);
        return;
    }
    let items: Vec<ListItem> = providers
        .iter()
        .map(|p| {
            let checkbox = if p.active { "[x]" } else { "[ ]" };
            ListItem::new(Line::from(format!("{} {}", checkbox, p.name)))
        })
        .collect();
    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");
    let mut state = ListState::default();
    if !providers.is_empty() {
        state.select(Some(selected));
    }
    frame.render_stateful_widget(list, area, &mut state);
}

fn install_status(
    config: &ConfigFile,
    vault_id: &str,
    name: &str,
    kind: &AssetKind,
    current_hash: &str,
) -> &'static str {
    match kind {
        AssetKind::Skill => {
            if let Some(installed_hash) = config.installed_skill_hash(vault_id, name) {
                if installed_hash != current_hash {
                    "[!]"
                } else {
                    "[✓]"
                }
            } else {
                "[ ]"
            }
        }
        AssetKind::Instruction => {
            if let Some(installed_hash) = config.installed_instruction_hash(vault_id, name) {
                if installed_hash != current_hash {
                    "[!]"
                } else {
                    "[✓]"
                }
            } else {
                "[ ]"
            }
        }
    }
}
