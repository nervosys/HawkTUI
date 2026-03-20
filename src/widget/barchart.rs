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
use unicode_width::UnicodeWidthStr;

/// A single bar in a bar chart.
#[derive(Debug, Clone)]
pub struct Bar {
    /// The numeric value of this bar.
    pub value: u64,
    /// Optional label displayed below the bar.
    pub label: Option<Span>,
    /// Style applied to the bar body.
    pub style: Style,
    /// Optional style for the value text above the bar.
    pub value_style: Style,
}

impl Bar {
    pub fn new(value: u64) -> Self {
        Self {
            value,
            label: None,
            style: Style::default(),
            value_style: Style::default(),
        }
    }

    pub fn label(mut self, label: impl Into<Span>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn value_style(mut self, style: Style) -> Self {
        self.value_style = style;
        self
    }
}

/// A group of bars rendered side-by-side with an optional group label.
#[derive(Debug, Clone)]
pub struct BarGroup {
    /// The bars in this group.
    pub bars: Vec<Bar>,
    /// Optional label for the group.
    pub label: Option<Span>,
}

impl BarGroup {
    pub fn new(bars: impl Into<Vec<Bar>>) -> Self {
        Self {
            bars: bars.into(),
            label: None,
        }
    }

    pub fn label(mut self, label: impl Into<Span>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// Direction for bar rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarDirection {
    Vertical,
    Horizontal,
}

/// Bar chart widget for visualizing categorical data.
///
/// Supports grouped bars, per-bar styling, value labels, and both
/// vertical and horizontal orientations.
///
/// # Example
///
/// ```ignore
/// use louie::widget::barchart::{BarChart, Bar, BarGroup};
///
/// let chart = BarChart::new(vec![
///     BarGroup::new(vec![Bar::new(80), Bar::new(60)]).label("Q1"),
///     BarGroup::new(vec![Bar::new(90), Bar::new(70)]).label("Q2"),
/// ])
/// .bar_width(5)
/// .bar_gap(1)
/// .group_gap(2);
/// ```
#[derive(Debug, Clone)]
pub struct BarChart {
    groups: Vec<BarGroup>,
    block: Option<Block>,
    style: Style,
    bar_width: u16,
    bar_gap: u16,
    group_gap: u16,
    max_value: Option<u64>,
    direction: BarDirection,
    value_style: Style,
    label_style: Style,
}

/// Bar symbols for sub-cell resolution (eighths).
const BAR_EIGHTHS: [&str; 9] = [" ", "▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];

/// Horizontal bar symbols (eighths, left-to-right).
const HBAR_EIGHTHS: [&str; 9] = [" ", "▏", "▎", "▍", "▌", "▋", "▊", "▉", "█"];

impl BarChart {
    pub fn new(groups: impl Into<Vec<BarGroup>>) -> Self {
        Self {
            groups: groups.into(),
            block: None,
            style: Style::default(),
            bar_width: 3,
            bar_gap: 1,
            group_gap: 2,
            max_value: None,
            direction: BarDirection::Vertical,
            value_style: Style::default(),
            label_style: Style::default(),
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

    /// Width of each bar in terminal columns.
    pub fn bar_width(mut self, width: u16) -> Self {
        self.bar_width = width.max(1);
        self
    }

    /// Gap between bars within a group.
    pub fn bar_gap(mut self, gap: u16) -> Self {
        self.bar_gap = gap;
        self
    }

    /// Gap between groups.
    pub fn group_gap(mut self, gap: u16) -> Self {
        self.group_gap = gap;
        self
    }

    /// Manually set the maximum value. If `None`, auto-detects from data.
    pub fn max_value(mut self, max: u64) -> Self {
        self.max_value = Some(max);
        self
    }

    /// Set bar direction (vertical or horizontal).
    pub fn direction(mut self, direction: BarDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Style for value labels displayed alongside bars.
    pub fn value_style(mut self, style: Style) -> Self {
        self.value_style = style;
        self
    }

    /// Style for bar/group labels.
    pub fn label_style(mut self, style: Style) -> Self {
        self.label_style = style;
        self
    }

    fn effective_max(&self) -> u64 {
        self.max_value
            .unwrap_or_else(|| {
                self.groups
                    .iter()
                    .flat_map(|g| g.bars.iter())
                    .map(|b| b.value)
                    .max()
                    .unwrap_or(1)
            })
            .max(1)
    }

    fn render_vertical(self, inner: Rect, buf: &mut Buffer) {
        if inner.is_empty() {
            return;
        }

        let max = self.effective_max();
        // Reserve 1 row for bar labels and 1 for group labels if any groups have labels
        let has_bar_labels = self
            .groups
            .iter()
            .any(|g| g.bars.iter().any(|b| b.label.is_some()));
        let has_group_labels = self.groups.iter().any(|g| g.label.is_some());
        let label_rows = has_bar_labels as u16 + has_group_labels as u16;
        let chart_height = inner.height.saturating_sub(label_rows);
        if chart_height == 0 {
            return;
        }

        let mut x = inner.x;
        for (gi, group) in self.groups.iter().enumerate() {
            if gi > 0 {
                x += self.group_gap;
            }
            let group_start_x = x;

            for (bi, bar) in group.bars.iter().enumerate() {
                if bi > 0 {
                    x += self.bar_gap;
                }
                if x >= inner.right() {
                    break;
                }

                let bar_w = self.bar_width.min(inner.right().saturating_sub(x));
                if bar_w == 0 {
                    break;
                }

                // Calculate bar height in eighths
                let scaled =
                    (bar.value.min(max) as f64 / max as f64 * (chart_height as f64 * 8.0)) as u64;
                let full_rows = (scaled / 8) as u16;
                let remainder = (scaled % 8) as usize;

                let bar_style = if bar.style != Style::default() {
                    bar.style
                } else {
                    self.style
                };

                // Draw bar from bottom up
                let bar_bottom = inner.y + chart_height;
                for row in 0..chart_height {
                    let y = bar_bottom - 1 - row;
                    let symbol = if row < full_rows {
                        BAR_EIGHTHS[8]
                    } else if row == full_rows && remainder > 0 {
                        BAR_EIGHTHS[remainder]
                    } else {
                        break;
                    };
                    for bx in x..x + bar_w {
                        if bx < inner.right() {
                            buf[(bx, y)].set_symbol(symbol);
                            buf[(bx, y)].set_style(bar_style);
                        }
                    }
                }

                // Draw bar label
                if has_bar_labels {
                    let label_y = inner.y + chart_height;
                    if let Some(ref label) = bar.label {
                        let text = &label.content;
                        let tw = text.width() as u16;
                        let lx = x + bar_w.saturating_sub(tw) / 2;
                        buf.set_string(lx, label_y, text, self.label_style);
                    }
                }

                x += bar_w;
            }

            // Draw group label centered under the group
            if has_group_labels {
                if let Some(ref label) = group.label {
                    let group_end_x = x;
                    let group_w = group_end_x.saturating_sub(group_start_x);
                    let label_y = inner.y + chart_height + has_bar_labels as u16;
                    let text = &label.content;
                    let tw = text.width() as u16;
                    let lx = group_start_x + group_w.saturating_sub(tw) / 2;
                    if label_y < inner.bottom() {
                        buf.set_string(lx, label_y, text, self.label_style);
                    }
                }
            }
        }
    }

    fn render_horizontal(self, inner: Rect, buf: &mut Buffer) {
        if inner.is_empty() {
            return;
        }

        let max = self.effective_max();
        // Reserve columns for value labels
        let value_col_width = {
            let max_digits = format!("{}", max).len() as u16;
            max_digits + 1
        };
        let chart_width = inner.width.saturating_sub(value_col_width);
        if chart_width == 0 {
            return;
        }

        let mut y = inner.y;
        for (gi, group) in self.groups.iter().enumerate() {
            if gi > 0 {
                y += self.group_gap;
            }

            // Group label
            if let Some(ref label) = group.label {
                if y < inner.bottom() {
                    buf.set_string(inner.x, y, &label.content, self.label_style);
                    y += 1;
                }
            }

            for (bi, bar) in group.bars.iter().enumerate() {
                if bi > 0 {
                    y += self.bar_gap;
                }
                if y >= inner.bottom() {
                    break;
                }

                let bar_style = if bar.style != Style::default() {
                    bar.style
                } else {
                    self.style
                };

                // Value label at left
                let value_text = format!("{}", bar.value);
                let vw = value_text.len() as u16;
                let vx = inner.x + value_col_width.saturating_sub(vw + 1);
                buf.set_string(vx, y, &value_text, self.value_style);

                // Calculate bar width in eighths
                let bar_start = inner.x + value_col_width;
                let scaled =
                    (bar.value.min(max) as f64 / max as f64 * (chart_width as f64 * 8.0)) as u64;
                let full_cols = (scaled / 8) as u16;
                let remainder = (scaled % 8) as usize;

                for col in 0..chart_width {
                    let bx = bar_start + col;
                    if bx >= inner.right() {
                        break;
                    }
                    let symbol = if col < full_cols {
                        HBAR_EIGHTHS[8]
                    } else if col == full_cols && remainder > 0 {
                        HBAR_EIGHTHS[remainder]
                    } else {
                        break;
                    };
                    buf[(bx, y)].set_symbol(symbol);
                    buf[(bx, y)].set_style(bar_style);
                }

                // Bar label at end
                if let Some(ref label) = bar.label {
                    let lx = bar_start + full_cols + (remainder > 0) as u16 + 1;
                    if lx < inner.right() {
                        buf.set_string(lx, y, &label.content, self.label_style);
                    }
                }

                y += 1;
            }
        }
    }
}

impl Default for BarChart {
    fn default() -> Self {
        Self::new(Vec::<BarGroup>::new())
    }
}

impl Widget for BarChart {
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

        match self.direction {
            BarDirection::Vertical => self.render_vertical(inner, buf),
            BarDirection::Horizontal => self.render_horizontal(inner, buf),
        }
    }
}

impl Discoverable for BarChart {
    fn schema() -> WidgetSchema {
        WidgetSchema {
            name: "BarChart".into(),
            description: "A bar chart for visualizing categorical data with grouped bars.".into(),
            default_role: SemanticRole::DataVisualization,
            properties: vec![
                PropertySchema {
                    name: "groups".into(),
                    description: "Bar groups, each containing one or more bars.".into(),
                    property_type: PropertyType::Array(Box::new(PropertyType::Object(vec![]))),
                    required: true,
                    default_value: None,
                    constraints: vec![],
                },
                PropertySchema {
                    name: "bar_width".into(),
                    description: "Width of each bar in terminal columns.".into(),
                    property_type: PropertyType::Integer,
                    required: false,
                    default_value: Some(serde_json::json!(3)),
                    constraints: vec![PropertyConstraint::Min(1.0)],
                },
                PropertySchema {
                    name: "direction".into(),
                    description: "Rendering direction: Vertical or Horizontal.".into(),
                    property_type: PropertyType::Enum(vec!["Vertical".into(), "Horizontal".into()]),
                    required: false,
                    default_value: Some(serde_json::json!("Vertical")),
                    constraints: vec![],
                },
            ],
            actions: vec![],

            usage_hint: Some("Use for categorical comparisons. Each group can have multiple bars for multi-series data.".into()),
            tags: vec!["chart".into(), "bar".into(), "data".into(), "visualization".into()],
        }
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::DataVisualization
    }

    fn agent_state(&self) -> serde_json::Value {
        let bars: Vec<serde_json::Value> = self
            .groups
            .iter()
            .map(|g| {
                serde_json::json!({
                    "label": g.label.as_ref().map(|l| l.content.as_ref()),
                    "bars": g.bars.iter().map(|b| {
                        serde_json::json!({
                            "value": b.value,
                            "label": b.label.as_ref().map(|l| l.content.as_ref()),
                        })
                    }).collect::<Vec<_>>(),
                })
            })
            .collect();
        serde_json::json!({
            "groups": bars,
            "max_value": self.effective_max(),
            "direction": format!("{:?}", self.direction),
        })
    }

    fn execute_action(
        &mut self,
        _action: &str,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        Err("BarChart has no executable actions".into())
    }
}
