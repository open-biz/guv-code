use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};
use crate::ui::theme;

pub struct StatusBar<'a> {
    budget_consumed: f64,
    budget_limit: f64,
    model_name: &'a str,
    mode: &'a str,
    is_indexing: bool,
    memory_hit: bool,
}

impl<'a> StatusBar<'a> {
    pub fn new(
        budget_consumed: f64,
        budget_limit: f64,
        model_name: &'a str,
        mode: &'a str,
    ) -> Self {
        Self {
            budget_consumed,
            budget_limit,
            model_name,
            mode,
            is_indexing: false,
            memory_hit: false,
        }
    }

    pub fn indexing(mut self, indexing: bool) -> Self {
        self.is_indexing = indexing;
        self
    }

    pub fn memory_hit(mut self, hit: bool) -> Self {
        self.memory_hit = hit;
        self
    }
}

impl<'a> Widget for StatusBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let remaining = self.budget_limit - self.budget_consumed;
        let budget_color = if remaining < 1.0 {
            theme::RED
        } else if remaining < 3.0 {
            theme::YELLOW
        } else {
            theme::FG_MUTED
        };

        let mut spans: Vec<Span> = vec![
            Span::styled(" ", Style::default().bg(theme::BG_SUBTLE)),
            // Budget
            Span::styled(
                format!(
                    " ${:.2}/{:.2} ",
                    self.budget_consumed, self.budget_limit
                ),
                Style::default().fg(budget_color),
            ),
            Span::styled(" • ", Style::default().fg(theme::FG_DIM)),
            // Model
            Span::styled(
                format!(" {} ", self.model_name),
                Style::default().fg(theme::BLUE),
            ),
            Span::styled(" • ", Style::default().fg(theme::FG_DIM)),
            // Mode
            Span::styled(
                format!(" {} ", self.mode),
                Style::default()
                    .fg(theme::ACCENT_SECONDARY)
                    .add_modifier(Modifier::BOLD),
            ),
        ];

        if self.is_indexing {
            spans.push(Span::styled(" • ", Style::default().fg(theme::FG_DIM)));
            spans.push(Span::styled(
                " Indexing... ",
                Style::default()
                    .fg(theme::YELLOW)
                    .add_modifier(Modifier::ITALIC),
            ));
        }

        if self.memory_hit {
            spans.push(Span::styled(" • ", Style::default().fg(theme::FG_DIM)));
            spans.push(Span::styled(
                " ⚡ Memory Hit ",
                Style::default()
                    .fg(theme::GREEN)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        // Right-aligned keybindings
        let keys_text = " ^C quit  Esc cancel  j/k scroll  ? help ";
        let used_width: usize = spans.iter().map(|s| s.content.len()).sum();
        let remaining_width = (area.width as usize).saturating_sub(used_width);

        if remaining_width > keys_text.len() {
            let padding = remaining_width - keys_text.len();
            spans.push(Span::raw(" ".repeat(padding)));
        }
        spans.push(Span::styled(keys_text, Style::default().fg(theme::FG_SUBTLE)));

        let line = Line::from(spans);
        let bar = Paragraph::new(line).style(Style::default().bg(theme::BG_SUBTLE));
        bar.render(area, buf);
    }
}
