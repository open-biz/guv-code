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
    is_streaming: bool,
    help_visible: bool,
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
            is_streaming: false,
            help_visible: false,
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

    pub fn streaming(mut self, streaming: bool) -> Self {
        self.is_streaming = streaming;
        self
    }

    pub fn help_visible(mut self, visible: bool) -> Self {
        self.help_visible = visible;
        self
    }
}

fn sep<'a>() -> Span<'a> {
    Span::styled(" · ", theme::status_help_sep())
}

fn key<'a>(k: &'a str) -> Span<'a> {
    Span::styled(k, theme::status_help_key())
}

fn desc<'a>(d: &'a str) -> Span<'a> {
    Span::styled(d, theme::status_help_desc())
}

impl<'a> Widget for StatusBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        // ── Left side: status indicators ────────────────────────────
        let mut left_spans: Vec<Span> = Vec::new();

        if self.is_indexing {
            left_spans.push(Span::styled(
                " Indexing... ",
                Style::default()
                    .fg(theme::YELLOW)
                    .add_modifier(Modifier::ITALIC),
            ));
            left_spans.push(sep());
        }

        if self.memory_hit {
            left_spans.push(Span::styled(
                "⚡ Memory Hit",
                Style::default()
                    .fg(theme::GREEN)
                    .add_modifier(Modifier::BOLD),
            ));
            left_spans.push(sep());
        }

        // ── Right side: keybind hints (minimal when help panel is visible) ──
        let mut right_spans: Vec<Span> = Vec::new();

        if self.is_streaming {
            right_spans.push(key("esc"));
            right_spans.push(desc(" cancel"));
            right_spans.push(sep());
        }

        if self.help_visible {
            // Help panel is showing the full grid — just show toggle hint
            right_spans.push(key("ctrl+g"));
            right_spans.push(desc(" less"));
        } else {
            // Compact hints when help panel is hidden
            right_spans.push(key("/"));
            right_spans.push(desc(" commands"));
            right_spans.push(sep());
            right_spans.push(key("ctrl+c"));
            right_spans.push(desc(" quit"));
            right_spans.push(sep());
            right_spans.push(key("ctrl+g"));
            right_spans.push(desc(" more"));
        }

        // Calculate widths for padding
        let left_width: usize = left_spans.iter().map(|s| s.content.len()).sum();
        let right_width: usize = right_spans.iter().map(|s| s.content.len()).sum();
        let total = left_width + right_width + 2; // 2 for edge padding
        let padding = (area.width as usize).saturating_sub(total);

        let mut spans = Vec::new();
        spans.push(Span::raw(" "));
        spans.extend(left_spans);
        spans.push(Span::raw(" ".repeat(padding)));
        spans.extend(right_spans);
        spans.push(Span::raw(" "));

        let line = Line::from(spans);
        let bar = Paragraph::new(line).style(Style::default().bg(theme::BG_SUBTLE));
        bar.render(area, buf);
    }
}
