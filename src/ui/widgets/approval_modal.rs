use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget, Wrap},
};
use crate::ui::theme;

#[derive(Debug, Clone, PartialEq)]
pub enum ModalSelection {
    Approve,
    Deny,
}

pub struct ApprovalModal<'a> {
    command: &'a str,
    selected: ModalSelection,
}

impl<'a> ApprovalModal<'a> {
    pub fn new(command: &'a str) -> Self {
        Self {
            command,
            selected: ModalSelection::Approve,
        }
    }

    pub fn selected(mut self, selected: ModalSelection) -> Self {
        self.selected = selected;
        self
    }
}

impl<'a> Widget for ApprovalModal<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Center the modal (60% width, 40% height, clamped)
        let modal_width = (area.width * 60 / 100).max(40).min(area.width.saturating_sub(4));
        let modal_height = (area.height * 40 / 100).max(10).min(area.height.saturating_sub(2));
        let x = area.x + (area.width.saturating_sub(modal_width)) / 2;
        let y = area.y + (area.height.saturating_sub(modal_height)) / 2;
        let modal_area = Rect::new(x, y, modal_width, modal_height);

        // Clear the area behind the modal
        Clear.render(modal_area, buf);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme::YELLOW))
            .title(Span::styled(
                " ⚠ Approval Required ",
                Style::default()
                    .fg(theme::YELLOW)
                    .add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(theme::BG_OVERLAY));

        let inner = block.inner(modal_area);
        block.render(modal_area, buf);

        if inner.height < 4 || inner.width < 10 {
            return;
        }

        // Layout: warning text, command, buttons
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Warning text
                Constraint::Length(1), // Spacer
                Constraint::Min(2),    // Command display
                Constraint::Length(1), // Spacer
                Constraint::Length(1), // Buttons
                Constraint::Length(1), // Help
            ])
            .split(inner);

        // Warning text
        let warning = Paragraph::new(Line::from(vec![
            Span::styled(
                " This command may have destructive side-effects.",
                Style::default()
                    .fg(theme::YELLOW)
                    .add_modifier(Modifier::ITALIC),
            ),
        ]))
        .alignment(Alignment::Center);
        warning.render(chunks[0], buf);

        // Command display with code background
        let cmd_lines = vec![
            Line::from(vec![
                Span::styled("  $ ", Style::default().fg(theme::GREEN)),
                Span::styled(
                    self.command,
                    Style::default()
                        .fg(theme::FG_BASE)
                        .bg(theme::BG_LIGHTER)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ];
        let cmd_para = Paragraph::new(cmd_lines).wrap(Wrap { trim: false });
        cmd_para.render(chunks[2], buf);

        // Buttons
        let (approve_style, deny_style) = match self.selected {
            ModalSelection::Approve => (
                Style::default()
                    .fg(theme::BG_BASE)
                    .bg(theme::GREEN)
                    .add_modifier(Modifier::BOLD),
                Style::default()
                    .fg(theme::FG_MUTED)
                    .bg(theme::BG_SUBTLE),
            ),
            ModalSelection::Deny => (
                Style::default()
                    .fg(theme::FG_MUTED)
                    .bg(theme::BG_SUBTLE),
                Style::default()
                    .fg(theme::BG_BASE)
                    .bg(theme::RED)
                    .add_modifier(Modifier::BOLD),
            ),
        };

        let buttons = Line::from(vec![
            Span::raw("        "),
            Span::styled("  Yes (Enter)  ", approve_style),
            Span::raw("   "),
            Span::styled("  No (Esc)  ", deny_style),
        ]);
        let btn_para = Paragraph::new(buttons).alignment(Alignment::Center);
        btn_para.render(chunks[4], buf);

        // Help hint
        if chunks.len() > 5 {
            let help = Paragraph::new(Line::from(Span::styled(
                " Tab to switch  •  Enter to confirm  •  Esc to deny ",
                Style::default().fg(theme::FG_SUBTLE),
            )))
            .alignment(Alignment::Center);
            help.render(chunks[5], buf);
        }
    }
}

/// Calculate a centered rect within the given area
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
