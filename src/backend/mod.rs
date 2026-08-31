//! Terminal backend abstractions.
//!
//! The backend trait defines the interface between hawktui's rendering engine
//! and the actual terminal. The default implementation uses crossterm.

pub mod ansi;
pub mod test;

#[cfg(feature = "crossterm")]
pub mod crossterm_backend;

use crate::core::cell::Cell;
use crate::core::rect::{Position, Size};
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

    /// Write changed cells, each with an optional OSC 8 hyperlink target.
    ///
    /// Backends that cannot render hyperlinks — or do not care to — inherit the
    /// default, which drops the targets and draws the cells normally.
    fn draw_linked<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell, Option<&'a str>)>,
    {
        self.draw(content.map(|(x, y, cell, _)| (x, y, cell)))
    }

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
