use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
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

const ACCENT: Color = Color::Cyan;
const DIM: Color = Color::Rgb(80, 80, 80);
const BORDER_DIM: Color = Color::Rgb(40, 40, 40);
const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

#[derive(Clone)]
enum MsgKind {
    User,
    Agent,
    Error,
    System,
}

#[derive(Clone)]
struct ChatMsg {
    kind: MsgKind,
    text: String,
}

pub struct App {
    input: String,
    cursor_pos: usize,
    messages: Vec<ChatMsg>,
    agent_logs: Vec<(LogStatus, String)>,
    chat_scroll: u16,
    repo_path: std::path::PathBuf,
    config: Config,
    orchestrator: Option<Orchestrator>,
    is_streaming: bool,
    tick: usize,
}

#[derive(Clone, Copy)]
enum LogStatus {
    Pending,
    Done,
    Failed,
    Info,
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
            agent_logs: vec![(LogStatus::Info, "Awaiting instructions".into())],
            chat_scroll: 0,
            repo_path,
            config,
            orchestrator,
            is_streaming: false,
            tick: 0,
        }
    }

    pub async fn run<B: Backend>(mut self, terminal: &mut Terminal<B>) -> Result<()> {
        let (ui_tx, mut ui_rx) = mpsc::channel(100);

        loop {
            self.tick = self.tick.wrapping_add(1);
            terminal.draw(|f| self.ui(f))?;

            if event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    match (key.code, key.modifiers) {
                        (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Ok(()),
                        (KeyCode::Char('d'), KeyModifiers::CONTROL) => return Ok(()),
                        (KeyCode::Char(c), _) if !self.is_streaming => {
                            self.input.insert(self.cursor_pos, c);
                            self.cursor_pos += c.len_utf8();
                        }
                        (KeyCode::Backspace, _) if !self.is_streaming => {
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
                        (KeyCode::Delete, _) if !self.is_streaming => {
                            if self.cursor_pos < self.input.len() {
                                self.input.remove(self.cursor_pos);
                            }
                        }
                        (KeyCode::Left, _) => {
                            if self.cursor_pos > 0 {
                                let prev = self.input[..self.cursor_pos]
                                    .chars()
                                    .last()
                                    .map(|c| c.len_utf8())
                                    .unwrap_or(0);
                                self.cursor_pos -= prev;
                            }
                        }
                        (KeyCode::Right, _) => {
                            if self.cursor_pos < self.input.len() {
                                let next = self.input[self.cursor_pos..]
                                    .chars()
                                    .next()
                                    .map(|c| c.len_utf8())
                                    .unwrap_or(0);
                                self.cursor_pos += next;
                            }
                        }
                        (KeyCode::Home, _) => self.cursor_pos = 0,
                        (KeyCode::End, _) => self.cursor_pos = self.input.len(),
                        (KeyCode::Up, _) => {
                            self.chat_scroll = self.chat_scroll.saturating_add(1);
                        }
                        (KeyCode::Down, _) => {
                            self.chat_scroll = self.chat_scroll.saturating_sub(1);
                        }
                        (KeyCode::Enter, _) if !self.is_streaming => {
                            if self.input.is_empty() {
                                continue;
                            }
                            let query: String = self.input.drain(..).collect();
                            self.cursor_pos = 0;
                            self.chat_scroll = 0;

                            if query == "exit" || query == "quit" || query == "q" {
                                return Ok(());
                            }

                            self.messages.push(ChatMsg {
                                kind: MsgKind::User,
                                text: query.clone(),
                            });

                            if let Some(orch) = &self.orchestrator {
                                let tx = ui_tx.clone();
                                let q = query.clone();
                                let o = orch.clone();
                                tokio::spawn(async move {
                                    let _ = o.run(q, tx).await;
                                });
                            } else {
                                self.messages.push(ChatMsg {
                                    kind: MsgKind::Error,
                                    text: "API keys not configured. Run `guv auth`.".into(),
                                });
                            }
                        }
                        (KeyCode::Esc, _) => {
                            if self.is_streaming {
                                self.agent_logs.push((LogStatus::Info, "Cancelled by user".into()));
                                self.is_streaming = false;
                            }
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
                self.is_streaming = true;
                self.agent_logs.push((LogStatus::Pending, "Scout: Analyzing repo...".into()));
            }
            AgentMessage::PlanUpdate(text) => {
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
            }
            AgentMessage::CoderStarted(file) => {
                self.agent_logs.push((LogStatus::Pending, format!("Coder: {}", file)));
                self.messages.push(ChatMsg {
                    kind: MsgKind::Agent,
                    text: String::new(),
                });
                self.is_streaming = true;
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
            }
            AgentMessage::ReviewPassed(file) => {
                if let Some(last) = self.agent_logs.last_mut() {
                    *last = (LogStatus::Done, format!("Review: {} passed", file));
                }
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
            AgentMessage::Error(e) => {
                self.messages.push(ChatMsg {
                    kind: MsgKind::Error,
                    text: e,
                });
                self.is_streaming = false;
            }
        }
    }

    fn spinner(&self) -> &str {
        SPINNER_FRAMES[(self.tick / 2) % SPINNER_FRAMES.len()]
    }

    fn ui(&self, f: &mut Frame) {
        let area = f.size();

        let root = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(4),
                Constraint::Length(3),
                Constraint::Length(1),
            ])
            .split(area);

        self.render_header(f, root[0]);
        self.render_main(f, root[1]);
        self.render_input(f, root[2]);
        self.render_footer(f, root[3]);
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        let mut spans = vec![
            Span::styled(" 🎩 guv ", Style::default().fg(Color::Black).bg(ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" {} ", self.repo_path.display()), Style::default().fg(DIM)),
        ];
        if self.is_streaming {
            spans.push(Span::styled(
                format!(" {} working... ", self.spinner()),
                Style::default().fg(ACCENT),
            ));
        }
        f.render_widget(Paragraph::new(Line::from(spans)), area);
    }

    fn render_main(&self, f: &mut Frame, area: Rect) {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(40), Constraint::Length(30)])
            .split(area);

        self.render_chat(f, cols[0]);
        self.render_activity(f, cols[1]);
    }

    fn render_chat(&self, f: &mut Frame, area: Rect) {
        let inner = Block::default()
            .borders(Borders::NONE)
            .style(Style::default());
        let inner_area = inner.inner(area);

        let mut lines: Vec<Line> = Vec::new();
        for msg in &self.messages {
            let (prefix, style) = match msg.kind {
                MsgKind::User => ("▸ ", Style::default().fg(Color::Yellow)),
                MsgKind::Agent => ("● ", Style::default().fg(ACCENT)),
                MsgKind::Error => ("✘ ", Style::default().fg(Color::Red)),
                MsgKind::System => ("  ", Style::default().fg(DIM)),
            };

            if msg.text.is_empty() {
                continue;
            }

            lines.push(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(&msg.text, style),
            ]));
            lines.push(Line::from(""));
        }

        let total_lines = lines.len() as u16;
        let visible = inner_area.height;
        let max_scroll = total_lines.saturating_sub(visible);
        let scroll = self.chat_scroll.min(max_scroll);

        let chat = Paragraph::new(lines)
            .block(inner)
            .wrap(Wrap { trim: false })
            .scroll((max_scroll.saturating_sub(scroll), 0));

        f.render_widget(chat, area);
    }

    fn render_activity(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(BORDER_DIM))
            .title(Span::styled(" activity ", Style::default().fg(DIM)));

        let items: Vec<ListItem> = self.agent_logs.iter().rev().map(|(status, text)| {
            let (icon, color) = match status {
                LogStatus::Pending => (self.spinner(), ACCENT),
                LogStatus::Done => ("✔", Color::Green),
                LogStatus::Failed => ("✘", Color::Red),
                LogStatus::Info => ("·", DIM),
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {} ", icon), Style::default().fg(color)),
                Span::styled(text, Style::default().fg(color)),
            ]))
        }).collect();

        f.render_widget(List::new(items).block(block), area);
    }

    fn render_input(&self, f: &mut Frame, area: Rect) {
        let title = if self.is_streaming {
            Span::styled(" streaming... ", Style::default().fg(DIM))
        } else {
            Span::styled(" prompt ", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))
        };

        let border_color = if self.is_streaming { BORDER_DIM } else { ACCENT };
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(title);

        let display_text = if self.input.is_empty() && !self.is_streaming {
            Span::styled(
                "Describe what you want to build...",
                Style::default().fg(DIM),
            )
        } else {
            Span::raw(&self.input)
        };

        f.render_widget(Paragraph::new(display_text).block(block), area);

        if !self.is_streaming {
            let x = area.x + 1 + self.input[..self.cursor_pos].chars().count() as u16;
            let y = area.y + 1;
            if x < area.x + area.width - 1 {
                f.set_cursor(x, y);
            }
        }
    }

    fn render_footer(&self, f: &mut Frame, area: Rect) {
        let budget_remaining = self.config.budget.limit - self.config.budget.consumed;
        let budget_color = if budget_remaining < 1.0 {
            Color::Red
        } else if budget_remaining < 3.0 {
            Color::Yellow
        } else {
            DIM
        };

        let footer = Line::from(vec![
            Span::styled(" ^C", Style::default().fg(DIM)),
            Span::styled(" quit  ", Style::default().fg(Color::Rgb(50, 50, 50))),
            Span::styled("Esc", Style::default().fg(DIM)),
            Span::styled(" cancel  ", Style::default().fg(Color::Rgb(50, 50, 50))),
            Span::styled("↑↓", Style::default().fg(DIM)),
            Span::styled(" scroll  ", Style::default().fg(Color::Rgb(50, 50, 50))),
            Span::styled(
                format!("${:.2} / ${:.2}", self.config.budget.consumed, self.config.budget.limit),
                Style::default().fg(budget_color),
            ),
        ]);

        f.render_widget(Paragraph::new(footer), area);
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
