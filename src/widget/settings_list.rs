use crate::core::buffer::Buffer;
use crate::core::rect::Rect;
use crate::core::style::Style;
use crate::ontology::{
    ActionParam, ActionParamType, AgentAction, AgentCapability, Discoverable, PropertySchema,
    PropertyType, SemanticRole, WidgetSchema,
};
use crate::widget::block::Block;
use crate::widget::{StatefulWidget, Widget};
use unicode_width::UnicodeWidthStr;

/// A single setting entry with a label, a set of possible values, and
/// an optional description.
#[derive(Debug, Clone)]
pub struct Setting {
    /// The setting label (displayed on the left).
    pub label: String,
    /// Available values to cycle through.
    pub values: Vec<String>,
    /// Currently selected value index.
    pub selected: usize,
    /// Optional description shown below the setting.
    pub description: Option<String>,
}

impl Setting {
    pub fn new(label: impl Into<String>, values: impl Into<Vec<String>>) -> Self {
        Self {
            label: label.into(),
            values: values.into(),
            selected: 0,
            description: None,
        }
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index.min(self.values.len().saturating_sub(1));
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Get the currently selected value.
    pub fn current_value(&self) -> &str {
        self.values
            .get(self.selected)
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    /// Cycle to the next value.
    pub fn cycle_next(&mut self) {
        if !self.values.is_empty() {
            self.selected = (self.selected + 1) % self.values.len();
        }
    }

    /// Cycle to the previous value.
    pub fn cycle_prev(&mut self) {
        if !self.values.is_empty() {
            self.selected = if self.selected == 0 {
                self.values.len() - 1
            } else {
                self.selected - 1
            };
        }
    }
}

/// State for the settings list (tracks cursor position and scroll offset).
#[derive(Debug, Clone)]
pub struct SettingsListState {
    /// Currently focused setting index.
    pub cursor: usize,
    /// Scroll offset for when settings exceed the display area.
    pub offset: usize,
}

impl SettingsListState {
    pub fn new() -> Self {
        Self {
            cursor: 0,
            offset: 0,
        }
    }

    pub fn move_up(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn move_down(&mut self, item_count: usize) {
        if item_count > 0 {
            self.cursor = (self.cursor + 1).min(item_count - 1);
        }
    }
}

impl Default for SettingsListState {
    fn default() -> Self {
        Self::new()
    }
}

/// An interactive settings list widget displaying key-value pairs with
/// cycleable values.
///
/// Each setting has a label, a set of possible values, and an optional
/// description. The user (or agent) can navigate with cursor keys and
/// cycle values with Enter or arrow keys.
///
/// # Example
///
/// ```ignore
/// use hawktui::widget::settings_list::{SettingsList, Setting, SettingsListState};
///
/// let settings = vec![
///     Setting::new("Theme", vec!["Dark".into(), "Light".into(), "Auto".into()])
///         .description("Color scheme for the editor"),
///     Setting::new("Font Size", vec!["12".into(), "14".into(), "16".into(), "18".into()])
///         .selected(1),
///     Setting::new("Word Wrap", vec!["On".into(), "Off".into()]),
/// ];
///
/// let widget = SettingsList::new(settings);
/// let mut state = SettingsListState::new();
/// ```
#[derive(Debug, Clone)]
pub struct SettingsList {
    settings: Vec<Setting>,
    block: Option<Block>,
    style: Style,
    cursor_style: Style,
    label_style: Style,
    value_style: Style,
    description_style: Style,
    cursor_symbol: String,
    value_separator: String,
}

impl SettingsList {
    pub fn new(settings: impl Into<Vec<Setting>>) -> Self {
        Self {
            settings: settings.into(),
            block: None,
            style: Style::default(),
            cursor_style: Style::default(),
            label_style: Style::default(),
            value_style: Style::default(),
            description_style: Style::default(),
            cursor_symbol: "▸ ".into(),
            value_separator: " : ".into(),
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

    pub fn label_style(mut self, style: Style) -> Self {
        self.label_style = style;
        self
    }

    pub fn value_style(mut self, style: Style) -> Self {
        self.value_style = style;
        self
    }

    pub fn description_style(mut self, style: Style) -> Self {
        self.description_style = style;
        self
    }

    pub fn cursor_symbol(mut self, symbol: impl Into<String>) -> Self {
        self.cursor_symbol = symbol.into();
        self
    }

    pub fn value_separator(mut self, sep: impl Into<String>) -> Self {
        self.value_separator = sep.into();
        self
    }

    /// Get a reference to the settings.
    pub fn settings(&self) -> &[Setting] {
        &self.settings
    }

    /// Get a mutable reference to the settings.
    pub fn settings_mut(&mut self) -> &mut [Setting] {
        &mut self.settings
    }
}

impl StatefulWidget for SettingsList {
    type State = SettingsListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if area.is_empty() {
            return;
        }

        buf.set_style(area, self.style);

        let inner = if let Some(ref block) = self.block {
            let inner = block.inner(area);
            block.clone().render(area, buf);
            inner
        } else {
            area
        };

        if inner.is_empty() || self.settings.is_empty() {
            return;
        }

        // Compute label column width
        let cursor_w = self.cursor_symbol.width() as u16;
        let sep_w = self.value_separator.width() as u16;
        let max_label_w = self
            .settings
            .iter()
            .map(|s| s.label.width() as u16)
            .max()
            .unwrap_or(0);
        let label_col = cursor_w + max_label_w;

        // Adjust scroll offset to keep cursor visible
        let visible_lines = inner.height as usize;
        if state.cursor < state.offset {
            state.offset = state.cursor;
        }
        // Each setting takes 1 line (+ 1 for description if it has one and is selected)
        // Ensure cursor stays within visible range
        let _ = state.offset..state.cursor;
        if state.cursor >= state.offset + visible_lines {
            state.offset = state.cursor + 1 - visible_lines;
        }

        let mut y = inner.y;
        for (i, setting) in self.settings.iter().enumerate().skip(state.offset) {
            if y >= inner.bottom() {
                break;
            }

            let is_selected = i == state.cursor;

            // Cursor marker
            let line_style = if is_selected {
                self.cursor_style
            } else {
                self.style
            };
            if is_selected {
                buf.set_string(inner.x, y, &self.cursor_symbol, self.cursor_style);
            }

            // Label
            let label_x = inner.x + cursor_w;
            let label_style = if is_selected {
                self.cursor_style
            } else {
                self.label_style
            };
            buf.set_string(label_x, y, &setting.label, label_style);

            // Separator
            let sep_x = inner.x + label_col;
            buf.set_string(sep_x, y, &self.value_separator, line_style);

            // Value
            let value_x = sep_x + sep_w;
            let value = setting.current_value();
            let value_style = if is_selected {
                self.cursor_style
            } else {
                self.value_style
            };
            let max_value_w = inner.right().saturating_sub(value_x) as usize;
            let truncated_value = if value.len() > max_value_w {
                &value[..max_value_w]
            } else {
                value
            };
            buf.set_string(value_x, y, truncated_value, value_style);

            y += 1;

            // Description for selected item
            if is_selected {
                if let Some(ref desc) = setting.description {
                    if y < inner.bottom() {
                        let desc_x = inner.x + cursor_w;
                        let max_desc_w = inner.right().saturating_sub(desc_x) as usize;
                        let truncated_desc = if desc.len() > max_desc_w {
                            &desc[..max_desc_w]
                        } else {
                            desc.as_str()
                        };
                        buf.set_string(desc_x, y, truncated_desc, self.description_style);
                        y += 1;
                    }
                }
            }
        }
    }
}

// Also implement Widget for non-stateful rendering (cursor at 0)
impl Widget for SettingsList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = SettingsListState::new();
        StatefulWidget::render(self, area, buf, &mut state);
    }
}

impl Discoverable for SettingsList {
    fn schema() -> WidgetSchema {
        WidgetSchema {
            name: "SettingsList".into(),
            description: "An interactive settings list with cycleable key-value pairs.".into(),
            default_role: SemanticRole::Navigation,
            properties: vec![PropertySchema {
                name: "settings".into(),
                description:
                    "Array of settings, each with label, values, and optional description.".into(),
                property_type: PropertyType::Array(Box::new(PropertyType::Object(vec![]))),
                required: true,
                default_value: None,
                constraints: vec![],
            }],
            actions: vec![],

            usage_hint: Some(
                "Navigate with up/down, cycle values with Enter or left/right.".into(),
            ),
            tags: vec![
                "settings".into(),
                "configuration".into(),
                "form".into(),
                "key-value".into(),
            ],
        }
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::Focusable,
            AgentCapability::Selectable {
                multi_select: false,
                item_count: self.settings.len(),
            },
        ]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![
            AgentAction {
                name: "cycle_next".into(),
                description: "Cycle the focused setting to its next value.".into(),
                params: vec![ActionParam {
                    name: "index".into(),
                    description: "Setting index to cycle (defaults to cursor position).".into(),
                    param_type: ActionParamType::Index,
                    required: false,
                    default_value: None,
                }],
                returns: Some("The new value after cycling.".into()),
                mutates: true,
                idempotent: false,
                shortcut: Some("Enter".into()),
            },
            AgentAction {
                name: "cycle_prev".into(),
                description: "Cycle the focused setting to its previous value.".into(),
                params: vec![ActionParam {
                    name: "index".into(),
                    description: "Setting index to cycle.".into(),
                    param_type: ActionParamType::Index,
                    required: false,
                    default_value: None,
                }],
                returns: Some("The new value after cycling.".into()),
                mutates: true,
                idempotent: false,
                shortcut: None,
            },
        ]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Navigation
    }

    fn agent_state(&self) -> serde_json::Value {
        let items: Vec<serde_json::Value> = self
            .settings
            .iter()
            .map(|s| {
                serde_json::json!({
                    "label": s.label,
                    "current_value": s.current_value(),
                    "values": s.values,
                    "selected_index": s.selected,
                })
            })
            .collect();
        serde_json::json!({
            "settings": items,
            "count": self.settings.len(),
        })
    }

    fn execute_action(
        &mut self,
        action: &str,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let index = params
            .get("index")
            .and_then(|v| v.as_u64())
            .map(|i| i as usize);

        match action {
            "cycle_next" => {
                let idx = index.unwrap_or(0);
                let setting = self.settings.get_mut(idx).ok_or("Invalid setting index")?;
                setting.cycle_next();
                Ok(serde_json::json!({ "value": setting.current_value() }))
            }
            "cycle_prev" => {
                let idx = index.unwrap_or(0);
                let setting = self.settings.get_mut(idx).ok_or("Invalid setting index")?;
                setting.cycle_prev();
                Ok(serde_json::json!({ "value": setting.current_value() }))
            }
            _ => Err(format!("Unknown action: {action}")),
        }
    }
}
