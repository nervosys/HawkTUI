//! Theming system with semantic color tokens.
//!
//! Provides a [`Theme`] struct with named style tokens for consistent styling
//! across all widgets. Includes built-in dark and light themes, and supports
//! custom themes via the builder API.
//!
//! # Example
//!
//! ```ignore
//! use louie::theme::{Theme, ThemeToken};
//!
//! let theme = Theme::dark();
//! let style = theme.get(ThemeToken::Primary);
//! ```

use crate::core::style::{Color, Modifier, Style};

/// Semantic token names for themed UI elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThemeToken {
    // ── Core UI ──────────────────────
    /// Primary foreground text.
    Primary,
    /// Secondary / muted text.
    Secondary,
    /// Accent color for highlights and active elements.
    Accent,
    /// Error / failure indicators.
    Error,
    /// Warning indicators.
    Warning,
    /// Success indicators.
    Success,
    /// Informational text.
    Info,

    // ── Backgrounds ──────────────────
    /// Default background.
    Background,
    /// Surface (slightly raised, e.g. panels).
    Surface,
    /// Overlay background (modals, dropdowns).
    Overlay,

    // ── Widget-specific ──────────────
    /// Borders and separators.
    Border,
    /// Focused border.
    BorderFocused,
    /// Selection highlight.
    Selection,
    /// Cursor / caret.
    Cursor,
    /// Placeholder text.
    Placeholder,
    /// Status bar.
    StatusBar,
    /// Title / heading text.
    Title,

    // ── Diff / Code ──────────────────
    /// Added lines.
    DiffAdd,
    /// Removed lines.
    DiffRemove,
    /// Code block background.
    CodeBlock,
}

/// A theme is a mapping from semantic tokens to styles.
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    tokens: Vec<(ThemeToken, Style)>,
}

impl Theme {
    /// Create a new empty theme with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tokens: Vec::new(),
        }
    }

    /// Set a style for a token.
    pub fn set(mut self, token: ThemeToken, style: Style) -> Self {
        if let Some(entry) = self.tokens.iter_mut().find(|(t, _)| *t == token) {
            entry.1 = style;
        } else {
            self.tokens.push((token, style));
        }
        self
    }

    /// Get the style for a token. Returns `Style::default()` if not set.
    pub fn get(&self, token: ThemeToken) -> Style {
        self.tokens
            .iter()
            .find(|(t, _)| *t == token)
            .map(|(_, s)| *s)
            .unwrap_or_default()
    }

    /// Built-in dark theme.
    pub fn dark() -> Self {
        Self::new("dark")
            .set(ThemeToken::Primary, Style::default().fg(Color::White))
            .set(ThemeToken::Secondary, Style::default().fg(Color::Gray))
            .set(ThemeToken::Accent, Style::default().fg(Color::Cyan))
            .set(ThemeToken::Error, Style::default().fg(Color::Red))
            .set(ThemeToken::Warning, Style::default().fg(Color::Yellow))
            .set(ThemeToken::Success, Style::default().fg(Color::Green))
            .set(ThemeToken::Info, Style::default().fg(Color::Blue))
            .set(ThemeToken::Background, Style::default().bg(Color::Black))
            .set(
                ThemeToken::Surface,
                Style::default().bg(Color::Rgb(30, 30, 30)),
            )
            .set(
                ThemeToken::Overlay,
                Style::default().bg(Color::Rgb(40, 40, 40)),
            )
            .set(ThemeToken::Border, Style::default().fg(Color::DarkGray))
            .set(ThemeToken::BorderFocused, Style::default().fg(Color::Cyan))
            .set(
                ThemeToken::Selection,
                Style::default().bg(Color::Rgb(50, 50, 80)),
            )
            .set(
                ThemeToken::Cursor,
                Style::default().fg(Color::Black).bg(Color::White),
            )
            .set(
                ThemeToken::Placeholder,
                Style::default().fg(Color::DarkGray),
            )
            .set(
                ThemeToken::StatusBar,
                Style::default().fg(Color::White).bg(Color::Rgb(30, 30, 50)),
            )
            .set(
                ThemeToken::Title,
                style_with_modifier(Color::White, Modifier::BOLD),
            )
            .set(ThemeToken::DiffAdd, Style::default().fg(Color::Green))
            .set(ThemeToken::DiffRemove, Style::default().fg(Color::Red))
            .set(
                ThemeToken::CodeBlock,
                Style::default().bg(Color::Rgb(25, 25, 25)),
            )
    }

    /// Built-in light theme.
    pub fn light() -> Self {
        Self::new("light")
            .set(ThemeToken::Primary, Style::default().fg(Color::Black))
            .set(ThemeToken::Secondary, Style::default().fg(Color::DarkGray))
            .set(ThemeToken::Accent, Style::default().fg(Color::Blue))
            .set(ThemeToken::Error, Style::default().fg(Color::Red))
            .set(ThemeToken::Warning, Style::default().fg(Color::Yellow))
            .set(ThemeToken::Success, Style::default().fg(Color::Green))
            .set(ThemeToken::Info, Style::default().fg(Color::Cyan))
            .set(ThemeToken::Background, Style::default().bg(Color::White))
            .set(
                ThemeToken::Surface,
                Style::default().bg(Color::Rgb(240, 240, 240)),
            )
            .set(
                ThemeToken::Overlay,
                Style::default().bg(Color::Rgb(230, 230, 230)),
            )
            .set(ThemeToken::Border, Style::default().fg(Color::Gray))
            .set(ThemeToken::BorderFocused, Style::default().fg(Color::Blue))
            .set(
                ThemeToken::Selection,
                Style::default().bg(Color::Rgb(180, 210, 255)),
            )
            .set(
                ThemeToken::Cursor,
                Style::default().fg(Color::White).bg(Color::Black),
            )
            .set(ThemeToken::Placeholder, Style::default().fg(Color::Gray))
            .set(
                ThemeToken::StatusBar,
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Rgb(230, 230, 240)),
            )
            .set(
                ThemeToken::Title,
                style_with_modifier(Color::Black, Modifier::BOLD),
            )
            .set(
                ThemeToken::DiffAdd,
                Style::default().fg(Color::Rgb(0, 100, 0)),
            )
            .set(
                ThemeToken::DiffRemove,
                Style::default().fg(Color::Rgb(150, 0, 0)),
            )
            .set(
                ThemeToken::CodeBlock,
                Style::default().bg(Color::Rgb(245, 245, 245)),
            )
    }
}

/// Helper to create a Style with fg color + modifier (since add_modifier is a field).
fn style_with_modifier(fg: Color, modifier: Modifier) -> Style {
    let mut s = Style::default().fg(fg);
    s.add_modifier = modifier;
    s
}
