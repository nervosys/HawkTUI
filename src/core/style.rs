use serde::{Deserialize, Serialize};
use std::fmt;

/// Terminal color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(Default)]
pub enum Color {
    #[default]
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,
    /// 256-color palette index.
    Indexed(u8),
    /// True color (24-bit RGB).
    Rgb(u8, u8, u8),
}


impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rgb(r, g, b) => write!(f, "#{r:02X}{g:02X}{b:02X}"),
            Self::Indexed(i) => write!(f, "color({i})"),
            other => write!(f, "{other:?}"),
        }
    }
}

/// Text modifiers (bitflags).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Modifier(u16);

impl Modifier {
    pub const NONE: Self = Self(0);
    pub const BOLD: Self = Self(1 << 0);
    pub const DIM: Self = Self(1 << 1);
    pub const ITALIC: Self = Self(1 << 2);
    pub const UNDERLINED: Self = Self(1 << 3);
    pub const SLOW_BLINK: Self = Self(1 << 4);
    pub const RAPID_BLINK: Self = Self(1 << 5);
    pub const REVERSED: Self = Self(1 << 6);
    pub const HIDDEN: Self = Self(1 << 7);
    pub const CROSSED_OUT: Self = Self(1 << 8);
    pub const DOUBLE_UNDERLINED: Self = Self(1 << 9);
    pub const UNDERCURLED: Self = Self(1 << 10);
    pub const UNDERDOTTED: Self = Self(1 << 11);
    pub const UNDERDASHED: Self = Self(1 << 12);
    pub const OVERLINED: Self = Self(1 << 13);
    pub const SUPERSCRIPT: Self = Self(1 << 14);
    pub const SUBSCRIPT: Self = Self(1 << 15);

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn difference(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl std::ops::BitOr for Modifier {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for Modifier {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::Not for Modifier {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

/// Complete styling specification for a terminal cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub underline_color: Option<Color>,
    pub add_modifier: Modifier,
    pub sub_modifier: Modifier,
}

impl Style {
    pub const fn new() -> Self {
        Self {
            fg: None,
            bg: None,
            underline_color: None,
            add_modifier: Modifier::NONE,
            sub_modifier: Modifier::NONE,
        }
    }

    /// Reset all style fields.
    pub const fn reset() -> Self {
        Self {
            fg: Some(Color::Reset),
            bg: Some(Color::Reset),
            underline_color: Some(Color::Reset),
            add_modifier: Modifier::NONE,
            sub_modifier: Modifier::NONE,
        }
    }

    pub const fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    pub const fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    pub const fn underline_color(mut self, color: Color) -> Self {
        self.underline_color = Some(color);
        self
    }

    pub const fn bold(mut self) -> Self {
        self.add_modifier = self.add_modifier.union(Modifier::BOLD);
        self
    }

    pub const fn dim(mut self) -> Self {
        self.add_modifier = self.add_modifier.union(Modifier::DIM);
        self
    }

    pub const fn italic(mut self) -> Self {
        self.add_modifier = self.add_modifier.union(Modifier::ITALIC);
        self
    }

    pub const fn underlined(mut self) -> Self {
        self.add_modifier = self.add_modifier.union(Modifier::UNDERLINED);
        self
    }

    pub const fn reversed(mut self) -> Self {
        self.add_modifier = self.add_modifier.union(Modifier::REVERSED);
        self
    }

    pub const fn crossed_out(mut self) -> Self {
        self.add_modifier = self.add_modifier.union(Modifier::CROSSED_OUT);
        self
    }

    pub const fn hidden(mut self) -> Self {
        self.add_modifier = self.add_modifier.union(Modifier::HIDDEN);
        self
    }

    pub const fn slow_blink(mut self) -> Self {
        self.add_modifier = self.add_modifier.union(Modifier::SLOW_BLINK);
        self
    }

    pub const fn rapid_blink(mut self) -> Self {
        self.add_modifier = self.add_modifier.union(Modifier::RAPID_BLINK);
        self
    }

    pub const fn overlined(mut self) -> Self {
        self.add_modifier = self.add_modifier.union(Modifier::OVERLINED);
        self
    }

    pub const fn not_bold(mut self) -> Self {
        self.sub_modifier = self.sub_modifier.union(Modifier::BOLD);
        self
    }

    pub const fn not_italic(mut self) -> Self {
        self.sub_modifier = self.sub_modifier.union(Modifier::ITALIC);
        self
    }

    pub const fn not_underlined(mut self) -> Self {
        self.sub_modifier = self.sub_modifier.union(Modifier::UNDERLINED);
        self
    }

    /// Merge another style on top of this one.
    /// `other`'s non-None fields override `self`'s.
    pub fn patch(mut self, other: Style) -> Self {
        if other.fg.is_some() {
            self.fg = other.fg;
        }
        if other.bg.is_some() {
            self.bg = other.bg;
        }
        if other.underline_color.is_some() {
            self.underline_color = other.underline_color;
        }
        self.add_modifier = self
            .add_modifier
            .difference(other.sub_modifier)
            .union(other.add_modifier);
        self.sub_modifier = self
            .sub_modifier
            .difference(other.add_modifier)
            .union(other.sub_modifier);
        self
    }
}

/// [Stylize] trait enables a fluent shorthand API.
///
/// ```
/// use louie::core::style::*;
/// let style = Style::new().red().on_blue().bold();
/// ```
pub trait Stylize: Sized {
    fn style(self, style: Style) -> Self;

    fn red(self) -> Self {
        self.style(Style::new().fg(Color::Red))
    }
    fn green(self) -> Self {
        self.style(Style::new().fg(Color::Green))
    }
    fn yellow(self) -> Self {
        self.style(Style::new().fg(Color::Yellow))
    }
    fn blue(self) -> Self {
        self.style(Style::new().fg(Color::Blue))
    }
    fn magenta(self) -> Self {
        self.style(Style::new().fg(Color::Magenta))
    }
    fn cyan(self) -> Self {
        self.style(Style::new().fg(Color::Cyan))
    }
    fn white(self) -> Self {
        self.style(Style::new().fg(Color::White))
    }
    fn gray(self) -> Self {
        self.style(Style::new().fg(Color::Gray))
    }

    fn on_red(self) -> Self {
        self.style(Style::new().bg(Color::Red))
    }
    fn on_green(self) -> Self {
        self.style(Style::new().bg(Color::Green))
    }
    fn on_yellow(self) -> Self {
        self.style(Style::new().bg(Color::Yellow))
    }
    fn on_blue(self) -> Self {
        self.style(Style::new().bg(Color::Blue))
    }
    fn on_magenta(self) -> Self {
        self.style(Style::new().bg(Color::Magenta))
    }
    fn on_cyan(self) -> Self {
        self.style(Style::new().bg(Color::Cyan))
    }
    fn on_white(self) -> Self {
        self.style(Style::new().bg(Color::White))
    }

    fn bold(self) -> Self {
        self.style(Style::new().bold())
    }
    fn dim(self) -> Self {
        self.style(Style::new().dim())
    }
    fn italic(self) -> Self {
        self.style(Style::new().italic())
    }
    fn underlined(self) -> Self {
        self.style(Style::new().underlined())
    }
    fn reversed(self) -> Self {
        self.style(Style::new().reversed())
    }
    fn crossed_out(self) -> Self {
        self.style(Style::new().crossed_out())
    }
}

impl Stylize for Style {
    fn style(self, other: Style) -> Self {
        self.patch(other)
    }
}
