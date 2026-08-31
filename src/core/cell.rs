use super::style::{Color, Modifier, Style};
use super::symbol::Symbol;

/// A single cell in the terminal buffer.
///
/// `Cell` is `Copy` and 24 bytes wide: the grapheme cluster is stored inline
/// (see [`Symbol`]) rather than behind a heap pointer, so allocating, resetting,
/// cloning, and diffing a buffer never touch the allocator and never run a
/// destructor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    /// The grapheme cluster displayed in this cell.
    pub symbol: Symbol,
    /// Foreground color.
    pub fg: Color,
    /// Background color.
    pub bg: Color,
    /// Underline color.
    pub underline_color: Color,
    /// Active text modifiers.
    pub modifier: Modifier,
}

impl Default for Cell {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl Cell {
    /// The empty/space cell constant.
    pub const EMPTY_SYMBOL: &'static str = " ";

    /// A reset cell: a space with default colors and no modifiers.
    pub const EMPTY: Self = Self {
        symbol: Symbol::SPACE,
        fg: Color::Reset,
        bg: Color::Reset,
        underline_color: Color::Reset,
        modifier: Modifier::NONE,
    };

    /// Set the grapheme cluster for this cell.
    pub fn set_symbol(&mut self, symbol: &str) -> &mut Self {
        self.symbol = Symbol::new(symbol);
        self
    }

    /// Set a single character.
    pub fn set_char(&mut self, ch: char) -> &mut Self {
        self.symbol = Symbol::from_char(ch);
        self
    }

    /// The grapheme cluster as a string slice.
    pub fn symbol(&self) -> &str {
        self.symbol.as_str()
    }

    /// Apply a style to this cell (merging).
    pub fn set_style(&mut self, style: Style) -> &mut Self {
        if let Some(fg) = style.fg {
            self.fg = fg;
        }
        if let Some(bg) = style.bg {
            self.bg = bg;
        }
        if let Some(uc) = style.underline_color {
            self.underline_color = uc;
        }
        self.modifier = self
            .modifier
            .difference(style.sub_modifier)
            .union(style.add_modifier);
        self
    }

    /// Get the current style of this cell.
    pub fn style(&self) -> Style {
        Style {
            fg: Some(self.fg),
            bg: Some(self.bg),
            underline_color: Some(self.underline_color),
            add_modifier: self.modifier,
            sub_modifier: Modifier::NONE,
        }
    }

    /// Reset this cell to empty.
    pub fn reset(&mut self) {
        *self = Self::EMPTY;
    }

    /// Whether this cell has the default empty content.
    pub fn is_empty(&self) -> bool {
        self.symbol == Symbol::SPACE
            && self.fg == Color::Reset
            && self.bg == Color::Reset
            && self.modifier.is_empty()
    }
}
