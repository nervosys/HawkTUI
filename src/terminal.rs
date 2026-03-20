//! Terminal management: double-buffered rendering with Frame API.

use crate::backend::Backend;
use crate::core::buffer::Buffer;
use crate::core::rect::{Position, Rect};
use crate::event::HitMap;
use crate::widget::{StatefulWidget, Widget};
use std::io;

/// Manages the terminal lifecycle and double-buffered rendering.
pub struct Terminal<B: Backend> {
    backend: B,
    /// Front buffer (what's currently on screen).
    buffers: [Buffer; 2],
    /// Index of the current (front) buffer.
    current: usize,
    /// Hit map for mouse click resolution.
    hit_map: HitMap,
    /// Whether the cursor should be hidden.
    hidden_cursor: bool,
    /// Viewport area.
    viewport_area: Rect,
}

impl<B: Backend> Terminal<B> {
    pub fn new(backend: B) -> io::Result<Self> {
        let size = backend.size()?;
        let area = Rect::new(0, 0, size.width, size.height);
        Ok(Self {
            backend,
            buffers: [Buffer::empty(area), Buffer::empty(area)],
            current: 0,
            hit_map: HitMap::new(),
            hidden_cursor: false,
            viewport_area: area,
        })
    }

    /// Get a reference to the backend.
    pub fn backend(&self) -> &B {
        &self.backend
    }

    /// Get a mutable reference to the backend.
    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    /// Get the current viewport area.
    pub fn area(&self) -> Rect {
        self.viewport_area
    }

    /// Get the hit map (for mouse event resolution).
    pub fn hit_map(&self) -> &HitMap {
        &self.hit_map
    }

    /// Initialize the terminal (alternate screen, raw mode, mouse, hide cursor).
    pub fn init(&mut self) -> io::Result<()> {
        self.backend.enable_raw_mode()?;
        self.backend.enter_alternate_screen()?;
        self.backend.enable_mouse_capture()?;
        self.backend.enable_bracketed_paste()?;
        self.backend.hide_cursor()?;
        self.backend.clear()?;
        self.backend.flush()?;
        self.hidden_cursor = true;
        Ok(())
    }

    /// Restore the terminal to its original state.
    pub fn restore(&mut self) -> io::Result<()> {
        self.backend.disable_bracketed_paste()?;
        self.backend.disable_mouse_capture()?;
        self.backend.leave_alternate_screen()?;
        self.backend.show_cursor()?;
        self.backend.disable_raw_mode()?;
        self.backend.flush()?;
        Ok(())
    }

    /// Automatically detect and apply the current terminal size.
    pub fn autoresize(&mut self) -> io::Result<()> {
        let size = self.backend.size()?;
        let area = Rect::new(0, 0, size.width, size.height);
        if area != self.viewport_area {
            self.viewport_area = area;
            self.buffers[0].resize(area);
            self.buffers[1].resize(area);
        }
        Ok(())
    }

    /// Draw a frame: clears the back buffer, runs the render closure,
    /// diffs against the front buffer, and writes changes to the terminal.
    pub fn draw<F>(&mut self, render: F) -> io::Result<()>
    where
        F: FnOnce(&mut Frame),
    {
        self.autoresize()?;

        // Clear hit map for this frame
        self.hit_map.clear();

        // Prepare the back buffer
        let back = 1 - self.current;
        self.buffers[back].reset();

        // Create a frame and let the user render into it
        let area = self.viewport_area;
        let mut frame = Frame {
            buffer: &mut self.buffers[back],
            area,
            hit_map: &mut self.hit_map,
            cursor_position: None,
        };
        render(&mut frame);

        let cursor_position = frame.cursor_position;

        // Begin synchronized output for flicker-free rendering
        self.backend.begin_sync()?;

        // Diff the buffers and write changes
        let changes = self.buffers[self.current].diff(&self.buffers[back]);
        self.backend.draw(changes.into_iter())?;

        // Handle cursor
        if let Some(pos) = cursor_position {
            self.backend.show_cursor()?;
            self.backend.set_cursor_position(pos)?;
            self.hidden_cursor = false;
        } else if !self.hidden_cursor {
            self.backend.hide_cursor()?;
            self.hidden_cursor = true;
        }

        // End synchronized output and flush
        self.backend.end_sync()?;
        self.backend.flush()?;

        // Swap buffers
        self.current = back;

        Ok(())
    }
}

impl<B: Backend> Drop for Terminal<B> {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

/// A single frame for rendering.
///
/// The frame provides the primary API that widgets use to write into the
/// terminal buffer. It also tracks hit regions for mouse interaction.
pub struct Frame<'a> {
    buffer: &'a mut Buffer,
    area: Rect,
    hit_map: &'a mut HitMap,
    cursor_position: Option<Position>,
}

impl<'a> Frame<'a> {
    /// The full area available for rendering.
    pub fn area(&self) -> Rect {
        self.area
    }

    /// The underlying buffer.
    pub fn buffer_mut(&mut self) -> &mut Buffer {
        self.buffer
    }

    /// Render a widget into the given area.
    pub fn render_widget(&mut self, widget: impl Widget, area: Rect) {
        widget.render(area, self.buffer);
    }

    /// Render a stateful widget into the given area.
    pub fn render_stateful_widget<W: StatefulWidget>(
        &mut self,
        widget: W,
        area: Rect,
        state: &mut W::State,
    ) {
        widget.render(area, self.buffer, state);
    }

    /// Set the cursor position (makes it visible for this frame).
    pub fn set_cursor_position(&mut self, position: Position) {
        self.cursor_position = Some(position);
    }

    /// Register a clickable region for mouse hit-testing.
    pub fn register_clickable(&mut self, agent_id: impl Into<String>, area: Rect, z_index: u16) {
        self.hit_map.register(agent_id, area, z_index);
    }
}
