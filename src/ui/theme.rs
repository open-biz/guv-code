use ratatui::style::{Color, Modifier, Style};

// ── Charm / Lip Gloss Inspired Palette ──────────────────────────────────────
// Deep charcoal base with vibrant accent pops.

// Backgrounds
pub const BG_BASE: Color = Color::Rgb(18, 18, 18);        // #121212 - Deep charcoal
pub const BG_LIGHTER: Color = Color::Rgb(30, 30, 30);     // #1E1E1E - Slightly lighter
pub const BG_SUBTLE: Color = Color::Rgb(40, 40, 40);      // #282828 - Subtle panels
pub const BG_OVERLAY: Color = Color::Rgb(50, 50, 50);     // #323232 - Overlays/modals

// Foregrounds
pub const FG_BASE: Color = Color::Rgb(210, 210, 210);     // #D2D2D2 - Primary text
pub const FG_MUTED: Color = Color::Rgb(120, 120, 120);    // #787878 - Secondary info
pub const FG_HALF_MUTED: Color = Color::Rgb(160, 160, 160); // #A0A0A0 - Half-dimmed
pub const FG_SUBTLE: Color = Color::Rgb(80, 80, 80);      // #505050 - Ghost hints
pub const FG_DIM: Color = Color::Rgb(60, 60, 60);         // #3C3C3C - Very dim

// Primary Accents (Charm Pink + Codebuff Cyan)
pub const CHARM_PINK: Color = Color::Rgb(255, 95, 135);   // #FF5F87 - Hot pink
pub const CODEBUFF_CYAN: Color = Color::Rgb(0, 215, 215); // #00D7D7 - Vibrant cyan
pub const ACCENT_PRIMARY: Color = CHARM_PINK;
pub const ACCENT_SECONDARY: Color = CODEBUFF_CYAN;

// Status Colors
pub const GREEN: Color = Color::Rgb(95, 215, 135);        // #5FD787 - Success
pub const GREEN_DARK: Color = Color::Rgb(70, 160, 100);   // #46A064 - Muted success
pub const RED: Color = Color::Rgb(255, 95, 95);           // #FF5F5F - Error
pub const RED_DARK: Color = Color::Rgb(180, 70, 70);      // #B44646 - Muted error
pub const YELLOW: Color = Color::Rgb(255, 215, 95);       // #FFD75F - Warning
pub const BLUE: Color = Color::Rgb(95, 175, 255);         // #5FAFFF - Info / tool names
pub const BLUE_DARK: Color = Color::Rgb(70, 130, 180);    // #4682B4 - Muted info

// Diff Colors (from Crush ref)
pub const DIFF_INSERT_BG: Color = Color::Rgb(50, 57, 49); // #323931
pub const DIFF_INSERT_FG: Color = Color::Rgb(98, 150, 87);// #629657
pub const DIFF_DELETE_BG: Color = Color::Rgb(56, 48, 48); // #383030
pub const DIFF_DELETE_FG: Color = Color::Rgb(164, 92, 89);// #A45C59
pub const DIFF_EQUAL_FG: Color = FG_MUTED;

// Borders
pub const BORDER_DIM: Color = Color::Rgb(45, 45, 45);     // #2D2D2D
pub const BORDER_FOCUS: Color = CHARM_PINK;

// Icons (matching Crush's icon set)
pub const ICON_CHECK: &str = "✓";
pub const ICON_CROSS: &str = "×";
pub const ICON_PENDING: &str = "●";
pub const ICON_ARROW: &str = "→";
pub const ICON_RADIO_ON: &str = "◉";
pub const ICON_RADIO_OFF: &str = "○";
pub const ICON_BORDER_THICK: &str = "▌";
pub const ICON_SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
pub const ICON_IMAGE: &str = "■";
pub const ICON_THINKING: &str = "💭";

// ── Reusable Style Presets ──────────────────────────────────────────────────

pub fn base() -> Style {
    Style::default().fg(FG_BASE)
}

pub fn muted() -> Style {
    Style::default().fg(FG_MUTED)
}

pub fn half_muted() -> Style {
    Style::default().fg(FG_HALF_MUTED)
}

pub fn subtle() -> Style {
    Style::default().fg(FG_SUBTLE)
}

pub fn accent() -> Style {
    Style::default().fg(ACCENT_PRIMARY).add_modifier(Modifier::BOLD)
}

pub fn accent_secondary() -> Style {
    Style::default().fg(ACCENT_SECONDARY).add_modifier(Modifier::BOLD)
}

pub fn success() -> Style {
    Style::default().fg(GREEN)
}

pub fn error() -> Style {
    Style::default().fg(RED)
}

pub fn warning() -> Style {
    Style::default().fg(YELLOW)
}

pub fn info() -> Style {
    Style::default().fg(BLUE)
}

pub fn tag_error() -> Style {
    Style::default().fg(Color::White).bg(RED_DARK).add_modifier(Modifier::BOLD)
}

pub fn tag_info() -> Style {
    Style::default().fg(Color::White).bg(BLUE_DARK).add_modifier(Modifier::BOLD)
}

pub fn tag_success() -> Style {
    Style::default().fg(Color::White).bg(GREEN_DARK).add_modifier(Modifier::BOLD)
}

pub fn tag_warning() -> Style {
    Style::default().fg(BG_BASE).bg(YELLOW).add_modifier(Modifier::BOLD)
}

// Panel styles with rounded borders
pub fn panel_block_style() -> Style {
    Style::default().fg(BORDER_DIM)
}

pub fn panel_focused_style() -> Style {
    Style::default().fg(BORDER_FOCUS)
}

pub fn ghost_hint() -> Style {
    Style::default().fg(FG_DIM).add_modifier(Modifier::ITALIC)
}

pub fn bold_primary() -> Style {
    Style::default().fg(FG_BASE).add_modifier(Modifier::BOLD)
}

pub fn dim_italic() -> Style {
    Style::default().fg(FG_MUTED).add_modifier(Modifier::ITALIC)
}
