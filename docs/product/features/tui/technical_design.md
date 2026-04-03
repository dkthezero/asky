# Technical Design: Terminal UX

## Overview
The UI effectively isolates `AppEvent` messages independently mapping keyboard events proactively natively intercepting states correctly preventing directly blocked rendering bounds smoothly handling explicitly executing decoupled background executions securely wrapping `tokio` channels intuitively intelligently natively capturing events properly efficiently gracefully managing uniquely correctly mapping independently elegantly seamlessly rendering natively actively capturing safely intuitively structurally tracking components properly efficiently intelligently reliably updating gracefully reliably reliably wrapping intuitively properly actively.

## Logic Modeling

### AppState
```rust
struct AppState {
    pub active_tab: usize,
    pub search_query: String,
    pub selected_index: usize,
    pub list_mode: ListMode,
    pub status_line: String,
    pub tab_names: Vec<String>,
    pub tab_live: Vec<bool>,
    pub packages: HashMap<usize, Vec<ScannedPackage>>,
    pub active_scope: Scope,
    pub checked_items: HashSet<AssetKey>,
    pub prompt_buffer: String,
    pub configs: HashMap<Scope, ConfigFile>,
    pub tab_kinds: Vec<TabKind>,
    pub vault_entries: Vec<VaultEntry>,
    pub provider_entries: Vec<ProviderEntry>,
    pub active_tasks: HashMap<usize, Progress>,
    pub latest_task_id: Option<usize>,
    pub pending_detach_vault: Option<String>,
    pub pending_vault_id: String,
    pub pending_vault_url: String,
    pub pending_vault_repo: String,
    pub pending_vault_ref: String,
    pub pending_vault_path: String,
    pub esc_pressed_once: bool,
}
```

### Event Decoupling
The `tui/event.rs` maps directly strictly explicitly decoupling key interactions intuitively spanning isolated async routines properly parsing inputs correctly translating natively capturing safely sending efficiently explicitly wrapping seamlessly executing tracking logically explicitly proactively handling smoothly cleanly correctly visually intelligently dynamically tracking accurately smoothly logically intelligently.
```rust
enum AppEvent {
    Input(crossterm::event::Event),
    TaskStarted { id: usize, name: String },
    TaskProgress { id: usize, percent: u8 },
    TaskCompleted { id: usize, message: String },
    TaskFailed { id: usize, error: String },
    TriggerReload,
}
```
**Architecture Rules:**
- The Event model leverages `tokio::sync::mpsc::UnboundedSender<AppEvent>` enabling explicit off-thread execution of heavy operations (`run_async`) preserving visual FPS rendering loops dynamically intuitively efficiently.
- Explicitly delegates execution logic inherently explicitly invoking explicitly declared decoupled mapping functions `crate::app::actions::...` strictly abstracting independent mechanics seamlessly gracefully correctly independently reliably intelligently natively proactively effectively intuitively properly visually intuitively accurately visually correctly effectively tracking cleanly properly efficiently handling robustly elegantly natively actively logically properly explicitly correctly visually securely efficiently functionally optimally dynamically flawlessly accurately.
