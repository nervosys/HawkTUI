//! Interactive select list widget.
//!
//! A scrollable list with single or multi-select, keyboard navigation,
//! and optional filtering. Fully discoverable by agent systems.

use crate::core::buffer::Buffer;
use crate::core::rect::Rect;
use crate::core::style::Style;
use crate::ontology::{
    ActionParam, ActionParamType, AgentAction, AgentCapability, Discoverable, PropertySchema,
    PropertyType, SemanticRole, WidgetSchema,
};
use crate::widget::block::Block;
use crate::widget::{StatefulWidget, Widget};

/// Selection mode for the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectMode {
    Single,
    Multi,
}

/// An item in a [`SelectList`].
#[derive(Debug, Clone)]
pub struct SelectItem {
    pub label: String,
    pub value: String,
}

impl SelectItem {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }

    /// Convenience: label and value are the same.
    pub fn simple(s: impl Into<String>) -> Self {
        let s = s.into();
        Self {
            label: s.clone(),
            value: s,
        }
    }
}

/// An interactive select list widget.
#[derive(Debug, Clone)]
pub struct SelectList {
    block: Option<Block>,
    items: Vec<SelectItem>,
    mode: SelectMode,
    style: Style,
    highlight_style: Style,
    selected_marker: &'static str,
    unselected_marker: &'static str,
}

impl SelectList {
    pub fn new(items: Vec<SelectItem>) -> Self {
        Self {
            block: None,
            items,
            mode: SelectMode::Single,
            style: Style::default(),
            highlight_style: Style::default().reversed(),
            selected_marker: "[x] ",
            unselected_marker: "[ ] ",
        }
    }

    pub fn block(mut self, block: Block) -> Self {
        self.block = Some(block);
        self
    }

    pub fn mode(mut self, mode: SelectMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }

    pub fn markers(mut self, selected: &'static str, unselected: &'static str) -> Self {
        self.selected_marker = selected;
        self.unselected_marker = unselected;
        self
    }
}

/// State for a [`SelectList`].
#[derive(Debug, Clone, Default)]
pub struct SelectListState {
    /// Currently highlighted index.
    pub cursor: usize,
    /// Set of selected indices.
    pub selected: Vec<usize>,
    /// Vertical scroll offset.
    pub scroll: usize,
    /// Optional filter text. Only items containing this substring are shown.
    pub filter: String,
}

impl SelectListState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Move cursor up within the visible item count.
    pub fn move_up(&mut self, visible_count: usize) {
        if visible_count == 0 {
            return;
        }
        if self.cursor > 0 {
            self.cursor -= 1;
        } else {
            self.cursor = visible_count.saturating_sub(1);
        }
    }

    /// Move cursor down within the visible item count.
    pub fn move_down(&mut self, visible_count: usize) {
        if visible_count == 0 {
            return;
        }
        if self.cursor + 1 < visible_count {
            self.cursor += 1;
        } else {
            self.cursor = 0;
        }
    }

    /// Toggle selection of the item at the given original index.
    pub fn toggle_select(&mut self, original_idx: usize, mode: SelectMode) {
        match mode {
            SelectMode::Single => {
                self.selected.clear();
                self.selected.push(original_idx);
            }
            SelectMode::Multi => {
                if let Some(pos) = self.selected.iter().position(|&i| i == original_idx) {
                    self.selected.remove(pos);
                } else {
                    self.selected.push(original_idx);
                }
            }
        }
    }

    /// Returns the selected indices.
    pub fn selected_indices(&self) -> &[usize] {
        &self.selected
    }
}

impl StatefulWidget for SelectList {
    type State = SelectListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut SelectListState) {
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

        // Filter items
        let filter_lower = state.filter.to_lowercase();
        let visible_items: Vec<(usize, &SelectItem)> = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                if state.filter.is_empty() {
                    true
                } else {
                    item.label
                        .to_lowercase()
                        .contains(&filter_lower)
                }
            })
            .collect();

        let visible_count = visible_items.len();

        // Clamp cursor
        if visible_count > 0 {
            state.cursor = state.cursor.min(visible_count - 1);
        }

        // Adjust scroll to keep cursor visible
        let rows = inner.height as usize;
        if state.cursor < state.scroll {
            state.scroll = state.cursor;
        } else if state.cursor >= state.scroll + rows {
            state.scroll = state.cursor - rows + 1;
        }

        for row_offset in 0..rows {
            let vi = state.scroll + row_offset;
            if vi >= visible_count {
                break;
            }

            let (original_idx, item) = visible_items[vi];
            let y = inner.y + row_offset as u16;
            let is_highlighted = vi == state.cursor;
            let is_selected = state.selected.contains(&original_idx);

            let marker = if self.mode == SelectMode::Multi {
                if is_selected {
                    self.selected_marker
                } else {
                    self.unselected_marker
                }
            } else if is_selected {
                "> "
            } else {
                "  "
            };

            let max_w = inner.width as usize;
            let text = format!("{marker}{}", item.label);
            let truncated: String = text.chars().take(max_w).collect();

            let row_style = if is_highlighted {
                self.highlight_style
            } else {
                self.style
            };

            buf.set_string(inner.x, y, &truncated, row_style);
        }
    }
}

impl Discoverable for SelectList {
    fn schema() -> WidgetSchema {
        WidgetSchema {
            name: "SelectList".into(),
            description: "An interactive select list supporting single or multi-select with filtering.".into(),
            default_role: SemanticRole::Input,
            properties: vec![
                PropertySchema {
                    name: "mode".into(),
                    description: "Selection mode: Single or Multi.".into(),
                    property_type: PropertyType::String,
                    required: false,
                    default_value: Some(serde_json::json!("Single")),
                    constraints: vec![],
                },
                PropertySchema {
                    name: "items".into(),
                    description: "List of selectable items.".into(),
                    property_type: PropertyType::Array(Box::new(PropertyType::String)),
                    required: true,
                    default_value: None,
                    constraints: vec![],
                },
            ],
            usage_hint: Some("SelectList::new(items).mode(SelectMode::Multi)".into()),
            tags: vec!["select".into(), "list".into(), "input".into(), "menu".into()],
        }
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::Focusable,
            AgentCapability::Selectable {
                multi_select: self.mode == SelectMode::Multi,
                item_count: self.items.len(),
            },
            AgentCapability::Scrollable {
                vertical: true,
                horizontal: false,
            },
            AgentCapability::Searchable,
            AgentCapability::HasKeyBindings {
                bindings: vec![
                    ("Up/k".into(), "Move cursor up".into()),
                    ("Down/j".into(), "Move cursor down".into()),
                    ("Space/Enter".into(), "Toggle/select item".into()),
                    ("Home".into(), "Jump to first item".into()),
                    ("End".into(), "Jump to last item".into()),
                ],
            },
        ]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![
            AgentAction {
                name: "select_index".into(),
                description: "Select an item by index.".into(),
                params: vec![ActionParam {
                    name: "index".into(),
                    description: "Zero-based index of the item to select.".into(),
                    param_type: ActionParamType::Integer,
                    required: true,
                    default_value: None,
                }],
                returns: None,
                mutates: true,
                idempotent: true,
                shortcut: None,
            },
            AgentAction {
                name: "set_filter".into(),
                description: "Set the filter string for fuzzy matching.".into(),
                params: vec![ActionParam {
                    name: "filter".into(),
                    description: "Substring to filter items by.".into(),
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
                name: "get_selected".into(),
                description: "Get the currently selected item values.".into(),
                params: vec![],
                returns: Some("Array of selected item values.".into()),
                mutates: false,
                idempotent: true,
                shortcut: None,
            },
        ]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Input
    }

    fn agent_state(&self) -> serde_json::Value {
        let labels: Vec<&str> = self.items.iter().map(|i| i.label.as_str()).collect();
        serde_json::json!({
            "widget": "SelectList",
            "item_count": self.items.len(),
            "items": labels,
            "mode": format!("{:?}", self.mode),
        })
    }

    fn execute_action(
        &mut self,
        action: &str,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        Err(format!(
            "SelectList actions require SelectListState; use stateful dispatch. Action: {action}"
        ))
    }
}
