//! Markdown renderer widget.
//!
//! Renders a subset of Markdown as styled terminal text:
//! headings, bold, italic, code spans, code blocks, lists, and blockquotes.

use crate::core::buffer::Buffer;
use crate::core::rect::Rect;
use crate::core::style::{Color, Style};
use crate::core::text::{Line, Span};
use crate::ontology::{
    AgentAction, AgentCapability, Discoverable, PropertySchema, PropertyType, SemanticRole,
    WidgetSchema,
};
use crate::widget::block::Block;
use crate::widget::Widget;

/// A widget that renders Markdown text with basic formatting.
#[derive(Debug, Clone)]
pub struct Markdown {
    source: String,
    block: Option<Block>,
    style: Style,
    heading_style: Style,
    code_style: Style,
    bold_style: Style,
    italic_style: Style,
    quote_style: Style,
}

impl Markdown {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            block: None,
            style: Style::default(),
            heading_style: Style::default().bold().fg(Color::Cyan),
            code_style: Style::default().fg(Color::Green),
            bold_style: Style::default().bold(),
            italic_style: Style::default().italic(),
            quote_style: Style::default().fg(Color::DarkGray).italic(),
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

    pub fn heading_style(mut self, style: Style) -> Self {
        self.heading_style = style;
        self
    }

    pub fn code_style(mut self, style: Style) -> Self {
        self.code_style = style;
        self
    }

    /// Parse the source into styled lines.
    fn parse_lines(&self) -> Vec<Line> {
        let mut lines = Vec::new();
        let mut in_code_block = false;

        for raw_line in self.source.lines() {
            if raw_line.starts_with("```") {
                in_code_block = !in_code_block;
                if in_code_block {
                    // Code block start — skip the fence line
                    continue;
                } else {
                    // Code block end
                    continue;
                }
            }

            if in_code_block {
                lines.push(Line {
                    spans: vec![Span::styled(format!("  {raw_line}"), self.code_style)],
                    alignment: None,
                });
                continue;
            }

            // Headings
            if let Some(rest) = raw_line.strip_prefix("### ") {
                lines.push(Line {
                    spans: vec![Span::styled(format!("   {rest}"), self.heading_style)],
                    alignment: None,
                });
                continue;
            }
            if let Some(rest) = raw_line.strip_prefix("## ") {
                lines.push(Line {
                    spans: vec![Span::styled(format!("  {rest}"), self.heading_style.bold())],
                    alignment: None,
                });
                continue;
            }
            if let Some(rest) = raw_line.strip_prefix("# ") {
                lines.push(Line {
                    spans: vec![Span::styled(rest.to_uppercase(), self.heading_style.bold())],
                    alignment: None,
                });
                continue;
            }

            // Blockquotes
            if let Some(rest) = raw_line.strip_prefix("> ") {
                lines.push(Line {
                    spans: vec![
                        Span::styled("│ ", self.quote_style),
                        Span::styled(rest.to_string(), self.quote_style),
                    ],
                    alignment: None,
                });
                continue;
            }

            // Unordered list items
            let list_line = raw_line.strip_prefix("- ").or(raw_line.strip_prefix("* "));
            if let Some(rest) = list_line {
                let mut spans = vec![Span::styled("  • ", self.style)];
                spans.extend(self.parse_inline(rest));
                lines.push(Line {
                    spans,
                    alignment: None,
                });
                continue;
            }

            // Regular paragraph line (with inline formatting)
            let spans = self.parse_inline(raw_line);
            lines.push(Line {
                spans,
                alignment: None,
            });
        }

        lines
    }

    /// Parse inline formatting: **bold**, *italic*, `code`.
    fn parse_inline(&self, text: &str) -> Vec<Span> {
        let mut spans = Vec::new();
        let mut remaining = text;

        while !remaining.is_empty() {
            // Check for code span
            if let Some(pos) = remaining.find('`') {
                if pos > 0 {
                    spans.push(Span::styled(remaining[..pos].to_string(), self.style));
                }
                remaining = &remaining[pos + 1..];
                if let Some(end) = remaining.find('`') {
                    spans.push(Span::styled(remaining[..end].to_string(), self.code_style));
                    remaining = &remaining[end + 1..];
                    continue;
                } else {
                    spans.push(Span::styled("`".to_string(), self.style));
                    continue;
                }
            }

            // Check for bold
            if let Some(pos) = remaining.find("**") {
                if pos > 0 {
                    spans.push(Span::styled(remaining[..pos].to_string(), self.style));
                }
                remaining = &remaining[pos + 2..];
                if let Some(end) = remaining.find("**") {
                    spans.push(Span::styled(remaining[..end].to_string(), self.bold_style));
                    remaining = &remaining[end + 2..];
                    continue;
                } else {
                    spans.push(Span::styled("**".to_string(), self.style));
                    continue;
                }
            }

            // Check for italic
            if let Some(pos) = remaining.find('*') {
                if pos > 0 {
                    spans.push(Span::styled(remaining[..pos].to_string(), self.style));
                }
                remaining = &remaining[pos + 1..];
                if let Some(end) = remaining.find('*') {
                    spans.push(Span::styled(
                        remaining[..end].to_string(),
                        self.italic_style,
                    ));
                    remaining = &remaining[end + 1..];
                    continue;
                } else {
                    spans.push(Span::styled("*".to_string(), self.style));
                    continue;
                }
            }

            // Plain text
            spans.push(Span::styled(remaining.to_string(), self.style));
            break;
        }

        if spans.is_empty() {
            spans.push(Span::raw(""));
        }
        spans
    }
}

impl Widget for Markdown {
    fn render(self, area: Rect, buf: &mut Buffer) {
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

        if inner.is_empty() {
            return;
        }

        let lines = self.parse_lines();
        for (i, line) in lines.iter().enumerate() {
            let y = inner.y + i as u16;
            if y >= inner.bottom() {
                break;
            }
            buf.set_line(inner.x, y, line, inner.width);
        }
    }
}

impl Discoverable for Markdown {
    fn schema() -> WidgetSchema {
        WidgetSchema {
            name: "Markdown".into(),
            description:
                "Renders Markdown text with headings, bold, italic, code, lists, and blockquotes."
                    .into(),
            default_role: SemanticRole::Display,
            properties: vec![PropertySchema {
                name: "source".into(),
                description: "The Markdown source text to render.".into(),
                property_type: PropertyType::String,
                required: true,
                default_value: None,
                constraints: vec![],
            }],
            actions: vec![],

            usage_hint: Some("Markdown::new(\"# Hello\\n\\nSome **bold** text\")".into()),
            tags: vec![
                "markdown".into(),
                "text".into(),
                "display".into(),
                "rich-text".into(),
            ],
        }
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![AgentCapability::Scrollable {
            vertical: true,
            horizontal: false,
        }]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![AgentAction {
            name: "get_source".into(),
            description: "Get the Markdown source text.".into(),
            params: vec![],
            returns: Some("The Markdown source string.".into()),
            mutates: false,
            idempotent: true,
            shortcut: None,
        }]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Display
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "source_length": self.source.len(),
            "line_count": self.source.lines().count(),
        })
    }

    fn execute_action(
        &mut self,
        action: &str,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match action {
            "get_source" => Ok(serde_json::json!(self.source)),
            _ => Err(format!("Unknown action: {action}")),
        }
    }
}
