use crate::domain::asset::{AssetKind, ProviderEntry, ScannedPackage, VaultEntry};
use crate::domain::config::ConfigFile;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    packages: &[&ScannedPackage],
    selected: usize,
    is_stub: bool,
    config: &ConfigFile,
) {
    let block = Block::default().borders(Borders::ALL).title("Packages");

    if is_stub {
        let items = vec![ListItem::new(Line::from("  [STUB] Not yet implemented"))];
        frame.render_widget(List::new(items).block(block), area);
        return;
    }

    let items: Vec<ListItem> = packages
        .iter()
        .map(|pkg| {
            let version = pkg.identity.version.as_deref().unwrap_or("--");
            let status = install_status(
                config,
                &pkg.vault_id,
                &pkg.identity.name,
                &pkg.kind,
                &pkg.identity.sha10,
            );
            ListItem::new(Line::from(format!(
                "{} {:<32} {:<8} {}",
                status, pkg.identity.name, version, pkg.vault_id
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
