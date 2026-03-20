//! Animated spinner/loader widget.
//!
//! Displays a spinner animation with an optional message string.
//! Useful for indicating background work is in progress.

use crate::core::buffer::Buffer;
use crate::core::rect::Rect;
use crate::core::style::Style;
use crate::ontology::{
    AgentAction, AgentCapability, Discoverable, PropertySchema, PropertyType, SemanticRole,
    WidgetSchema,
};
use crate::widget::Widget;

/// Spinner style presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpinnerStyle {
    /// Braille dots: ⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏
    Braille,
    /// Simple line: |/-\
    Line,
    /// Dots: ⣾⣽⣻⢿⡿⣟⣯⣷
    Dots,
    /// Arc: ◜◠◝◞◡◟
    Arc,
}

impl SpinnerStyle {
    pub fn frames(self) -> &'static [char] {
        match self {
            SpinnerStyle::Braille => &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'],
            SpinnerStyle::Line => &['|', '/', '-', '\\'],
            SpinnerStyle::Dots => &['⣾', '⣽', '⣻', '⢿', '⡿', '⣟', '⣯', '⣷'],
            SpinnerStyle::Arc => &['◜', '◠', '◝', '◞', '◡', '◟'],
        }
    }
}

/// An animated spinner/loader widget.
#[derive(Debug, Clone)]
pub struct Loader {
    message: String,
    spinner_style: SpinnerStyle,
    style: Style,
    spinner_fg: Style,
    tick: usize,
}

impl Loader {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            spinner_style: SpinnerStyle::Braille,
            style: Style::default(),
            spinner_fg: Style::default(),
            tick: 0,
        }
    }

    pub fn spinner_style(mut self, style: SpinnerStyle) -> Self {
        self.spinner_style = style;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn spinner_fg(mut self, style: Style) -> Self {
        self.spinner_fg = style;
        self
    }

    pub fn tick(mut self, tick: usize) -> Self {
        self.tick = tick;
        self
    }
}

impl Widget for Loader {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() || area.height == 0 {
            return;
        }

        buf.set_style(area, self.style);

        let frames = self.spinner_style.frames();
        let frame_char = frames[self.tick % frames.len()];

        let y = area.y;
        // Spinner character
        buf.set_string(area.x, y, &frame_char.to_string(), self.spinner_fg);

        // Message after spinner
        if !self.message.is_empty() && area.width > 2 {
            let max_msg = (area.width - 2) as usize;
            let msg: String = self.message.chars().take(max_msg).collect();
            buf.set_string(area.x + 2, y, &msg, self.style);
        }
    }
}

impl Discoverable for Loader {
    fn schema() -> WidgetSchema {
        WidgetSchema {
            name: "Loader".into(),
            description: "An animated spinner/loader with an optional message.".into(),
            default_role: SemanticRole::StatusBar,
            properties: vec![
                PropertySchema {
                    name: "message".into(),
                    description: "Text displayed alongside the spinner.".into(),
                    property_type: PropertyType::String,
                    required: false,
                    default_value: Some(serde_json::json!("")),
                    constraints: vec![],
                },
                PropertySchema {
                    name: "spinner_style".into(),
                    description: "Spinner animation preset: Braille, Line, Dots, or Arc.".into(),
                    property_type: PropertyType::String,
                    required: false,
                    default_value: Some(serde_json::json!("Braille")),
                    constraints: vec![],
                },
            ],
            actions: vec![],

            usage_hint: Some(
                "Loader::new(\"Loading...\").spinner_style(SpinnerStyle::Braille).tick(n)".into(),
            ),
            tags: vec![
                "loader".into(),
                "spinner".into(),
                "progress".into(),
                "animation".into(),
            ],
        }
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![AgentCapability::Animated { playing: true }]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::StatusBar
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "widget": "Loader",
            "message": self.message,
            "tick": self.tick,
            "spinner_style": format!("{:?}", self.spinner_style),
        })
    }

    fn execute_action(
        &mut self,
        action: &str,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        Err(format!("Unknown action: {action}"))
    }
}
