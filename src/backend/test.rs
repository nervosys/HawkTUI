use super::Backend;
use crate::core::buffer::Buffer;
use crate::core::cell::Cell;
use crate::core::rect::{Position, Size};
use std::io;

/// In-memory backend for testing.
///
/// Records all operations for assertions without touching a real terminal.
pub struct TestBackend {
    buffer: Buffer,
    cursor: Position,
    cursor_visible: bool,
    mouse_capture: bool,
}

impl TestBackend {
    pub fn new(width: u16, height: u16) -> Self {
        use crate::core::rect::Rect;
        Self {
            buffer: Buffer::empty(Rect::new(0, 0, width, height)),
            cursor: Position::new(0, 0),
            cursor_visible: true,
            mouse_capture: false,
        }
    }

    /// Get the current buffer contents (for test assertions).
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    /// Get the text content at a specific row.
    pub fn row_text(&self, y: u16) -> String {
        let mut text = String::new();
        for x in 0..self.buffer.area.width {
            if let Some(cell) = self.buffer.cell(Position::new(x, y)) {
                text.push_str(cell.symbol.as_str());
            }
        }
        // Trim trailing spaces
        text.trim_end().to_string()
    }
}

impl Backend for TestBackend {
    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        for (x, y, cell) in content {
            if x < self.buffer.area.width && y < self.buffer.area.height {
                self.buffer[(x, y)] = *cell;
            }
        }
        Ok(())
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        self.cursor_visible = false;
        Ok(())
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        self.cursor_visible = true;
        Ok(())
    }

    fn set_cursor_position(&mut self, position: Position) -> io::Result<()> {
        self.cursor = position;
        Ok(())
    }

    fn get_cursor_position(&mut self) -> io::Result<Position> {
        Ok(self.cursor)
    }

    fn clear(&mut self) -> io::Result<()> {
        self.buffer.reset();
        Ok(())
    }

    fn size(&self) -> io::Result<Size> {
        Ok(Size::new(self.buffer.area.width, self.buffer.area.height))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn enable_mouse_capture(&mut self) -> io::Result<()> {
        self.mouse_capture = true;
        Ok(())
    }

    fn disable_mouse_capture(&mut self) -> io::Result<()> {
        self.mouse_capture = false;
        Ok(())
    }

    fn enter_alternate_screen(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn leave_alternate_screen(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn enable_raw_mode(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn disable_raw_mode(&mut self) -> io::Result<()> {
        Ok(())
    }
}
