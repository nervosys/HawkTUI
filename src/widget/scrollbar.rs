use crate::core::buffer::Buffer;
use crate::core::rect::Rect;
use crate::core::style::Style;
use crate::ontology::{
    AgentAction, AgentCapability, Discoverable, PropertySchema, PropertyType, SemanticRole,
    WidgetSchema,
};
use crate::widget::StatefulWidget;

/// Orientation of the scrollbar track.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollbarOrientation {
    Vertical,
    Horizontal,
}

/// Scrollbar state tracking content and viewport sizes.
#[derive(Debug, Clone, Default)]
pub struct ScrollbarState {
    /// Total number of content units (e.g., lines).
    pub content_length: usize,
    /// Number of visible content units (viewport size).
    pub viewport_length: usize,
    /// Current scroll offset.
    pub position: usize,
}

impl ScrollbarState {
    pub fn new(content_length: usize, viewport_length: usize) -> Self {
        Self {
            content_length,
            viewport_length,
            position: 0,
        }
    }

    pub fn position(mut self, position: usize) -> Self {
        self.position = position;
        self
    }

    pub fn scroll_down(&mut self, amount: usize) {
        let max = self
            .content_length
            .saturating_sub(self.viewport_length);
        self.position = (self.position + amount).min(max);
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.position = self.position.saturating_sub(amount);
    }

    pub fn scroll_to_top(&mut self) {
        self.position = 0;
    }

    pub fn scroll_to_bottom(&mut self) {
        self.position = self.content_length.saturating_sub(self.viewport_length);
    }

    /// Returns the fraction scrolled (0.0..=1.0).
    pub fn fraction(&self) -> f64 {
        let max = self
            .content_length
            .saturating_sub(self.viewport_length);
        if max == 0 {
            0.0
        } else {
            self.position as f64 / max as f64
        }
    }
}

/// Symbols used to render a scrollbar.
#[derive(Debug, Clone)]
pub struct ScrollbarSymbols {
    pub track: &'static str,
    pub thumb: &'static str,
    pub begin: &'static str,
    pub end: &'static str,
}

impl Default for ScrollbarSymbols {
    fn default() -> Self {
        Self {
            track: "│",
            thumb: "█",
            begin: "↑",
            end: "↓",
        }
    }
}

/// A scrollbar indicator widget.
#[derive(Debug, Clone)]
pub struct Scrollbar {
    orientation: ScrollbarOrientation,
    style: Style,
    thumb_style: Style,
    symbols: ScrollbarSymbols,
}

impl Scrollbar {
    pub fn new(orientation: ScrollbarOrientation) -> Self {
        let symbols = match orientation {
            ScrollbarOrientation::Vertical => ScrollbarSymbols {
                track: "│",
                thumb: "█",
                begin: "↑",
                end: "↓",
            },
            ScrollbarOrientation::Horizontal => ScrollbarSymbols {
                track: "─",
                thumb: "█",
                begin: "←",
                end: "→",
            },
        };
        Self {
            orientation,
            style: Style::default(),
            thumb_style: Style::default(),
            symbols,
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn thumb_style(mut self, style: Style) -> Self {
        self.thumb_style = style;
        self
    }

    pub fn symbols(mut self, symbols: ScrollbarSymbols) -> Self {
        self.symbols = symbols;
        self
    }
}

impl Default for Scrollbar {
    fn default() -> Self {
        Self::new(ScrollbarOrientation::Vertical)
    }
}

impl StatefulWidget for Scrollbar {
    type State = ScrollbarState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut ScrollbarState) {
        if area.is_empty() {
            return;
        }

        match self.orientation {
            ScrollbarOrientation::Vertical => {
                self.render_vertical(area, buf, state);
            }
            ScrollbarOrientation::Horizontal => {
                self.render_horizontal(area, buf, state);
            }
        }
    }
}

impl Scrollbar {
    fn render_vertical(self, area: Rect, buf: &mut Buffer, state: &ScrollbarState) {
        let track_height = area.height as usize;
        if track_height < 3 {
            return;
        }

        let x = area.x;

        // Begin arrow
        buf[(x, area.y)].set_symbol(self.symbols.begin);
        buf[(x, area.y)].set_style(self.style);

        // End arrow
        buf[(x, area.bottom() - 1)].set_symbol(self.symbols.end);
        buf[(x, area.bottom() - 1)].set_style(self.style);

        let inner_height = track_height - 2;
        if inner_height == 0 {
            return;
        }

        // Draw track
        for i in 0..inner_height {
            let y = area.y + 1 + i as u16;
            buf[(x, y)].set_symbol(self.symbols.track);
            buf[(x, y)].set_style(self.style);
        }

        // Calculate thumb
        if state.content_length <= state.viewport_length {
            return; // No scrollbar needed
        }

        let thumb_size = ((state.viewport_length as f64 / state.content_length as f64)
            * inner_height as f64)
            .ceil() as usize;
        let thumb_size = thumb_size.max(1).min(inner_height);

        let max_offset = inner_height - thumb_size;
        let thumb_offset = (state.fraction() * max_offset as f64).round() as usize;

        for i in 0..thumb_size {
            let y = area.y + 1 + (thumb_offset + i) as u16;
            buf[(x, y)].set_symbol(self.symbols.thumb);
            buf[(x, y)].set_style(self.thumb_style);
        }
    }

    fn render_horizontal(self, area: Rect, buf: &mut Buffer, state: &ScrollbarState) {
        let track_width = area.width as usize;
        if track_width < 3 {
            return;
        }

        let y = area.y;

        // Begin arrow
        buf[(area.x, y)].set_symbol(self.symbols.begin);
        buf[(area.x, y)].set_style(self.style);

        // End arrow
        buf[(area.right() - 1, y)].set_symbol(self.symbols.end);
        buf[(area.right() - 1, y)].set_style(self.style);

        let inner_width = track_width - 2;
        if inner_width == 0 {
            return;
        }

        // Draw track
        for i in 0..inner_width {
            let x = area.x + 1 + i as u16;
            buf[(x, y)].set_symbol(self.symbols.track);
            buf[(x, y)].set_style(self.style);
        }

        if state.content_length <= state.viewport_length {
            return;
        }

        let thumb_size = ((state.viewport_length as f64 / state.content_length as f64)
            * inner_width as f64)
            .ceil() as usize;
        let thumb_size = thumb_size.max(1).min(inner_width);

        let max_offset = inner_width - thumb_size;
        let thumb_offset = (state.fraction() * max_offset as f64).round() as usize;

        for i in 0..thumb_size {
            let x = area.x + 1 + (thumb_offset + i) as u16;
            buf[(x, y)].set_symbol(self.symbols.thumb);
            buf[(x, y)].set_style(self.thumb_style);
        }
    }
}

impl Discoverable for Scrollbar {
    fn schema() -> WidgetSchema {
        WidgetSchema {
            name: "Scrollbar".into(),
            description: "A scroll indicator showing viewport position within content.".into(),
            default_role: SemanticRole::Scrollable,
            properties: vec![PropertySchema {
                name: "orientation".into(),
                description: "Vertical or Horizontal.".into(),
                property_type: PropertyType::Enum(vec!["Vertical".into(), "Horizontal".into()]),
                required: false,
                default_value: Some(serde_json::json!("Vertical")),
                constraints: vec![],
            }],
            usage_hint: Some(
                "Scrollbar::new(ScrollbarOrientation::Vertical)".into(),
            ),
            tags: vec!["scrollbar".into(), "scroll".into(), "indicator".into()],
        }
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![AgentCapability::Scrollable {
            vertical: self.orientation == ScrollbarOrientation::Vertical,
            horizontal: self.orientation == ScrollbarOrientation::Horizontal,
        }]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Scrollable
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "orientation": match self.orientation {
                ScrollbarOrientation::Vertical => "vertical",
                ScrollbarOrientation::Horizontal => "horizontal",
            }
        })
    }

    fn execute_action(
        &mut self,
        _action: &str,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        Err("Scrollbar actions require ScrollbarState.".into())
    }
}
