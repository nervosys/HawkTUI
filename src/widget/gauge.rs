use crate::core::buffer::Buffer;
use crate::core::rect::Rect;
use crate::core::style::Style;
use crate::core::text::Span;
use crate::ontology::{
    ActionParam, ActionParamType, AgentAction, AgentCapability, Discoverable, PropertyConstraint,
    PropertySchema, PropertyType, SemanticRole, WidgetSchema,
};
use crate::widget::block::Block;
use crate::widget::Widget;

/// A progress bar widget.
#[derive(Debug, Clone)]
pub struct Gauge {
    block: Option<Block>,
    ratio: f64,
    label: Option<Span>,
    style: Style,
    gauge_style: Style,
}

impl Gauge {
    pub fn new() -> Self {
        Self {
            block: None,
            ratio: 0.0,
            label: None,
            style: Style::default(),
            gauge_style: Style::default(),
        }
    }

    pub fn block(mut self, block: Block) -> Self {
        self.block = Some(block);
        self
    }

    /// Set the ratio (0.0..=1.0). Values are clamped.
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

    pub fn gauge_style(mut self, style: Style) -> Self {
        self.gauge_style = style;
        self
    }
}

impl Default for Gauge {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Gauge {
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

        let filled_width = (inner.width as f64 * self.ratio).round() as u16;

        // Draw filled portion
        for y in inner.y..inner.bottom() {
            for x in inner.x..inner.x + filled_width.min(inner.width) {
                let cell = &mut buf[(x, y)];
                cell.set_symbol("█");
                cell.set_style(self.gauge_style);
            }
        }

        // Draw label centered
        let label_text = self
            .label
            .map(|l| l.content.to_string())
            .unwrap_or_else(|| format!("{}%", (self.ratio * 100.0).round() as u16));

        let label_width = label_text.len() as u16;
        if label_width <= inner.width {
            let label_x = inner.x + (inner.width - label_width) / 2;
            let label_y = inner.y + inner.height / 2;
            buf.set_string(label_x, label_y, &label_text, self.style);
        }
    }
}

impl Discoverable for Gauge {
    fn schema() -> WidgetSchema {
        WidgetSchema {
            name: "Gauge".into(),
            description: "A horizontal progress bar displaying a ratio or percentage.".into(),
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
                    description: "Text displayed over the gauge. Defaults to percentage.".into(),
                    property_type: PropertyType::String,
                    required: false,
                    default_value: None,
                    constraints: vec![],
                },
            ],
            actions: vec![
                AgentAction {
                    name: "set_ratio".into(),
                    description: "Set the gauge progress ratio.".into(),
                    params: vec![ActionParam {
                        name: "ratio".into(),
                        description: "Value between 0.0 and 1.0.".into(),
                        param_type: ActionParamType::Float,
                        required: true,
                        default_value: None,
                    }],
                    returns: None,
                    mutates: true,
                    idempotent: true,
                    shortcut: None,
                },
                AgentAction {
                    name: "get_ratio".into(),
                    description: "Get the current progress ratio.".into(),
                    params: vec![],
                    returns: Some("Current ratio as float.".into()),
                    mutates: false,
                    idempotent: true,
                    shortcut: None,
                },
            ],

            usage_hint: Some("Gauge::new().percent(42).label(\"Loading...\")".into()),
            tags: vec![
                "gauge".into(),
                "progress".into(),
                "bar".into(),
                "loading".into(),
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
        vec![
            AgentAction {
                name: "set_ratio".into(),
                description: "Set the gauge progress ratio.".into(),
                params: vec![ActionParam {
                    name: "ratio".into(),
                    description: "Value between 0.0 and 1.0.".into(),
                    param_type: ActionParamType::Float,
                    required: true,
                    default_value: None,
                }],
                returns: None,
                mutates: true,
                idempotent: true,
                shortcut: None,
            },
            AgentAction {
                name: "get_ratio".into(),
                description: "Get the current progress ratio.".into(),
                params: vec![],
                returns: Some("Current ratio as float.".into()),
                mutates: false,
                idempotent: true,
                shortcut: None,
            },
        ]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Progress
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "ratio": self.ratio,
            "percent": (self.ratio * 100.0).round() as u16,
        })
    }

    fn execute_action(
        &mut self,
        action: &str,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match action {
            "get_ratio" => Ok(serde_json::json!(self.ratio)),
            "set_ratio" => {
                let r = params
                    .get("ratio")
                    .and_then(|v| v.as_f64())
                    .ok_or("Missing ratio parameter")?;
                self.ratio = r.clamp(0.0, 1.0);
                Ok(serde_json::json!({"ratio": self.ratio}))
            }
            _ => Err(format!("Unknown action: {action}")),
        }
    }
}
