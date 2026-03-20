use super::style::{Color, Modifier, Style};
use compact_str::CompactString;

/// A single cell in the terminal buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    /// The grapheme cluster displayed in this cell.
    pub symbol: CompactString,
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
        Self {
            symbol: CompactString::const_new(" "),
            fg: Color::Reset,
            bg: Color::Reset,
            underline_color: Color::Reset,
            modifier: Modifier::NONE,
        }
    }
}

impl Cell {
    /// The empty/space cell constant.
    pub const EMPTY_SYMBOL: &'static str = " ";

    /// Set the grapheme cluster for this cell.
    pub fn set_symbol(&mut self, symbol: &str) -> &mut Self {
        self.symbol = CompactString::from(symbol);
        self
    }

    /// Set a single character.
    pub fn set_char(&mut self, ch: char) -> &mut Self {
        let mut buf = [0u8; 4];
        self.symbol = CompactString::from(ch.encode_utf8(&mut buf) as &str);
        self
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
        self.symbol = CompactString::const_new(" ");
        self.fg = Color::Reset;
        self.bg = Color::Reset;
        self.underline_color = Color::Reset;
        self.modifier = Modifier::NONE;
    }

    /// Whether this cell has the default empty content.
    pub fn is_empty(&self) -> bool {
        self.symbol == " "
            && self.fg == Color::Reset
            && self.bg == Color::Reset
            && self.modifier.is_empty()
    }
}
