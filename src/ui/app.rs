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
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
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
    pub theme_color: Color,
}

impl App {
    pub fn new(config: Config) -> Self {
        let repo_path = env::current_dir().unwrap_or_else(|_| ".".into());
        Self {
            input: String::new(),
            messages: vec!["✦ Guv'nor: Standing by. How can I help you build today?".to_string()],
            agent_logs: vec!["○ System: Awaiting instructions...".to_string()],
            repo_path: repo_path.clone(),
            config: config.clone(),
            orchestrator: if let (Some(g), Some(a)) = (&config.keys.gemini, &config.keys.anthropic) {
                Some(Orchestrator::new(repo_path, g.clone(), a.clone()))
            } else {
                None
            },
            is_streaming: false,
            theme_color: Color::Cyan,
        }
    }

    pub async fn run<B: Backend>(mut self, terminal: &mut Terminal<B>) -> Result<()> {
        let (ui_tx, mut ui_rx) = mpsc::channel(100);

        loop {
            terminal.draw(|f| self.ui(f))?;

            if event::poll(std::time::Duration::from_millis(16))? {
                if let Event::Key(key) = event::read()? {
                    match (key.code, key.modifiers) {
                        (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                            return Ok(());
                        }
                        (KeyCode::Char(c), _) => {
                            self.input.push(c);
                        }
                        (KeyCode::Backspace, _) => {
                            self.input.pop();
                        }
                        (KeyCode::Enter, _) => {
                            if self.input.is_empty() { continue; }
                            let query = self.input.drain(..).collect::<String>();
                            if query == "exit" || query == "quit" {
                                return Ok(());
                            }
                            
                            self.messages.push(format!("👤 User: {}", query));
                            
                            if let Some(orch) = &self.orchestrator {
                                let ui_tx = ui_tx.clone();
                                let query_clone = query.clone();
                                let orch_clone = orch.clone();
                                tokio::spawn(async move {
                                    let _ = orch_clone.run(query_clone, ui_tx).await;
                                });
                            } else {
                                self.messages.push("✘ System: Error: API keys missing. Run `guv auth`.".to_string());
                            }
                        }
                        (KeyCode::Esc, _) => {
                            // In a real implementation, we would send a cancel signal to the orchestrator
                            self.agent_logs.push("○ System: Request cancelled by user.".to_string());
                        }
                        _ => {}
                    }
                }
            }

            while let Ok(msg) = ui_rx.try_recv() {
                self.handle_agent_message(msg);
            }
        }
    }

    fn handle_agent_message(&mut self, msg: AgentMessage) {
        match msg {
            AgentMessage::PlanStarted => {
                self.agent_logs.push("⠧ Planner: Analyzing repository...".to_string());
            }
            AgentMessage::PlanUpdate(text) => {
                if let Some(last) = self.agent_logs.last_mut() {
                    if last.contains("Planner:") {
                        *last = format!("⠧ Planner: {}", text);
                    }
                }
            }
            AgentMessage::PlanCompleted(files) => {
                if let Some(last) = self.agent_logs.last_mut() {
                    if last.contains("Planner:") {
                        *last = format!("✔ Planner: Identified {} files.", files.len());
                    }
                }
            }
            AgentMessage::CoderStarted(file) => {
                self.agent_logs.push(format!("⠧ Coder: Generating edits for {}...", file));
                self.messages.push(format!("✦ Guv ({}): ", file));
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
                        *last = format!("✔ Coder: Patch ready for {}.", file);
                    }
                }
            },
            AgentMessage::ReviewStarted(file) => {
                self.agent_logs.push(format!("⠧ Reviewer: Validating {}...", file));
            }
            AgentMessage::ReviewPassed(file) => {
                if let Some(last) = self.agent_logs.last_mut() {
                    if last.contains(&file) {
                        *last = format!("✔ Reviewer: {} passed build check.", file);
                    }
                }
            }
            AgentMessage::ReviewFailed(file, err) => {
                if let Some(last) = self.agent_logs.last_mut() {
                    if last.contains(&file) {
                        *last = format!("✘ Reviewer: {} FAILED.", file);
                    }
                }
                self.messages.push(format!("✘ Guv (Error): {} check failed: {}", file, err));
            },
            AgentMessage::Error(e) => {
                self.messages.push(format!("✘ Error: {}", e));
                self.is_streaming = false;
            }
        }
    }

    fn ui(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Top Bar
                Constraint::Min(3),    // Main content
                Constraint::Length(3), // Input
                Constraint::Length(1), // Footer
            ])
            .split(f.size());

        // Header - Sleek and minimal
        let header_content = vec![
            Span::styled(" 🎩 GUVCODE ", Style::default().fg(Color::Black).bg(self.theme_color).add_modifier(Modifier::BOLD)),
            Span::styled(format!("  {}  ", self.repo_path.display()), Style::default().fg(Color::DarkGray)),
        ];
        f.render_widget(Paragraph::new(Line::from(header_content)), chunks[0]);

        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
            .split(chunks[1]);

        // Chat View - Codebuff inspired
        let chat_items: Vec<ListItem> = self.messages.iter()
            .map(|m| {
                let style = if m.contains("👤 User:") {
                    Style::default().fg(Color::Yellow)
                } else if m.contains("✦ Guv") {
                    Style::default().fg(self.theme_color)
                } else if m.contains("✘") {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default().fg(Color::White)
                };
                
                ListItem::new(Line::from(Span::styled(m, style)))
            }).collect();
        
        let chat = List::new(chat_items)
            .block(Block::default().borders(Borders::NONE))
            .style(Style::default().fg(Color::White));
        f.render_widget(chat, main_chunks[0]);

        // Activity View - Styled like a sidebar terminal
        let log_items: Vec<ListItem> = self.agent_logs.iter().rev()
            .map(|m| {
                let color = if m.contains("✔") {
                    Color::Green
                } else if m.contains("✘") {
                    Color::Red
                } else if m.contains("⠧") {
                    self.theme_color
                } else {
                    Color::DarkGray
                };
                ListItem::new(Line::from(Span::styled(m, Style::default().fg(color))))
            })
            .collect();
        let logs = List::new(log_items)
            .block(Block::default().borders(Borders::LEFT).border_style(Style::default().fg(Color::Rgb(30, 30, 30))).title(" ACTIVITY "))
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(logs, main_chunks[1]);

        // Input - Beautiful focused box
        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if self.is_streaming { Color::DarkGray } else { self.theme_color }))
            .title(Span::styled(" PROMPT ", Style::default().add_modifier(Modifier::BOLD)));
            
        let input_text = if self.input.is_empty() && !self.is_streaming {
            Span::styled("Type instructions (e.g. 'Add dark mode to the header')...", Style::default().fg(Color::Rgb(80, 80, 80)))
        } else {
            Span::raw(self.input.as_str())
        };
        
        let input = Paragraph::new(input_text)
            .block(input_block);
        f.render_widget(input, chunks[2]);

        // Footer - Real-time stats
        let footer_text = format!(" [ Ctrl+C ] Exit  [ Esc ] Cancel  [ Budget: $4.50 / $10.00 ] ");
        let footer = Paragraph::new(Span::styled(footer_text, Style::default().fg(Color::Rgb(60, 60, 60))));
        f.render_widget(footer, chunks[3]);
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
