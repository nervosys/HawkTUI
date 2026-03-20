//! Terminal backend abstractions.
//!
//! The backend trait defines the interface between louie's rendering engine
//! and the actual terminal. The default implementation uses crossterm.

pub mod test;

#[cfg(feature = "crossterm")]
pub mod crossterm_backend;

use crate::core::cell::Cell;
use crate::core::rect::{Position, Size};
use crate::core::style::{Color, Modifier};
use std::io;

/// Trait for terminal backends.
///
/// A backend is responsible for writing styled characters to the terminal,
/// managing cursor visibility and position, and querying the terminal size.
pub trait Backend {
    /// Write changed cells to the terminal.
    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>;

    /// Hide the cursor.
    fn hide_cursor(&mut self) -> io::Result<()>;

    /// Show the cursor.
    fn show_cursor(&mut self) -> io::Result<()>;

    /// Move the cursor to a position.
    fn set_cursor_position(&mut self, position: Position) -> io::Result<()>;

    /// Get the cursor position.
    fn get_cursor_position(&mut self) -> io::Result<Position>;

    /// Clear the terminal.
    fn clear(&mut self) -> io::Result<()>;

    /// Get the terminal size.
    fn size(&self) -> io::Result<Size>;

    /// Flush pending output.
    fn flush(&mut self) -> io::Result<()>;

    /// Enable mouse capture.
    fn enable_mouse_capture(&mut self) -> io::Result<()>;

    /// Disable mouse capture.
    fn disable_mouse_capture(&mut self) -> io::Result<()>;

    /// Enter alternate screen mode.
    fn enter_alternate_screen(&mut self) -> io::Result<()>;

    /// Leave alternate screen mode.
    fn leave_alternate_screen(&mut self) -> io::Result<()>;

    /// Enable raw mode.
    fn enable_raw_mode(&mut self) -> io::Result<()>;

    /// Disable raw mode.
    fn disable_raw_mode(&mut self) -> io::Result<()>;

    /// Enable bracketed paste.
    fn enable_bracketed_paste(&mut self) -> io::Result<()> {
        Ok(())
    }

    /// Disable bracketed paste.
    fn disable_bracketed_paste(&mut self) -> io::Result<()> {
        Ok(())
    }

    /// Begin synchronized output (CSI ?2026h).
    ///
    /// When supported by the terminal, all output between `begin_sync` and
    /// `end_sync` is buffered and rendered atomically, eliminating flicker.
    fn begin_sync(&mut self) -> io::Result<()> {
        Ok(())
    }

    /// End synchronized output (CSI ?2026l).
    fn end_sync(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Map louie Color to crossterm Color.
#[cfg(feature = "crossterm")]
pub(crate) fn to_crossterm_color(color: Color) -> crossterm::style::Color {
    match color {
        Color::Reset => crossterm::style::Color::Reset,
        Color::Black => crossterm::style::Color::Black,
        Color::Red => crossterm::style::Color::DarkRed,
        Color::Green => crossterm::style::Color::DarkGreen,
        Color::Yellow => crossterm::style::Color::DarkYellow,
        Color::Blue => crossterm::style::Color::DarkBlue,
        Color::Magenta => crossterm::style::Color::DarkMagenta,
        Color::Cyan => crossterm::style::Color::DarkCyan,
        Color::Gray => crossterm::style::Color::Grey,
        Color::DarkGray => crossterm::style::Color::DarkGrey,
        Color::LightRed => crossterm::style::Color::Red,
        Color::LightGreen => crossterm::style::Color::Green,
        Color::LightYellow => crossterm::style::Color::Yellow,
        Color::LightBlue => crossterm::style::Color::Blue,
        Color::LightMagenta => crossterm::style::Color::Magenta,
        Color::LightCyan => crossterm::style::Color::Cyan,
        Color::White => crossterm::style::Color::White,
        Color::Indexed(i) => crossterm::style::Color::AnsiValue(i),
        Color::Rgb(r, g, b) => crossterm::style::Color::Rgb { r, g, b },
    }
}

/// Map louie Modifier to crossterm Attributes.
#[cfg(feature = "crossterm")]
pub(crate) fn to_crossterm_attributes(modifier: Modifier) -> Vec<crossterm::style::Attribute> {
    use crossterm::style::Attribute;
    let mut attrs = Vec::new();
    if modifier.contains(Modifier::BOLD) {
        attrs.push(Attribute::Bold);
    }
    if modifier.contains(Modifier::DIM) {
        attrs.push(Attribute::Dim);
    }
    if modifier.contains(Modifier::ITALIC) {
        attrs.push(Attribute::Italic);
    }
    if modifier.contains(Modifier::UNDERLINED) {
        attrs.push(Attribute::Underlined);
    }
    if modifier.contains(Modifier::SLOW_BLINK) {
        attrs.push(Attribute::SlowBlink);
    }
    if modifier.contains(Modifier::RAPID_BLINK) {
        attrs.push(Attribute::RapidBlink);
    }
    if modifier.contains(Modifier::REVERSED) {
        attrs.push(Attribute::Reverse);
    }
    if modifier.contains(Modifier::HIDDEN) {
        attrs.push(Attribute::Hidden);
    }
    if modifier.contains(Modifier::CROSSED_OUT) {
        attrs.push(Attribute::CrossedOut);
    }
    if modifier.contains(Modifier::OVERLINED) {
        attrs.push(Attribute::OverLined);
    }
    attrs
}
