use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};
use crate::ui::theme;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum ImageEncoding {
    Kitty,
    BlockChars,
}

/// Detects if the terminal supports Kitty graphics protocol
pub fn detect_kitty_support() -> bool {
    std::env::var("TERM").map(|t| t.contains("kitty")).unwrap_or(false)
        || std::env::var("TERM_PROGRAM")
            .map(|t| t.contains("kitty") || t.contains("WezTerm"))
            .unwrap_or(false)
}

/// Transform a path like `@path/to/img.png` into an interactive image tag
pub fn transform_image_path(input: &str) -> Option<ImageTag> {
    let trimmed = input.trim();
    if !trimmed.starts_with('@') {
        return None;
    }

    let path_str = &trimmed[1..];
    let path = Path::new(path_str);

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    let is_image = matches!(
        ext.as_str(),
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg" | "tiff" | "ico"
    );

    if !is_image {
        return None;
    }

    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path_str)
        .to_string();

    Some(ImageTag {
        full_path: path_str.to_string(),
        filename,
        mime: format!("image/{}", if ext == "jpg" { "jpeg".to_string() } else { ext }),
    })
}

#[derive(Debug, Clone)]
pub struct ImageTag {
    pub full_path: String,
    pub filename: String,
    pub mime: String,
}

/// Renders a collapsed image tag: `[Image filename.ext]`
pub fn render_image_tag(tag: &ImageTag, expanded: bool) -> Line<'static> {
    if expanded {
        Line::from(vec![
            Span::styled(
                format!("{} ", theme::ICON_IMAGE),
                Style::default().fg(theme::ACCENT_PRIMARY),
            ),
            Span::styled(
                format!("[Image {}]", tag.filename),
                Style::default()
                    .fg(theme::ACCENT_PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}", tag.full_path),
                Style::default()
                    .fg(theme::FG_MUTED)
                    .add_modifier(Modifier::ITALIC),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled(
                format!("{} ", theme::ICON_IMAGE),
                Style::default().fg(theme::ACCENT_PRIMARY),
            ),
            Span::styled(
                format!("[Image {}]", tag.filename),
                Style::default()
                    .fg(theme::ACCENT_PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    }
}

/// Block-character fallback renderer for non-Kitty terminals.
/// Uses Unicode quadrant/block characters to approximate image colors.
pub struct BlockCharImage {
    pub width: u16,
    pub height: u16,
    // Rows of (char, fg_color, bg_color) tuples
    pub cells: Vec<Vec<(char, ratatui::style::Color, ratatui::style::Color)>>,
}

impl BlockCharImage {
    /// Create a block-char approximation from raw RGBA pixels
    pub fn from_rgba(data: &[u8], img_width: u32, img_height: u32, cols: u16, rows: u16) -> Self {
        let mut cells = Vec::new();

        if img_width == 0 || img_height == 0 || cols == 0 || rows == 0 {
            return Self {
                width: cols,
                height: rows,
                cells,
            };
        }

        // Each cell represents 1 column x 2 rows of pixels (using ▄ half-block)
        let cell_rows = rows as u32;
        let cell_cols = cols as u32;

        for cy in 0..cell_rows {
            let mut row = Vec::new();
            for cx in 0..cell_cols {
                // Map cell to image coordinates
                let img_x = (cx * img_width / cell_cols).min(img_width - 1);
                let top_y = (cy * 2 * img_height / (cell_rows * 2)).min(img_height - 1);
                let bot_y = ((cy * 2 + 1) * img_height / (cell_rows * 2)).min(img_height - 1);

                let top_pixel = pixel_at(data, img_width, img_x, top_y);
                let bot_pixel = pixel_at(data, img_width, img_x, bot_y);

                let fg = ratatui::style::Color::Rgb(bot_pixel.0, bot_pixel.1, bot_pixel.2);
                let bg = ratatui::style::Color::Rgb(top_pixel.0, top_pixel.1, top_pixel.2);

                row.push(('▄', fg, bg));
            }
            cells.push(row);
        }

        Self {
            width: cols,
            height: rows,
            cells,
        }
    }
}

impl Widget for BlockCharImage {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for (y, row) in self.cells.iter().enumerate() {
            if y as u16 >= area.height {
                break;
            }
            for (x, (ch, fg, bg)) in row.iter().enumerate() {
                if x as u16 >= area.width {
                    break;
                }
                let cell = buf.get_mut(area.x + x as u16, area.y + y as u16);
                cell.set_char(*ch);
                cell.set_fg(*fg);
                cell.set_bg(*bg);
            }
        }
    }
}

/// Generates Kitty Graphics Protocol escape sequence for image display
pub fn kitty_graphics_display(
    image_data: &[u8],
    cols: u16,
    rows: u16,
    image_id: u32,
) -> String {
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(image_data);

    // Chunked transmission (4096 byte chunks)
    let chunk_size = 4096;
    let chunks: Vec<&str> = encoded
        .as_bytes()
        .chunks(chunk_size)
        .map(|c| std::str::from_utf8(c).unwrap_or(""))
        .collect();

    let mut result = String::new();
    for (i, chunk) in chunks.iter().enumerate() {
        let more = if i < chunks.len() - 1 { 1 } else { 0 };
        if i == 0 {
            // First chunk: transmit + display
            result.push_str(&format!(
                "\x1b_Ga=T,f=32,s={cols},v={rows},i={image_id},m={more};{chunk}\x1b\\",
                cols = cols,
                rows = rows,
                image_id = image_id,
                more = more,
                chunk = chunk,
            ));
        } else {
            // Continuation chunks
            result.push_str(&format!(
                "\x1b_Gm={more};{chunk}\x1b\\",
                more = more,
                chunk = chunk,
            ));
        }
    }

    result
}

fn pixel_at(data: &[u8], width: u32, x: u32, y: u32) -> (u8, u8, u8) {
    let idx = ((y * width + x) * 4) as usize;
    if idx + 2 < data.len() {
        (data[idx], data[idx + 1], data[idx + 2])
    } else {
        (0, 0, 0)
    }
}
