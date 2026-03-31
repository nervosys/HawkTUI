use crate::core::buffer::Buffer;
use crate::core::rect::Rect;
use crate::core::style::Style;
use crate::core::text::Span;
use crate::ontology::{
    AgentAction, AgentCapability, Discoverable, PropertyConstraint, PropertySchema, PropertyType,
    SemanticRole, WidgetSchema,
};
use crate::widget::block::Block;
use crate::widget::Widget;

/// A thin, single-line progress gauge.
///
/// Unlike [`Gauge`](super::gauge::Gauge) which fills a multi-row area,
/// `LineGauge` renders as a single horizontal line using unicode line-drawing
/// characters for sub-cell precision.
///
/// # Example
///
/// ```ignore
/// use louie::widget::line_gauge::LineGauge;
///
/// let gauge = LineGauge::new()
///     .ratio(0.65)
///     .label("Progress")
///     .filled_symbol("━")
///     .unfilled_symbol("─");
/// ```
#[derive(Debug, Clone)]
pub struct LineGauge {
    block: Option<Block>,
    ratio: f64,
    label: Option<Span>,
    style: Style,
    line_style: Style,
    filled_symbol: &'static str,
    unfilled_symbol: &'static str,
}

impl LineGauge {
    pub fn new() -> Self {
        Self {
            block: None,
            ratio: 0.0,
            label: None,
            style: Style::default(),
            line_style: Style::default(),
            filled_symbol: "━",
            unfilled_symbol: "─",
        }
    }

    pub fn block(mut self, block: Block) -> Self {
        self.block = Some(block);
        self
    }

    /// Set the ratio (0.0..=1.0). Clamped to valid range.
    pub fn ratio(mut self, ratio: f64) -> Self {
        self.ratio = ratio.clamp(0.0, 1.0);
        self
    }

    /// Set from a percentage (0..=100).
    pub fn percent(mut self, percent: u16) -> Self {
        self.ratio = (percent.min(100) as f64) / 100.0;
        self
    }

    pub fn label(mut self, label: impl Into<Span>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn line_style(mut self, style: Style) -> Self {
        self.line_style = style;
        self
    }

    pub fn filled_symbol(mut self, symbol: &'static str) -> Self {
        self.filled_symbol = symbol;
        self
    }

    pub fn unfilled_symbol(mut self, symbol: &'static str) -> Self {
        self.unfilled_symbol = symbol;
        self
    }
}

impl Default for LineGauge {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for LineGauge {
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

        if inner.is_empty() {
            return;
        }

        // Determine label and gauge widths
        let label_text = self.label.as_ref().map(|l| l.content.to_string());
        let label_width = label_text.as_ref().map_or(0, |t| t.len() as u16 + 1); // +1 for space
        let gauge_width = inner.width.saturating_sub(label_width);

        if gauge_width == 0 {
            return;
        }

        let y = inner.y;
        let mut x = inner.x;

        // Render label
        if let Some(ref text) = label_text {
            let label_style = self
                .label
                .as_ref()
                .map_or(self.style, |l| self.style.patch(l.style));
            buf.set_string_truncated(x, y, text, label_width.saturating_sub(1), label_style);
            x += label_width;
        }

        // Render gauge line
        let filled = (gauge_width as f64 * self.ratio).round() as u16;

        for col in 0..gauge_width {
            let symbol = if col < filled {
                self.filled_symbol
            } else {
                self.unfilled_symbol
            };
            buf.set_string(x + col, y, symbol, self.line_style);
        }
    }
}

impl Discoverable for LineGauge {
    fn schema() -> WidgetSchema {
        WidgetSchema {
            name: "LineGauge".into(),
            description: "A thin, single-line progress bar using line-drawing characters.".into(),
            default_role: SemanticRole::Progress,
            properties: vec![
                PropertySchema {
                    name: "ratio".into(),
                    description: "Progress value between 0.0 and 1.0.".into(),
                    property_type: PropertyType::Float,
                    required: true,
                    default_value: Some(serde_json::json!(0.0)),
                    constraints: vec![PropertyConstraint::Min(0.0), PropertyConstraint::Max(1.0)],
                },
                PropertySchema {
                    name: "label".into(),
                    description: "Text label displayed before the gauge line.".into(),
                    property_type: PropertyType::String,
                    required: false,
                    default_value: None,
                    constraints: vec![],
                },
            ],
            actions: vec![],

            usage_hint: Some(r#"LineGauge::new().percent(65).label("Progress")"#.into()),
            tags: vec![
                "gauge".into(),
                "progress".into(),
                "line".into(),
                "bar".into(),
            ],
        }
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![AgentCapability::RangeEditable {
            min: 0.0,
            max: 1.0,
            step: Some(0.01),
        }]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![AgentAction {
            name: "set_progress".into(),
            description: "Set the progress ratio.".into(),
            params: vec![],
            returns: None,
            mutates: true,
            idempotent: true,
            shortcut: None,
        }]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Progress
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "ratio": self.ratio,
            "label": self.label.as_ref().map(|l| l.content.to_string()),
        })
    }

    fn execute_action(
        &mut self,
        action: &str,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match action {
            "set_progress" => {
                if let Some(r) = params.get("ratio").and_then(|v| v.as_f64()) {
                    self.ratio = r.clamp(0.0, 1.0);
                    Ok(serde_json::json!({ "ratio": self.ratio }))
                } else {
                    Err("Missing 'ratio' parameter".into())
                }
            }
            _ => Err(format!("Unknown action: {action}")),
        }
    }
}
