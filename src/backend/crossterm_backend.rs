use super::Backend;
use crate::core::cell::Cell;
use crate::core::rect::{Position, Size};
use crate::core::style::{Color, Modifier};
use std::io::{self, Write};

/// Crossterm-based terminal backend.
///
/// Uses the crossterm crate for cross-platform terminal control.
pub struct CrosstermBackend<W: Write> {
    writer: W,
    /// Scratch buffer for one frame of escape sequences, reused across draws so
    /// steady-state rendering performs no allocation.
    scratch: Vec<u8>,
}

impl<W: Write> CrosstermBackend<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            scratch: Vec::new(),
        }
    }

    pub fn writer(&self) -> &W {
        &self.writer
    }

    pub fn writer_mut(&mut self) -> &mut W {
        &mut self.writer
    }
}

impl<W: Write> CrosstermBackend<W> {
    /// Encode a frame of cells, each with an optional hyperlink target, into
    /// the scratch buffer and write it out in one call.
    ///
    /// Escape sequences are written directly rather than through a command
    /// layer, and every attribute — position, colors, modifiers, hyperlink — is
    /// emitted only when it actually changes from the previous cell.
    fn encode_frame<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell, Option<&'a str>)>,
    {
        use super::ansi;

        let out = &mut self.scratch;
        out.clear();

        let mut last_fg: Option<Color> = None;
        let mut last_bg: Option<Color> = None;
        let mut last_underline: Option<Color> = None;
        let mut last_modifier: Option<Modifier> = None;
        let mut next_pos: Option<(u16, u16)> = None;
        let mut open_link: Option<&str> = None;

        for (x, y, cell, link) in content {
            // Skip the continuation half of a wide grapheme.
            if cell.symbol.is_empty() {
                continue;
            }

            if next_pos != Some((x, y)) {
                ansi::move_to(out, x, y);
            }

            // Emit only the attributes that actually changed. Turning a single
            // flag off costs one short sequence and — unlike a full SGR reset —
            // leaves the colors intact, so they need no re-assertion.
            if last_modifier != Some(cell.modifier) {
                let previous = last_modifier.unwrap_or(Modifier::NONE);
                ansi::diff_modifiers(out, previous, cell.modifier);
                last_modifier = Some(cell.modifier);
            }

            if last_fg != Some(cell.fg) {
                ansi::set_color(out, cell.fg, true);
                last_fg = Some(cell.fg);
            }
            if last_bg != Some(cell.bg) {
                ansi::set_color(out, cell.bg, false);
                last_bg = Some(cell.bg);
            }
            if cell.underline_color != Color::Reset && last_underline != Some(cell.underline_color)
            {
                ansi::set_underline_color(out, cell.underline_color);
                last_underline = Some(cell.underline_color);
            }

            if open_link != link {
                match link {
                    Some(url) => ansi::open_hyperlink(out, url),
                    None => ansi::close_hyperlink(out),
                }
                open_link = link;
            }

            out.extend_from_slice(cell.symbol.as_str().as_bytes());
            next_pos = Some((x.saturating_add(1), y));
        }

        if open_link.is_some() {
            ansi::close_hyperlink(out);
        }
        ansi::reset_attributes(out);
        self.writer.write_all(out)
    }
}

impl<W: Write> Backend for CrosstermBackend<W> {
    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        self.encode_frame(content.map(|(x, y, cell)| (x, y, cell, None)))
    }

    fn draw_linked<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell, Option<&'a str>)>,
    {
        self.encode_frame(content)
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
