use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tokio::sync::mpsc;
use crate::agent_logic::AgentMessage;
use crate::orchestrator::Orchestrator;
use crate::config::Config;
use anyhow::Result;
use std::env;

pub struct App {
    pub input: String,
    pub messages: Vec<String>,
    pub agent_logs: Vec<String>,
    pub repo_path: std::path::PathBuf,
    pub config: Config,
    pub orchestrator: Option<Orchestrator>,
    pub is_streaming: bool,
}

impl App {
    pub fn new(config: Config) -> Self {
        let repo_path = env::current_dir().unwrap_or_else(|_| ".".into());
        Self {
            input: String::new(),
            messages: vec!["Guv'nor: Ready to serve.".to_string()],
            agent_logs: vec!["System: Awaiting orders...".to_string()],
            repo_path: repo_path.clone(),
            config: config.clone(),
            orchestrator: if let (Some(g), Some(a)) = (&config.keys.gemini, &config.keys.anthropic) {
                Some(Orchestrator::new(repo_path, g.clone(), a.clone()))
            } else {
                None
            },
            is_streaming: false,
        }
    }

    pub async fn run<B: Backend>(mut self, terminal: &mut Terminal<B>) -> Result<()> {
        let (ui_tx, mut ui_rx) = mpsc::channel(100);

        loop {
            terminal.draw(|f| self.ui(f))?;

            if event::poll(std::time::Duration::from_millis(16))? { // ~60fps poll
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char(c) => {
                            self.input.push(c);
                        }
                        KeyCode::Backspace => {
                            self.input.pop();
                        }
                        KeyCode::Enter => {
                            if self.input.is_empty() { continue; }
                            let query = self.input.drain(..).collect::<String>();
                            if query == "exit" || query == "quit" {
                                return Ok(());
                            }
                            
                            self.messages.push(format!("User: {}", query));
                            
                            if let Some(orch) = &self.orchestrator {
                                let ui_tx = ui_tx.clone();
                                let query_clone = query.clone();
                                let orch_clone = orch.clone();
                                tokio::spawn(async move {
                                    let _ = orch_clone.run(query_clone, ui_tx).await;
                                });
                            } else {
                                self.messages.push("System: Error: API keys missing. Use `guv auth`.".to_string());
                            }
                        }
                        KeyCode::Esc => {
                            // Cancel logic could be added here
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            }

            // Non-blocking recv for agent messages
            while let Ok(msg) = ui_rx.try_recv() {
                self.handle_agent_message(msg);
            }
        }
    }

    fn handle_agent_message(&mut self, msg: AgentMessage) {
        match msg {
            AgentMessage::PlanStarted => {
                self.agent_logs.push("✔ Planner: Analyzing...".to_string());
            }
            AgentMessage::PlanCompleted(files) => {
                self.agent_logs.push(format!("✔ Planner: Identified {} files.", files.len()));
            }
            AgentMessage::CoderStarted(file) => {
                self.agent_logs.push(format!("⠧ Coder: Patching {}...", file));
                self.messages.push(format!("Guv ({}): ", file));
                self.is_streaming = true;
            }
            AgentMessage::CoderUpdate(text) => {
                if let Some(last) = self.messages.last_mut() {
                    last.push_str(&text);
                }
            }
            AgentMessage::CoderCompleted(file, _) => {
                self.is_streaming = false;
                if let Some(last) = self.agent_logs.last_mut() {
                    if last.contains(&file) {
                        *last = format!("✔ Coder: Finished {}.", file);
                    }
                }
            },
            AgentMessage::ReviewStarted(file) => {
                self.agent_logs.push(format!("⠧ Reviewer: Validating {}...", file));
            }
            AgentMessage::ReviewPassed(file) => {
                if let Some(last) = self.agent_logs.last_mut() {
                    if last.contains(&file) {
                        *last = format!("✔ Reviewer: {} is clean.", file);
                    }
                }
            }
            AgentMessage::ReviewFailed(file, err) => {
                if let Some(last) = self.agent_logs.last_mut() {
                    if last.contains(&file) {
                        *last = format!("✘ Reviewer: {} FAILED.", file);
                    }
                }
                self.messages.push(format!("Guv (Error): {} check failed: {}", file, err));
            },
            AgentMessage::Error(e) => {
                self.messages.push(format!("Error: {}", e));
                self.is_streaming = false;
            }
        }
    }

    fn ui(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Header
                Constraint::Min(3),    // Main content
                Constraint::Length(3), // Input
                Constraint::Length(1), // Status
            ])
            .split(f.size());

        // Header (Crush-style minimal)
        let header = Paragraph::new(Span::styled(" 🎩 GUV-Code v0.2.0 ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
        f.render_widget(header, chunks[0]);

        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(chunks[1]);

        // Chat History (Codebuff-style with clear separators)
        let chat_items: Vec<ListItem> = self.messages.iter()
            .map(|m| {
                let (prefix, color) = if m.starts_with("User:") {
                    (" 👤 ", Color::Yellow)
                } else if m.starts_with("Guv (") {
                    (" 🪄 ", Color::Cyan)
                } else {
                    (" 🎩 ", Color::White)
                };
                
                ListItem::new(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(color)),
                    Span::styled(m, Style::default().fg(Color::White)),
                ]))
            }).collect();
        
        let chat = List::new(chat_items)
            .block(Block::default().borders(Borders::LEFT).border_style(Style::default().fg(Color::DarkGray)).title(" Conversation "))
            .style(Style::default().fg(Color::White));
        f.render_widget(chat, main_chunks[0]);

        // Agent Activity (Sleek right sidebar)
        let log_items: Vec<ListItem> = self.agent_logs.iter().rev()
            .map(|m| {
                let color = if m.starts_with("✔") { Color::Green } else if m.starts_with("✘") { Color::Red } else { Color::Blue };
                ListItem::new(Line::from(Span::styled(m, Style::default().fg(color).add_modifier(Modifier::DIM))))
            })
            .collect();
        let logs = List::new(log_items)
            .block(Block::default().borders(Borders::LEFT).border_style(Style::default().fg(Color::DarkGray)).title(" Activity "))
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(logs, main_chunks[1]);

        // Input Box (Sleek, rounded feel via unicode)
        let input_text = if self.input.is_empty() && !self.is_streaming {
            " Type your instructions, Guv'nor... ".to_string()
        } else {
            self.input.clone()
        };
        let input_style = if self.input.is_empty() { Style::default().fg(Color::DarkGray) } else { Style::default().fg(Color::Yellow) };
        
        let input = Paragraph::new(input_text)
            .style(input_style)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)).title(" Prompt "));
        f.render_widget(input, chunks[2]);

        // Status Line
        let status_text = format!(" [ Repo: {} ] [ Gemini: Pro ] [ Opus: Active ] ", 
            self.repo_path.file_name().unwrap_or_default().to_string_lossy());
        let status = Paragraph::new(Span::styled(status_text, Style::default().fg(Color::DarkGray)));
        f.render_widget(status, chunks[3]);
    }
}

pub async fn start_tui(config: Config) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new(config);
    let res = app.run(&mut terminal).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    res
}
