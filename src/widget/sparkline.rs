use crate::core::buffer::Buffer;
use crate::core::rect::Rect;
use crate::core::style::Style;
use crate::ontology::{
    AgentAction, AgentCapability, Discoverable, PropertyConstraint, PropertySchema, PropertyType,
    SemanticRole, WidgetSchema,
};
use crate::widget::block::Block;
use crate::widget::Widget;

/// A mini chart showing a data trend as vertical bars.
#[derive(Debug, Clone)]
pub struct Sparkline {
    data: Vec<u64>,
    block: Option<Block>,
    style: Style,
    bar_style: Style,
    max: Option<u64>,
}

impl Sparkline {
    pub fn new(data: impl Into<Vec<u64>>) -> Self {
        Self {
            data: data.into(),
            block: None,
            style: Style::default(),
            bar_style: Style::default(),
            max: None,
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

    pub fn bar_style(mut self, style: Style) -> Self {
        self.bar_style = style;
        self
    }

    /// Manually set the maximum value. If `None`, auto-detects from data.
    pub fn max(mut self, max: u64) -> Self {
        self.max = Some(max);
        self
    }
}

/// Bar symbols for sparkline rendering (eighths).
const BAR_SYMBOLS: [&str; 9] = [" ", "▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];

impl Widget for Sparkline {
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

        if inner.is_empty() || self.data.is_empty() {
            return;
        }

        let max = self
            .max
            .unwrap_or_else(|| self.data.iter().copied().max().unwrap_or(1))
            .max(1);
        let height = inner.height as u64;

        // Show the last `inner.width` data points
        let data_start = self.data.len().saturating_sub(inner.width as usize);
        let visible = &self.data[data_start..];

        for (i, &value) in visible.iter().enumerate() {
            let x = inner.x + i as u16;
            if x >= inner.right() {
                break;
            }

            // Scale value to available height (in eighths)
            let scaled = (value.min(max) as f64 / max as f64 * (height * 8) as f64) as u64;
            let full_rows = (scaled / 8) as u16;
            let remainder = (scaled % 8) as usize;

            // Draw from bottom up
            for row in 0..inner.height {
                let y = inner.bottom() - 1 - row;
                let symbol = if row < full_rows {
                    BAR_SYMBOLS[8]
                } else if row == full_rows && remainder > 0 {
                    BAR_SYMBOLS[remainder]
                } else {
                    " "
                };
                buf[(x, y)].set_symbol(symbol);
                buf[(x, y)].set_style(self.bar_style);
            }
        }
    }
}

impl Discoverable for Sparkline {
    fn schema() -> WidgetSchema {
        WidgetSchema {
            name: "Sparkline".into(),
            description: "A compact chart displaying data trends as vertical bars.".into(),
            default_role: SemanticRole::DataVisualization,
            properties: vec![
                PropertySchema {
                    name: "data".into(),
                    description: "Array of unsigned integer data points.".into(),
                    property_type: PropertyType::Array(Box::new(PropertyType::Integer)),
                    required: true,
                    default_value: None,
                    constraints: vec![],
                },
                PropertySchema {
                    name: "max".into(),
                    description: "Maximum value for scaling. Auto-detected if omitted.".into(),
                    property_type: PropertyType::Integer,
                    required: false,
                    default_value: None,
                    constraints: vec![PropertyConstraint::Min(1.0)],
                },
            ],
            actions: vec![],

            usage_hint: Some("Sparkline::new(vec![0, 1, 3, 7, 5, 2])".into()),
            tags: vec![
                "sparkline".into(),
                "chart".into(),
                "trend".into(),
                "data".into(),
            ],
        }
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![AgentAction {
            name: "get_data".into(),
            description: "Get the sparkline data points.".into(),
            params: vec![],
            returns: Some("Array of integers.".into()),
            mutates: false,
            idempotent: true,
            shortcut: None,
        }]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::DataVisualization
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "data_points": self.data.len(),
            "min": self.data.iter().min().unwrap_or(&0),
            "max": self.data.iter().max().unwrap_or(&0),
        })
    }

    fn execute_action(
        &mut self,
        action: &str,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match action {
            "get_data" => Ok(serde_json::json!(self.data)),
            _ => Err(format!("Unknown action: {action}")),
        }
    }
}
