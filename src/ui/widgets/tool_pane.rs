use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget, Wrap},
};
use crate::agent_logic::ToolStatus;
use crate::ui::theme;

#[derive(Debug, Clone)]
pub struct ToolEntry {
    pub name: String,
    pub description: String,
    pub status: ToolStatus,
    pub output_lines: Vec<String>,
}

pub struct ToolPane<'a> {
    entries: &'a [ToolEntry],
    scroll: u16,
    tick: usize,
    focused: bool,
    block: Option<Block<'a>>,
}

impl<'a> ToolPane<'a> {
    pub fn new(entries: &'a [ToolEntry]) -> Self {
        Self {
            entries,
            scroll: 0,
            tick: 0,
            focused: false,
            block: None,
        }
    }

    pub fn scroll(mut self, scroll: u16) -> Self {
        self.scroll = scroll;
        self
    }

    pub fn tick(mut self, tick: usize) -> Self {
        self.tick = tick;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    fn spinner_frame(&self) -> &'static str {
        theme::ICON_SPINNER_FRAMES[(self.tick / 2) % theme::ICON_SPINNER_FRAMES.len()]
    }

    fn status_icon(&self, status: &ToolStatus) -> (&'static str, Style) {
        match status {
            ToolStatus::Pending => (
                theme::ICON_PENDING,
                Style::default().fg(theme::FG_MUTED),
            ),
            ToolStatus::Executing => (
                self.spinner_frame(),
                Style::default().fg(theme::ACCENT_SECONDARY),
            ),
            ToolStatus::Success => (
                theme::ICON_CHECK,
                Style::default().fg(theme::GREEN),
            ),
            ToolStatus::Error => (
                theme::ICON_CROSS,
                Style::default().fg(theme::RED),
            ),
            ToolStatus::Cancelled => (
                theme::ICON_PENDING,
                Style::default().fg(theme::FG_MUTED),
            ),
        }
    }

    fn status_label(status: &ToolStatus) -> (&'static str, Style) {
        match status {
            ToolStatus::Pending => ("PENDING", Style::default().fg(theme::FG_MUTED)),
            ToolStatus::Executing => (
                "RUNNING",
                Style::default()
                    .fg(theme::ACCENT_SECONDARY)
                    .add_modifier(Modifier::BOLD),
            ),
            ToolStatus::Success => ("SUCCESS", Style::default().fg(theme::GREEN)),
            ToolStatus::Error => (
                "ERROR",
                Style::default()
                    .fg(theme::RED)
                    .add_modifier(Modifier::BOLD),
            ),
            ToolStatus::Cancelled => ("CANCELLED", Style::default().fg(theme::FG_MUTED)),
        }
    }
}

impl<'a> Widget for ToolPane<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = if let Some(block) = &self.block {
            let inner = block.inner(area);
            block.clone().render(area, buf);
            inner
        } else {
            area
        };

        if inner.height == 0 || inner.width == 0 || self.entries.is_empty() {
            return;
        }

        let mut lines: Vec<Line> = Vec::new();

        for entry in self.entries.iter().rev() {
            let (icon, icon_style) = self.status_icon(&entry.status);
            let (status_label, status_style) = Self::status_label(&entry.status);

            // ── Sticky Header ───────────────────────────────────────
            // Tool name + status pinned at top of each tool's output
            lines.push(Line::from(vec![
                Span::styled(format!(" {} ", icon), icon_style),
                Span::styled(
                    &entry.name,
                    Style::default()
                        .fg(theme::BLUE)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(status_label, status_style),
                Span::raw("  "),
                Span::styled(
                    &entry.description,
                    Style::default().fg(theme::FG_MUTED),
                ),
            ]));

            // Output lines with subtle background
            let max_visible = 8;
            let output = &entry.output_lines;
            let start = output.len().saturating_sub(max_visible);

            if start > 0 {
                lines.push(Line::from(Span::styled(
                    format!("   … ({} lines hidden)", start),
                    Style::default()
                        .fg(theme::FG_SUBTLE)
                        .add_modifier(Modifier::ITALIC),
                )));
            }

            for line in &output[start..] {
                lines.push(Line::from(vec![
                    Span::raw("   "),
                    Span::styled(
                        line.as_str(),
                        Style::default().fg(theme::FG_HALF_MUTED).bg(theme::BG_LIGHTER),
                    ),
                ]));
            }

            // Separator between tool entries
            lines.push(Line::from(""));
        }

        let para = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0));
        para.render(inner, buf);
    }
}
