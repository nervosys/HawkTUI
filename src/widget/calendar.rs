use crate::core::buffer::Buffer;
use crate::core::rect::Rect;
use crate::core::style::Style;
use crate::ontology::{
    AgentAction, AgentCapability, Discoverable, PropertyConstraint, PropertySchema, PropertyType,
    SemanticRole, WidgetSchema,
};
use crate::widget::block::Block;
use crate::widget::Widget;

/// A month-view calendar widget.
///
/// Renders a single month as a grid with day-of-week headers and numbered
/// day cells. Specific dates can be highlighted with a custom style.
///
/// # Example
///
/// ```ignore
/// use louie::widget::calendar::Calendar;
///
/// let cal = Calendar::new(2026, 3)
///     .show_header(true)
///     .highlight_day(18, Style::default().reversed());
/// ```
#[derive(Debug, Clone)]
pub struct Calendar {
    year: i32,
    month: u32,
    block: Option<Block>,
    style: Style,
    header_style: Style,
    highlight_today: Option<Style>,
    highlighted_days: Vec<(u32, Style)>,
    show_header: bool,
}

impl Calendar {
    /// Create a new calendar for the given year and month (1-12).
    pub fn new(year: i32, month: u32) -> Self {
        Self {
            year,
            month: month.clamp(1, 12),
            block: None,
            style: Style::default(),
            header_style: Style::default(),
            highlight_today: None,
            highlighted_days: Vec::new(),
            show_header: true,
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

    pub fn header_style(mut self, style: Style) -> Self {
        self.header_style = style;
        self
    }

    pub fn show_header(mut self, show: bool) -> Self {
        self.show_header = show;
        self
    }

    /// Highlight a specific day with a style.
    pub fn highlight_day(mut self, day: u32, style: Style) -> Self {
        self.highlighted_days.push((day, style));
        self
    }

    pub fn highlight_today(mut self, style: Style) -> Self {
        self.highlight_today = Some(style);
        self
    }

    /// Returns the number of days in the given month/year.
    fn days_in_month(year: i32, month: u32) -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                    29
                } else {
                    28
                }
            }
            _ => 30,
        }
    }

    /// Day of week for the 1st of this month (0=Monday, 6=Sunday).
    /// Uses Tomohiko Sakamoto's algorithm.
    fn first_weekday(year: i32, month: u32) -> u32 {
        let t = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
        let y = if month < 3 { year - 1 } else { year };
        let m = month as usize;
        ((y + y / 4 - y / 100 + y / 400 + t[m - 1] + 1) % 7).unsigned_abs()
    }

    fn style_for_day(&self, day: u32) -> Style {
        for &(d, s) in &self.highlighted_days {
            if d == day {
                return self.style.patch(s);
            }
        }
        self.style
    }
}

const DAY_HEADERS: [&str; 7] = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];

impl Widget for Calendar {
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

        if inner.width < 20 || inner.height < 1 {
            return;
        }

        let cell_width = 3u16; // "DD " per cell
        let mut row = 0u16;

        // Header row
        if self.show_header {
            let mut x = inner.x;
            for header in &DAY_HEADERS {
                if x + cell_width > inner.right() {
                    break;
                }
                buf.set_string_truncated(x, inner.y + row, header, cell_width, self.header_style);
                x += cell_width;
            }
            row += 1;
        }

        let num_days = Self::days_in_month(self.year, self.month);
        let first_dow = Self::first_weekday(self.year, self.month) as u16;

        let mut day = 1u32;
        let mut col = first_dow;

        while day <= num_days {
            if inner.y + row >= inner.bottom() {
                break;
            }

            if col >= 7 {
                col = 0;
                row += 1;
                if inner.y + row >= inner.bottom() {
                    break;
                }
            }

            let x = inner.x + col * cell_width;
            if x + 2 <= inner.right() {
                let text = format!("{day:>2}");
                let style = self.style_for_day(day);
                buf.set_string_truncated(x, inner.y + row, &text, cell_width, style);
            }

            day += 1;
            col += 1;
        }
    }
}

impl Discoverable for Calendar {
    fn schema() -> WidgetSchema {
        WidgetSchema {
            name: "Calendar".into(),
            description: "A month-view calendar grid with day-of-week headers and day highlighting."
                .into(),
            default_role: SemanticRole::Display,
            properties: vec![
                PropertySchema {
                    name: "year".into(),
                    description: "The year to display.".into(),
                    property_type: PropertyType::Integer,
                    required: true,
                    default_value: None,
                    constraints: vec![],
                },
                PropertySchema {
                    name: "month".into(),
                    description: "The month to display (1-12).".into(),
                    property_type: PropertyType::Integer,
                    required: true,
                    default_value: None,
                    constraints: vec![
                        PropertyConstraint::Min(1.0),
                        PropertyConstraint::Max(12.0),
                    ],
                },
            ],
            actions: vec![],

            usage_hint: Some("Calendar::new(2026, 3).show_header(true)".into()),
            tags: vec!["calendar".into(), "date".into(), "display".into()],
        }
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Display
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "year": self.year,
            "month": self.month,
            "highlighted_days": self.highlighted_days.iter().map(|(d, _)| d).collect::<Vec<_>>(),
        })
    }

    fn execute_action(
        &mut self,
        action: &str,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        Err(format!("Unknown action: {action}"))
    }
}
