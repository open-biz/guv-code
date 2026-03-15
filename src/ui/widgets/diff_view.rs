use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget, Wrap},
};
use similar::{ChangeTag, TextDiff};
use crate::ui::theme;

pub struct DiffView<'a> {
    old_text: &'a str,
    new_text: &'a str,
    file_path: &'a str,
    block: Option<Block<'a>>,
    scroll: u16,
}

impl<'a> DiffView<'a> {
    pub fn new(old_text: &'a str, new_text: &'a str, file_path: &'a str) -> Self {
        Self {
            old_text,
            new_text,
            file_path,
            block: None,
            scroll: 0,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn scroll(mut self, scroll: u16) -> Self {
        self.scroll = scroll;
        self
    }
}

impl<'a> Widget for DiffView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = if let Some(block) = &self.block {
            let inner = block.inner(area);
            block.clone().render(area, buf);
            inner
        } else {
            area
        };

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        let diff = TextDiff::from_lines(self.old_text, self.new_text);
        let mut lines: Vec<Line> = Vec::new();

        // File header
        lines.push(Line::from(vec![
            Span::styled(" ", Style::default().fg(theme::BLUE)),
            Span::styled(self.file_path.to_string(), Style::default().fg(theme::BLUE)),
        ]));
        lines.push(Line::from(""));

        let mut old_line: usize = 1;
        let mut new_line: usize = 1;

        for change in diff.iter_all_changes() {
            let (prefix, line_num, style, bg) = match change.tag() {
                ChangeTag::Delete => {
                    let ln = old_line;
                    old_line += 1;
                    (
                        "−",
                        format!("{:>4}", ln),
                        Style::default().fg(theme::DIFF_DELETE_FG),
                        Style::default().fg(theme::DIFF_DELETE_FG).bg(theme::DIFF_DELETE_BG),
                    )
                }
                ChangeTag::Insert => {
                    let ln = new_line;
                    new_line += 1;
                    (
                        "+",
                        format!("{:>4}", ln),
                        Style::default().fg(theme::DIFF_INSERT_FG),
                        Style::default().fg(theme::DIFF_INSERT_FG).bg(theme::DIFF_INSERT_BG),
                    )
                }
                ChangeTag::Equal => {
                    let ln = old_line;
                    old_line += 1;
                    new_line += 1;
                    (
                        " ",
                        format!("{:>4}", ln),
                        Style::default().fg(theme::FG_MUTED),
                        Style::default().fg(theme::FG_MUTED),
                    )
                }
            };

            let content = change.as_str().unwrap_or("").trim_end_matches('\n').to_string();

            lines.push(Line::from(vec![
                Span::styled(line_num, style),
                Span::styled(" ", style),
                Span::styled(prefix.to_string(), style),
                Span::styled(" ", bg),
                Span::styled(content, bg),
            ]));
        }

        let para = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0));
        para.render(inner, buf);
    }
}

/// Render a streaming diff: new text arrives incrementally.
/// Shows the latest N lines of diff output.
pub fn render_streaming_diff_lines(
    accumulated_text: &str,
    max_lines: usize,
) -> Vec<Line<'static>> {
    let text_lines: Vec<&str> = accumulated_text.lines().collect();
    let start = text_lines.len().saturating_sub(max_lines);

    text_lines[start..]
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let line_no = start + i + 1;
            let trimmed = line.trim_start();

            if trimmed.starts_with('+') || trimmed.starts_with("SEARCH") {
                Line::from(vec![
                    Span::styled(
                        format!("{:>4} ", line_no),
                        Style::default().fg(theme::DIFF_INSERT_FG),
                    ),
                    Span::styled(
                        line.to_string(),
                        Style::default()
                            .fg(theme::DIFF_INSERT_FG)
                            .bg(theme::DIFF_INSERT_BG),
                    ),
                ])
            } else if trimmed.starts_with('-') || trimmed.starts_with("REPLACE") {
                Line::from(vec![
                    Span::styled(
                        format!("{:>4} ", line_no),
                        Style::default().fg(theme::DIFF_DELETE_FG),
                    ),
                    Span::styled(
                        line.to_string(),
                        Style::default()
                            .fg(theme::DIFF_DELETE_FG)
                            .bg(theme::DIFF_DELETE_BG),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::styled(
                        format!("{:>4} ", line_no),
                        Style::default().fg(theme::FG_SUBTLE),
                    ),
                    Span::styled(line.to_string(), Style::default().fg(theme::FG_MUTED)),
                ])
            }
        })
        .collect()
}
