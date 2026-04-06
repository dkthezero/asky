use crate::tui::app::{AppState, ListMode};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};

pub enum ControlFlow {
    Continue,
    Quit,
}

use std::sync::Arc;

pub enum AppEvent {
    Input(crossterm::event::Event),
    TaskStarted {
        id: usize,
        name: String,
    },
    TaskProgress {
        id: usize,
        percent: u8,
    },
    TaskCompleted {
        id: usize,
        message: String,
    },
    TaskFailed {
        id: usize,
        error: String,
    },
    TriggerReload,
    VaultRefreshRequired {
        id: String,
        config: crate::domain::config::VaultConfig,
    },
    ClawHubSearchResults {
        packages: Vec<crate::domain::asset::ScannedPackage>,
    },
}

pub struct EventContext {
    pub store: Arc<dyn crate::app::ports::ConfigStorePort>,
    pub registry: Arc<crate::app::registry::Registry>,
    pub tx: tokio::sync::mpsc::UnboundedSender<AppEvent>,
    pub workspace_root: std::path::PathBuf,
}

fn is_clawhub_active(ctx: &EventContext) -> bool {
    ctx.store
        .load(crate::domain::scope::Scope::Global)
        .map(|c| c.vaults.contains(&"clawhub".to_string()))
        .unwrap_or(false)
}

fn dispatch_clawhub_search(state: &mut AppState, ctx: &EventContext) {
    state.clawhub_searching = true;
    let query = state.search_query.clone();
    let tx = ctx.tx.clone();
    tokio::task::spawn_blocking(move || {
        match crate::infra::vault::clawhub::cli_search(&query) {
            Ok(packages) => {
                let _ = tx.send(AppEvent::ClawHubSearchResults { packages });
            }
            Err(_) => {
                let _ = tx.send(AppEvent::ClawHubSearchResults {
                    packages: Vec::new(),
                });
            }
        }
    });
}

pub fn handle(
    state: &mut AppState,
    ctx: &EventContext,
    evt: crossterm::event::Event,
) -> Result<ControlFlow> {
    if let crossterm::event::Event::Key(key) = evt {
        // Ctrl+C always quits
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Ok(ControlFlow::Quit);
        }

        if key.code != KeyCode::Esc {
            state.esc_pressed_once = false;
        }

        match &key.code {
            KeyCode::Char('y') | KeyCode::Char('Y')
                if state.list_mode == ListMode::ConfirmDetachVault =>
            {
                return handle_detach_confirm(state, ctx);
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc
                if state.list_mode == ListMode::ConfirmDetachVault =>
            {
                return handle_detach_cancel(state);
            }
            KeyCode::Char(c @ '1'..='9') if state.list_mode == ListMode::Normal => {
                let idx = (*c as usize) - ('1' as usize);
                apply_tab_switch(state, idx, state.tab_names.len());
            }
            KeyCode::Up | KeyCode::Down => {
                handle_navigation(state, &key.code);
            }
            KeyCode::Char(_) | KeyCode::Backspace | KeyCode::Enter
                if state.is_attach_vault_mode() =>
            {
                handle_attach_vault_input(state, ctx, &key.code)?;
            }
            KeyCode::Esc => {
                return handle_esc(state);
            }
            KeyCode::Backspace => {
                handle_backspace(state);
            }
            KeyCode::Char(' ') if state.list_mode == ListMode::Normal => {
                handle_space(state, ctx)?;
            }
            KeyCode::Enter if state.list_mode == ListMode::Normal => {
                handle_enter(state, ctx)?;
            }
            KeyCode::F(5) | KeyCode::F(4) | KeyCode::F(2)
                if state.list_mode == ListMode::Normal =>
            {
                handle_f_keys(state, ctx, &key.code)?;
            }
            KeyCode::Tab if state.list_mode == ListMode::Normal => {
                apply_scope_toggle(state);
                let _ = ctx.tx.send(AppEvent::TriggerReload);
            }
            KeyCode::Char(c) => {
                let active_kind = state.tab_kinds.get(state.active_tab).copied();
                if active_kind != Some(crate::tui::app::TabKind::Vault) {
                    apply_search_char(state, *c);
                    if active_kind == Some(crate::tui::app::TabKind::Asset)
                        && is_clawhub_active(ctx)
                        && !state.search_query.is_empty()
                    {
                        dispatch_clawhub_search(state, ctx);
                    }
                }
            }
            _ => {}
        }
    }

    Ok(ControlFlow::Continue)
}

fn handle_detach_confirm(state: &mut AppState, ctx: &EventContext) -> Result<ControlFlow> {
    if let Some(vault_id) = state.pending_detach_vault.take() {
        let store = ctx.store.clone();
        let tx = ctx.tx.clone();
        let id = crate::tui::app::NEXT_TASK_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        tokio::task::spawn_blocking(move || {
            let _ = tx.send(AppEvent::TaskStarted {
                id,
                name: format!("Detaching vault '{}'", vault_id),
            });
            match crate::app::actions::detach_vault(&vault_id, store.as_ref()) {
                Ok(()) => {
                    let _ = tx.send(AppEvent::TaskProgress { id, percent: 100 });
                    let _ = tx.send(AppEvent::TriggerReload);
                    let _ = tx.send(AppEvent::TaskCompleted {
                        id,
                        message: format!("Detached vault '{}'", vault_id),
                    });
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::TaskFailed {
                        id,
                        error: format!("Detach failed: {}", e),
                    });
                }
            }
        });
    }
    state.list_mode = ListMode::Normal;
    state.status_line.clear();
    Ok(ControlFlow::Continue)
}

fn handle_detach_cancel(state: &mut AppState) -> Result<ControlFlow> {
    state.list_mode = ListMode::Normal;
    state.status_line = "Cancelled detach".to_string();
    state.pending_detach_vault = None;
    Ok(ControlFlow::Continue)
}

fn handle_navigation(state: &mut AppState, code: &KeyCode) {
    match code {
        KeyCode::Up => {
            if state.selected_index > 0 {
                state.selected_index -= 1;
            }
        }
        KeyCode::Down => {
            let count = state.list_length();
            if state.selected_index + 1 < count {
                state.selected_index += 1;
            }
        }
        _ => {}
    }
}

fn handle_attach_vault_input(
    state: &mut AppState,
    ctx: &EventContext,
    code: &KeyCode,
) -> Result<()> {
    match code {
        KeyCode::Char(c) => {
            state.prompt_buffer.push(*c);
            update_attach_status(state);
        }
        KeyCode::Backspace => {
            state.prompt_buffer.pop();
            update_attach_status(state);
        }
        KeyCode::Enter => match state.list_mode {
            ListMode::AttachVault => {
                let input = std::mem::take(&mut state.prompt_buffer);
                if input.is_empty() {
                    state.list_mode = ListMode::Normal;
                    state.status_line = "Cancelled \u{2014} empty path".to_string();
                } else if let Some((id, repo)) = parse_github_url(&input) {
                    state.pending_vault_id = id;
                    state.pending_vault_repo = repo;
                    state.pending_vault_url = input;
                    state.list_mode = ListMode::AttachVaultBranch;
                    state.prompt_buffer = "main".to_string();
                    update_attach_status(state);
                } else {
                    state.list_mode = ListMode::Normal;
                    let id = std::path::Path::new(&input)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned();
                    let vault_config = crate::domain::config::VaultConfig::Local(
                        crate::domain::config::LocalVaultSource { path: input },
                    );
                    execute_attach_vault(ctx, id, vault_config);
                }
            }
            ListMode::AttachVaultBranch => {
                let branch = std::mem::take(&mut state.prompt_buffer);
                state.pending_vault_ref = if branch.trim().is_empty() {
                    "main".to_string()
                } else {
                    branch
                };
                state.list_mode = ListMode::AttachVaultPath;
                state.prompt_buffer = "skills/".to_string();
                update_attach_status(state);
            }
            ListMode::AttachVaultPath => {
                let subfolder = std::mem::take(&mut state.prompt_buffer);
                state.pending_vault_path = if subfolder.trim().is_empty() {
                    "skills/".to_string()
                } else {
                    subfolder
                };
                state.list_mode = ListMode::Normal;

                let id = state.pending_vault_id.clone();
                let vault_config = crate::domain::config::VaultConfig::Github(
                    crate::domain::config::GithubVaultSource {
                        repo: state.pending_vault_repo.clone(),
                        r#ref: state.pending_vault_ref.clone(),
                        path: state.pending_vault_path.clone(),
                    },
                );
                execute_attach_vault(ctx, id, vault_config);
            }
            _ => {}
        },
        _ => {}
    }
    Ok(())
}

fn handle_esc(state: &mut AppState) -> Result<ControlFlow> {
    if state.list_mode == ListMode::AttachVault
        || state.list_mode == ListMode::AttachVaultBranch
        || state.list_mode == ListMode::AttachVaultPath
    {
        state.list_mode = ListMode::Normal;
        state.prompt_buffer.clear();
        state.status_line = "Cancelled".to_string();
        return Ok(ControlFlow::Continue);
    }

    let active_kind = state.tab_kinds.get(state.active_tab).copied();
    if state.list_mode == ListMode::Normal && state.search_query.is_empty() {
        if state.esc_pressed_once {
            return Ok(ControlFlow::Quit);
        } else {
            state.esc_pressed_once = true;
            state.status_line = "Press ESC again to quit".to_string();
        }
    } else if active_kind != Some(crate::tui::app::TabKind::Vault) {
        apply_esc(state);
    }
    Ok(ControlFlow::Continue)
}

fn handle_backspace(state: &mut AppState) {
    let active_kind = state.tab_kinds.get(state.active_tab).copied();
    if state.list_mode == ListMode::AttachVault
        || state.list_mode == ListMode::AttachVaultBranch
        || state.list_mode == ListMode::AttachVaultPath
    {
        state.prompt_buffer.pop();
        update_attach_status(state);
    } else if active_kind != Some(crate::tui::app::TabKind::Vault) {
        state.search_query.pop();
        if state.search_query.is_empty() {
            state.list_mode = ListMode::Normal;
            state.remote_packages.clear();
            state.clawhub_searching = false;
        }
        state.selected_index = 0;
    }
}

fn handle_space(state: &mut AppState, ctx: &EventContext) -> Result<()> {
    let active_kind = state.tab_kinds.get(state.active_tab).cloned();
    match active_kind {
        Some(crate::tui::app::TabKind::Provider) => handle_space_provider(state, ctx),
        Some(crate::tui::app::TabKind::Vault) => handle_space_vault(state, ctx),
        Some(crate::tui::app::TabKind::Asset) => {
            if !state.active_scope_has_provider() {
                let providers_idx = state
                    .tab_names
                    .iter()
                    .position(|n| n == "Providers")
                    .unwrap_or(2);
                apply_space_no_provider(state, providers_idx);
                Ok(())
            } else {
                handle_space_asset(state, ctx)
            }
        }
        _ => Ok(()),
    }
}

fn handle_space_provider(state: &mut AppState, ctx: &EventContext) -> Result<()> {
    if let Some(p) = state.provider_entries.get(state.selected_index) {
        let provider_id = p.id.clone();
        let scope = state.active_scope;
        let store = ctx.store.clone();
        let tx = ctx.tx.clone();
        let registry = ctx.registry.clone();

        let mut installed_pkgs = Vec::new();
        for tab_pkgs in state.packages.values() {
            for pkg in tab_pkgs {
                if state.is_installed(&pkg.vault_id, &pkg.identity.name, &pkg.kind) {
                    installed_pkgs.push(pkg.clone());
                }
            }
        }

        let id = crate::tui::app::NEXT_TASK_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        tokio::task::spawn_blocking(move || {
            let mut config = store.load(scope).unwrap_or_default();
            if config.providers.contains(&provider_id) {
                let _ = tx.send(AppEvent::TaskStarted {
                    id,
                    name: "Deactivating Provider".into(),
                });
                let total = installed_pkgs.len();
                if let Ok(provider) = registry.get_provider(&provider_id) {
                    config.providers.retain(|p| p != &provider_id);
                    let _ = store.save(scope, &config);

                    for (i, pkg) in installed_pkgs.iter().enumerate() {
                        let _ = provider.remove(&pkg.identity, &pkg.kind, scope);
                        let percent = (((i + 1) as f32 / total.max(1) as f32) * 100.0) as u8;
                        let _ = tx.send(AppEvent::TaskProgress { id, percent });
                    }
                }
                let _ = tx.send(AppEvent::TriggerReload);
                let _ = tx.send(AppEvent::TaskCompleted {
                    id,
                    message: format!("Deactivated '{}'", provider_id),
                });
            } else {
                let _ = tx.send(AppEvent::TaskStarted {
                    id,
                    name: "Activating Provider".into(),
                });
                let total = installed_pkgs.len();
                if let Ok(provider) = registry.get_provider(&provider_id) {
                    config.providers.push(provider_id.clone());
                    let _ = store.save(scope, &config);

                    for (i, pkg) in installed_pkgs.iter().enumerate() {
                        let _ = crate::app::actions::install_asset(
                            scope,
                            pkg,
                            store.as_ref(),
                            provider,
                        );
                        let percent = (((i + 1) as f32 / total.max(1) as f32) * 100.0) as u8;
                        let _ = tx.send(AppEvent::TaskProgress { id, percent });
                    }
                }
                let _ = tx.send(AppEvent::TriggerReload);
                let _ = tx.send(AppEvent::TaskCompleted {
                    id,
                    message: format!("Activated '{}'", provider_id),
                });
            }
        });
    }
    Ok(())
}

fn handle_space_vault(state: &mut AppState, ctx: &EventContext) -> Result<()> {
    if let Some(vault) = state.vault_entries.get(state.selected_index) {
        let vault_id = vault.id.clone();

        let is_attached = if let Ok(config) = ctx.store.load(crate::domain::scope::Scope::Global) {
            config.vaults.contains(&vault_id)
        } else {
            false
        };

        if is_attached {
            state.list_mode = ListMode::ConfirmDetachVault;
            state.pending_detach_vault = Some(vault_id.clone());
            state.status_line = format!(
                "Detach vault '{}'? This will hide all its uninstalled skills. [y/N]",
                vault_id
            );
        } else {
            let store = ctx.store.clone();
            let tx = ctx.tx.clone();
            let id =
                crate::tui::app::NEXT_TASK_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            tokio::task::spawn_blocking(move || {
                let _ = tx.send(AppEvent::TaskStarted {
                    id,
                    name: format!("Attaching vault '{}'", vault_id),
                });
                if let Ok(mut config) = store.load(crate::domain::scope::Scope::Global) {
                    config.vaults.push(vault_id.clone());
                    let _ = store.save(crate::domain::scope::Scope::Global, &config);
                }
                let _ = tx.send(AppEvent::TaskProgress { id, percent: 100 });
                let _ = tx.send(AppEvent::TriggerReload);
                let _ = tx.send(AppEvent::TaskCompleted {
                    id,
                    message: format!("Attached vault '{}'", vault_id),
                });
            });
        }
    }
    Ok(())
}

fn handle_space_asset(state: &mut AppState, ctx: &EventContext) -> Result<()> {
    let pkg_opt = {
        let filtered = state.filtered_packages();
        filtered.get(state.selected_index).copied().cloned()
    };
    if let Some(pkg) = pkg_opt {
        if pkg.is_remote {
            return handle_install_remote_clawhub(state, ctx, &pkg);
        }
        let is_installed = state.is_installed(&pkg.vault_id, &pkg.identity.name, &pkg.kind);
        let store = ctx.store.clone();
        let active_scope = state.active_scope;
        let tx = ctx.tx.clone();
        let registry = ctx.registry.clone();

        let id = crate::tui::app::NEXT_TASK_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        tokio::task::spawn_blocking(move || {
            let action = if is_installed {
                "Uninstalling"
            } else {
                "Installing"
            };
            let _ = tx.send(AppEvent::TaskStarted {
                id,
                name: format!("{} '{}'", action, pkg.identity.name),
            });

            let config = store.load(active_scope).unwrap_or_default();
            let providers = active_providers(&registry, &config);

            if providers.is_empty() {
                let _ = tx.send(AppEvent::TaskFailed {
                    id,
                    error: "No active providers to install to".into(),
                });
                return;
            }

            let mut success = true;
            for provider in providers {
                if is_installed {
                    if crate::app::actions::remove_asset(
                        active_scope,
                        &pkg.identity,
                        &pkg.kind,
                        &pkg.vault_id,
                        store.as_ref(),
                        provider,
                    )
                    .is_err()
                    {
                        success = false;
                    }
                } else if crate::app::actions::install_asset(
                    active_scope,
                    &pkg,
                    store.as_ref(),
                    provider,
                )
                .is_err()
                {
                    success = false;
                }
            }
            let _ = tx.send(AppEvent::TaskProgress { id, percent: 100 });
            let _ = tx.send(AppEvent::TriggerReload);
            if success {
                let done = if is_installed {
                    "Uninstalled"
                } else {
                    "Installed"
                };
                let _ = tx.send(AppEvent::TaskCompleted {
                    id,
                    message: format!("{} '{}'", done, pkg.identity.name),
                });
            } else {
                let _ = tx.send(AppEvent::TaskFailed {
                    id,
                    error: format!(
                        "Failed to {} '{}'",
                        action.to_lowercase(),
                        pkg.identity.name
                    ),
                });
            }
        });
    }
    Ok(())
}

fn handle_install_remote_clawhub(
    state: &mut AppState,
    ctx: &EventContext,
    pkg: &crate::domain::asset::ScannedPackage,
) -> Result<()> {
    let slug = pkg.identity.name.clone();
    let store = ctx.store.clone();
    let tx = ctx.tx.clone();
    let registry = ctx.registry.clone();
    let active_scope = state.active_scope;

    let fetch_id = crate::tui::app::NEXT_TASK_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let install_id =
        crate::tui::app::NEXT_TASK_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    let _ = tx.send(AppEvent::TaskStarted {
        id: fetch_id,
        name: format!("Fetching '{}' from ClawHub", slug),
    });
    let _ = tx.send(AppEvent::TaskStarted {
        id: install_id,
        name: format!("Installing '{}' to {:?}", slug, active_scope),
    });

    tokio::task::spawn_blocking(move || {
        match crate::infra::vault::clawhub::cli_install(&slug) {
            Ok(()) => {
                let _ = tx.send(AppEvent::TaskProgress {
                    id: fetch_id,
                    percent: 100,
                });
                let _ = tx.send(AppEvent::TaskCompleted {
                    id: fetch_id,
                    message: format!("Fetched '{}' from ClawHub", slug),
                });
            }
            Err(e) => {
                let _ = tx.send(AppEvent::TaskFailed {
                    id: fetch_id,
                    error: format!("Failed to fetch '{}': {}", slug, e),
                });
                let _ = tx.send(AppEvent::TaskFailed {
                    id: install_id,
                    error: "Cancelled — fetch failed".into(),
                });
                return;
            }
        }

        let cache_dir = crate::domain::paths::clawhub_cache_dir();
        let local = crate::infra::vault::local::LocalVaultAdapter::new("clawhub", cache_dir);
        let feature = crate::infra::feature::skill::SkillFeatureSet;
        use crate::app::ports::VaultPort;
        let cached_pkgs = match local.list_packages(&feature) {
            Ok(pkgs) => pkgs,
            Err(e) => {
                let _ = tx.send(AppEvent::TaskFailed {
                    id: install_id,
                    error: format!("Failed to scan cached package: {}", e),
                });
                return;
            }
        };

        let cached_pkg = cached_pkgs.iter().find(|p| p.identity.name == slug);
        if let Some(pkg) = cached_pkg {
            let config = store.load(active_scope).unwrap_or_default();
            let providers = active_providers(&registry, &config);
            if providers.is_empty() {
                let _ = tx.send(AppEvent::TaskFailed {
                    id: install_id,
                    error: "No active providers to install to".into(),
                });
                return;
            }
            let mut success = true;
            for provider in providers {
                if crate::app::actions::install_asset(active_scope, pkg, store.as_ref(), provider)
                    .is_err()
                {
                    success = false;
                }
            }
            let _ = tx.send(AppEvent::TaskProgress {
                id: install_id,
                percent: 100,
            });
            let _ = tx.send(AppEvent::TriggerReload);
            if success {
                let _ = tx.send(AppEvent::TaskCompleted {
                    id: install_id,
                    message: format!("Installed '{}' to {:?}", slug, active_scope),
                });
            } else {
                let _ = tx.send(AppEvent::TaskFailed {
                    id: install_id,
                    error: format!("Failed to install '{}'", slug),
                });
            }
        } else {
            let _ = tx.send(AppEvent::TaskFailed {
                id: install_id,
                error: format!("Skill '{}' not found in ClawHub cache after fetch", slug),
            });
        }
    });
    Ok(())
}

fn handle_enter(state: &mut AppState, ctx: &EventContext) -> Result<()> {
    let active_kind = state
        .tab_kinds
        .get(state.active_tab)
        .cloned()
        .unwrap_or(crate::tui::app::TabKind::Asset);
    if active_kind != crate::tui::app::TabKind::Asset {
        state.status_line = "Update only applies to Skills/Instructions tabs".to_string();
    } else if !state.active_scope_has_provider() {
        let providers_idx = state
            .tab_names
            .iter()
            .position(|n| n == "Providers")
            .unwrap_or(2);
        apply_space_no_provider(state, providers_idx);
    } else {
        let pkg_clone = {
            let filtered = state.filtered_packages();
            filtered.get(state.selected_index).map(|p| (*p).clone())
        };
        if let Some(pkg) = pkg_clone {
            let is_installed = state.is_installed(&pkg.vault_id, &pkg.identity.name, &pkg.kind);
            if !is_installed {
                state.status_line =
                    "Item not installed \u{2014} use Space to install first".to_string();
            } else {
                let providers = if let Ok(config) = ctx.store.load(state.active_scope) {
                    active_providers(&ctx.registry, &config)
                } else {
                    vec![]
                };

                if providers.is_empty() {
                    state.status_line = "No active providers to update to".to_string();
                } else {
                    let mut success = true;
                    for provider in providers {
                        if let Err(e) = crate::app::actions::update_asset(
                            state.active_scope,
                            &pkg,
                            ctx.store.as_ref(),
                            provider,
                        ) {
                            state.status_line =
                                format!("Update failed for {}: {}", provider.name(), e);
                            success = false;
                            break;
                        }
                    }
                    if success {
                        if let Ok(config) = ctx.store.load(state.active_scope) {
                            state.configs.insert(state.active_scope, config);
                        }
                        state.status_line = format!("Updated '{}'", pkg.identity.name);
                    }
                }
            }
        }
    }
    Ok(())
}

fn handle_f_keys(state: &mut AppState, ctx: &EventContext, code: &KeyCode) -> Result<()> {
    match code {
        KeyCode::F(5) => handle_f5_update_all(state, ctx),
        KeyCode::F(4) => {
            let tx = ctx.tx.clone();
            let registry = ctx.registry.clone();
            tokio::spawn(async move {
                let _ = refresh_all_vaults(registry, tx, "").await;
            });
            Ok(())
        }
        KeyCode::F(2) => {
            let vaults_idx = state
                .tab_names
                .iter()
                .position(|n| n == "Vaults")
                .unwrap_or(3);
            if state.active_tab == vaults_idx {
                apply_enter_attach_vault(state);
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn handle_f5_update_all(state: &mut AppState, ctx: &EventContext) -> Result<()> {
    let mut pkgs_to_update = Vec::new();
    for pkg_list in state.packages.values() {
        for pkg in pkg_list {
            if state.is_installed(&pkg.vault_id, &pkg.identity.name, &pkg.kind) {
                pkgs_to_update.push(pkg.clone());
            }
        }
    }

    if pkgs_to_update.is_empty() {
        state.status_line = "No installed items to update".into();
        return Ok(());
    }

    let tx = ctx.tx.clone();
    let store = ctx.store.clone();
    let registry = ctx.registry.clone();
    let scope = state.active_scope;
    let id = crate::tui::app::NEXT_TASK_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    tokio::task::spawn_blocking(move || {
        let _ = tx.send(AppEvent::TaskStarted {
            id,
            name: format!("Updating {} items...", pkgs_to_update.len()),
        });

        let providers = if let Ok(config) = store.load(scope) {
            active_providers(&registry, &config)
        } else {
            vec![]
        };

        if providers.is_empty() {
            let _ = tx.send(AppEvent::TaskFailed {
                id,
                error: "No active providers for update".into(),
            });
            return;
        }

        let mut success = 0;
        for pkg in pkgs_to_update {
            for provider in &providers {
                if crate::app::actions::update_asset(scope, &pkg, store.as_ref(), *provider).is_ok()
                {
                    success += 1;
                }
            }
        }
        let _ = tx.send(AppEvent::TaskProgress { id, percent: 100 });
        let _ = tx.send(AppEvent::TriggerReload);
        let _ = tx.send(AppEvent::TaskCompleted {
            id,
            message: format!("Updated {} items successfully", success),
        });
    });

    state.checked_items.clear();
    Ok(())
}

pub fn apply_tab_switch(state: &mut AppState, idx: usize, tab_count: usize) {
    if idx < tab_count {
        state.active_tab = idx;
        state.selected_index = 0;
        state.search_query.clear();
        state.status_line.clear();
        state.list_mode = ListMode::Normal;
    }
}

pub fn apply_search_char(state: &mut AppState, c: char) {
    state.search_query.push(c);
    state.list_mode = ListMode::Searching;
    state.selected_index = 0;
    state.status_line.clear();
}

pub fn apply_esc(state: &mut AppState) {
    state.search_query.clear();
    state.list_mode = ListMode::Normal;
    state.selected_index = 0;
    state.remote_packages.clear();
    state.clawhub_searching = false;
}

pub fn apply_scope_toggle(state: &mut AppState) {
    state.toggle_scope();
    state.status_line = format!("Scope: {}", state.scope_label());
}

pub fn apply_space_no_provider(state: &mut AppState, providers_tab_idx: usize) {
    apply_tab_switch(state, providers_tab_idx, state.tab_names.len());
    state.status_line = "No provider configured \u{2014} please select one".to_string();
}

pub fn apply_enter_attach_vault(state: &mut AppState) {
    state.list_mode = ListMode::AttachVault;
    state.prompt_buffer = String::new();
    state.status_line =
        "Attach vault \u{2014} enter local path or Github URL (Enter to confirm, Esc to cancel):"
            .to_string();
}

fn parse_github_url(url: &str) -> Option<(String, String)> {
    let url = url.trim();
    let url = url.strip_suffix(".git").unwrap_or(url);

    let path = url
        .strip_prefix("https://github.com/")
        .or_else(|| url.strip_prefix("github.com/"));

    if let Some(path) = path {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 2 {
            let repo = format!("{}/{}", parts[0], parts[1]);
            let id = parts[1].to_string();
            return Some((id, repo));
        }
    }
    None
}

pub async fn refresh_all_vaults(
    registry: Arc<crate::app::registry::Registry>,
    tx: tokio::sync::mpsc::UnboundedSender<AppEvent>,
    message_prefix: &str,
) -> Result<()> {
    let id = crate::tui::app::NEXT_TASK_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let _ = tx.send(AppEvent::TaskStarted {
        id,
        name: format!("{}refreshing vaults...", message_prefix),
    });
    let mut errs = Vec::new();
    let total = registry.vaults.len();
    for (i, vault) in registry.vaults.iter().enumerate() {
        if let Err(e) = vault.refresh().await {
            errs.push(format!("{}: {}", vault.id(), e));
        }
        let percent = (((i + 1) as f32 / total.max(1) as f32) * 100.0) as u8;
        let _ = tx.send(AppEvent::TaskProgress { id, percent });
    }
    let _ = tx.send(AppEvent::TriggerReload);
    if errs.is_empty() {
        let _ = tx.send(AppEvent::TaskCompleted {
            id,
            message: format!("{}refreshed successfully", message_prefix),
        });
    } else {
        let _ = tx.send(AppEvent::TaskFailed {
            id,
            error: format!("{}refresh issues: {}", message_prefix, errs.join(", ")),
        });
    }
    Ok(())
}

fn active_providers<'a>(
    registry: &'a crate::app::registry::Registry,
    config: &crate::domain::config::ConfigFile,
) -> Vec<&'a dyn crate::app::ports::ProviderPort> {
    registry
        .providers
        .iter()
        .filter(|p| config.providers.contains(&p.id().to_string()))
        .map(|p| p.as_ref())
        .collect()
}

fn update_attach_status(state: &mut AppState) {
    match state.list_mode {
        ListMode::AttachVault => {
            state.status_line = format!("Path/URL: {}", state.prompt_buffer);
        }
        ListMode::AttachVaultBranch => {
            state.status_line = format!("Branch (default: main): {}", state.prompt_buffer);
        }
        ListMode::AttachVaultPath => {
            state.status_line = format!("Subfolder (default: skills/): {}", state.prompt_buffer);
        }
        _ => {}
    }
}

fn execute_attach_vault(
    ctx: &EventContext,
    vault_id: String,
    vault_config: crate::domain::config::VaultConfig,
) {
    let id = crate::tui::app::NEXT_TASK_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let _ = ctx.tx.send(AppEvent::TaskStarted {
        id,
        name: format!("Attaching vault '{}'", vault_id),
    });

    let store = ctx.store.clone();
    let tx = ctx.tx.clone();

    tokio::task::spawn_blocking(move || {
        let vault_config_clone = vault_config.clone();
        match crate::app::actions::attach_vault(vault_id.clone(), vault_config, store.as_ref()) {
            Ok(()) => {
                let _ = tx.send(AppEvent::TaskProgress { id, percent: 100 });
                let _ = tx.send(AppEvent::TriggerReload);
                let _ = tx.send(AppEvent::TaskCompleted {
                    id,
                    message: format!("Attached vault '{}'", vault_id),
                });

                // Signal that we need a refresh (async work handled elsewhere)
                let _ = tx.send(AppEvent::VaultRefreshRequired {
                    id: vault_id,
                    config: vault_config_clone,
                });
            }
            Err(e) => {
                let _ = tx.send(AppEvent::TaskFailed {
                    id,
                    error: format!("Failed to attach: {}", e),
                });
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::app::{AppState, ListMode};
    use std::collections::HashMap;

    fn empty_state(tab_count: usize) -> AppState {
        AppState::new(
            (0..tab_count).map(|i| format!("Tab{}", i)).collect(),
            vec![true; tab_count],
            HashMap::new(),
        )
    }

    #[test]
    fn switch_tab_updates_active_tab() {
        let mut state = empty_state(4);
        apply_tab_switch(&mut state, 2, 4);
        assert_eq!(state.active_tab, 2);
    }

    #[test]
    fn switch_tab_resets_selection_and_search() {
        let mut state = empty_state(4);
        state.selected_index = 3;
        state.search_query = "foo".to_string();
        apply_tab_switch(&mut state, 1, 4);
        assert_eq!(state.selected_index, 0);
        assert!(state.search_query.is_empty());
    }

    #[test]
    fn switch_tab_ignores_out_of_range() {
        let mut state = empty_state(4);
        apply_tab_switch(&mut state, 9, 4);
        assert_eq!(state.active_tab, 0);
    }

    #[test]
    fn search_query_appends_char() {
        let mut state = empty_state(1);
        apply_search_char(&mut state, 'a');
        apply_search_char(&mut state, 'b');
        assert_eq!(state.search_query, "ab");
        assert_eq!(state.list_mode, ListMode::Searching);
    }

    #[test]
    fn esc_clears_search() {
        let mut state = empty_state(1);
        state.search_query = "hello".to_string();
        state.list_mode = ListMode::Searching;
        apply_esc(&mut state);
        assert!(state.search_query.is_empty());
        assert_eq!(state.list_mode, ListMode::Normal);
    }

    #[test]
    fn space_redirects_to_providers_tab_when_no_provider() {
        let mut state = empty_state(4);
        apply_space_no_provider(&mut state, 2);
        assert_eq!(state.active_tab, 2);
        assert!(!state.status_line.is_empty());
    }

    #[test]
    fn a_key_on_vaults_tab_enters_attach_mode() {
        let mut state = empty_state(4);
        state.active_tab = 3; // Vaults tab
        apply_enter_attach_vault(&mut state);
        assert_eq!(state.list_mode, ListMode::AttachVault);
    }

    #[test]
    fn s_key_toggles_scope() {
        let mut state = empty_state(4);
        use crate::domain::scope::Scope;
        assert_eq!(state.active_scope, Scope::Workspace);
        apply_scope_toggle(&mut state);
        assert_eq!(state.active_scope, Scope::Global);
    }

    #[test]
    fn parse_github_url_works() {
        assert_eq!(
            parse_github_url("https://github.com/obra/superpowers"),
            Some(("superpowers".to_string(), "obra/superpowers".to_string()))
        );
        assert_eq!(
            parse_github_url("https://github.com/obra/superpowers.git"),
            Some(("superpowers".to_string(), "obra/superpowers".to_string()))
        );
        assert_eq!(
            parse_github_url("github.com/obra/superpowers"),
            Some(("superpowers".to_string(), "obra/superpowers".to_string()))
        );
        assert_eq!(
            parse_github_url("https://github.com/obra/superpowers/tree/main"),
            Some(("superpowers".to_string(), "obra/superpowers".to_string()))
        );
        assert!(parse_github_url("/local/path").is_none());
    }

    #[test]
    fn test_handle_navigation_down() {
        let mut state = empty_state(1);
        state.packages.insert(
            0,
            vec![
                crate::domain::asset::ScannedPackage {
                    identity: crate::domain::identity::AssetIdentity::new("a", None, "hash"),
                    path: std::path::PathBuf::from("a"),
                    vault_id: "v".into(),
                    kind: crate::domain::asset::AssetKind::Skill,
                    is_remote: false,
                },
                crate::domain::asset::ScannedPackage {
                    identity: crate::domain::identity::AssetIdentity::new("b", None, "hash"),
                    path: std::path::PathBuf::from("b"),
                    vault_id: "v".into(),
                    kind: crate::domain::asset::AssetKind::Skill,
                    is_remote: false,
                },
            ],
        );
        state.tab_kinds = vec![crate::tui::app::TabKind::Asset];

        let (tx, _) = tokio::sync::mpsc::unbounded_channel();
        let registry = Arc::new(crate::app::registry::Registry::new());
        let store = Arc::new(crate::infra::config::toml_store::TomlConfigStore::new(
            std::path::PathBuf::from("g"),
            std::path::PathBuf::from("w"),
        ));
        let ctx = EventContext {
            store,
            registry,
            tx,
            workspace_root: std::path::PathBuf::from("."),
        };

        let event = crossterm::event::Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Down,
            KeyModifiers::empty(),
        ));
        handle(&mut state, &ctx, event).unwrap();
        assert_eq!(state.selected_index, 1);
    }

    #[test]
    fn test_handle_esc_quit() {
        let mut state = empty_state(1);
        state.esc_pressed_once = true;
        let (tx, _) = tokio::sync::mpsc::unbounded_channel();
        let registry = Arc::new(crate::app::registry::Registry::new());
        let store = Arc::new(crate::infra::config::toml_store::TomlConfigStore::new(
            std::path::PathBuf::from("g"),
            std::path::PathBuf::from("w"),
        ));
        let ctx = EventContext {
            store,
            registry,
            tx,
            workspace_root: std::path::PathBuf::from("."),
        };

        let event = crossterm::event::Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Esc,
            KeyModifiers::empty(),
        ));
        let res = handle(&mut state, &ctx, event).unwrap();
        assert!(matches!(res, ControlFlow::Quit));
    }

    use crate::app::ports::{ConfigStorePort, ProviderPort};
    use crate::domain::asset::AssetKind;
    use crate::domain::config::ConfigFile;
    use crate::domain::identity::AssetIdentity;
    use crate::domain::scope::Scope;

    struct FakeStore {
        config: std::sync::Mutex<ConfigFile>,
    }
    impl FakeStore {
        fn new(config: ConfigFile) -> Self {
            Self {
                config: std::sync::Mutex::new(config),
            }
        }
    }
    impl ConfigStorePort for FakeStore {
        fn load(&self, _scope: Scope) -> Result<ConfigFile> {
            Ok(self.config.lock().unwrap().clone())
        }
        fn save(&self, _scope: Scope, config: &ConfigFile) -> Result<()> {
            *self.config.lock().unwrap() = config.clone();
            Ok(())
        }
    }

    struct FakeProvider {
        id: String,
    }
    impl ProviderPort for FakeProvider {
        fn id(&self) -> &str {
            &self.id
        }
        fn name(&self) -> &str {
            &self.id
        }
        fn install(
            &self,
            _pkg: &crate::domain::asset::ScannedPackage,
            _scope: Scope,
        ) -> Result<()> {
            Ok(())
        }
        fn remove(
            &self,
            _identity: &AssetIdentity,
            _kind: &AssetKind,
            _scope: Scope,
        ) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_handle_space_install() {
        let mut state = empty_state(1);
        let pkg = crate::domain::asset::ScannedPackage {
            identity: crate::domain::identity::AssetIdentity::new("my-skill", None, "hash"),
            path: std::path::PathBuf::from("a"),
            vault_id: "v".into(),
            kind: crate::domain::asset::AssetKind::Skill,
            is_remote: false,
        };
        state.packages.insert(0, vec![pkg.clone()]);
        state.tab_kinds = vec![crate::tui::app::TabKind::Asset];
        state.active_tab = 0;
        state.selected_index = 0;

        let mut config = ConfigFile::default();
        config.providers.push("fake".into());
        state.configs.insert(Scope::Workspace, config.clone());

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let mut registry = crate::app::registry::Registry::new();
        registry.register_provider(Box::new(FakeProvider { id: "fake".into() }));
        let registry = Arc::new(registry);

        let store = Arc::new(FakeStore::new(config));
        let ctx = EventContext {
            store,
            registry,
            tx,
            workspace_root: std::path::PathBuf::from("."),
        };

        let event = crossterm::event::Event::Key(crossterm::event::KeyEvent::new(
            KeyCode::Char(' '),
            KeyModifiers::empty(),
        ));
        handle(&mut state, &ctx, event).unwrap();

        // Use a small sleep to let the background task run (since it's spawn_blocking)
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Check events
        let mut events = Vec::new();
        while let Ok(e) = rx.try_recv() {
            events.push(e);
        }

        // Should have TaskStarted, TaskProgress, TriggerReload, TaskCompleted
        assert!(events
            .iter()
            .any(|e| matches!(e, AppEvent::TaskStarted { .. })));
        assert!(events.iter().any(|e| matches!(e, AppEvent::TriggerReload)));
        assert!(events
            .iter()
            .any(|e| matches!(e, AppEvent::TaskCompleted { .. })));
    }
}
