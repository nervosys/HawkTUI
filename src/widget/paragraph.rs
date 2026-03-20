use crate::core::buffer::Buffer;
use crate::core::rect::Rect;
use crate::core::reflow::{CharWrapper, LineTruncator, WordWrapper};
use crate::core::style::Style;
use crate::core::text::{Alignment, Line, Text};
use crate::ontology::{
    AgentAction, AgentCapability, Discoverable, PropertySchema, PropertyType, SemanticRole,
    WidgetSchema,
};
use crate::widget::block::Block;
use crate::widget::Widget;

/// Word-wrap mode for paragraph text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Wrap {
    /// No wrapping, truncate lines that exceed width.
    #[default]
    None,
    /// Wrap at word boundaries.
    Word,
    /// Wrap at character boundaries.
    Char,
}

/// A text display widget with optional wrapping, scrolling, and alignment.
#[derive(Debug, Clone)]
pub struct Paragraph {
    text: Text,
    block: Option<Block>,
    style: Style,
    wrap: Wrap,
    scroll: (u16, u16),
    alignment: Alignment,
}

impl Paragraph {
    pub fn new(text: impl Into<Text>) -> Self {
        Self {
            text: text.into(),
            block: None,
            style: Style::default(),
            wrap: Wrap::None,
            scroll: (0, 0),
            alignment: Alignment::Left,
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

    pub fn wrap(mut self, wrap: Wrap) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn scroll(mut self, scroll: (u16, u16)) -> Self {
        self.scroll = scroll;
        self
    }

    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn left(self) -> Self {
        self.alignment(Alignment::Left)
    }

    pub fn centered(self) -> Self {
        self.alignment(Alignment::Center)
    }

    pub fn right(self) -> Self {
        self.alignment(Alignment::Right)
    }

    /// Reflow the text according to the wrap mode, returning owned Lines.
    fn reflow_lines(&self, max_width: u16) -> Vec<Line> {
        match self.wrap {
            Wrap::None => LineTruncator::truncate(&self.text.lines, max_width, self.style),
            Wrap::Word => WordWrapper::wrap(&self.text.lines, max_width, self.style),
            Wrap::Char => CharWrapper::wrap(&self.text.lines, max_width, self.style),
        }
    }
}

impl Widget for Paragraph {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        // Apply background style
        buf.set_style(area, self.style);

        // Render block and get inner area
        let inner = if let Some(block) = self.block.take() {
            let inner = block.inner(area);
            block.render(area, buf);
            inner
        } else {
            area
        };

        if inner.is_empty() {
            return;
        }

        // Reflow text according to wrap mode
        let lines = self.reflow_lines(inner.width);

        let (scroll_y, scroll_x) = self.scroll;

        for (i, line) in lines.iter().enumerate().skip(scroll_y as usize) {
            let y = inner.y + (i as u16 - scroll_y);
            if y >= inner.bottom() {
                break;
            }

            let line_width = line.width() as u16;
            let alignment = line.alignment.unwrap_or(self.alignment);

            let x_offset = match alignment {
                Alignment::Left => 0,
                Alignment::Center => inner.width.saturating_sub(line_width) / 2,
                Alignment::Right => inner.width.saturating_sub(line_width),
            };

            let x = inner.x + x_offset;
            let max_width = inner.width.saturating_sub(x_offset);

            // Render each span
            let mut col = 0u16;
            for span in &line.spans {
                if col >= max_width {
                    break;
                }
                let style = self.style.patch(span.style);
                let content = if scroll_x > 0 && col == 0 {
                    let skip = scroll_x as usize;
                    let chars: String = span.content.chars().skip(skip).collect();
                    chars
                } else {
                    span.content.to_string()
                };
                let written =
                    buf.set_string_truncated(x + col, y, &content, max_width - col, style);
                col += written;
            }
        }
    }
}

impl Discoverable for Paragraph {
    fn schema() -> WidgetSchema {
        WidgetSchema {
            name: "Paragraph".into(),
            description: "Displays styled text with optional wrapping, scrolling, and alignment."
                .into(),
            default_role: SemanticRole::Display,
            properties: vec![
                PropertySchema {
                    name: "text".into(),
                    description: "The text content to display.".into(),
                    property_type: PropertyType::String,
                    required: true,
                    default_value: None,
                    constraints: vec![],
                },
                PropertySchema {
                    name: "wrap".into(),
                    description: "Word wrapping mode.".into(),
                    property_type: PropertyType::Enum(vec![
                        "none".into(),
                        "word".into(),
                        "char".into(),
                    ]),
                    required: false,
                    default_value: Some(serde_json::json!("none")),
                    constraints: vec![],
                },
                PropertySchema {
                    name: "alignment".into(),
                    description: "Text alignment.".into(),
                    property_type: PropertyType::Enum(vec![
                        "left".into(),
                        "center".into(),
                        "right".into(),
                    ]),
                    required: false,
                    default_value: Some(serde_json::json!("left")),
                    constraints: vec![],
                },
                PropertySchema {
                    name: "scroll".into(),
                    description: "Scroll offset (y, x).".into(),
                    property_type: PropertyType::Object(vec![]),
                    required: false,
                    default_value: Some(serde_json::json!([0, 0])),
                    constraints: vec![],
                },
            ],
            actions: vec![],

            usage_hint: Some("Paragraph::new(\"Hello, world!\").centered()".into()),
            tags: vec!["text".into(), "display".into(), "paragraph".into()],
        }
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        let mut caps = vec![];
        if self.scroll != (0, 0) || self.text.lines.len() > 1 {
            caps.push(AgentCapability::Scrollable {
                vertical: true,
                horizontal: true,
            });
        }
        caps.push(AgentCapability::Copyable);
        caps
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![
            AgentAction {
                name: "scroll_to".into(),
                description: "Scroll to a specific position.".into(),
                params: vec![
                    crate::ontology::ActionParam {
                        name: "y".into(),
                        description: "Vertical scroll offset.".into(),
                        param_type: crate::ontology::ActionParamType::Integer,
                        required: false,
                        default_value: Some(serde_json::json!(0)),
                    },
                    crate::ontology::ActionParam {
                        name: "x".into(),
                        description: "Horizontal scroll offset.".into(),
                        param_type: crate::ontology::ActionParamType::Integer,
                        required: false,
                        default_value: Some(serde_json::json!(0)),
                    },
                ],
                returns: None,
                mutates: true,
                idempotent: true,
                shortcut: None,
            },
            AgentAction {
                name: "get_text".into(),
                description: "Get the full text content.".into(),
                params: vec![],
                returns: Some("Full text content as string.".into()),
                mutates: false,
                idempotent: true,
                shortcut: None,
            },
        ]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Display
    }

    fn agent_state(&self) -> serde_json::Value {
        let text: String = self
            .text
            .lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|s| s.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");
        serde_json::json!({
            "text": text,
            "line_count": self.text.lines.len(),
            "scroll_y": self.scroll.0,
            "scroll_x": self.scroll.1,
            "alignment": format!("{:?}", self.alignment),
            "wrap": format!("{:?}", self.wrap),
        })
    }

    fn execute_action(
        &mut self,
        action: &str,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match action {
            "scroll_to" => {
                let y = params.get("y").and_then(|v| v.as_u64()).unwrap_or(0) as u16;
                let x = params.get("x").and_then(|v| v.as_u64()).unwrap_or(0) as u16;
                self.scroll = (y, x);
                Ok(serde_json::json!({"scrolled_to": [y, x]}))
            }
            "get_text" => {
                let text: String = self
                    .text
                    .lines
                    .iter()
                    .map(|line| {
                        line.spans
                            .iter()
                            .map(|s| s.content.as_ref())
                            .collect::<String>()
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                Ok(serde_json::json!(text))
            }
            _ => Err(format!("Unknown action: {action}")),
        }
    }
}
