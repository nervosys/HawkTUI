//! Widget system.
//!
//! All visual components implement the [`Widget`] trait. Stateful components
//! use [`StatefulWidget`]. Both support agent discoverability via the
//! [`Discoverable`](crate::ontology::Discoverable) trait.

pub mod barchart;
pub mod block;
pub mod calendar;
pub mod cancellable_loader;
pub mod canvas;
pub mod chart;
pub mod editor;
pub mod gauge;
pub mod image;
pub mod input;
pub mod line_gauge;
pub mod list;
pub mod loader;
pub mod markdown;
pub mod paragraph;
pub mod scrollbar;
pub mod select_list;
pub mod settings_list;
pub mod sparkline;
pub mod table;
pub mod tabs;

use crate::core::buffer::Buffer;
use crate::core::rect::Rect;

/// A stateless widget that renders itself into a buffer.
///
/// Widgets are consumed when rendered (take `self`). Implement on `&MyWidget`
/// for reusable rendering.
pub trait Widget {
    fn render(self, area: Rect, buf: &mut Buffer);
}

/// A stateful widget that maintains state between render calls.
pub trait StatefulWidget {
    /// The mutable state type for this widget.
    type State;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State);
}

// Blanket implementations for references
impl Widget for &str {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_string(area.x, area.y, self, crate::core::style::Style::default());
    }
}

impl Widget for String {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_string(area.x, area.y, &self, crate::core::style::Style::default());
    }
}
