use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::llm::{GeminiProvider, AnthropicProvider};
use crate::index::RepoIndex;
use crate::agent::{ScoutAgent, CoderAgent};
use crate::config::Config;
use std::path::PathBuf;
use std::env;
use anyhow::Result;

pub struct TuiState {
    pub messages: Vec<String>,
    pub agent_logs: Vec<String>,
    pub input: String,
    pub status: String,
    pub repo_path: PathBuf,
}

impl TuiState {
    pub fn new() -> Self {
        let repo_path = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            messages: vec!["Guv'nor: Ready to serve.".to_string()],
            agent_logs: vec!["System: Awaiting orders...".to_string()],
            input: String::new(),
            status: "[ Budget: $10.00 / $10.00 ]".to_string(),
            repo_path,
        }
    }
}

pub async fn run_tui() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let mut state = TuiState::new();
    let res = run_loop(&mut terminal, &mut state).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    res?;
    Ok(())
}

async fn run_loop<B: Backend>(terminal: &mut ratatui::Terminal<B>, state: &mut TuiState) -> Result<()> {
    let config = Config::load()?;
    let gemini_key = config.keys.gemini.clone().unwrap_or_default();
    let anthropic_key = config.keys.anthropic.clone().unwrap_or_default();

    loop {
        terminal.draw(|f| ui(f, state))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char(c) => {
                        state.input.push(c);
                    }
                    KeyCode::Backspace => {
                        state.input.pop();
                    }
                    KeyCode::Enter => {
                        let query = state.input.drain(..).collect::<String>();
                        if query == "exit" || query == "quit" {
                            return Ok(());
                        }
                        
                        state.messages.push(format!("User: {}", query));
                        state.agent_logs.push(format!("System: Processing..."));
                        terminal.draw(|f| ui(f, state))?;

                        // Process request
                        let scout_provider = GeminiProvider::new(gemini_key.clone());
                        let coder_provider = AnthropicProvider::new(anthropic_key.clone());

                        let scout = ScoutAgent::new(&scout_provider);
                        let coder = CoderAgent::new(&coder_provider);

                        state.agent_logs.push("Index: Updating...".to_string());
                        let mut index = RepoIndex::load_or_create(&state.repo_path)?;
                        index.update(&state.repo_path)?;
                        index.save(&state.repo_path)?;

                        state.agent_logs.push("Scout: Identifying files...".to_string());
                        let relevant_files = scout.find_files(&index, &query).await?;
                        
                        state.agent_logs.push(format!("Scout: Found {} files.", relevant_files.len()));
                        state.agent_logs.push("Coder: Generating edits...".to_string());
                        
                        let mut file_contents = Vec::new();
                        for path_str in relevant_files {
                            let path = state.repo_path.join(&path_str);
                            if path.exists() {
                                let content = std::fs::read_to_string(path)?;
                                file_contents.push((path_str, content));
                            }
                        }

                        let edits = coder.generate_edits(&query, file_contents).await?;
                        state.messages.push(format!("Guv: Edits generated for your review."));
                        state.agent_logs.push("Coder: Done.".to_string());
                    }
                    KeyCode::Esc => {
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut Frame, state: &TuiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(f.size());

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[0]);

    // Chat Window
    let chat_items: Vec<ListItem> = state
        .messages
        .iter()
        .map(|m| {
            let content = Line::from(Span::raw(m));
            ListItem::new(content)
        })
        .collect();
    let chat = List::new(chat_items)
        .block(Block::default().borders(Borders::ALL).title(" 🎩 GUV Chat "))
        .style(Style::default().fg(Color::White));
    f.render_widget(chat, main_chunks[0]);

    // Agent Window
    let log_items: Vec<ListItem> = state
        .agent_logs
        .iter()
        .map(|m| {
            let content = Line::from(Span::styled(m, Style::default().fg(Color::Cyan)));
            ListItem::new(content)
        })
        .collect();
    let logs = List::new(log_items)
        .block(Block::default().borders(Borders::ALL).title(" 🤖 Agent Activity "))
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(logs, main_chunks[1]);

    // Input Window
    let input = Paragraph::new(state.input.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title(" Prompt (Guv'nor?) "));
    f.render_widget(input, chunks[1]);

    // Status Bar
    let status = Paragraph::new(state.status.as_str())
        .style(Style::default().add_modifier(Modifier::BOLD).bg(Color::Blue).fg(Color::White));
    f.render_widget(status, chunks[2]);
}
