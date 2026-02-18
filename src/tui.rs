use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

pub struct TuiState {
    pub messages: Vec<String>,
    pub agent_logs: Vec<String>,
    pub input: String,
    pub status: String,
}

impl TuiState {
    pub fn new() -> Self {
        Self {
            messages: vec!["Guv'nor: Ready to serve.".to_string()],
            agent_logs: vec!["System: Awaiting orders...".to_string()],
            input: String::new(),
            status: "[ Budget: $10.00 / $10.00 ]".to_string(),
        }
    }
}

pub fn run_tui() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let mut state = TuiState::new();
    let res = run_loop(&mut terminal, &mut state);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_loop<B: Backend>(terminal: &mut ratatui::Terminal<B>, state: &mut TuiState) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, state))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char(c) => {
                    state.input.push(c);
                }
                KeyCode::Backspace => {
                    state.input.pop();
                }
                KeyCode::Enter => {
                    let msg = state.input.drain(..).collect::<String>();
                    if msg == "exit" || msg == "quit" {
                        return Ok(());
                    }
                    state.messages.push(format!("User: {}", msg));
                    state.agent_logs.push(format!("Scout: Analyzing for '{}'...", msg));
                }
                KeyCode::Esc => {
                    return Ok(());
                }
                _ => {}
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
        .split(f.area());

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
