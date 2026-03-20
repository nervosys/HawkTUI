//! Cancellable loader widget.
//!
//! Extends [`Loader`](super::loader::Loader) with cancellation support.
//! Displays a spinner with a message and an "Esc to cancel" hint.
//! Agents can cancel via the `cancel` action.

use crate::core::buffer::Buffer;
use crate::core::rect::Rect;
use crate::core::style::Style;
use crate::ontology::{
    AgentAction, AgentCapability, Discoverable, PropertySchema, PropertyType, SemanticRole,
    WidgetSchema,
};
use crate::widget::Widget;

use super::loader::SpinnerStyle;

/// A cancellable animated loader widget.
///
/// Renders a spinner + message + cancel hint, and tracks whether the user
/// or agent has requested cancellation.
#[derive(Debug, Clone)]
pub struct CancellableLoader {
    message: String,
    cancel_hint: String,
    spinner_style: SpinnerStyle,
    style: Style,
    spinner_fg: Style,
    hint_style: Style,
    tick: usize,
    cancelled: bool,
}

impl CancellableLoader {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            cancel_hint: "Esc to cancel".into(),
            spinner_style: SpinnerStyle::Braille,
            style: Style::default(),
            spinner_fg: Style::default(),
            hint_style: Style::default(),
            tick: 0,
            cancelled: false,
        }
    }

    pub fn cancel_hint(mut self, hint: impl Into<String>) -> Self {
        self.cancel_hint = hint.into();
        self
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

    pub fn hint_style(mut self, style: Style) -> Self {
        self.hint_style = style;
        self
    }

    pub fn tick(mut self, tick: usize) -> Self {
        self.tick = tick;
        self
    }

    pub fn cancelled(mut self, cancelled: bool) -> Self {
        self.cancelled = cancelled;
        self
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Widget for CancellableLoader {
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

        // Cancel hint on the right side
        if !self.cancel_hint.is_empty() && area.width > 4 {
            let hint_len = self.cancel_hint.len() as u16;
            if hint_len < area.width.saturating_sub(4) {
                let hint_x = area.x + area.width - hint_len;
                buf.set_string(hint_x, y, &self.cancel_hint, self.hint_style);
            }
        }
    }
}

impl Discoverable for CancellableLoader {
    fn schema() -> WidgetSchema {
        WidgetSchema {
            name: "CancellableLoader".into(),
            description: "An animated spinner/loader with cancellation support.".into(),
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
                    name: "cancelled".into(),
                    description: "Whether cancellation has been requested.".into(),
                    property_type: PropertyType::Boolean,
                    required: false,
                    default_value: Some(serde_json::json!(false)),
                    constraints: vec![],
                },
            ],
            actions: vec![],

            usage_hint: Some("CancellableLoader::new(\"Processing...\").tick(n)".into()),
            tags: vec![
                "loader".into(),
                "spinner".into(),
                "progress".into(),
                "cancellable".into(),
            ],
        }
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![AgentCapability::Animated { playing: true }]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![AgentAction {
            name: "cancel".into(),
            description: "Cancel the running operation.".into(),
            params: vec![],
            returns: None,
            mutates: true,
            idempotent: true,
            shortcut: Some("Esc".into()),
        }]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::StatusBar
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "widget": "CancellableLoader",
            "message": self.message,
            "cancelled": self.cancelled,
            "tick": self.tick,
        })
    }

    fn execute_action(
        &mut self,
        action: &str,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match action {
            "cancel" => {
                self.cancelled = true;
                Ok(serde_json::json!({"cancelled": true}))
            }
            _ => Err(format!("Unknown action: {action}")),
        }
    }
}
