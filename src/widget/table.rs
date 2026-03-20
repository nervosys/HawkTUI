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

/// Constraint for a table column width.
#[derive(Debug, Clone, Copy)]
pub enum TableColumnWidth {
    /// Fixed character width.
    Fixed(u16),
    /// Percentage of available space.
    Percent(u16),
    /// Fill remaining space equally with other Fill columns.
    Fill,
}

/// A column definition for a [`Table`].
#[derive(Debug, Clone)]
pub struct TableColumn {
    pub header: Line,
    pub width: TableColumnWidth,
}

impl TableColumn {
    pub fn new(header: impl Into<Line>, width: TableColumnWidth) -> Self {
        Self {
            header: header.into(),
            width,
        }
    }
}

/// A single row in a [`Table`].
#[derive(Debug, Clone)]
pub struct TableRow {
    pub cells: Vec<Line>,
    pub style: Style,
    pub height: u16,
}

impl TableRow {
    pub fn new(cells: impl IntoIterator<Item = impl Into<Line>>) -> Self {
        Self {
            cells: cells.into_iter().map(Into::into).collect(),
            style: Style::default(),
            height: 1,
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn height(mut self, height: u16) -> Self {
        self.height = height;
        self
    }
}

/// Table state for scrolling and selection.
#[derive(Debug, Clone, Default)]
pub struct TableState {
    pub selected: Option<usize>,
    pub offset: usize,
}

impl TableState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn select(&mut self, index: Option<usize>) {
        self.selected = index;
    }

    pub fn select_next(&mut self, row_count: usize) {
        if row_count == 0 {
            return;
        }
        let i = match self.selected {
            Some(i) => (i + 1).min(row_count - 1),
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
}

/// A data table widget with column headers and row selection.
#[derive(Debug, Clone)]
pub struct Table {
    columns: Vec<TableColumn>,
    rows: Vec<TableRow>,
    block: Option<Block>,
    style: Style,
    header_style: Style,
    highlight_style: Style,
    column_spacing: u16,
}

impl Table {
    pub fn new(
        columns: impl IntoIterator<Item = TableColumn>,
        rows: impl IntoIterator<Item = TableRow>,
    ) -> Self {
        Self {
            columns: columns.into_iter().collect(),
            rows: rows.into_iter().collect(),
            block: None,
            style: Style::default(),
            header_style: Style::default().bold(),
            highlight_style: Style::default().reversed(),
            column_spacing: 1,
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

    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }

    pub fn column_spacing(mut self, spacing: u16) -> Self {
        self.column_spacing = spacing;
        self
    }

    fn resolve_column_widths(&self, available: u16) -> Vec<u16> {
        let col_count = self.columns.len();
        if col_count == 0 {
            return vec![];
        }

        let total_spacing = self.column_spacing * (col_count as u16).saturating_sub(1);
        let usable = available.saturating_sub(total_spacing);

        let mut widths = vec![0u16; col_count];
        let mut remaining = usable;
        let mut fill_count = 0u16;

        // First pass: fixed and percentage
        for (i, col) in self.columns.iter().enumerate() {
            match col.width {
                TableColumnWidth::Fixed(w) => {
                    widths[i] = w.min(remaining);
                    remaining = remaining.saturating_sub(widths[i]);
                }
                TableColumnWidth::Percent(p) => {
                    let w = ((usable as u32 * p as u32) / 100) as u16;
                    widths[i] = w.min(remaining);
                    remaining = remaining.saturating_sub(widths[i]);
                }
                TableColumnWidth::Fill => {
                    fill_count += 1;
                }
            }
        }

        // Second pass: fill columns
        if fill_count > 0 {
            let per_fill = remaining / fill_count;
            let mut extra = remaining % fill_count;
            for (i, col) in self.columns.iter().enumerate() {
                if matches!(col.width, TableColumnWidth::Fill) {
                    widths[i] = per_fill
                        + if extra > 0 {
                            extra -= 1;
                            1
                        } else {
                            0
                        };
                }
            }
        }

        widths
    }
}

impl StatefulWidget for Table {
    type State = TableState;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut TableState) {
        if area.is_empty() {
            return;
        }

        buf.set_style(area, self.style);

        let inner = if let Some(block) = self.block.take() {
            let inner = block.inner(area);
            block.render(area, buf);
            inner
        } else {
            area
        };

        if inner.is_empty() || self.columns.is_empty() {
            return;
        }

        let col_widths = self.resolve_column_widths(inner.width);
        let mut y = inner.y;

        // Render header
        {
            let mut x = inner.x;
            for (i, col) in self.columns.iter().enumerate() {
                if y >= inner.bottom() {
                    break;
                }
                let w = col_widths[i];
                buf.set_line(x, y, &col.header, w);
                buf.set_style(Rect::new(x, y, w, 1), self.header_style);
                x += w + self.column_spacing;
            }
            y += 1;

            // Separator line
            if y < inner.bottom() {
                for x in inner.x..inner.right() {
                    buf[(x, y)].set_symbol("─");
                    buf[(x, y)].set_style(self.header_style);
                }
                y += 1;
            }
        }

        let visible_height = inner.bottom().saturating_sub(y) as usize;

        // Adjust scroll
        if let Some(selected) = state.selected {
            if selected < state.offset {
                state.offset = selected;
            } else if selected >= state.offset + visible_height {
                state.offset = selected - visible_height + 1;
            }
        }

        // Render rows
        for (row_i, row) in self
            .rows
            .iter()
            .enumerate()
            .skip(state.offset)
            .take(visible_height)
        {
            if y >= inner.bottom() {
                break;
            }

            let is_selected = state.selected == Some(row_i);
            let row_style = if is_selected {
                self.style.patch(row.style).patch(self.highlight_style)
            } else {
                self.style.patch(row.style)
            };

            buf.set_style(Rect::new(inner.x, y, inner.width, 1), row_style);

            let mut x = inner.x;
            for (col_i, cell) in row.cells.iter().enumerate() {
                if col_i >= col_widths.len() {
                    break;
                }
                let w = col_widths[col_i];
                buf.set_line(x, y, cell, w);
                x += w + self.column_spacing;
            }

            y += 1;
        }
    }
}

impl Discoverable for Table {
    fn schema() -> WidgetSchema {
        WidgetSchema {
            name: "Table".into(),
            description: "A data table with column headers, row selection, and scrolling.".into(),
            default_role: SemanticRole::DataVisualization,
            properties: vec![
                PropertySchema {
                    name: "columns".into(),
                    description: "Column definitions with headers and widths.".into(),
                    property_type: PropertyType::Array(Box::new(PropertyType::Object(vec![]))),
                    required: true,
                    default_value: None,
                    constraints: vec![],
                },
                PropertySchema {
                    name: "rows".into(),
                    description: "Data rows containing cells for each column.".into(),
                    property_type: PropertyType::Array(Box::new(PropertyType::Array(Box::new(
                        PropertyType::String,
                    )))),
                    required: true,
                    default_value: None,
                    constraints: vec![],
                },
            ],
            actions: vec![],

            usage_hint: Some(
                "Table::new([TableColumn::new(\"Name\", Fill)], [TableRow::new([\"Alice\"])])"
                    .into(),
            ),
            tags: vec![
                "table".into(),
                "data".into(),
                "grid".into(),
                "columns".into(),
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
                item_count: self.rows.len(),
            },
            AgentCapability::Sortable {
                columns: self
                    .columns
                    .iter()
                    .map(|c| {
                        c.header
                            .spans
                            .iter()
                            .map(|s| s.content.as_ref())
                            .collect::<String>()
                    })
                    .collect(),
            },
            AgentCapability::HasKeyBindings {
                bindings: vec![
                    ("Up/k".into(), "Previous row".into()),
                    ("Down/j".into(), "Next row".into()),
                    ("Home".into(), "First row".into()),
                    ("End".into(), "Last row".into()),
                ],
            },
        ]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![
            AgentAction {
                name: "select_row".into(),
                description: "Select a row by index.".into(),
                params: vec![ActionParam {
                    name: "index".into(),
                    description: "Zero-based row index.".into(),
                    param_type: ActionParamType::Index,
                    required: true,
                    default_value: None,
                }],
                returns: Some("The selected row data.".into()),
                mutates: true,
                idempotent: true,
                shortcut: None,
            },
            AgentAction {
                name: "get_cell".into(),
                description: "Get the value of a specific cell.".into(),
                params: vec![
                    ActionParam {
                        name: "row".into(),
                        description: "Zero-based row index.".into(),
                        param_type: ActionParamType::Index,
                        required: true,
                        default_value: None,
                    },
                    ActionParam {
                        name: "column".into(),
                        description: "Zero-based column index.".into(),
                        param_type: ActionParamType::Index,
                        required: true,
                        default_value: None,
                    },
                ],
                returns: Some("Cell text content.".into()),
                mutates: false,
                idempotent: true,
                shortcut: None,
            },
            AgentAction {
                name: "get_row_count".into(),
                description: "Get the total number of rows.".into(),
                params: vec![],
                returns: Some("Row count as integer.".into()),
                mutates: false,
                idempotent: true,
                shortcut: None,
            },
        ]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::DataVisualization
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "column_count": self.columns.len(),
            "row_count": self.rows.len(),
            "columns": self.columns.iter().map(|c|
                c.header.spans.iter().map(|s| s.content.as_ref()).collect::<String>()
            ).collect::<Vec<_>>(),
        })
    }

    fn execute_action(
        &mut self,
        _action: &str,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        Err("Table actions require TableState. Use the runtime to dispatch actions.".into())
    }
}
