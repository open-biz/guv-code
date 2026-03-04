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
        }
    }

    pub async fn run<B: Backend>(mut self, terminal: &mut Terminal<B>) -> Result<()> {
        let (ui_tx, mut ui_rx) = mpsc::channel(100);

        loop {
            terminal.draw(|f| self.ui(f))?;

            if event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char(c) => {
                            self.input.push(c);
                        }
                        KeyCode::Backspace => {
                            self.input.pop();
                        }
                        KeyCode::Enter => {
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
            AgentMessage::PlanStarted => self.agent_logs.push("Planner: Analyzing...".to_string()),
            AgentMessage::PlanCompleted(files) => self.agent_logs.push(format!("Planner: Selected {} files.", files.len())),
            AgentMessage::CoderStarted(file) => self.agent_logs.push(format!("Coder: Patching {}...", file)),
            AgentMessage::CoderUpdate(_) => {}, // Could animate something
            AgentMessage::CoderCompleted(file, _) => {
                self.agent_logs.push(format!("Coder: Finished {}.", file));
                self.messages.push(format!("Guv: Patch ready for {}.", file));
            },
            AgentMessage::ReviewStarted(file) => self.agent_logs.push(format!("Reviewer: Validating {}...", file)),
            AgentMessage::ReviewPassed(file) => self.agent_logs.push(format!("Reviewer: {} passed build check.", file)),
            AgentMessage::ReviewFailed(file, err) => {
                self.agent_logs.push(format!("Reviewer: {} FAILED.", file));
                self.messages.push(format!("Guv (Error): {} check failed: {}", file, err));
            },
            AgentMessage::Error(e) => self.messages.push(format!("Error: {}", e)),
            _ => {}
        }
    }

    fn ui(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(3),    // Main content
                Constraint::Length(3), // Input
                Constraint::Length(1), // Status
            ])
            .split(f.size());

        // Header
        let header = Paragraph::new(Line::from(vec![
            Span::styled(" 🎩 GUV-Code ", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)),
            Span::raw(" - Right away, Guv'nor. "),
        ]))
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
        f.render_widget(header, chunks[0]);

        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(chunks[1]);

        // Chat History
        let chat_items: Vec<ListItem> = self.messages.iter().rev().take(f.size().height as usize)
            .map(|m| {
                let color = if m.starts_with("User:") { Color::Yellow } else { Color::White };
                ListItem::new(Line::from(Span::styled(m, Style::default().fg(color))))
            }).collect();
        let chat = List::new(chat_items)
            .block(Block::default().borders(Borders::ALL).title(" Chat "))
            .style(Style::default().fg(Color::White));
        f.render_widget(chat, main_chunks[0]);

        // Agent Logs
        let log_items: Vec<ListItem> = self.agent_logs.iter().rev().take(f.size().height as usize)
            .map(|m| ListItem::new(Line::from(Span::styled(m, Style::default().fg(Color::DarkGray)))))
            .collect();
        let logs = List::new(log_items)
            .block(Block::default().borders(Borders::ALL).title(" Agent Activity "))
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(logs, main_chunks[1]);

        // Input
        let input = Paragraph::new(self.input.as_str())
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL).title(" Guv'nor? "));
        f.render_widget(input, chunks[2]);

        // Status
        let status_text = format!(" [ Repo: {} ] [ Gemini: {} ] [ Anthropic: {} ] ", 
            self.repo_path.file_name().unwrap_or_default().to_string_lossy(),
            if self.config.keys.gemini.is_some() { "OK" } else { "MISSING" },
            if self.config.keys.anthropic.is_some() { "OK" } else { "MISSING" }
        );
        let status = Paragraph::new(Span::styled(status_text, Style::default().bg(Color::Blue).fg(Color::White)));
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
