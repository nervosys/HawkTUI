//! Overlay / modal system.
//!
//! Overlays are rendered on top of the main application content. They capture
//! keyboard focus and can be dismissed programmatically or by the user.

use crate::core::buffer::Buffer;
use crate::core::rect::Rect;
use crate::core::style::Style;
use crate::widget::Widget;

/// An overlay entry in the overlay stack.
pub struct Overlay {
    /// Unique ID for agent addressability.
    pub id: String,
    /// Area where the overlay renders (absolute terminal coordinates).
    pub area: Rect,
    /// Whether this overlay captures all keyboard input.
    pub captures_focus: bool,
}

/// Manages a stack of overlays rendered on top of the main content.
///
/// Overlays are drawn in order (bottom to top). The topmost overlay that
/// captures focus receives all keyboard events.
#[derive(Default)]
pub struct OverlayStack {
    entries: Vec<Overlay>,
}

impl OverlayStack {
    pub fn new() -> Self {
        Self::default()
    }

    /// Push an overlay onto the stack.
    pub fn push(&mut self, overlay: Overlay) {
        self.entries.push(overlay);
    }

    /// Remove an overlay by ID. Returns true if found and removed.
    pub fn remove(&mut self, id: &str) -> bool {
        let before = self.entries.len();
        self.entries.retain(|o| o.id != id);
        self.entries.len() < before
    }

    /// The topmost overlay, if any.
    pub fn top(&self) -> Option<&Overlay> {
        self.entries.last()
    }

    /// Whether any overlay is currently capturing focus.
    pub fn has_focus_capture(&self) -> bool {
        self.entries.iter().rev().any(|o| o.captures_focus)
    }

    /// The ID of the overlay currently capturing focus, if any.
    pub fn focus_capture_id(&self) -> Option<&str> {
        self.entries
            .iter()
            .rev()
            .find(|o| o.captures_focus)
            .map(|o| o.id.as_str())
    }

    /// Whether the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Number of overlays on the stack.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Iterate over overlays from bottom to top.
    pub fn iter(&self) -> impl Iterator<Item = &Overlay> {
        self.entries.iter()
    }

    /// Clear all overlays.
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

/// A simple centered modal box widget.
///
/// Renders a bordered box centered in the given area, with a dimmed background.
pub struct ModalBox {
    title: String,
    style: Style,
    border_style: Style,
    /// Width as percentage of parent (0-100).
    width_percent: u16,
    /// Height as percentage of parent (0-100).
    height_percent: u16,
}

impl ModalBox {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            style: Style::default(),
            border_style: Style::default(),
            width_percent: 60,
            height_percent: 40,
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn border_style(mut self, style: Style) -> Self {
        self.border_style = style;
        self
    }

    pub fn width_percent(mut self, pct: u16) -> Self {
        self.width_percent = pct.min(100);
        self
    }

    pub fn height_percent(mut self, pct: u16) -> Self {
        self.height_percent = pct.min(100);
        self
    }

    /// Calculate the inner area for the modal content, given the parent area.
    pub fn inner_area(&self, parent: Rect) -> Rect {
        let w = (parent.width as u32 * self.width_percent as u32 / 100) as u16;
        let h = (parent.height as u32 * self.height_percent as u32 / 100) as u16;
        let x = parent.x + (parent.width.saturating_sub(w)) / 2;
        let y = parent.y + (parent.height.saturating_sub(h)) / 2;
        // Inner area is 1 cell smaller on each side for the border
        Rect::new(x + 1, y + 1, w.saturating_sub(2), h.saturating_sub(2))
    }
}

impl Widget for ModalBox {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Dim the background
        let dim_style = Style::default().dim();
        buf.set_style(area, dim_style);

        // Calculate modal rect
        let w = (area.width as u32 * self.width_percent as u32 / 100) as u16;
        let h = (area.height as u32 * self.height_percent as u32 / 100) as u16;
        let x = area.x + (area.width.saturating_sub(w)) / 2;
        let y = area.y + (area.height.saturating_sub(h)) / 2;
        let modal_area = Rect::new(x, y, w, h);

        // Fill modal background
        buf.fill(modal_area, " ", self.style);

        // Draw border
        if w >= 2 && h >= 2 {
            // Top and bottom edges
            for bx in x..x + w {
                buf.set_string(bx, y, "─", self.border_style);
                buf.set_string(bx, y + h - 1, "─", self.border_style);
            }
            // Left and right edges
            for by in y..y + h {
                buf.set_string(x, by, "│", self.border_style);
                buf.set_string(x + w - 1, by, "│", self.border_style);
            }
            // Corners
            buf.set_string(x, y, "┌", self.border_style);
            buf.set_string(x + w - 1, y, "┐", self.border_style);
            buf.set_string(x, y + h - 1, "└", self.border_style);
            buf.set_string(x + w - 1, y + h - 1, "┘", self.border_style);

            // Title
            if !self.title.is_empty() && w > 4 {
                let max_title = (w - 4) as usize;
                let title: String = self.title.chars().take(max_title).collect();
                buf.set_string(x + 2, y, &title, self.border_style);
            }
        }
    }
}
