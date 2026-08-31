use crate::core::buffer::Buffer;
use crate::core::rect::Rect;
use crate::core::style::Style;
use crate::core::text::Line;
use crate::ontology::{
    ActionParam, ActionParamType, AgentAction, AgentCapability, Discoverable, PropertySchema,
    PropertyType, SemanticRole, WidgetSchema,
};
use crate::widget::block::Block;
use crate::widget::{StatefulWidget, Widget};

/// A single item in a [`List`].
#[derive(Debug, Clone)]
pub struct ListItem {
    pub content: Line,
    pub style: Style,
}

impl ListItem {
    pub fn new(content: impl Into<Line>) -> Self {
        Self {
            content: content.into(),
            style: Style::default(),
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl<T: Into<Line>> From<T> for ListItem {
    fn from(content: T) -> Self {
        ListItem::new(content)
    }
}

/// State for a [`List`] widget.
#[derive(Debug, Clone, Default)]
pub struct ListState {
    pub selected: Option<usize>,
    pub offset: usize,
}

impl ListState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_selected(mut self, selected: Option<usize>) -> Self {
        self.selected = selected;
        self
    }

    pub fn select(&mut self, index: Option<usize>) {
        self.selected = index;
    }

    pub fn select_next(&mut self, item_count: usize) {
        if item_count == 0 {
            return;
        }
        let i = match self.selected {
            Some(i) => (i + 1).min(item_count - 1),
            None => 0,
        };
        self.selected = Some(i);
    }

    pub fn select_previous(&mut self) {
        let i = match self.selected {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        self.selected = Some(i);
    }

    pub fn select_first(&mut self) {
        self.selected = Some(0);
    }

    pub fn select_last(&mut self, item_count: usize) {
        if item_count > 0 {
            self.selected = Some(item_count - 1);
        }
    }
}

/// Direction in which list items are rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ListDirection {
    /// Render items from top to bottom (default).
    #[default]
    TopToBottom,
    /// Render items from bottom to top (newest at bottom, like a chat log).
    BottomToTop,
}

/// A scrollable list widget with selection support.
#[derive(Debug, Clone)]
pub struct List {
    items: Vec<ListItem>,
    block: Option<Block>,
    style: Style,
    highlight_style: Style,
    highlight_symbol: Option<String>,
    /// Rows kept visible above and below the selection while scrolling.
    scroll_padding: usize,
    direction: ListDirection,
}

impl List {
    pub fn new(items: impl IntoIterator<Item = impl Into<ListItem>>) -> Self {
        Self {
            items: items.into_iter().map(Into::into).collect(),
            block: None,
            style: Style::default(),
            highlight_style: Style::default().reversed(),
            highlight_symbol: None,
            scroll_padding: 0,
            direction: ListDirection::TopToBottom,
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

    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }

    pub fn highlight_symbol(mut self, symbol: impl Into<String>) -> Self {
        self.highlight_symbol = Some(symbol.into());
        self
    }

    /// Keep `padding` rows visible above and below the selection when
    /// scrolling, so the cursor never sits flush against the edge.
    pub fn scroll_padding(mut self, padding: usize) -> Self {
        self.scroll_padding = padding;
        self
    }

    pub fn direction(mut self, direction: ListDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl StatefulWidget for List {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut ListState) {
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

        if inner.is_empty() || self.items.is_empty() {
            return;
        }

        let visible_height = inner.height as usize;

        // Adjust the scroll offset to keep the selection visible, honoring the
        // requested breathing room above and below it. The padding is capped so
        // it can never demand more rows than the viewport has.
        if let Some(selected) = state.selected {
            let pad = self
                .scroll_padding
                .min(visible_height.saturating_sub(1) / 2);
            let max_offset = self.items.len().saturating_sub(visible_height);

            if selected < state.offset + pad {
                state.offset = selected.saturating_sub(pad);
            } else if selected + pad >= state.offset + visible_height {
                state.offset = (selected + pad + 1).saturating_sub(visible_height);
            }
            state.offset = state.offset.min(max_offset);
        }

        let highlight_symbol_width = self.highlight_symbol.as_ref().map_or(0, |s| s.len() as u16);

        // Build the visible items iterator based on direction
        let items_iter: Vec<(usize, &ListItem)> = match self.direction {
            ListDirection::TopToBottom => self
                .items
                .iter()
                .enumerate()
                .skip(state.offset)
                .take(visible_height)
                .collect(),
            ListDirection::BottomToTop => {
                // Show items anchored to the bottom of the area
                let total = self.items.len();
                let end = total.saturating_sub(state.offset);
                let start = end.saturating_sub(visible_height);
                self.items[start..end]
                    .iter()
                    .enumerate()
                    .map(|(i, item)| (start + i, item))
                    .collect()
            }
        };

        for (vi, (i, item)) in items_iter.iter().enumerate() {
            let y = match self.direction {
                ListDirection::TopToBottom => inner.y + vi as u16,
                ListDirection::BottomToTop => inner.bottom().saturating_sub(1) - vi as u16,
            };
            let is_selected = state.selected == Some(*i);

            let style = if is_selected {
                self.style.patch(item.style).patch(self.highlight_style)
            } else {
                self.style.patch(item.style)
            };

            // Clear the line
            buf.set_style(Rect::new(inner.x, y, inner.width, 1), style);

            let mut x = inner.x;

            // Draw highlight symbol
            if is_selected {
                if let Some(ref symbol) = self.highlight_symbol {
                    buf.set_string(x, y, symbol, style);
                    x += highlight_symbol_width;
                }
            } else if self.highlight_symbol.is_some() {
                x += highlight_symbol_width;
            }

            // Draw item content. The row was already painted with `style`
            // above, and writing a span only patches the fields its own style
            // sets — so an item with no styling of its own needs no second
            // pass over the row.
            let max_width = inner.right().saturating_sub(x);
            buf.set_line(x, y, &item.content, max_width);
            if item
                .content
                .spans
                .iter()
                .any(|s| s.style != Style::default())
            {
                buf.set_style(Rect::new(x, y, max_width, 1), style);
            }
        }
    }
}

impl Discoverable for List {
    fn schema() -> WidgetSchema {
        WidgetSchema {
            name: "List".into(),
            description:
                "A scrollable list with item selection, highlight, and keyboard navigation.".into(),
            default_role: SemanticRole::Selection,
            properties: vec![
                PropertySchema {
                    name: "items".into(),
                    description: "The list items to display.".into(),
                    property_type: PropertyType::Array(Box::new(PropertyType::String)),
                    required: true,
                    default_value: None,
                    constraints: vec![],
                },
                PropertySchema {
                    name: "highlight_symbol".into(),
                    description: "Symbol displayed next to the selected item.".into(),
                    property_type: PropertyType::String,
                    required: false,
                    default_value: Some(serde_json::json!(">> ")),
                    constraints: vec![],
                },
            ],
            actions: vec![],

            usage_hint: Some(
                "List::new([\"Item 1\", \"Item 2\"]).highlight_symbol(\">> \")".into(),
            ),
            tags: vec![
                "list".into(),
                "selection".into(),
                "scrollable".into(),
                "menu".into(),
            ],
        }
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::Focusable,
            AgentCapability::Scrollable {
                vertical: true,
                horizontal: false,
            },
            AgentCapability::Selectable {
                multi_select: false,
                item_count: self.items.len(),
            },
            AgentCapability::HasKeyBindings {
                bindings: vec![
                    ("Up/k".into(), "Select previous item".into()),
                    ("Down/j".into(), "Select next item".into()),
                    ("Home/g".into(), "Select first item".into()),
                    ("End/G".into(), "Select last item".into()),
                    ("Enter".into(), "Confirm selection".into()),
                ],
            },
        ]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![
            AgentAction {
                name: "select".into(),
                description: "Select an item by index.".into(),
                params: vec![ActionParam {
                    name: "index".into(),
                    description: "Zero-based item index.".into(),
                    param_type: ActionParamType::Index,
                    required: true,
                    default_value: None,
                }],
                returns: Some("The selected item text.".into()),
                mutates: true,
                idempotent: true,
                shortcut: None,
            },
            AgentAction {
                name: "select_next".into(),
                description: "Move selection to the next item.".into(),
                params: vec![],
                returns: None,
                mutates: true,
                idempotent: false,
                shortcut: Some("Down".into()),
            },
            AgentAction {
                name: "select_previous".into(),
                description: "Move selection to the previous item.".into(),
                params: vec![],
                returns: None,
                mutates: true,
                idempotent: false,
                shortcut: Some("Up".into()),
            },
            AgentAction {
                name: "get_items".into(),
                description: "Get all list item texts.".into(),
                params: vec![],
                returns: Some("Array of item texts.".into()),
                mutates: false,
                idempotent: true,
                shortcut: None,
            },
            AgentAction {
                name: "get_selected".into(),
                description: "Get the currently selected item index and text.".into(),
                params: vec![],
                returns: Some("Object with index and text, or null.".into()),
                mutates: false,
                idempotent: true,
                shortcut: None,
            },
        ]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Selection
    }

    fn agent_state(&self) -> serde_json::Value {
        let items: Vec<String> = self
            .items
            .iter()
            .map(|item| {
                item.content
                    .spans
                    .iter()
                    .map(|s| s.content.as_ref())
                    .collect::<String>()
            })
            .collect();
        serde_json::json!({
            "item_count": self.items.len(),
            "items": items,
        })
    }

    fn execute_action(
        &mut self,
        _action: &str,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        // Actions that modify state need to go through the StatefulWidget pattern
        Err("List actions require a ListState. Use the runtime to dispatch actions.".into())
    }
}
