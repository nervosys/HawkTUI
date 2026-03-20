use super::Backend;
use crate::core::cell::Cell;
use crate::core::rect::{Position, Size};
use std::io::{self, Write};

/// Crossterm-based terminal backend.
///
/// Uses the crossterm crate for cross-platform terminal control.
pub struct CrosstermBackend<W: Write> {
    writer: W,
}

impl<W: Write> CrosstermBackend<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub fn writer(&self) -> &W {
        &self.writer
    }

    pub fn writer_mut(&mut self) -> &mut W {
        &mut self.writer
    }
}

impl<W: Write> Backend for CrosstermBackend<W> {
    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        use crossterm::{
            cursor::MoveTo,
            queue,
            style::{Attribute, Print, SetAttribute, SetBackgroundColor, SetForegroundColor},
        };

        let mut last_fg = None;
        let mut last_bg = None;
        let mut last_modifier = None;
        let mut last_pos: Option<(u16, u16)> = None;

        for (x, y, cell) in content {
            // Skip empty continuation cells (wide characters)
            if cell.symbol.is_empty() {
                continue;
            }

            // Only move cursor if not at the expected next position
            let needs_move = match last_pos {
                Some((lx, ly)) => !(ly == y && lx + 1 == x),
                None => true,
            };
            if needs_move {
                queue!(self.writer, MoveTo(x, y))?;
            }

            // Update foreground color if changed
            let fg = super::to_crossterm_color(cell.fg);
            if last_fg != Some(fg) {
                queue!(self.writer, SetForegroundColor(fg))?;
                last_fg = Some(fg);
            }

            // Update background color if changed
            let bg = super::to_crossterm_color(cell.bg);
            if last_bg != Some(bg) {
                queue!(self.writer, SetBackgroundColor(bg))?;
                last_bg = Some(bg);
            }

            // Update modifiers if changed
            if last_modifier != Some(cell.modifier) {
                queue!(self.writer, SetAttribute(Attribute::Reset))?;
                let attrs = super::to_crossterm_attributes(cell.modifier);
                for attr in attrs.into_iter() {
                    queue!(self.writer, SetAttribute(attr))?;
                }
                // Re-set colors after reset
                if let Some(fg) = last_fg {
                    queue!(self.writer, SetForegroundColor(fg))?;
                }
                if let Some(bg) = last_bg {
                    queue!(self.writer, SetBackgroundColor(bg))?;
                }
                last_modifier = Some(cell.modifier);
            }

            queue!(self.writer, Print(&cell.symbol))?;
            last_pos = Some((x, y));
        }

        // Reset attributes at the end
        queue!(
            self.writer,
            crossterm::style::SetAttribute(crossterm::style::Attribute::Reset)
        )?;

        Ok(())
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        crossterm::queue!(self.writer, crossterm::cursor::Hide)
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        crossterm::queue!(self.writer, crossterm::cursor::Show)
    }

    fn set_cursor_position(&mut self, position: Position) -> io::Result<()> {
        crossterm::queue!(
            self.writer,
            crossterm::cursor::MoveTo(position.x, position.y)
        )
    }

    fn get_cursor_position(&mut self) -> io::Result<Position> {
        let (x, y) = crossterm::cursor::position()?;
        Ok(Position::new(x, y))
    }

    fn clear(&mut self) -> io::Result<()> {
        crossterm::queue!(
            self.writer,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
        )
    }

    fn size(&self) -> io::Result<Size> {
        let (cols, rows) = crossterm::terminal::size()?;
        Ok(Size::new(cols, rows))
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }

    fn enable_mouse_capture(&mut self) -> io::Result<()> {
        crossterm::queue!(self.writer, crossterm::event::EnableMouseCapture)
    }

    fn disable_mouse_capture(&mut self) -> io::Result<()> {
        crossterm::queue!(self.writer, crossterm::event::DisableMouseCapture)
    }

    fn enter_alternate_screen(&mut self) -> io::Result<()> {
        crossterm::queue!(self.writer, crossterm::terminal::EnterAlternateScreen)
    }

    fn leave_alternate_screen(&mut self) -> io::Result<()> {
        crossterm::queue!(self.writer, crossterm::terminal::LeaveAlternateScreen)
    }

    fn enable_raw_mode(&mut self) -> io::Result<()> {
        crossterm::terminal::enable_raw_mode()
    }

    fn disable_raw_mode(&mut self) -> io::Result<()> {
        crossterm::terminal::disable_raw_mode()
    }

    fn enable_bracketed_paste(&mut self) -> io::Result<()> {
        crossterm::queue!(self.writer, crossterm::event::EnableBracketedPaste)
    }

    fn disable_bracketed_paste(&mut self) -> io::Result<()> {
        crossterm::queue!(self.writer, crossterm::event::DisableBracketedPaste)
    }

    fn begin_sync(&mut self) -> io::Result<()> {
        // CSI ?2026h — Begin Synchronized Output
        self.writer.write_all(b"\x1b[?2026h")
    }

    fn end_sync(&mut self) -> io::Result<()> {
        // CSI ?2026l — End Synchronized Output
        self.writer.write_all(b"\x1b[?2026l")
    }
}
