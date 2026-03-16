use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tokio::sync::mpsc;
use crate::agent_logic::{AgentMessage, AgentPhase, ToolStatus};
use crate::orchestrator::Orchestrator;
use crate::config::{self as config, Config};
use crate::ui::theme;
use crate::ui::widgets::stepper::{self, AgentStepper};
use crate::ui::widgets::diff_view;
use crate::ui::widgets::status_bar::StatusBar;
use crate::ui::widgets::tool_pane::{ToolEntry, ToolPane};
use crate::ui::widgets::approval_modal::{ApprovalModal, ModalSelection};
use crate::ui::widgets::image_view;
use crate::ui::widgets::command_palette::{CommandPalette, CommandPaletteState};
use crate::clipboard;
use crate::auth;
use crate::git::GitManager;
use anyhow::Result;
use std::env;

// ── Agent Interaction Mode (Crush-style lifecycle) ──────────────────────────
#[derive(Clone, Copy, PartialEq, Debug)]
enum AgentMode {
    Manual,      // User must approve every edit
    AutoAccept,  // Edits are auto-accepted
    Plan,        // AI generates plan only, no edits
    Yolo,        // Full auto: no approval, no review
}

impl AgentMode {
    /// Cycle to next mode with shift+tab (Manual → AutoAccept → Plan → Manual)
    fn cycle_next(self) -> Self {
        match self {
            Self::Manual => Self::AutoAccept,
            Self::AutoAccept => Self::Plan,
            Self::Plan => Self::Manual,
            Self::Yolo => Self::Manual, // Yolo exits to Manual
        }
    }
}

// ── Auth Sub-Menu Options ───────────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq, Debug)]
enum AuthMenuItem {
    LoginGoogle,
    LoginOpenRouter,
    SetApiKey,
    Status,
    Logout,
}

impl AuthMenuItem {
    fn all() -> &'static [AuthMenuItem] {
        &[
            AuthMenuItem::LoginGoogle,
            AuthMenuItem::LoginOpenRouter,
            AuthMenuItem::SetApiKey,
            AuthMenuItem::Status,
            AuthMenuItem::Logout,
        ]
    }

    fn label(&self) -> &'static str {
        match self {
            Self::LoginGoogle => "Sign in with Google OAuth",
            Self::LoginOpenRouter => "Sign in with OpenRouter",
            Self::SetApiKey => "Set API key manually",
            Self::Status => "View auth status",
            Self::Logout => "Log out (clear credentials)",
        }
    }
}

// ── Model Sub-Menu State ─────────────────────────────────────────────────────
#[derive(Clone, PartialEq, Debug)]
enum ModelMenuLevel {
    PickProvider,
    PickModel(config::Provider),
}

// ── Focus Zones ─────────────────────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq)]
enum FocusPane {
    Chat,
    Sidebar,
    ToolOutput,
    Input,
}

// ── Chat Message Types ──────────────────────────────────────────────────────
#[derive(Clone)]
enum MsgKind {
    User,
    Agent,
    Error,
    System,
    Diff,
}

#[derive(Clone)]
struct ChatMsg {
    kind: MsgKind,
    text: String,
}

// ── Activity Log Entry ──────────────────────────────────────────────────────
#[derive(Clone, Copy)]
enum LogStatus {
    Pending,
    Done,
    Failed,
    Info,
}

// ── App State ───────────────────────────────────────────────────────────────
pub struct App {
    // Input
    input: String,
    cursor_pos: usize,

    // Chat
    messages: Vec<ChatMsg>,
    chat_scroll: u16,

    // Activity sidebar
    agent_logs: Vec<(LogStatus, String)>,
    sidebar_scroll: u16,

    // Thinking sidebar (raw LLM inner monologue)
    thinking_lines: Vec<String>,

    // Agent stepper
    agent_phase: AgentPhase,

    // Tool tracking
    tool_entries: Vec<ToolEntry>,
    tool_scroll: u16,

    // Approval modal
    show_approval: bool,
    approval_command: String,
    approval_selection: ModalSelection,

    // Image tags
    image_tags: Vec<image_view::ImageTag>,

    // System state
    repo_path: std::path::PathBuf,
    config: Config,
    orchestrator: Option<Orchestrator>,
    is_streaming: bool,
    is_indexing: bool,
    memory_hit: bool,
    tick: usize,
    focus: FocusPane,

    // Response channel for sending approval decisions back to orchestrator
    orch_response_tx: Option<mpsc::Sender<AgentMessage>>,

    // Command palette
    palette: CommandPaletteState,

    // Agent interaction mode (Crush-style lifecycle)
    agent_mode: AgentMode,

    // Help panel visibility (Ctrl+G toggle)
    show_help: bool,

    // Tool pane visibility (toggle)
    show_tools: bool,

    // Available tools catalog (shown when tool pane is open with no active tools)
    available_tools: Vec<(&'static str, &'static str)>,

    // Auth sub-menu state
    show_auth_menu: bool,
    auth_menu_selected: usize,

    // Model sub-menu state
    show_model_menu: bool,
    model_menu_level: ModelMenuLevel,
    model_menu_selected: usize,
}

impl App {
    pub fn new(config: Config) -> Self {
        let repo_path = env::current_dir().unwrap_or_else(|_| ".".into());
        let orchestrator = if let (Some(g), Some(a)) = (&config.keys.gemini, &config.keys.anthropic) {
            Some(Orchestrator::new(repo_path.clone(), g.clone(), a.clone()))
        } else {
            None
        };
        Self {
            input: String::new(),
            cursor_pos: 0,
            messages: vec![ChatMsg {
                kind: MsgKind::Agent,
                text: "Standing by. What are we building?".into(),
            }],
            chat_scroll: 0,
            agent_logs: vec![(LogStatus::Info, "Awaiting instructions".into())],
            sidebar_scroll: 0,
            thinking_lines: Vec::new(),
            agent_phase: AgentPhase::Idle,
            tool_entries: Vec::new(),
            tool_scroll: 0,
            show_approval: false,
            approval_command: String::new(),
            approval_selection: ModalSelection::Approve,
            image_tags: Vec::new(),
            repo_path,
            config,
            orchestrator,
            is_streaming: false,
            is_indexing: false,
            memory_hit: false,
            tick: 0,
            focus: FocusPane::Input,
            orch_response_tx: None,
            palette: CommandPaletteState::new(),
            agent_mode: AgentMode::Manual,
            show_help: false,
            show_tools: false,
            show_auth_menu: false,
            auth_menu_selected: 0,
            show_model_menu: false,
            model_menu_level: ModelMenuLevel::PickProvider,
            model_menu_selected: 0,
            available_tools: vec![
                ("read_file", "Read file contents from workspace"),
                ("edit_file", "Apply targeted edits to files"),
                ("create_file", "Create new files in workspace"),
                ("shell", "Execute shell commands (with approval)"),
                ("search", "Search codebase with regex patterns"),
                ("list_dir", "List directory contents"),
                ("cargo_check", "Run cargo check for diagnostics"),
                ("git_diff", "Show git diff of working tree"),
                ("git_log", "Show recent git history"),
                ("web_fetch", "Fetch content from URLs"),
            ],
        }
    }

    // ── Main Event Loop ─────────────────────────────────────────────────────
    pub async fn run<B: Backend>(mut self, terminal: &mut Terminal<B>, initial_prompt: Option<String>) -> Result<()> {
        let (ui_tx, mut ui_rx) = mpsc::channel(256);

        // Auto-submit initial prompt if provided via CLI (gemini-cli style)
        if let Some(prompt) = initial_prompt {
            self.messages.push(ChatMsg {
                kind: MsgKind::User,
                text: prompt.clone(),
            });

            // Check for image path transformations
            for word in prompt.split_whitespace() {
                if let Some(tag) = image_view::transform_image_path(word) {
                    self.image_tags.push(tag);
                }
            }

            if let Some(orch) = &self.orchestrator {
                let tx = ui_tx.clone();
                let q = prompt;
                let o = orch.clone();
                self.agent_phase = AgentPhase::Mapping;
                let (resp_tx, resp_rx) = mpsc::channel(32);
                self.orch_response_tx = Some(resp_tx);
                tokio::spawn(async move {
                    let _ = o.run(q, tx, resp_rx).await;
                });
            } else {
                self.messages.push(ChatMsg {
                    kind: MsgKind::Error,
                    text: "API keys not configured. Use /auth or /login.".into(),
                });
            }
        }

        loop {
            self.tick = self.tick.wrapping_add(1);
            terminal.draw(|f| self.ui(f))?;

            if event::poll(std::time::Duration::from_millis(33))? {
                let ev = event::read()?;
                let key = match ev {
                    Event::Resize(_, _) => {
                        // Ratatui redraws on next loop iteration automatically.
                        // Drain messages then continue so we don't skip them.
                        while let Ok(msg) = ui_rx.try_recv() {
                            self.handle_agent_message(msg);
                        }
                        continue;
                    }
                    Event::Key(k) => k,
                    _ => {
                        while let Ok(msg) = ui_rx.try_recv() {
                            self.handle_agent_message(msg);
                        }
                        continue;
                    }
                };

                // ── Modal Intercept ──────────────────────────────────
                if self.show_approval {
                    match key.code {
                        KeyCode::Tab => {
                            self.approval_selection = match self.approval_selection {
                                ModalSelection::Approve => ModalSelection::Deny,
                                ModalSelection::Deny => ModalSelection::Approve,
                            };
                        }
                        KeyCode::Enter => {
                            let cmd = self.approval_command.clone();
                            self.show_approval = false;
                            let response = match self.approval_selection {
                                ModalSelection::Approve => AgentMessage::ShellApproved(cmd),
                                ModalSelection::Deny => AgentMessage::ShellDenied(cmd),
                            };
                            if let Some(tx) = &self.orch_response_tx {
                                let _ = tx.send(response).await;
                            }
                        }
                        KeyCode::Esc => {
                            let cmd = self.approval_command.clone();
                            self.show_approval = false;
                            if let Some(tx) = &self.orch_response_tx {
                                let _ = tx.send(AgentMessage::ShellDenied(cmd)).await;
                            }
                        }
                        KeyCode::Char('y') => {
                            let cmd = self.approval_command.clone();
                            self.show_approval = false;
                            if let Some(tx) = &self.orch_response_tx {
                                let _ = tx.send(AgentMessage::ShellApproved(cmd)).await;
                            }
                        }
                        KeyCode::Char('n') => {
                            let cmd = self.approval_command.clone();
                            self.show_approval = false;
                            if let Some(tx) = &self.orch_response_tx {
                                let _ = tx.send(AgentMessage::ShellDenied(cmd)).await;
                            }
                        }
                        _ => {}
                    }
                    while let Ok(msg) = ui_rx.try_recv() {
                        self.handle_agent_message(msg);
                    }
                    continue;
                }

                // ── Auth Sub-Menu Intercept ───────────────────────────
                if self.show_auth_menu {
                    let items = AuthMenuItem::all();
                    match key.code {
                        KeyCode::Esc => {
                            self.show_auth_menu = false;
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            self.auth_menu_selected = self.auth_menu_selected.saturating_sub(1);
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if self.auth_menu_selected + 1 < items.len() {
                                self.auth_menu_selected += 1;
                            }
                        }
                        KeyCode::Enter => {
                            let selected = items[self.auth_menu_selected];
                            self.show_auth_menu = false;
                            match selected {
                                AuthMenuItem::LoginGoogle => {
                                    self.messages.push(ChatMsg {
                                        kind: MsgKind::System,
                                        text: "Starting Google OAuth login... Opening browser.".into(),
                                    });
                                    let tx = ui_tx.clone();
                                    tokio::spawn(async move {
                                        match Self::run_google_oauth_login().await {
                                            Ok(_) => {
                                                let _ = tx.send(AgentMessage::Thinking("Google OAuth login successful.".into())).await;
                                            }
                                            Err(e) => {
                                                let _ = tx.send(AgentMessage::Error(format!("Google OAuth failed: {}", e))).await;
                                            }
                                        }
                                    });
                                }
                                AuthMenuItem::LoginOpenRouter => {
                                    self.messages.push(ChatMsg {
                                        kind: MsgKind::System,
                                        text: "Starting OpenRouter OAuth login... Opening browser.".into(),
                                    });
                                    let tx = ui_tx.clone();
                                    tokio::spawn(async move {
                                        match Self::run_openrouter_oauth_login().await {
                                            Ok(_) => {
                                                let _ = tx.send(AgentMessage::Thinking("OpenRouter login successful.".into())).await;
                                            }
                                            Err(e) => {
                                                let _ = tx.send(AgentMessage::Error(format!("OpenRouter login failed: {}", e))).await;
                                            }
                                        }
                                    });
                                }
                                AuthMenuItem::SetApiKey => {
                                    self.messages.push(ChatMsg {
                                        kind: MsgKind::System,
                                        text: "Set keys via CLI: guv auth -g <GEMINI_KEY> or guv auth -a <ANTHROPIC_KEY>".into(),
                                    });
                                }
                                AuthMenuItem::Status => {
                                    self.handle_slash_command("auth_status", &ui_tx).await;
                                }
                                AuthMenuItem::Logout => {
                                    match auth::clear_credentials() {
                                        Ok(_) => {
                                            self.messages.push(ChatMsg {
                                                kind: MsgKind::System,
                                                text: "Logged out. Credentials cleared.".into(),
                                            });
                                        }
                                        Err(e) => {
                                            self.messages.push(ChatMsg {
                                                kind: MsgKind::Error,
                                                text: format!("Logout failed: {}", e),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                    while let Ok(msg) = ui_rx.try_recv() {
                        self.handle_agent_message(msg);
                    }
                    continue;
                }

                // ── Model Sub-Menu Intercept ─────────────────────────
                if self.show_model_menu {
                    match key.code {
                        KeyCode::Esc => {
                            match &self.model_menu_level {
                                ModelMenuLevel::PickModel(_) => {
                                    // Go back to provider picker
                                    self.model_menu_level = ModelMenuLevel::PickProvider;
                                    self.model_menu_selected = 0;
                                }
                                ModelMenuLevel::PickProvider => {
                                    self.show_model_menu = false;
                                }
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            self.model_menu_selected = self.model_menu_selected.saturating_sub(1);
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            let max = match &self.model_menu_level {
                                ModelMenuLevel::PickProvider => 3, // Google, Anthropic, OpenRouter
                                ModelMenuLevel::PickModel(p) => config::models_for_provider(p).len(),
                            };
                            if self.model_menu_selected + 1 < max {
                                self.model_menu_selected += 1;
                            }
                        }
                        KeyCode::Enter => {
                            match self.model_menu_level.clone() {
                                ModelMenuLevel::PickProvider => {
                                    let provider = match self.model_menu_selected {
                                        0 => config::Provider::Google,
                                        1 => config::Provider::Anthropic,
                                        _ => config::Provider::OpenRouter,
                                    };
                                    self.model_menu_level = ModelMenuLevel::PickModel(provider);
                                    self.model_menu_selected = 0;
                                }
                                ModelMenuLevel::PickModel(ref provider) => {
                                    let models = config::models_for_provider(provider);
                                    if let Some((model_id, _)) = models.get(self.model_menu_selected) {
                                        self.config.model = config::ModelChoice {
                                            provider: provider.clone(),
                                            model_id: model_id.to_string(),
                                        };
                                        self.messages.push(ChatMsg {
                                            kind: MsgKind::System,
                                            text: format!("Model set to {}", self.config.model.display_name()),
                                        });
                                        self.show_model_menu = false;
                                        self.model_menu_level = ModelMenuLevel::PickProvider;
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                    while let Ok(msg) = ui_rx.try_recv() {
                        self.handle_agent_message(msg);
                    }
                    continue;
                }

                // ── Command Palette Intercept ────────────────────────
                if self.palette.visible {
                    match key.code {
                        KeyCode::Esc => {
                            self.palette.close();
                        }
                        KeyCode::Enter => {
                            if let Some(cmd) = self.palette.confirm() {
                                self.handle_slash_command(&cmd.name.to_string(), &ui_tx).await;
                            }
                        }
                        KeyCode::Up => {
                            self.palette.move_up();
                        }
                        KeyCode::Down => {
                            self.palette.move_down();
                        }
                        KeyCode::Tab => {
                            self.palette.move_down();
                        }
                        KeyCode::BackTab => {
                            self.palette.move_up();
                        }
                        KeyCode::Backspace => {
                            self.palette.backspace();
                        }
                        KeyCode::Char(c) => {
                            self.palette.type_char(c);
                        }
                        _ => {}
                    }
                    while let Ok(msg) = ui_rx.try_recv() {
                        self.handle_agent_message(msg);
                    }
                    continue;
                }

                // ── Global Keybindings ──────────────────────────────
                match (key.code, key.modifiers) {
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Ok(()),
                    (KeyCode::Char('d'), KeyModifiers::CONTROL) => return Ok(()),

                    // Command palette triggers
                    (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                        self.palette.open();
                    }

                    // Ctrl+Y: Toggle YOLO mode
                    (KeyCode::Char('y'), KeyModifiers::CONTROL) => {
                        if self.agent_mode == AgentMode::Yolo {
                            self.agent_mode = AgentMode::Manual;
                            self.messages.push(ChatMsg {
                                kind: MsgKind::System,
                                text: "Switched to manual mode".into(),
                            });
                        } else {
                            self.agent_mode = AgentMode::Yolo;
                            self.messages.push(ChatMsg {
                                kind: MsgKind::System,
                                text: "YOLO mode enabled – all edits auto-applied".into(),
                            });
                        }
                    }

                    // Ctrl+G: Toggle help panel
                    (KeyCode::Char('g'), KeyModifiers::CONTROL) => {
                        self.show_help = !self.show_help;
                    }

                    // Ctrl+F: Add image (open file path prompt)
                    (KeyCode::Char('f'), KeyModifiers::CONTROL) if self.focus == FocusPane::Input && !self.is_streaming => {
                        self.messages.push(ChatMsg {
                            kind: MsgKind::System,
                            text: "Paste an image path or drag-and-drop a file. Use Ctrl+V for clipboard.".into(),
                        });
                    }

                    // Ctrl+V: Paste clipboard image
                    (KeyCode::Char('v'), KeyModifiers::CONTROL) if self.focus == FocusPane::Input && !self.is_streaming => {
                        if clipboard::clipboard_has_image() {
                            if let Ok(Some(path)) = clipboard::save_clipboard_image(&self.repo_path) {
                                clipboard::cleanup_old_clipboard_images(&self.repo_path);
                                let rel = path.strip_prefix(&self.repo_path)
                                    .unwrap_or(&path)
                                    .display()
                                    .to_string();
                                let insert = format!("@{}", rel);
                                // Insert at cursor
                                let before = if self.cursor_pos > 0 && !self.input[..self.cursor_pos].ends_with(' ') {
                                    " "
                                } else {
                                    ""
                                };
                                let after = if self.cursor_pos < self.input.len() && !self.input[self.cursor_pos..].starts_with(' ') {
                                    " "
                                } else {
                                    ""
                                };
                                let text = format!("{}{}{}", before, insert, after);
                                self.input.insert_str(self.cursor_pos, &text);
                                self.cursor_pos += text.len();

                                if let Some(tag) = image_view::transform_image_path(&path.display().to_string()) {
                                    self.image_tags.push(tag);
                                }
                            }
                        }
                    }

                    // Ctrl+T: Toggle tool pane
                    (KeyCode::Char('t'), KeyModifiers::CONTROL) => {
                        self.show_tools = !self.show_tools;
                    }

                    // Vim-style scroll (when not typing)
                    (KeyCode::Char('j'), _) if self.focus != FocusPane::Input => {
                        match self.focus {
                            FocusPane::Chat => self.chat_scroll = self.chat_scroll.saturating_sub(1),
                            FocusPane::Sidebar => self.sidebar_scroll = self.sidebar_scroll.saturating_add(1),
                            FocusPane::ToolOutput => self.tool_scroll = self.tool_scroll.saturating_add(1),
                            _ => {}
                        }
                    }
                    (KeyCode::Char('k'), _) if self.focus != FocusPane::Input => {
                        match self.focus {
                            FocusPane::Chat => self.chat_scroll = self.chat_scroll.saturating_add(1),
                            FocusPane::Sidebar => self.sidebar_scroll = self.sidebar_scroll.saturating_sub(1),
                            FocusPane::ToolOutput => self.tool_scroll = self.tool_scroll.saturating_sub(1),
                            _ => {}
                        }
                    }

                    // Tab to cycle focus
                    (KeyCode::Tab, _) if !self.is_streaming => {
                        self.focus = match self.focus {
                            FocusPane::Input => FocusPane::Chat,
                            FocusPane::Chat => FocusPane::Sidebar,
                            FocusPane::Sidebar => FocusPane::ToolOutput,
                            FocusPane::ToolOutput => FocusPane::Input,
                        };
                    }
                    // Shift+Tab: Cycle agent mode (Manual → AutoAccept → Plan → Manual)
                    (KeyCode::BackTab, _) if !self.is_streaming => {
                        self.agent_mode = self.agent_mode.cycle_next();
                        let mode_name = match self.agent_mode {
                            AgentMode::Manual => "manual",
                            AgentMode::AutoAccept => "auto-accept edits",
                            AgentMode::Plan => "plan",
                            AgentMode::Yolo => "YOLO",
                        };
                        self.messages.push(ChatMsg {
                            kind: MsgKind::System,
                            text: format!("Switched to {} mode", mode_name),
                        });
                    }

                    // Undo shortcut
                    (KeyCode::Char('u'), _) if self.focus != FocusPane::Input && !self.is_streaming => {
                        self.messages.push(ChatMsg {
                            kind: MsgKind::System,
                            text: "Undo requested (guv undo)".into(),
                        });
                    }

                    // ── Input-mode keys ─────────────────────────────
                    (KeyCode::Char('/'), _) if self.focus == FocusPane::Input && !self.is_streaming && self.input.is_empty() => {
                        // Open command palette when / is typed on empty input (Codebuff-style)
                        self.palette.open();
                    }
                    (KeyCode::Char(c), _) if self.focus == FocusPane::Input && !self.is_streaming => {
                        self.input.insert(self.cursor_pos, c);
                        self.cursor_pos += c.len_utf8();
                    }
                    (KeyCode::Backspace, _) if self.focus == FocusPane::Input && !self.is_streaming => {
                        if self.cursor_pos > 0 {
                            let prev = self.input[..self.cursor_pos]
                                .chars()
                                .last()
                                .map(|c| c.len_utf8())
                                .unwrap_or(0);
                            self.cursor_pos -= prev;
                            self.input.remove(self.cursor_pos);
                        }
                    }
                    (KeyCode::Delete, _) if self.focus == FocusPane::Input && !self.is_streaming => {
                        if self.cursor_pos < self.input.len() {
                            self.input.remove(self.cursor_pos);
                        }
                    }
                    (KeyCode::Left, _) if self.focus == FocusPane::Input => {
                        if self.cursor_pos > 0 {
                            let prev = self.input[..self.cursor_pos]
                                .chars()
                                .last()
                                .map(|c| c.len_utf8())
                                .unwrap_or(0);
                            self.cursor_pos -= prev;
                        }
                    }
                    (KeyCode::Right, _) if self.focus == FocusPane::Input => {
                        if self.cursor_pos < self.input.len() {
                            let next = self.input[self.cursor_pos..]
                                .chars()
                                .next()
                                .map(|c| c.len_utf8())
                                .unwrap_or(0);
                            self.cursor_pos += next;
                        }
                    }
                    (KeyCode::Home, _) if self.focus == FocusPane::Input => {
                        self.cursor_pos = 0;
                    }
                    (KeyCode::End, _) if self.focus == FocusPane::Input => {
                        self.cursor_pos = self.input.len();
                    }

                    // Arrow scroll (works everywhere)
                    (KeyCode::Up, _) => {
                        self.chat_scroll = self.chat_scroll.saturating_add(1);
                    }
                    (KeyCode::Down, _) => {
                        self.chat_scroll = self.chat_scroll.saturating_sub(1);
                    }

                    // Enter to submit / approve
                    (KeyCode::Enter, _) if self.focus == FocusPane::Input && !self.is_streaming => {
                        if self.input.is_empty() {
                            continue;
                        }
                        let query: String = self.input.drain(..).collect();
                        self.cursor_pos = 0;
                        self.chat_scroll = 0;

                        if query == "exit" || query == "quit" || query == "q" {
                            return Ok(());
                        }

                        // Image path transformations
                        for word in query.split_whitespace() {
                            if let Some(tag) = image_view::transform_image_path(word) {
                                self.image_tags.push(tag);
                            }
                        }

                        self.messages.push(ChatMsg {
                            kind: MsgKind::User,
                            text: query.clone(),
                        });

                        if let Some(orch) = &self.orchestrator {
                            let tx = ui_tx.clone();
                            let q = query.clone();
                            let o = orch.clone();
                            self.agent_phase = AgentPhase::Mapping;
                            let (resp_tx, resp_rx) = mpsc::channel(32);
                            self.orch_response_tx = Some(resp_tx);
                            tokio::spawn(async move {
                                let _ = o.run(q, tx, resp_rx).await;
                            });
                        } else {
                            self.messages.push(ChatMsg {
                                kind: MsgKind::Error,
                                text: "API keys not configured. Run `guv auth`.".into(),
                            });
                        }
                    }

                    // Esc to cancel or return to input
                    (KeyCode::Esc, _) => {
                        if self.is_streaming {
                            self.agent_logs.push((LogStatus::Info, "Cancelled by user".into()));
                            self.is_streaming = false;
                            self.agent_phase = AgentPhase::Idle;
                        } else if self.focus != FocusPane::Input {
                            self.focus = FocusPane::Input;
                        }
                    }
                    _ => {}
                }
            }

            // ── Drain Agent Messages ────────────────────────────────────
            while let Ok(msg) = ui_rx.try_recv() {
                self.handle_agent_message(msg);
            }
        }
    }

    // ── Slash Command Handler ────────────────────────────────────────────────
    async fn handle_slash_command(&mut self, name: &str, ui_tx: &mpsc::Sender<AgentMessage>) {
        match name {
            "quit" => {
                // Handled in caller
            }
            "new" => {
                self.messages.clear();
                self.messages.push(ChatMsg {
                    kind: MsgKind::Agent,
                    text: "New session. What are we building?".into(),
                });
                self.thinking_lines.clear();
                self.agent_logs.clear();
                self.agent_logs.push((LogStatus::Info, "Awaiting instructions".into()));
                self.tool_entries.clear();
                self.image_tags.clear();
                self.agent_phase = AgentPhase::Idle;
                self.is_streaming = false;
                self.memory_hit = false;
            }
            "undo" => {
                if GitManager::is_repo(&self.repo_path) {
                    match GitManager::auto_stage_all(&self.repo_path)
                        .and_then(|_| GitManager::undo(&self.repo_path))
                    {
                        Ok(_) => {
                            self.messages.push(ChatMsg {
                                kind: MsgKind::System,
                                text: "Undone last AI edit.".into(),
                            });
                        }
                        Err(e) => {
                            self.messages.push(ChatMsg {
                                kind: MsgKind::Error,
                                text: format!("Undo failed: {}", e),
                            });
                        }
                    }
                } else {
                    self.messages.push(ChatMsg {
                        kind: MsgKind::Error,
                        text: "Not a git repository.".into(),
                    });
                }
            }
            "help" => {
                self.show_help = !self.show_help;
            }
            "model" => {
                self.show_model_menu = true;
                self.model_menu_level = ModelMenuLevel::PickProvider;
                self.model_menu_selected = 0;
            }
            "history" => {
                self.messages.push(ChatMsg {
                    kind: MsgKind::System,
                    text: "Session history not yet implemented.".into(),
                });
            }
            "init" => {
                self.messages.push(ChatMsg {
                    kind: MsgKind::System,
                    text: "Initializing project...".into(),
                });
                // Trigger indexing via orchestrator
                if let Some(orch) = &self.orchestrator {
                    let tx = ui_tx.clone();
                    let o = orch.clone();
                    let (resp_tx, resp_rx) = mpsc::channel(32);
                    self.orch_response_tx = Some(resp_tx);
                    self.agent_phase = AgentPhase::Mapping;
                    tokio::spawn(async move {
                        let _ = o.run("Initialize project and create AGENTS.md".into(), tx, resp_rx).await;
                    });
                }
            }
            "review" => {
                self.messages.push(ChatMsg {
                    kind: MsgKind::System,
                    text: "Running cargo check review...".into(),
                });
            }
            "sidebar" => {
                self.focus = if self.focus == FocusPane::Sidebar {
                    FocusPane::Input
                } else {
                    FocusPane::Sidebar
                };
            }
            "yolo" => {
                if self.agent_mode == AgentMode::Yolo {
                    self.agent_mode = AgentMode::Manual;
                } else {
                    self.agent_mode = AgentMode::Yolo;
                }
            }
            "tools" => {
                self.show_tools = !self.show_tools;
            }
            "editor" => {
                self.messages.push(ChatMsg {
                    kind: MsgKind::System,
                    text: "External editor not yet implemented.".into(),
                });
            }
            "sessions" => {
                self.messages.push(ChatMsg {
                    kind: MsgKind::System,
                    text: "Session browser not yet implemented.".into(),
                });
            }
            "image" => {
                self.messages.push(ChatMsg {
                    kind: MsgKind::System,
                    text: "Paste an image path or use Ctrl+V.".into(),
                });
            }
            "auth" => {
                self.show_auth_menu = true;
                self.auth_menu_selected = 0;
            }
            "auth_status" => {
                let stored = auth::load_credentials().ok().flatten();
                let env_auth = auth::detect_auth_from_env();
                let mut status_lines = Vec::new();
                if let Some(ref creds) = stored {
                    status_lines.push(format!("Auth: {:?}", creds.auth_type));
                    let valid = creds.token.as_ref().map(|t| !t.is_expired()).unwrap_or(false);
                    status_lines.push(if valid { "Token: valid".into() } else { "Token: expired".into() });
                } else if let Some(ref at) = env_auth {
                    status_lines.push(format!("Auth: {:?} (from env)", at));
                } else {
                    status_lines.push("Auth: not configured".into());
                }
                let has_gemini = self.config.keys.gemini.is_some();
                let has_anthropic = self.config.keys.anthropic.is_some();
                let has_openrouter = std::env::var("OPENROUTER_API_KEY").is_ok();
                status_lines.push(format!("Gemini key: {}", if has_gemini { "set" } else { "not set" }));
                status_lines.push(format!("Anthropic key: {}", if has_anthropic { "set" } else { "not set" }));
                status_lines.push(format!("OpenRouter key: {}", if has_openrouter { "set" } else { "not set" }));
                for line in status_lines {
                    self.messages.push(ChatMsg { kind: MsgKind::System, text: line });
                }
            }
            "usage" => {
                self.messages.push(ChatMsg {
                    kind: MsgKind::System,
                    text: format!(
                        "Budget: ${:.2} / ${:.2} remaining",
                        self.config.budget.consumed, self.config.budget.limit
                    ),
                });
            }
            "feedback" => {
                self.messages.push(ChatMsg {
                    kind: MsgKind::System,
                    text: "Feedback: github.com/openbiz/guv-code/issues".into(),
                });
            }
            "files" => {
                self.messages.push(ChatMsg {
                    kind: MsgKind::System,
                    text: "File picker not yet implemented.".into(),
                });
            }
            _ => {
                self.messages.push(ChatMsg {
                    kind: MsgKind::Error,
                    text: format!("Unknown command: /{}", name),
                });
            }
        }
    }

    // ── Agent Message Handler ───────────────────────────────────────────────
    fn handle_agent_message(&mut self, msg: AgentMessage) {
        match msg {
            AgentMessage::PlanStarted => {
                self.is_streaming = true;
                self.agent_phase = AgentPhase::Planning;
                self.agent_logs.push((LogStatus::Pending, "Scout: Analyzing repo...".into()));
            }
            AgentMessage::PlanUpdate(text) => {
                self.thinking_lines.push(format!("Scout: {}", text));
                if let Some(last) = self.agent_logs.last_mut() {
                    if matches!(last.0, LogStatus::Pending) {
                        last.1 = format!("Scout: {}", text);
                    }
                }
            }
            AgentMessage::PlanCompleted(files) => {
                if let Some(last) = self.agent_logs.last_mut() {
                    *last = (LogStatus::Done, format!("Scout: {} files identified", files.len()));
                }
                self.agent_phase = AgentPhase::Coding;
            }
            AgentMessage::CoderStarted(file) => {
                self.agent_logs.push((LogStatus::Pending, format!("Coder: {}", file)));
                self.messages.push(ChatMsg {
                    kind: MsgKind::Diff,
                    text: String::new(),
                });
                self.is_streaming = true;
                self.agent_phase = AgentPhase::Coding;
            }
            AgentMessage::CoderUpdate(text) => {
                if let Some(last) = self.messages.last_mut() {
                    last.text.push_str(&text);
                }
                self.chat_scroll = 0;
            }
            AgentMessage::CoderCompleted(file, _) => {
                self.is_streaming = false;
                if let Some(last) = self.agent_logs.last_mut() {
                    *last = (LogStatus::Done, format!("Coder: {} done", file));
                }
            }
            AgentMessage::ReviewStarted(file) => {
                self.agent_logs.push((LogStatus::Pending, format!("Review: {}", file)));
                self.agent_phase = AgentPhase::Reviewing;
            }
            AgentMessage::ReviewPassed(file) => {
                if let Some(last) = self.agent_logs.last_mut() {
                    *last = (LogStatus::Done, format!("Review: {} passed", file));
                }
                self.agent_phase = AgentPhase::Complete;
            }
            AgentMessage::ReviewFailed(file, err) => {
                if let Some(last) = self.agent_logs.last_mut() {
                    *last = (LogStatus::Failed, format!("Review: {} failed", file));
                }
                self.messages.push(ChatMsg {
                    kind: MsgKind::Error,
                    text: format!("{}: {}", file, err),
                });
            }
            AgentMessage::ToolStarted { name, description } => {
                self.tool_entries.push(ToolEntry {
                    name: name.clone(),
                    description,
                    status: ToolStatus::Executing,
                    output_lines: Vec::new(),
                });
                self.agent_logs.push((LogStatus::Pending, format!("[{}] started", name)));
            }
            AgentMessage::ToolOutput { name, line } => {
                if let Some(entry) = self.tool_entries.iter_mut().rev().find(|e| e.name == name) {
                    entry.output_lines.push(line);
                }
            }
            AgentMessage::ToolCompleted { name, status } => {
                if let Some(entry) = self.tool_entries.iter_mut().rev().find(|e| e.name == name) {
                    entry.status = status.clone();
                }
                let log_status = match status {
                    ToolStatus::Success => LogStatus::Done,
                    ToolStatus::Error => LogStatus::Failed,
                    _ => LogStatus::Info,
                };
                if let Some(last) = self.agent_logs.last_mut() {
                    *last = (log_status, format!("[{}] completed", name));
                }
            }
            AgentMessage::ShellRequested { command, destructive } => {
                if destructive {
                    self.show_approval = true;
                    self.approval_command = command;
                    self.approval_selection = ModalSelection::Approve;
                } else {
                    self.agent_logs.push((LogStatus::Pending, format!("[SHELL] {}", command)));
                }
            }
            AgentMessage::ShellApproved(cmd) => {
                self.agent_logs.push((LogStatus::Done, format!("[SHELL] approved: {}", cmd)));
            }
            AgentMessage::ShellDenied(cmd) => {
                self.agent_logs.push((LogStatus::Failed, format!("[SHELL] denied: {}", cmd)));
            }
            AgentMessage::ShellOutput(line) => {
                if let Some(entry) = self.tool_entries.last_mut() {
                    entry.output_lines.push(line);
                }
            }
            AgentMessage::ShellCompleted { exit_code } => {
                let status = if exit_code == 0 { LogStatus::Done } else { LogStatus::Failed };
                self.agent_logs.push((status, format!("[SHELL] exit {}", exit_code)));
            }
            AgentMessage::Thinking(text) => {
                if text.starts_with("Memory Hit:") {
                    self.memory_hit = true;
                }
                self.thinking_lines.push(text);
            }
            AgentMessage::PhaseChange(phase) => {
                self.agent_phase = phase;
            }
            AgentMessage::ImageAttached { path, mime } => {
                self.image_tags.push(image_view::ImageTag {
                    full_path: path.clone(),
                    filename: std::path::Path::new(&path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(&path)
                        .to_string(),
                    mime,
                });
            }
            AgentMessage::Error(e) => {
                self.messages.push(ChatMsg {
                    kind: MsgKind::Error,
                    text: e,
                });
                self.is_streaming = false;
            }
            AgentMessage::IndexingStarted => {
                self.is_indexing = true;
            }
            AgentMessage::IndexingCompleted => {
                self.is_indexing = false;
            }
        }
    }

    // ── Google OAuth Login ─────────────────────────────────────────────────
    async fn run_google_oauth_login() -> anyhow::Result<()> {
        let oauth_config = auth::OAuthFlowConfig::google();
        let pkce = auth::PKCEParams::generate();
        let (port, code_rx) = auth::start_callback_server(&pkce.state).await?;
        let auth_url = auth::build_auth_url(&oauth_config, &pkce, port);
        auth::open_browser(&auth_url)?;
        let code = code_rx.await.map_err(|_| anyhow::anyhow!("OAuth callback cancelled"))?;
        let token = auth::exchange_code_for_token(&oauth_config, &code, &pkce.code_verifier, port).await?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let creds = auth::StoredCredentials {
            auth_type: auth::AuthType::GoogleOAuth,
            token: Some(token),
            api_key: None,
            updated_at: now,
        };
        auth::save_credentials(&creds)?;
        Ok(())
    }

    // ── OpenRouter OAuth Login ───────────────────────────────────────────────
    async fn run_openrouter_oauth_login() -> anyhow::Result<()> {
        auth::run_openrouter_oauth().await?;
        Ok(())
    }

    fn spinner(&self) -> &str {
        theme::ICON_SPINNER_FRAMES[(self.tick / 2) % theme::ICON_SPINNER_FRAMES.len()]
    }

    // ════════════════════════════════════════════════════════════════════════
    //  UI RENDERING
    // ════════════════════════════════════════════════════════════════════════

    fn ui(&self, f: &mut Frame) {
        let area = f.size();

        // Calculate dynamic heights
        let help_height = if self.show_help { 6u16 } else { 0 };
        let mode_height = 1u16; // Mode bar is always 1 line

        // ── Root: Header | Body | ModeBar | Input | Help | StatusBar ──
        let root = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),           // Header with padding
                Constraint::Min(8),              // Body
                Constraint::Length(mode_height),  // Mode bar above input
                Constraint::Length(3),            // Input
                Constraint::Length(help_height),  // Help panel (toggled)
                Constraint::Length(1),            // Status bar
            ])
            .split(area);

        self.render_header(f, root[0]);
        self.render_body(f, root[1]);
        self.render_mode_bar(f, root[2]);
        self.render_input(f, root[3]);
        if self.show_help {
            self.render_help_panel(f, root[4]);
        }
        self.render_status_bar(f, root[5]);

        // ── Modal Overlays ──────────────────────────────────────────────
        if self.show_approval {
            f.render_widget(
                ApprovalModal::new(&self.approval_command)
                    .selected(self.approval_selection.clone()),
                area,
            );
        }

        // Auth sub-menu overlay
        if self.show_auth_menu {
            self.render_auth_menu(f, area);
        }

        // Model sub-menu overlay
        if self.show_model_menu {
            self.render_model_menu(f, area);
        }

        // Command Palette overlay (renders on top of everything)
        if self.palette.visible {
            f.render_widget(CommandPalette::new(&self.palette), area);
        }
    }

    // ── Header ──────────────────────────────────────────────────────────
    fn render_header(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::BOTTOM)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme::BORDER_DIM))
            .style(Style::default().bg(theme::BG_BASE));

        let inner = block.inner(area);
        f.render_widget(block, area);

        let mut spans = vec![
            Span::styled(
                " 🎩 guv ",
                Style::default()
                    .fg(theme::BG_BASE)
                    .bg(theme::CHARM_PINK)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  {}  ", self.repo_path.display()),
                Style::default().fg(theme::FG_MUTED),
            ),
        ];

        if self.is_streaming {
            spans.push(Span::styled(
                format!(" {} working... ", self.spinner()),
                Style::default()
                    .fg(theme::CODEBUFF_CYAN)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        // Model indicator in header
        spans.push(Span::styled(
            format!(" {} ", self.config.model.display_name()),
            Style::default().fg(theme::FG_MUTED).add_modifier(Modifier::DIM),
        ));

        // Image tags in header
        for tag in &self.image_tags {
            spans.push(Span::raw("  "));
            spans.push(Span::styled(
                format!("{} [{}]", theme::ICON_IMAGE, tag.filename),
                Style::default().fg(theme::ACCENT_PRIMARY),
            ));
        }

        let header_line = Paragraph::new(Line::from(spans))
            .style(Style::default().bg(theme::BG_BASE));

        // Add padding: render inside the inner area offset by 1
        if inner.height > 0 {
            let padded = Rect {
                x: inner.x + 1,
                y: inner.y,
                width: inner.width.saturating_sub(2),
                height: 1,
            };
            f.render_widget(header_line, padded);
        }
    }

    // ── Body: Adaptive column layout (tools toggleable) ────────────────
    fn render_body(&self, f: &mut Frame, area: Rect) {
        if self.show_tools {
            // 3-column: Chat + Stepper | Tool Output | Sidebar
            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(45),
                    Constraint::Percentage(30),
                    Constraint::Percentage(25),
                ])
                .split(area);

            self.render_chat_pane(f, cols[0]);
            self.render_tool_output(f, cols[1]);
            self.render_thinking_sidebar(f, cols[2]);
        } else {
            // 2-column: Chat + Stepper | Sidebar (tools hidden)
            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(70),
                    Constraint::Percentage(30),
                ])
                .split(area);

            self.render_chat_pane(f, cols[0]);
            self.render_thinking_sidebar(f, cols[1]);
        }
    }

    // ── Chat Pane (Stepper + Messages) ──────────────────────────────────
    fn render_chat_pane(&self, f: &mut Frame, area: Rect) {
        let is_focused = self.focus == FocusPane::Chat;
        let border_color = if is_focused { theme::BORDER_FOCUS } else { theme::BORDER_DIM };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(
                " Chat ",
                if is_focused { theme::accent() } else { theme::muted() },
            ))
            .style(Style::default().bg(theme::BG_BASE));

        let inner = block.inner(area);
        f.render_widget(block, area);

        if inner.height < 3 || inner.width < 5 {
            return;
        }

        // Split inner: Stepper (top) + Chat messages (bottom)
        let show_stepper = !matches!(self.agent_phase, AgentPhase::Idle);
        let stepper_height = if show_stepper { 9u16 } else { 0 };

        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(stepper_height.min(inner.height / 3)),
                Constraint::Min(2),
            ])
            .split(inner);

        // Agent Stepper
        if show_stepper && inner_chunks[0].height > 0 {
            let steps = stepper::steps_from_phase(&self.agent_phase);
            let stepper_widget = AgentStepper::new(&steps).tick(self.tick);
            f.render_widget(stepper_widget, inner_chunks[0].inner(&Margin { vertical: 0, horizontal: 1 }));
        }

        // Chat messages with padding
        let chat_area = inner_chunks[1].inner(&Margin { vertical: 0, horizontal: 1 });
        self.render_chat_messages(f, chat_area);
    }

    fn render_chat_messages(&self, f: &mut Frame, area: Rect) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let mut lines: Vec<Line> = Vec::new();
        for msg in &self.messages {
            if msg.text.is_empty() {
                continue;
            }

            match msg.kind {
                MsgKind::User => {
                    lines.push(Line::from(vec![
                        Span::styled(
                            " ▸ ",
                            Style::default().fg(theme::YELLOW),
                        ),
                        Span::styled(
                            &msg.text,
                            Style::default()
                                .fg(theme::FG_BASE)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]));
                }
                MsgKind::Agent => {
                    lines.push(Line::from(vec![
                        Span::styled(
                            " ● ",
                            Style::default().fg(theme::CODEBUFF_CYAN),
                        ),
                        Span::styled(&msg.text, Style::default().fg(theme::FG_BASE)),
                    ]));
                }
                MsgKind::Error => {
                    lines.push(Line::from(vec![
                        Span::styled(
                            " × ",
                            Style::default()
                                .fg(theme::RED)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(&msg.text, Style::default().fg(theme::RED)),
                    ]));
                }
                MsgKind::System => {
                    lines.push(Line::from(vec![
                        Span::styled("   ", Style::default()),
                        Span::styled(
                            &msg.text,
                            Style::default()
                                .fg(theme::FG_SUBTLE)
                                .add_modifier(Modifier::ITALIC),
                        ),
                    ]));
                }
                MsgKind::Diff => {
                    // Inline diff streaming
                    let diff_lines = diff_view::render_streaming_diff_lines(
                        &msg.text,
                        area.height as usize,
                    );
                    lines.extend(diff_lines);
                }
            }
            lines.push(Line::from(""));
        }

        let total_lines = lines.len() as u16;
        let visible = area.height;
        let max_scroll = total_lines.saturating_sub(visible);
        let scroll = self.chat_scroll.min(max_scroll);

        let chat = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((max_scroll.saturating_sub(scroll), 0));

        f.render_widget(chat, area);
    }

    // ── Tool Output Pane (with Available Tools Catalog + Live Execution) ──
    fn render_tool_output(&self, f: &mut Frame, area: Rect) {
        let is_focused = self.focus == FocusPane::ToolOutput;
        let border_color = if is_focused { theme::BORDER_FOCUS } else { theme::BORDER_DIM };

        let active_count = self.tool_entries.iter()
            .filter(|e| matches!(e.status, ToolStatus::Executing))
            .count();
        let total_count = self.tool_entries.len();

        let title_text = if total_count > 0 {
            format!(" Tools ({}/{}) ", active_count, total_count)
        } else {
            format!(" Tools ({}) ", self.available_tools.len())
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(
                title_text,
                if is_focused { theme::accent() } else { theme::muted() },
            ))
            .style(Style::default().bg(theme::BG_BASE));

        if self.tool_entries.is_empty() {
            // Show available tools catalog
            let inner = block.inner(area);
            f.render_widget(block, area);
            if inner.height > 1 && inner.width > 5 {
                let padded = inner.inner(&Margin { vertical: 0, horizontal: 1 });
                let mut lines: Vec<Line> = Vec::new();
                lines.push(Line::from(Span::styled(
                    " Available Tools",
                    Style::default().fg(theme::DOLLY_PURPLE).add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(""));
                for (name, desc) in &self.available_tools {
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("  {} ", theme::ICON_PENDING),
                            Style::default().fg(theme::FG_SUBTLE),
                        ),
                        Span::styled(
                            *name,
                            Style::default().fg(theme::FG_BASE).add_modifier(Modifier::BOLD),
                        ),
                    ]));
                    lines.push(Line::from(vec![
                        Span::styled("    ", Style::default()),
                        Span::styled(
                            *desc,
                            Style::default().fg(theme::FG_MUTED),
                        ),
                    ]));
                }
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    " Tools activate as the agent works.",
                    theme::ghost_hint(),
                )));

                let para = Paragraph::new(lines)
                    .wrap(Wrap { trim: false })
                    .scroll((self.tool_scroll, 0));
                f.render_widget(para, padded);
            }
            return;
        }

        let tool_widget = ToolPane::new(&self.tool_entries)
            .scroll(self.tool_scroll)
            .tick(self.tick)
            .focused(is_focused)
            .block(block);

        f.render_widget(tool_widget, area);
    }

    // ── Right Sidebar (Crush-style: Logo, Model, Files, Activity) ────
    fn render_thinking_sidebar(&self, f: &mut Frame, area: Rect) {
        let is_focused = self.focus == FocusPane::Sidebar;
        let border_color = if is_focused { theme::BORDER_FOCUS } else { theme::BORDER_DIM };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(theme::BG_LIGHTER));

        let inner = block.inner(area);
        f.render_widget(block, area);

        if inner.height == 0 || inner.width < 3 {
            return;
        }

        let padded = inner.inner(&Margin { vertical: 0, horizontal: 1 });
        let w = padded.width as usize;
        let mut lines: Vec<Line> = Vec::new();

        // ── Logo / Brand Block ──────────────────────────────────────
        let diag_count = w.saturating_sub(8);
        let diagonals: String = theme::ICON_DIAGONAL.repeat(diag_count);
        lines.push(Line::from(vec![
            Span::styled(" guv", theme::dolly_bold()),
            Span::styled("code ", Style::default().fg(theme::CHARM_PINK).add_modifier(Modifier::BOLD)),
            Span::styled(diagonals, Style::default().fg(theme::CHARM_PINK)),
        ]));
        lines.push(Line::from(""));

        // ── Project Path ────────────────────────────────────────────
        let dir_name = self.repo_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(".");
        lines.push(Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled(dir_name, theme::sidebar_value()),
        ]));
        lines.push(Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled(
                self.repo_path.display().to_string(),
                theme::sidebar_label(),
            ),
        ]));
        lines.push(Line::from(""));

        // ── Model Info ──────────────────────────────────────────────
        let model_name = if self.config.keys.gemini.is_some() {
            "Gemini 2.5 Pro"
        } else if self.config.keys.anthropic.is_some() {
            "Claude Sonnet"
        } else {
            "No Model"
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!(" {} ", theme::ICON_MODEL),
                Style::default().fg(theme::DOLLY_PURPLE),
            ),
            Span::styled(model_name, theme::sidebar_value()),
        ]));

        // Budget line
        let remaining = self.config.budget.limit - self.config.budget.consumed;
        let budget_color = if remaining < 1.0 {
            theme::RED
        } else if remaining < 3.0 {
            theme::YELLOW
        } else {
            theme::FG_MUTED
        };
        lines.push(Line::from(vec![
            Span::styled("   ", Style::default()),
            Span::styled(
                format!("${:.2} / ${:.2}", self.config.budget.consumed, self.config.budget.limit),
                Style::default().fg(budget_color),
            ),
        ]));

        // Phase indicator
        let phase_str = match self.agent_phase {
            AgentPhase::Idle => "idle",
            AgentPhase::Mapping => "mapping",
            AgentPhase::Planning => "planning",
            AgentPhase::Coding => "coding",
            AgentPhase::Complete => "complete",
            AgentPhase::Reviewing => "reviewing",
        };
        if !matches!(self.agent_phase, AgentPhase::Idle) {
            lines.push(Line::from(vec![
                Span::styled("   ", Style::default()),
                Span::styled(
                    format!("{} {}", self.spinner(), phase_str),
                    Style::default().fg(theme::CODEBUFF_CYAN),
                ),
            ]));
        }
        lines.push(Line::from(""));

        // ── Section: Modified Files ─────────────────────────────────
        self.render_section_header(&mut lines, "Modified Files", w);

        // Extract modified files from tool entries
        let modified: Vec<&str> = self.tool_entries.iter()
            .filter_map(|e| {
                if e.name.contains("edit") || e.name.contains("write") || e.name.contains("create") {
                    Some(e.description.as_str())
                } else {
                    None
                }
            })
            .collect();

        if modified.is_empty() {
            lines.push(Line::from(Span::styled(
                "   No files modified",
                theme::ghost_hint(),
            )));
        } else {
            for path in modified.iter().take(8) {
                let short = path.rsplit('/').next().unwrap_or(path);
                lines.push(Line::from(vec![
                    Span::styled("   ", Style::default()),
                    Span::styled(short, Style::default().fg(theme::FG_BASE)),
                ]));
            }
            if modified.len() > 8 {
                lines.push(Line::from(Span::styled(
                    format!("   +{} more", modified.len() - 8),
                    theme::sidebar_label(),
                )));
            }
        }
        lines.push(Line::from(""));

        // ── Section: Activity ───────────────────────────────────────
        self.render_section_header(&mut lines, "Activity", w);

        let max_logs = 6;
        let log_start = self.agent_logs.len().saturating_sub(max_logs);
        for (status, text) in self.agent_logs[log_start..].iter().rev() {
            let (icon, color) = match status {
                LogStatus::Pending => (self.spinner(), theme::CODEBUFF_CYAN),
                LogStatus::Done => (theme::ICON_CHECK, theme::GREEN),
                LogStatus::Failed => (theme::ICON_CROSS, theme::RED),
                LogStatus::Info => ("·", theme::FG_SUBTLE),
            };
            lines.push(Line::from(vec![
                Span::styled(format!("  {} ", icon), Style::default().fg(color)),
                Span::styled(text, Style::default().fg(color)),
            ]));
        }
        lines.push(Line::from(""));

        // ── Section: Images ─────────────────────────────────────────
        if !self.image_tags.is_empty() {
            self.render_section_header(&mut lines, "Images", w);
            for tag in &self.image_tags {
                lines.push(image_view::render_image_tag(tag, true));

                let path = std::path::Path::new(&tag.full_path);
                let abs_path = if path.is_absolute() {
                    path.to_path_buf()
                } else {
                    self.repo_path.join(path)
                };

                if abs_path.exists() {
                    if let Ok(img) = image::open(&abs_path) {
                        let preview_cols = padded.width.saturating_sub(2).min(40);
                        let preview_rows = 6u16;
                        let rgba = img
                            .resize(
                                preview_cols as u32,
                                (preview_rows * 2) as u32,
                                image::imageops::FilterType::Triangle,
                            )
                            .to_rgba8();
                        let (iw, ih) = rgba.dimensions();
                        let block_img = image_view::BlockCharImage::from_rgba(
                            rgba.as_raw(),
                            iw,
                            ih,
                            preview_cols,
                            preview_rows,
                        );
                        for row in &block_img.cells {
                            let spans: Vec<Span> = row
                                .iter()
                                .map(|(ch, fg, bg)| {
                                    Span::styled(
                                        ch.to_string(),
                                        Style::default().fg(*fg).bg(*bg),
                                    )
                                })
                                .collect();
                            lines.push(Line::from(spans));
                        }
                    }
                }
                lines.push(Line::from(""));
            }
        }

        // ── Section: Thoughts ───────────────────────────────────────
        if !self.thinking_lines.is_empty() {
            self.render_section_header(&mut lines, "Thoughts", w);
            let max_thoughts = 10;
            let start = self.thinking_lines.len().saturating_sub(max_thoughts);
            for line in &self.thinking_lines[start..] {
                lines.push(Line::from(Span::styled(
                    format!("  {}", line),
                    Style::default()
                        .fg(theme::FG_MUTED)
                        .add_modifier(Modifier::ITALIC),
                )));
            }
        }

        let total_lines = lines.len() as u16;
        let visible = padded.height;
        let max_scroll = total_lines.saturating_sub(visible);
        let scroll = self.sidebar_scroll.min(max_scroll);

        let para = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((scroll, 0));
        f.render_widget(para, padded);
    }

    // ── Section Header (Crush-style separator line) ─────────────────
    fn render_section_header<'a>(&self, lines: &mut Vec<Line<'a>>, title: &'a str, width: usize) {
        let title_len = title.len() + 2; // " title "
        let sep_len = width.saturating_sub(title_len + 1);
        let sep: String = theme::ICON_SECTION_SEP.repeat(sep_len);
        lines.push(Line::from(vec![
            Span::styled(format!(" {}", title), theme::section_title()),
            Span::styled(format!(" {}", sep), theme::section_line()),
        ]));
    }

    // ── Mode Bar (above input, matching Crush screenshots) ─────────────
    fn render_mode_bar(&self, f: &mut Frame, area: Rect) {
        if area.height == 0 || area.width < 10 {
            return;
        }

        let spans: Vec<Span> = match self.agent_mode {
            AgentMode::Yolo => vec![
                Span::styled(" YOLO ", theme::mode_yolo()),
                Span::styled("ctrl+y", theme::mode_hint()),
            ],
            AgentMode::AutoAccept => vec![
                Span::styled(" auto-accept edits ", theme::mode_auto_accept()),
                Span::styled("shift+tab to plan", theme::mode_hint()),
            ],
            AgentMode::Plan => vec![
                Span::styled(" plan ", theme::mode_plan()),
                Span::styled("shift+tab to manual", theme::mode_hint()),
            ],
            AgentMode::Manual => vec![
                Span::styled(" shift+tab to accept edits", theme::mode_manual()),
            ],
        };

        let line = Paragraph::new(Line::from(spans))
            .style(Style::default().bg(theme::BG_BASE));
        f.render_widget(line, area);
    }

    // ── Help Panel (below input, toggled by Ctrl+G) ──────────────────
    fn render_help_panel(&self, f: &mut Frame, area: Rect) {
        if area.height == 0 || area.width < 20 {
            return;
        }

        // Grid layout matching Crush screenshot 2 (left-aligned columns)
        let lines: Vec<Line> = vec![
            Line::from(vec![
                Span::styled("tab", theme::help_key()),
                Span::styled("      focus chat      ", theme::help_desc()),
                Span::styled("shift+enter", theme::help_key()),
                Span::styled("  newline       ", theme::help_desc()),
                Span::styled("ctrl+g", theme::help_key()),
                Span::styled(" less", theme::help_desc()),
            ]),
            Line::from(vec![
                Span::styled("/ ", theme::help_key()),
                Span::styled("or ", theme::help_desc()),
                Span::styled("ctrl+p", theme::help_key()),
                Span::styled(" commands    ", theme::help_desc()),
                Span::styled("ctrl+f", theme::help_key()),
                Span::styled("       add image    ", theme::help_desc()),
            ]),
            Line::from(vec![
                Span::styled("ctrl+m", theme::help_key()),
                Span::styled("   models          ", theme::help_desc()),
                Span::styled("@", theme::help_key()),
                Span::styled("            mention file", theme::help_desc()),
            ]),
            Line::from(vec![
                Span::styled("ctrl+s", theme::help_key()),
                Span::styled("   sessions        ", theme::help_desc()),
                Span::styled("ctrl+o", theme::help_key()),
                Span::styled("       open editor  ", theme::help_desc()),
            ]),
            Line::from(vec![
                Span::styled("ctrl+n", theme::help_key()),
                Span::styled("   new session     ", theme::help_desc()),
                Span::styled("ctrl+y", theme::help_key()),
                Span::styled("       yolo mode    ", theme::help_desc()),
            ]),
        ];

        let para = Paragraph::new(lines)
            .style(Style::default().bg(theme::BG_BASE));
        f.render_widget(para, Rect {
            x: area.x + 1, // Left-aligned with small indent
            y: area.y,
            width: area.width.saturating_sub(2),
            height: area.height,
        });
    }

    // ── Auth Sub-Menu (centered overlay) ──────────────────────────────
    fn render_auth_menu(&self, f: &mut Frame, area: Rect) {
        // Center the menu
        let menu_w = 42u16.min(area.width.saturating_sub(4));
        let items = AuthMenuItem::all();
        let menu_h = (items.len() as u16 + 4).min(area.height.saturating_sub(4)); // +4 for border+title+hint
        let x = area.x + (area.width.saturating_sub(menu_w)) / 2;
        let y = area.y + (area.height.saturating_sub(menu_h)) / 2;
        let menu_area = Rect::new(x, y, menu_w, menu_h);

        // Clear background
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme::CHARM_PINK))
            .title(Span::styled(" Auth ", theme::accent()))
            .style(Style::default().bg(theme::BG_SUBTLE));

        let inner = block.inner(menu_area);
        f.render_widget(Clear, menu_area);
        f.render_widget(block, menu_area);

        if inner.height == 0 || inner.width < 5 {
            return;
        }

        let mut lines: Vec<Line> = Vec::new();
        for (i, item) in items.iter().enumerate() {
            let selected = i == self.auth_menu_selected;
            let prefix = if selected { " ▸ " } else { "   " };
            let style = if selected {
                Style::default().fg(theme::FG_BASE).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::FG_MUTED)
            };
            lines.push(Line::from(Span::styled(
                format!("{}{}", prefix, item.label()),
                style,
            )));
        }
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " esc to close",
            theme::ghost_hint(),
        )));

        let para = Paragraph::new(lines);
        f.render_widget(para, inner);
    }

    // ── Model Sub-Menu (centered overlay, two-level) ──────────────────
    fn render_model_menu(&self, f: &mut Frame, area: Rect) {
        let menu_w = 52u16.min(area.width.saturating_sub(4));

        let (title, items): (&str, Vec<(String, String)>) = match &self.model_menu_level {
            ModelMenuLevel::PickProvider => {
                let providers = vec![
                    ("Google".into(), format!("Gemini models ({})", if self.config.keys.gemini.is_some() { "key set" } else { "no key" })),
                    ("Anthropic".into(), format!("Claude models ({})", if self.config.keys.anthropic.is_some() { "key set" } else { "no key" })),
                    ("OpenRouter".into(), format!("Multi-provider ({})", if self.config.keys.openrouter.is_some() { "key set" } else { "no key" })),
                ];
                ("Select Provider", providers)
            }
            ModelMenuLevel::PickModel(provider) => {
                let models = config::models_for_provider(provider);
                let items: Vec<(String, String)> = models.iter().map(|(id, desc)| {
                    let current = if self.config.model.model_id == *id && self.config.model.provider == *provider {
                        " (current)"
                    } else {
                        ""
                    };
                    (format!("{}{}", id, current), desc.to_string())
                }).collect();
                (match provider {
                    config::Provider::Google => "Google Models",
                    config::Provider::Anthropic => "Anthropic Models",
                    config::Provider::OpenRouter => "OpenRouter Models",
                }, items)
            }
        };

        let menu_h = (items.len() as u16 + 5).min(area.height.saturating_sub(4));
        let x = area.x + (area.width.saturating_sub(menu_w)) / 2;
        let y = area.y + (area.height.saturating_sub(menu_h)) / 2;
        let menu_area = Rect::new(x, y, menu_w, menu_h);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme::CODEBUFF_CYAN))
            .title(Span::styled(format!(" {} ", title), theme::accent()))
            .style(Style::default().bg(theme::BG_SUBTLE));

        let inner = block.inner(menu_area);
        f.render_widget(Clear, menu_area);
        f.render_widget(block, menu_area);

        if inner.height == 0 || inner.width < 5 {
            return;
        }

        let mut lines: Vec<Line> = Vec::new();

        // Current model indicator
        lines.push(Line::from(Span::styled(
            format!(" Active: {}", self.config.model.display_name()),
            Style::default().fg(theme::GREEN),
        )));
        lines.push(Line::from(""));

        for (i, (label, desc)) in items.iter().enumerate() {
            let selected = i == self.model_menu_selected;
            let prefix = if selected { " ▸ " } else { "   " };
            if selected {
                lines.push(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(theme::FG_BASE).add_modifier(Modifier::BOLD)),
                    Span::styled(label.as_str(), Style::default().fg(theme::FG_BASE).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("  {}", desc), Style::default().fg(theme::FG_MUTED)),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(theme::FG_MUTED)),
                    Span::styled(label.as_str(), Style::default().fg(theme::FG_MUTED)),
                ]));
            }
        }

        lines.push(Line::from(""));
        let hint = match &self.model_menu_level {
            ModelMenuLevel::PickProvider => " esc to close",
            ModelMenuLevel::PickModel(_) => " esc to go back",
        };
        lines.push(Line::from(Span::styled(hint, theme::ghost_hint())));

        let para = Paragraph::new(lines);
        f.render_widget(para, inner);
    }

    // ── Input (Charm-style ::: prompt) ────────────────────────────────
    fn render_input(&self, f: &mut Frame, area: Rect) {
        let is_focused = self.focus == FocusPane::Input;

        let border_color = if self.is_streaming {
            theme::CODEBUFF_CYAN
        } else if is_focused {
            theme::BORDER_FOCUS
        } else {
            theme::BORDER_DIM
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(theme::BG_BASE));

        let inner = block.inner(area);
        f.render_widget(block, area);

        if inner.height == 0 || inner.width < 8 {
            return;
        }

        // Build input line with ::: prompt prefix (Crush editor style)
        let dots_style = if is_focused {
            theme::prompt_dots_focused()
        } else {
            theme::prompt_dots_blurred()
        };

        let mut spans = vec![
            Span::styled(" ", Style::default()),
            Span::styled(theme::ICON_PROMPT_DOTS, dots_style),
            Span::styled(" ", Style::default()),
        ];

        if self.is_streaming {
            spans.push(Span::styled(
                "Processing...",
                Style::default()
                    .fg(theme::CODEBUFF_CYAN)
                    .add_modifier(Modifier::ITALIC),
            ));
        } else if self.input.is_empty() {
            spans.push(Span::styled(
                "Describe what you want to build...",
                theme::ghost_hint(),
            ));
        } else {
            spans.push(Span::styled(
                &self.input,
                Style::default().fg(theme::FG_BASE),
            ));
        }

        let input_para = Paragraph::new(Line::from(spans));
        f.render_widget(input_para, inner);

        // Cursor positioning (after "::: " prefix = 5 chars)
        if is_focused && !self.is_streaming {
            let prefix_len = 5u16; // " ::: "
            let x = inner.x + prefix_len + self.input[..self.cursor_pos].chars().count() as u16;
            let y = inner.y;
            if x < inner.x + inner.width - 1 {
                f.set_cursor(x, y);
            }
        }
    }

    // ── Status Bar (Persistent) ─────────────────────────────────────────
    fn render_status_bar(&self, f: &mut Frame, area: Rect) {
        let model = if self.config.keys.gemini.is_some() {
            "Gemini 2.5 Pro"
        } else {
            "No Model"
        };

        let mode = if self.is_streaming { "Active" } else { "Standby" };

        let bar = StatusBar::new(
            self.config.budget.consumed,
            self.config.budget.limit,
            model,
            mode,
        )
        .indexing(self.is_indexing)
        .memory_hit(self.memory_hit)
        .streaming(self.is_streaming)
        .help_visible(self.show_help);

        f.render_widget(bar, area);
    }
}

// ════════════════════════════════════════════════════════════════════════════
//  TUI Startup Options (passed from CLI flags, like gemini-cli)
// ════════════════════════════════════════════════════════════════════════════

#[derive(Default, Clone)]
pub struct TuiOptions {
    /// Initial prompt to submit on launch
    pub prompt: Option<String>,
    /// Start in YOLO mode
    pub yolo: bool,
    /// Start in plan-only mode
    pub plan: bool,
    /// Override model name
    pub model: Option<String>,
    /// Show tools pane on startup
    pub show_tools: bool,
}

// ════════════════════════════════════════════════════════════════════════════
//  TUI Entry Point
// ════════════════════════════════════════════════════════════════════════════

pub async fn start_tui(config: Config, opts: TuiOptions) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(config);

    // Apply startup options
    if opts.yolo {
        app.agent_mode = AgentMode::Yolo;
    } else if opts.plan {
        app.agent_mode = AgentMode::Plan;
    }
    if opts.show_tools {
        app.show_tools = true;
    }
    if let Some(ref model_name) = opts.model {
        // Try to resolve provider from model name
        let catalog = config::model_catalog();
        if let Some((provider, id, _)) = catalog.iter().find(|(_, id, _)| *id == model_name.as_str()) {
            app.config.model = config::ModelChoice {
                provider: provider.clone(),
                model_id: id.to_string(),
            };
        } else {
            // Unknown model — assume it's an OpenRouter model if it contains /
            let (provider, model_id) = if model_name.contains('/') {
                (config::Provider::OpenRouter, model_name.clone())
            } else if model_name.starts_with("gemini") {
                (config::Provider::Google, model_name.clone())
            } else if model_name.starts_with("claude") {
                (config::Provider::Anthropic, model_name.clone())
            } else {
                (config::Provider::OpenRouter, model_name.clone())
            };
            app.config.model = config::ModelChoice { provider, model_id };
        }
    }

    let res = app.run(&mut terminal, opts.prompt).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    res
}
