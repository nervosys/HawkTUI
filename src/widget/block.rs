use crate::core::buffer::Buffer;
use crate::core::rect::{Margin, Rect};
use crate::core::style::Style;
use crate::core::text::{Alignment, Line};
use crate::ontology::{
    AgentAction, AgentCapability, Discoverable, PropertySchema, PropertyType, SemanticRole,
    WidgetSchema,
};
use crate::widget::Widget;

/// Border type for blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BorderType {
    #[default]
    Plain,
    Rounded,
    Double,
    Thick,
    QuadrantInside,
    QuadrantOutside,
}

impl BorderType {
    fn symbols(&self) -> BorderSymbols {
        match self {
            Self::Plain => BorderSymbols {
                top_left: "┌",
                top_right: "┐",
                bottom_left: "└",
                bottom_right: "┘",
                horizontal: "─",
                vertical: "│",
            },
            Self::Rounded => BorderSymbols {
                top_left: "╭",
                top_right: "╮",
                bottom_left: "╰",
                bottom_right: "╯",
                horizontal: "─",
                vertical: "│",
            },
            Self::Double => BorderSymbols {
                top_left: "╔",
                top_right: "╗",
                bottom_left: "╚",
                bottom_right: "╝",
                horizontal: "═",
                vertical: "║",
            },
            Self::Thick => BorderSymbols {
                top_left: "┏",
                top_right: "┓",
                bottom_left: "┗",
                bottom_right: "┛",
                horizontal: "━",
                vertical: "┃",
            },
            Self::QuadrantInside => BorderSymbols {
                top_left: "▗",
                top_right: "▖",
                bottom_left: "▝",
                bottom_right: "▘",
                horizontal: "▀",
                vertical: "▐",
            },
            Self::QuadrantOutside => BorderSymbols {
                top_left: "▛",
                top_right: "▜",
                bottom_left: "▙",
                bottom_right: "▟",
                horizontal: "▀",
                vertical: "▌",
            },
        }
    }
}

struct BorderSymbols {
    top_left: &'static str,
    top_right: &'static str,
    bottom_left: &'static str,
    bottom_right: &'static str,
    horizontal: &'static str,
    vertical: &'static str,
}

/// Which borders to display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Borders(u8);

impl Borders {
    pub const NONE: Self = Self(0);
    pub const TOP: Self = Self(1 << 0);
    pub const RIGHT: Self = Self(1 << 1);
    pub const BOTTOM: Self = Self(1 << 2);
    pub const LEFT: Self = Self(1 << 3);
    pub const ALL: Self = Self(0b1111);

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl Default for Borders {
    fn default() -> Self {
        Self::NONE
    }
}

impl std::ops::BitOr for Borders {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

/// A container widget that draws borders, titles, and background.
#[derive(Debug, Clone, Default)]
pub struct Block {
    title: Option<Line>,
    title_bottom: Option<Line>,
    title_alignment: Alignment,
    title_bottom_alignment: Alignment,
    borders: Borders,
    border_type: BorderType,
    border_style: Style,
    style: Style,
    padding: Margin,
}

impl Block {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bordered() -> Self {
        Self {
            borders: Borders::ALL,
            border_type: BorderType::Plain,
            ..Default::default()
        }
    }

    pub fn title(mut self, title: impl Into<Line>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn title_bottom(mut self, title: impl Into<Line>) -> Self {
        self.title_bottom = Some(title.into());
        self
    }

    pub fn title_alignment(mut self, alignment: Alignment) -> Self {
        self.title_alignment = alignment;
        self
    }

    pub fn title_bottom_alignment(mut self, alignment: Alignment) -> Self {
        self.title_bottom_alignment = alignment;
        self
    }

    pub fn borders(mut self, borders: Borders) -> Self {
        self.borders = borders;
        self
    }

    pub fn border_type(mut self, border_type: BorderType) -> Self {
        self.border_type = border_type;
        self
    }

    pub fn border_style(mut self, style: Style) -> Self {
        self.border_style = style;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn padding(mut self, padding: Margin) -> Self {
        self.padding = padding;
        self
    }

    /// Calculate the inner area after borders and padding.
    pub fn inner(&self, area: Rect) -> Rect {
        let mut inner = area;

        if self.borders.contains(Borders::LEFT) {
            inner.x = inner.x.saturating_add(1);
            inner.width = inner.width.saturating_sub(1);
        }
        if self.borders.contains(Borders::RIGHT) {
            inner.width = inner.width.saturating_sub(1);
        }
        if self.borders.contains(Borders::TOP) {
            inner.y = inner.y.saturating_add(1);
            inner.height = inner.height.saturating_sub(1);
        }
        if self.borders.contains(Borders::BOTTOM) {
            inner.height = inner.height.saturating_sub(1);
        }

        inner.inner(self.padding)
    }
}

impl Widget for Block {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        // Fill background
        buf.set_style(area, self.style);

        let symbols = self.border_type.symbols();

        // Draw borders
        if self.borders.contains(Borders::TOP) {
            for x in area.left()..area.right() {
                buf.set_string(x, area.top(), symbols.horizontal, self.border_style);
            }
        }
        if self.borders.contains(Borders::BOTTOM) {
            let y = area.bottom().saturating_sub(1);
            for x in area.left()..area.right() {
                buf.set_string(x, y, symbols.horizontal, self.border_style);
            }
        }
        if self.borders.contains(Borders::LEFT) {
            for y in area.top()..area.bottom() {
                buf.set_string(area.left(), y, symbols.vertical, self.border_style);
            }
        }
        if self.borders.contains(Borders::RIGHT) {
            let x = area.right().saturating_sub(1);
            for y in area.top()..area.bottom() {
                buf.set_string(x, y, symbols.vertical, self.border_style);
            }
        }

        // Draw corners
        if self.borders.contains(Borders::TOP) && self.borders.contains(Borders::LEFT) {
            buf.set_string(area.left(), area.top(), symbols.top_left, self.border_style);
        }
        if self.borders.contains(Borders::TOP) && self.borders.contains(Borders::RIGHT) {
            buf.set_string(
                area.right().saturating_sub(1),
                area.top(),
                symbols.top_right,
                self.border_style,
            );
        }
        if self.borders.contains(Borders::BOTTOM) && self.borders.contains(Borders::LEFT) {
            buf.set_string(
                area.left(),
                area.bottom().saturating_sub(1),
                symbols.bottom_left,
                self.border_style,
            );
        }
        if self.borders.contains(Borders::BOTTOM) && self.borders.contains(Borders::RIGHT) {
            buf.set_string(
                area.right().saturating_sub(1),
                area.bottom().saturating_sub(1),
                symbols.bottom_right,
                self.border_style,
            );
        }

        // Draw title
        if let Some(title) = &self.title {
            if self.borders.contains(Borders::TOP) && area.width > 2 {
                let max_width = area.width.saturating_sub(2);
                let title_width = title.width().min(max_width as usize) as u16;
                let title_x = match self.title_alignment {
                    Alignment::Left => area.left() + 1,
                    Alignment::Center => {
                        area.left() + 1 + (max_width.saturating_sub(title_width)) / 2
                    }
                    Alignment::Right => area.left() + 1 + max_width.saturating_sub(title_width),
                };
                buf.set_line(title_x, area.top(), title, max_width);
            }
        }

        // Draw bottom title
        if let Some(title) = &self.title_bottom {
            if self.borders.contains(Borders::BOTTOM) && area.width > 2 {
                let max_width = area.width.saturating_sub(2);
                let title_width = title.width().min(max_width as usize) as u16;
                let title_x = match self.title_bottom_alignment {
                    Alignment::Left => area.left() + 1,
                    Alignment::Center => {
                        area.left() + 1 + (max_width.saturating_sub(title_width)) / 2
                    }
                    Alignment::Right => area.left() + 1 + max_width.saturating_sub(title_width),
                };
                buf.set_line(title_x, area.bottom().saturating_sub(1), title, max_width);
            }
        }
    }
}

impl Discoverable for Block {
    fn schema() -> WidgetSchema {
        WidgetSchema {
            name: "Block".into(),
            description: "A container widget with borders, titles, and background styling.".into(),
            default_role: SemanticRole::Container,
            properties: vec![
                PropertySchema {
                    name: "title".into(),
                    description: "Title displayed in the top border.".into(),
                    property_type: PropertyType::String,
                    required: false,
                    default_value: None,
                    constraints: vec![],
                },
                PropertySchema {
                    name: "borders".into(),
                    description: "Which borders to display.".into(),
                    property_type: PropertyType::Enum(vec![
                        "none".into(),
                        "all".into(),
                        "top".into(),
                        "bottom".into(),
                        "left".into(),
                        "right".into(),
                    ]),
                    required: false,
                    default_value: Some(serde_json::json!("none")),
                    constraints: vec![],
                },
                PropertySchema {
                    name: "border_type".into(),
                    description: "Visual style of the border.".into(),
                    property_type: PropertyType::Enum(vec![
                        "plain".into(),
                        "rounded".into(),
                        "double".into(),
                        "thick".into(),
                    ]),
                    required: false,
                    default_value: Some(serde_json::json!("plain")),
                    constraints: vec![],
                },
            ],
            actions: vec![],

            usage_hint: Some("Block::bordered().title(\"My Panel\")".into()),
            tags: vec!["container".into(), "border".into(), "panel".into()],
        }
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Container
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "has_title": self.title.is_some(),
            "border_type": format!("{:?}", self.border_type),
        })
    }

    fn execute_action(
        &mut self,
        _action: &str,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        Err("Block has no executable actions.".into())
    }
}
