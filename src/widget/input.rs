use crate::core::buffer::Buffer;
use crate::core::rect::Rect;
use crate::core::style::Style;
use crate::ontology::{
    ActionParam, ActionParamType, AgentAction, AgentCapability, Discoverable, PropertySchema,
    PropertyType, SemanticRole, WidgetSchema,
};
use crate::widget::block::Block;
use crate::widget::Widget;

/// A single-line text input widget.
#[derive(Debug, Clone)]
pub struct Input {
    block: Option<Block>,
    style: Style,
    cursor_style: Style,
    placeholder: String,
    placeholder_style: Style,
}

impl Input {
    pub fn new() -> Self {
        Self {
            block: None,
            style: Style::default(),
            cursor_style: Style::default().reversed(),
            placeholder: String::new(),
            placeholder_style: Style::default().dim(),
        }
    }

    pub fn block(mut self, block: Block) -> Self {
        self.block = Some(block);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn cursor_style(mut self, style: Style) -> Self {
        self.cursor_style = style;
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn placeholder_style(mut self, style: Style) -> Self {
        self.placeholder_style = style;
        self
    }
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

/// State for an [`Input`] widget.
#[derive(Debug, Clone, Default)]
pub struct InputState {
    pub value: String,
    pub cursor: usize,
    pub scroll_offset: usize,
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self.cursor = self.value.len();
        self
    }

    pub fn insert_char(&mut self, ch: char) {
        if self.cursor <= self.value.len() {
            self.value.insert(self.cursor, ch);
            self.cursor += ch.len_utf8();
        }
    }

    pub fn delete_char_before(&mut self) {
        if self.cursor > 0 {
            let prev = self.value[..self.cursor]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.value.remove(prev);
            self.cursor = prev;
        }
    }

    pub fn delete_char_after(&mut self) {
        if self.cursor < self.value.len() {
            self.value.remove(self.cursor);
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor = self.value[..self.cursor]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor < self.value.len() {
            self.cursor = self.value[self.cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor + i)
                .unwrap_or(self.value.len());
        }
    }

    pub fn move_start(&mut self) {
        self.cursor = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor = self.value.len();
    }

    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
        self.scroll_offset = 0;
    }
}

impl crate::widget::StatefulWidget for Input {
    type State = InputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut InputState) {
        if area.is_empty() {
            return;
        }

        buf.set_style(area, self.style);

        let inner = if let Some(block) = self.block {
            let inner = block.inner(area);
            block.render(area, buf);
            inner
        } else {
            area
        };

        if inner.is_empty() {
            return;
        }

        let width = inner.width as usize;
        let y = inner.y;

        if state.value.is_empty() {
            // Show placeholder
            let display: String = self.placeholder.chars().take(width).collect();
            buf.set_string(inner.x, y, &display, self.placeholder_style);
            return;
        }

        // Ensure cursor is visible
        if state.cursor < state.scroll_offset {
            state.scroll_offset = state.cursor;
        } else if state.cursor >= state.scroll_offset + width {
            state.scroll_offset = state.cursor - width + 1;
        }

        // Render visible portion
        let visible: String = state
            .value
            .chars()
            .skip(state.scroll_offset)
            .take(width)
            .collect();
        buf.set_string(inner.x, y, &visible, self.style);

        // Cursor highlight
        let cursor_display_pos = state.cursor - state.scroll_offset;
        if cursor_display_pos < width {
            let cx = inner.x + cursor_display_pos as u16;
            buf.set_style(Rect::new(cx, y, 1, 1), self.cursor_style);
        }
    }
}

impl Discoverable for Input {
    fn schema() -> WidgetSchema {
        WidgetSchema {
            name: "Input".into(),
            description: "A single-line text input field with cursor navigation and editing."
                .into(),
            default_role: SemanticRole::Input,
            properties: vec![PropertySchema {
                name: "placeholder".into(),
                description: "Hint text shown when the input is empty.".into(),
                property_type: PropertyType::String,
                required: false,
                default_value: None,
                constraints: vec![],
            }],
            actions: vec![
                AgentAction {
                    name: "set_value".into(),
                    description: "Set the input text value.".into(),
                    params: vec![ActionParam {
                        name: "value".into(),
                        description: "The text to set.".into(),
                        param_type: ActionParamType::String,
                        required: true,
                        default_value: None,
                    }],
                    returns: None,
                    mutates: true,
                    idempotent: true,
                    shortcut: None,
                },
                AgentAction {
                    name: "get_value".into(),
                    description: "Get the current input text.".into(),
                    params: vec![],
                    returns: Some("Current text value.".into()),
                    mutates: false,
                    idempotent: true,
                    shortcut: None,
                },
                AgentAction {
                    name: "clear".into(),
                    description: "Clear the input.".into(),
                    params: vec![],
                    returns: None,
                    mutates: true,
                    idempotent: true,
                    shortcut: None,
                },
                AgentAction {
                    name: "insert_text".into(),
                    description: "Insert text at the cursor position.".into(),
                    params: vec![ActionParam {
                        name: "text".into(),
                        description: "Text to insert.".into(),
                        param_type: ActionParamType::String,
                        required: true,
                        default_value: None,
                    }],
                    returns: None,
                    mutates: true,
                    idempotent: false,
                    shortcut: None,
                },
            ],

            usage_hint: Some("Input::new().placeholder(\"Type here...\")".into()),
            tags: vec![
                "input".into(),
                "text".into(),
                "form".into(),
                "editable".into(),
            ],
        }
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::Focusable,
            AgentCapability::TextInput {
                multiline: false,
                max_length: None,
            },
            AgentCapability::Copyable,
            AgentCapability::HasKeyBindings {
                bindings: vec![
                    ("Left".into(), "Move cursor left".into()),
                    ("Right".into(), "Move cursor right".into()),
                    ("Home".into(), "Move to start".into()),
                    ("End".into(), "Move to end".into()),
                    ("Backspace".into(), "Delete char before cursor".into()),
                    ("Delete".into(), "Delete char after cursor".into()),
                ],
            },
        ]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![
            AgentAction {
                name: "set_value".into(),
                description: "Set the input text value.".into(),
                params: vec![ActionParam {
                    name: "value".into(),
                    description: "The text to set.".into(),
                    param_type: ActionParamType::String,
                    required: true,
                    default_value: None,
                }],
                returns: None,
                mutates: true,
                idempotent: true,
                shortcut: None,
            },
            AgentAction {
                name: "get_value".into(),
                description: "Get the current input text.".into(),
                params: vec![],
                returns: Some("Current text value.".into()),
                mutates: false,
                idempotent: true,
                shortcut: None,
            },
            AgentAction {
                name: "clear".into(),
                description: "Clear the input.".into(),
                params: vec![],
                returns: None,
                mutates: true,
                idempotent: true,
                shortcut: None,
            },
            AgentAction {
                name: "insert_text".into(),
                description: "Insert text at the cursor position.".into(),
                params: vec![ActionParam {
                    name: "text".into(),
                    description: "Text to insert.".into(),
                    param_type: ActionParamType::String,
                    required: true,
                    default_value: None,
                }],
                returns: None,
                mutates: true,
                idempotent: false,
                shortcut: None,
            },
        ]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Input
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "placeholder": self.placeholder,
        })
    }

    fn execute_action(
        &mut self,
        _action: &str,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        Err("Input actions require InputState. Use the runtime to dispatch actions.".into())
    }
}
