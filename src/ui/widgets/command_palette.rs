use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget},
};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use crate::ui::theme;

// ── Slash Command Definition ────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct SlashCommand {
    pub name: &'static str,
    pub description: &'static str,
    pub keybind: &'static str,
    pub category: CommandCategory,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CommandCategory {
    Session,
    Navigation,
    Agent,
    Settings,
    System,
}

// ── Default Command Registry ────────────────────────────────────────────────

pub fn default_commands() -> Vec<SlashCommand> {
    vec![
        SlashCommand {
            name: "new",
            description: "Clear conversation and start fresh",
            keybind: "ctrl+n",
            category: CommandCategory::Session,
        },
        SlashCommand {
            name: "help",
            description: "Display keyboard shortcuts and tips",
            keybind: "",
            category: CommandCategory::System,
        },
        SlashCommand {
            name: "undo",
            description: "Revert last git auto-commit",
            keybind: "u",
            category: CommandCategory::Agent,
        },
        SlashCommand {
            name: "model",
            description: "Switch active model",
            keybind: "ctrl+m",
            category: CommandCategory::Settings,
        },
        SlashCommand {
            name: "history",
            description: "Browse and resume past conversations",
            keybind: "ctrl+s",
            category: CommandCategory::Session,
        },
        SlashCommand {
            name: "init",
            description: "Initialize project and create AGENTS.md",
            keybind: "",
            category: CommandCategory::Agent,
        },
        SlashCommand {
            name: "review",
            description: "Review code changes with cargo check",
            keybind: "",
            category: CommandCategory::Agent,
        },
        SlashCommand {
            name: "files",
            description: "Open file picker for workspace",
            keybind: "@",
            category: CommandCategory::Navigation,
        },
        SlashCommand {
            name: "tools",
            description: "Toggle tool output pane",
            keybind: "ctrl+t",
            category: CommandCategory::Navigation,
        },
        SlashCommand {
            name: "editor",
            description: "Open in external editor",
            keybind: "ctrl+o",
            category: CommandCategory::Navigation,
        },
        SlashCommand {
            name: "sessions",
            description: "Browse and resume past sessions",
            keybind: "ctrl+s",
            category: CommandCategory::Session,
        },
        SlashCommand {
            name: "sidebar",
            description: "Toggle sidebar visibility",
            keybind: "",
            category: CommandCategory::Navigation,
        },
        SlashCommand {
            name: "yolo",
            description: "Toggle YOLO mode (auto-approve all)",
            keybind: "ctrl+y",
            category: CommandCategory::Settings,
        },
        SlashCommand {
            name: "usage",
            description: "View credits and subscription quota",
            keybind: "",
            category: CommandCategory::System,
        },
        SlashCommand {
            name: "feedback",
            description: "Share general feedback about GuvCode",
            keybind: "",
            category: CommandCategory::System,
        },
        SlashCommand {
            name: "image",
            description: "Attach an image file (or Ctrl+V to paste)",
            keybind: "",
            category: CommandCategory::Agent,
        },
        SlashCommand {
            name: "auth",
            description: "Authentication settings and login",
            keybind: "",
            category: CommandCategory::Settings,
        },
        SlashCommand {
            name: "quit",
            description: "Exit GuvCode",
            keybind: "ctrl+c",
            category: CommandCategory::System,
        },
    ]
}

// ── Command Palette State ───────────────────────────────────────────────────

pub struct CommandPaletteState {
    pub visible: bool,
    pub filter: String,
    pub cursor: usize,
    pub selected: usize,
    commands: Vec<SlashCommand>,
    filtered: Vec<(i64, SlashCommand)>, // (score, command)
    matcher: SkimMatcherV2,
}

impl CommandPaletteState {
    pub fn new() -> Self {
        let commands = default_commands();
        let filtered: Vec<(i64, SlashCommand)> = commands
            .iter()
            .map(|c| (0, c.clone()))
            .collect();
        Self {
            visible: false,
            filter: String::new(),
            cursor: 0,
            selected: 0,
            commands,
            filtered,
            matcher: SkimMatcherV2::default(),
        }
    }

    pub fn open(&mut self) {
        self.visible = true;
        self.filter.clear();
        self.cursor = 0;
        self.selected = 0;
        self.update_filter();
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.filter.clear();
    }

    pub fn update_filter(&mut self) {
        if self.filter.is_empty() {
            self.filtered = self.commands.iter().map(|c| (0, c.clone())).collect();
        } else {
            let mut scored: Vec<(i64, SlashCommand)> = self
                .commands
                .iter()
                .filter_map(|cmd| {
                    let haystack = format!("{} {}", cmd.name, cmd.description);
                    self.matcher
                        .fuzzy_match(&haystack, &self.filter)
                        .map(|score| (score, cmd.clone()))
                })
                .collect();
            scored.sort_by(|a, b| b.0.cmp(&a.0));
            self.filtered = scored;
        }
        // Clamp selection
        if self.selected >= self.filtered.len() {
            self.selected = self.filtered.len().saturating_sub(1);
        }
    }

    pub fn type_char(&mut self, c: char) {
        self.filter.insert(self.cursor, c);
        self.cursor += c.len_utf8();
        self.update_filter();
    }

    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            let prev = self.filter[..self.cursor]
                .chars()
                .last()
                .map(|c| c.len_utf8())
                .unwrap_or(0);
            self.cursor -= prev;
            self.filter.remove(self.cursor);
            self.update_filter();
        }
    }

    pub fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.filtered.len() {
            self.selected += 1;
        }
    }

    pub fn confirm(&mut self) -> Option<SlashCommand> {
        if let Some((_, cmd)) = self.filtered.get(self.selected) {
            let result = cmd.clone();
            self.close();
            Some(result)
        } else {
            None
        }
    }

    pub fn filtered_items(&self) -> &[(i64, SlashCommand)] {
        &self.filtered
    }
}

// ── Command Palette Widget ──────────────────────────────────────────────────

pub struct CommandPalette<'a> {
    state: &'a CommandPaletteState,
}

impl<'a> CommandPalette<'a> {
    pub fn new(state: &'a CommandPaletteState) -> Self {
        Self { state }
    }
}

impl<'a> Widget for CommandPalette<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if !self.state.visible {
            return;
        }

        // Calculate centered overlay dimensions (Crush-style: ~60% width, capped)
        let dialog_width = (area.width * 60 / 100).min(72).max(40);
        let max_items = self.state.filtered.len().min(14) as u16;
        let dialog_height = (max_items + 6).min(area.height.saturating_sub(4)); // header + input + items + footer + borders

        let x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
        let y = area.y + (area.height.saturating_sub(dialog_height)) / 3; // Slightly above center

        let dialog_area = Rect {
            x,
            y,
            width: dialog_width,
            height: dialog_height,
        };

        // Clear background
        Clear.render(dialog_area, buf);

        // Outer block with Charm-style rounded border, pink focus
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme::BORDER_CHARPLE))
            .style(Style::default().bg(theme::BG_BASE));

        let inner = block.inner(dialog_area);
        block.render(dialog_area, buf);

        if inner.height < 4 || inner.width < 10 {
            return;
        }

        // Layout: Title header | Filter input | Item list | Footer help
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Title with diagonal pattern
                Constraint::Length(1), // Filter input
                Constraint::Min(1),   // Items
                Constraint::Length(1), // Footer help
            ])
            .split(inner);

        // ── Title Bar with diagonal pattern (Crush signature) ───────
        let diag_count = (chunks[0].width as usize).saturating_sub(12);
        let diagonals: String = theme::ICON_DIAGONAL.repeat(diag_count);
        let title_line = Line::from(vec![
            Span::styled(" Commands ", Style::default().fg(theme::CHARM_PINK).add_modifier(Modifier::BOLD)),
            Span::styled(diagonals, Style::default().fg(theme::CHARM_PINK)),
        ]);
        Paragraph::new(title_line)
            .style(Style::default().bg(theme::BG_BASE))
            .render(chunks[0], buf);

        // ── Filter Input ────────────────────────────────────────────
        let filter_line = Line::from(vec![
            Span::styled(
                theme::ICON_FILTER_PROMPT,
                Style::default().fg(theme::BOK_GREEN).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                if self.state.filter.is_empty() {
                    "Type to filter"
                } else {
                    &self.state.filter
                },
                if self.state.filter.is_empty() {
                    Style::default().fg(theme::FG_SUBTLE)
                } else {
                    Style::default().fg(theme::FG_BASE)
                },
            ),
        ]);
        Paragraph::new(filter_line)
            .style(Style::default().bg(theme::BG_BASE))
            .render(chunks[1], buf);

        // ── Item List ───────────────────────────────────────────────
        let list_area = chunks[2].inner(&Margin { vertical: 0, horizontal: 0 });
        let visible_items = list_area.height as usize;

        // Scroll window
        let scroll_start = if self.state.selected >= visible_items {
            self.state.selected - visible_items + 1
        } else {
            0
        };

        let items = self.state.filtered_items();
        for (i, (_, cmd)) in items.iter().enumerate().skip(scroll_start).take(visible_items) {
            let row_y = list_area.y + (i - scroll_start) as u16;
            if row_y >= list_area.y + list_area.height {
                break;
            }

            let is_selected = i == self.state.selected;
            let row_area = Rect {
                x: list_area.x,
                y: row_y,
                width: list_area.width,
                height: 1,
            };

            let name_style = if is_selected {
                theme::palette_selected()
            } else {
                Style::default().fg(theme::FG_BASE)
            };
            let desc_style = if is_selected {
                theme::palette_selected()
            } else {
                Style::default().fg(theme::FG_MUTED)
            };

            // Build the line: command name + description + right-aligned keybind
            let name_text = format!("  /{}", cmd.name);
            let desc_text = format!("  {}", cmd.description);
            let keybind_text = if cmd.keybind.is_empty() {
                String::new()
            } else {
                cmd.keybind.to_string()
            };

            let name_len = name_text.len();
            let desc_len = desc_text.len();
            let kb_len = keybind_text.len();
            let total = name_len + desc_len + kb_len + 2;
            let padding = if total < row_area.width as usize {
                row_area.width as usize - total
            } else {
                1
            };

            let mut spans = vec![
                Span::styled(name_text, name_style.add_modifier(Modifier::BOLD)),
                Span::styled(desc_text, desc_style),
            ];

            if !keybind_text.is_empty() {
                spans.push(Span::styled(" ".repeat(padding), desc_style));
                spans.push(Span::styled(
                    keybind_text,
                    if is_selected {
                        theme::palette_selected()
                    } else {
                        theme::palette_keybind()
                    },
                ));
            }

            let line = Paragraph::new(Line::from(spans)).style(
                if is_selected {
                    Style::default().bg(theme::CHARM_PINK)
                } else {
                    Style::default().bg(theme::BG_BASE)
                },
            );
            line.render(row_area, buf);
        }

        // ── Footer Help ─────────────────────────────────────────────
        let footer = Line::from(vec![
            Span::styled("tab", theme::status_help_key()),
            Span::styled(" switch selection", theme::status_help_desc()),
            Span::styled(" · ", theme::status_help_sep()),
            Span::styled("↑↓", theme::status_help_key()),
            Span::styled(" choose", theme::status_help_desc()),
            Span::styled(" · ", theme::status_help_sep()),
            Span::styled("enter", theme::status_help_key()),
            Span::styled(" confirm", theme::status_help_desc()),
            Span::styled(" · ", theme::status_help_sep()),
            Span::styled("esc", theme::status_help_key()),
            Span::styled(" cancel", theme::status_help_desc()),
        ]);
        Paragraph::new(footer)
            .style(Style::default().bg(theme::BG_BASE))
            .render(chunks[3], buf);
    }
}
