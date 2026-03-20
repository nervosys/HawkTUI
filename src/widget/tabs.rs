use crate::core::buffer::Buffer;
use crate::core::rect::Rect;
use crate::core::style::Style;
use crate::core::text::{Line, Span};
use crate::ontology::{
    AgentAction, AgentCapability, Discoverable, PropertySchema, PropertyType, SemanticRole,
    WidgetSchema,
};
use crate::widget::block::Block;
use crate::widget::Widget;

/// A tab bar widget for section navigation.
#[derive(Debug, Clone)]
pub struct Tabs {
    titles: Vec<Line>,
    block: Option<Block>,
    selected: usize,
    style: Style,
    highlight_style: Style,
    divider: Span,
    padding_left: Span,
    padding_right: Span,
}

impl Tabs {
    pub fn new(titles: impl IntoIterator<Item = impl Into<Line>>) -> Self {
        Self {
            titles: titles.into_iter().map(Into::into).collect(),
            block: None,
            selected: 0,
            style: Style::default(),
            highlight_style: Style::default().reversed(),
            divider: Span::raw(" | "),
            padding_left: Span::raw(" "),
            padding_right: Span::raw(" "),
        }
    }

    pub fn block(mut self, block: Block) -> Self {
        self.block = Some(block);
        self
    }

    pub fn select(mut self, selected: usize) -> Self {
        self.selected = selected;
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

    pub fn divider(mut self, divider: impl Into<Span>) -> Self {
        self.divider = divider.into();
        self
    }

    pub fn padding(mut self, left: impl Into<Span>, right: impl Into<Span>) -> Self {
        self.padding_left = left.into();
        self.padding_right = right.into();
        self
    }
}

impl Widget for Tabs {
    fn render(self, area: Rect, buf: &mut Buffer) {
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

        if inner.is_empty() || self.titles.is_empty() {
            return;
        }

        let mut x = inner.x;
        let y = inner.y;
        let max_x = inner.right();

        for (i, title) in self.titles.iter().enumerate() {
            if x >= max_x {
                break;
            }

            // Divider between tabs
            if i > 0 {
                let divider_width = self.divider.content.len() as u16;
                if x + divider_width > max_x {
                    break;
                }
                buf.set_span(x, y, &self.divider, divider_width);
                x += divider_width;
            }

            let is_selected = i == self.selected;
            let style = if is_selected {
                self.style.patch(self.highlight_style)
            } else {
                self.style
            };

            // Left pad
            let pad_left_w = self.padding_left.content.len() as u16;
            if x + pad_left_w <= max_x {
                buf.set_span(x, y, &self.padding_left, pad_left_w);
                x += pad_left_w;
            }

            // Title
            let title_width: u16 = title.width() as u16;
            if x + title_width > max_x {
                break;
            }
            buf.set_line(x, y, title, max_x - x);
            buf.set_style(Rect::new(x, y, title_width, 1), style);
            x += title_width;

            // Right pad
            let pad_right_w = self.padding_right.content.len() as u16;
            if x + pad_right_w <= max_x {
                buf.set_span(x, y, &self.padding_right, pad_right_w);
                x += pad_right_w;
            }
        }
    }
}

impl Discoverable for Tabs {
    fn schema() -> WidgetSchema {
        WidgetSchema {
            name: "Tabs".into(),
            description: "A horizontal tab bar for section navigation.".into(),
            default_role: SemanticRole::Navigation,
            properties: vec![
                PropertySchema {
                    name: "titles".into(),
                    description: "Tab labels.".into(),
                    property_type: PropertyType::Array(Box::new(PropertyType::String)),
                    required: true,
                    default_value: None,
                    constraints: vec![],
                },
                PropertySchema {
                    name: "selected".into(),
                    description: "Zero-based index of the active tab.".into(),
                    property_type: PropertyType::Integer,
                    required: false,
                    default_value: Some(serde_json::json!(0)),
                    constraints: vec![],
                },
            ],
            usage_hint: Some("Tabs::new([\"Tab 1\", \"Tab 2\"]).select(0)".into()),
            tags: vec!["tabs".into(), "navigation".into(), "header".into()],
        }
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::Focusable,
            AgentCapability::Selectable {
                multi_select: false,
                item_count: self.titles.len(),
            },
            AgentCapability::HasKeyBindings {
                bindings: vec![
                    ("Left/h".into(), "Previous tab".into()),
                    ("Right/l".into(), "Next tab".into()),
                ],
            },
        ]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![
            AgentAction {
                name: "select_tab".into(),
                description: "Switch to a tab by index.".into(),
                params: vec![crate::ontology::ActionParam {
                    name: "index".into(),
                    description: "Zero-based tab index.".into(),
                    param_type: crate::ontology::ActionParamType::Index,
                    required: true,
                    default_value: None,
                }],
                returns: None,
                mutates: true,
                idempotent: true,
                shortcut: None,
            },
            AgentAction {
                name: "get_tabs".into(),
                description: "Get all tab titles.".into(),
                params: vec![],
                returns: Some("Array of tab title strings.".into()),
                mutates: false,
                idempotent: true,
                shortcut: None,
            },
        ]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Navigation
    }

    fn agent_state(&self) -> serde_json::Value {
        let titles: Vec<String> = self
            .titles
            .iter()
            .map(|t| t.spans.iter().map(|s| s.content.as_ref()).collect())
            .collect();
        serde_json::json!({
            "selected": self.selected,
            "tab_count": self.titles.len(),
            "titles": titles,
        })
    }

    fn execute_action(
        &mut self,
        _action: &str,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        Err("Tabs is consumed on render. Use builder pattern to set selected index.".into())
    }
}
