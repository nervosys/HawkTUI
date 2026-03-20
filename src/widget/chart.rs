use crate::core::buffer::Buffer;
use crate::core::rect::Rect;
use crate::core::style::Style;
use crate::core::text::Span;
use crate::ontology::{
    AgentAction, AgentCapability, Discoverable, PropertySchema, PropertyType, SemanticRole,
    WidgetSchema,
};
use crate::widget::block::Block;
use crate::widget::Widget;

/// A data point in a chart dataset.
pub type DataPoint = (f64, f64);

/// Marker style for data points.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marker {
    /// Braille dot (sub-cell resolution, 2x4 grid per cell).
    Braille,
    /// Full block character.
    Block,
    /// Half block characters for 2x vertical resolution.
    HalfBlock,
    /// A specific character.
    Char(char),
}

impl Default for Marker {
    fn default() -> Self {
        Self::Braille
    }
}

/// How to connect data points.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphType {
    /// Connect points with lines.
    Line,
    /// Show only individual points.
    Scatter,
}

impl Default for GraphType {
    fn default() -> Self {
        Self::Line
    }
}

/// A series of data points with styling.
#[derive(Debug, Clone)]
pub struct Dataset {
    /// Optional dataset name (shown in legend).
    pub name: Option<Span>,
    /// The data points.
    pub data: Vec<DataPoint>,
    /// Marker style.
    pub marker: Marker,
    /// Style for this dataset.
    pub style: Style,
    /// How to connect points.
    pub graph_type: GraphType,
}

impl Dataset {
    pub fn new(data: impl Into<Vec<DataPoint>>) -> Self {
        Self {
            name: None,
            data: data.into(),
            marker: Marker::default(),
            style: Style::default(),
            graph_type: GraphType::default(),
        }
    }

    pub fn name(mut self, name: impl Into<Span>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn marker(mut self, marker: Marker) -> Self {
        self.marker = marker;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn graph_type(mut self, graph_type: GraphType) -> Self {
        self.graph_type = graph_type;
        self
    }
}

/// Position for the chart legend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegendPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// An axis on the chart.
#[derive(Debug, Clone)]
pub struct Axis {
    /// Optional axis title.
    pub title: Option<Span>,
    /// Value bounds [min, max].
    pub bounds: [f64; 2],
    /// Tick labels at specific positions.
    pub labels: Vec<Span>,
    /// Style for axis elements.
    pub style: Style,
}

impl Axis {
    pub fn new(bounds: [f64; 2]) -> Self {
        Self {
            title: None,
            bounds,
            labels: Vec::new(),
            style: Style::default(),
        }
    }

    pub fn title(mut self, title: impl Into<Span>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn labels(mut self, labels: impl Into<Vec<Span>>) -> Self {
        self.labels = labels.into();
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

/// XY chart widget for line and scatter plots.
///
/// Supports multiple datasets, configurable axes, legends, and
/// both braille and block marker rendering.
///
/// # Example
///
/// ```ignore
/// use louie::widget::chart::{Chart, Dataset, Axis};
///
/// let data = Dataset::new(vec![(0.0, 0.0), (1.0, 1.0), (2.0, 4.0), (3.0, 9.0)])
///     .name("y = x²");
///
/// let chart = Chart::new(vec![data])
///     .x_axis(Axis::new([0.0, 3.0]).title("x"))
///     .y_axis(Axis::new([0.0, 9.0]).title("y"));
/// ```
#[derive(Debug, Clone)]
pub struct Chart {
    datasets: Vec<Dataset>,
    block: Option<Block>,
    x_axis: Axis,
    y_axis: Axis,
    style: Style,
    legend_position: Option<LegendPosition>,
}

impl Chart {
    pub fn new(datasets: impl Into<Vec<Dataset>>) -> Self {
        Self {
            datasets: datasets.into(),
            block: None,
            x_axis: Axis::new([0.0, 1.0]),
            y_axis: Axis::new([0.0, 1.0]),
            style: Style::default(),
            legend_position: None,
        }
    }

    pub fn block(mut self, block: Block) -> Self {
        self.block = Some(block);
        self
    }

    pub fn x_axis(mut self, axis: Axis) -> Self {
        self.x_axis = axis;
        self
    }

    pub fn y_axis(mut self, axis: Axis) -> Self {
        self.y_axis = axis;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn legend_position(mut self, pos: LegendPosition) -> Self {
        self.legend_position = Some(pos);
        self
    }

    /// Map a data value to a position in the chart area.
    fn map_x(&self, value: f64, width: u16) -> f64 {
        let range = self.x_axis.bounds[1] - self.x_axis.bounds[0];
        if range == 0.0 {
            return 0.0;
        }
        (value - self.x_axis.bounds[0]) / range * width as f64
    }

    fn map_y(&self, value: f64, height: u16) -> f64 {
        let range = self.y_axis.bounds[1] - self.y_axis.bounds[0];
        if range == 0.0 {
            return 0.0;
        }
        (value - self.y_axis.bounds[0]) / range * height as f64
    }

    fn render_braille_dataset(&self, dataset: &Dataset, chart_area: Rect, buf: &mut Buffer) {
        // Braille grid: each cell is 2 columns x 4 rows of dots
        let grid_w = chart_area.width as usize * 2;
        let grid_h = chart_area.height as usize * 4;

        if grid_w == 0 || grid_h == 0 {
            return;
        }

        let mut grid = vec![vec![false; grid_w]; grid_h];

        let points = &dataset.data;
        for i in 0..points.len() {
            let (px, py) = points[i];
            let gx = (self.map_x(px, grid_w as u16)).round() as isize;
            let gy = (grid_h as f64 - 1.0 - self.map_y(py, grid_h as u16)).round() as isize;

            if gx >= 0 && gx < grid_w as isize && gy >= 0 && gy < grid_h as isize {
                grid[gy as usize][gx as usize] = true;
            }

            // Draw line to next point
            if dataset.graph_type == GraphType::Line && i + 1 < points.len() {
                let (nx, ny) = points[i + 1];
                let ngx = (self.map_x(nx, grid_w as u16)).round() as isize;
                let ngy = (grid_h as f64 - 1.0 - self.map_y(ny, grid_h as u16)).round() as isize;
                self.bresenham_line(&mut grid, gx, gy, ngx, ngy, grid_w, grid_h);
            }
        }

        // Render braille characters
        // Braille dot positions in a 2x4 cell:
        // (0,0)=0x01 (1,0)=0x08
        // (0,1)=0x02 (1,1)=0x10
        // (0,2)=0x04 (1,2)=0x20
        // (0,3)=0x40 (1,3)=0x80
        let dot_map: [[u32; 4]; 2] = [[0x01, 0x02, 0x04, 0x40], [0x08, 0x10, 0x20, 0x80]];

        for cy in 0..chart_area.height {
            for cx in 0..chart_area.width {
                let mut braille = 0x2800u32;
                for dx in 0..2 {
                    for dy in 0..4 {
                        let gx = cx as usize * 2 + dx;
                        let gy = cy as usize * 4 + dy;
                        if gx < grid_w && gy < grid_h && grid[gy][gx] {
                            braille |= dot_map[dx][dy];
                        }
                    }
                }
                if braille != 0x2800 {
                    let ch = char::from_u32(braille).unwrap_or(' ');
                    let bx = chart_area.x + cx;
                    let by = chart_area.y + cy;
                    if bx < chart_area.right() && by < chart_area.bottom() {
                        buf[(bx, by)].set_symbol(&ch.to_string());
                        buf[(bx, by)].set_style(dataset.style);
                    }
                }
            }
        }
    }

    fn render_block_dataset(&self, dataset: &Dataset, chart_area: Rect, buf: &mut Buffer) {
        let symbol = match dataset.marker {
            Marker::Char(c) => c.to_string(),
            _ => "█".to_string(),
        };

        let points = &dataset.data;
        for i in 0..points.len() {
            let (px, py) = points[i];
            let cx = self.map_x(px, chart_area.width).round() as i32;
            let cy =
                (chart_area.height as f64 - 1.0 - self.map_y(py, chart_area.height)).round() as i32;

            if cx >= 0 && cx < chart_area.width as i32 && cy >= 0 && cy < chart_area.height as i32 {
                let bx = chart_area.x + cx as u16;
                let by = chart_area.y + cy as u16;
                buf[(bx, by)].set_symbol(&symbol);
                buf[(bx, by)].set_style(dataset.style);
            }

            // Draw line segments for Line graph type
            if dataset.graph_type == GraphType::Line && i + 1 < points.len() {
                let (nx, ny) = points[i + 1];
                let ncx = self.map_x(nx, chart_area.width).round() as i32;
                let ncy = (chart_area.height as f64 - 1.0 - self.map_y(ny, chart_area.height))
                    .round() as i32;
                self.bresenham_cells(buf, chart_area, cx, cy, ncx, ncy, &symbol, dataset.style);
            }
        }
    }

    fn bresenham_line(
        &self,
        grid: &mut [Vec<bool>],
        x0: isize,
        y0: isize,
        x1: isize,
        y1: isize,
        w: usize,
        h: usize,
    ) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx: isize = if x0 < x1 { 1 } else { -1 };
        let sy: isize = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut cx = x0;
        let mut cy = y0;

        loop {
            if cx >= 0 && cx < w as isize && cy >= 0 && cy < h as isize {
                grid[cy as usize][cx as usize] = true;
            }
            if cx == x1 && cy == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                cx += sx;
            }
            if e2 <= dx {
                err += dx;
                cy += sy;
            }
        }
    }

    fn bresenham_cells(
        &self,
        buf: &mut Buffer,
        area: Rect,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        symbol: &str,
        style: Style,
    ) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx: i32 = if x0 < x1 { 1 } else { -1 };
        let sy: i32 = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut cx = x0;
        let mut cy = y0;

        loop {
            if cx >= 0 && cx < area.width as i32 && cy >= 0 && cy < area.height as i32 {
                let bx = area.x + cx as u16;
                let by = area.y + cy as u16;
                buf[(bx, by)].set_symbol(symbol);
                buf[(bx, by)].set_style(style);
            }
            if cx == x1 && cy == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                cx += sx;
            }
            if e2 <= dx {
                err += dx;
                cy += sy;
            }
        }
    }

    fn render_legend(&self, chart_area: Rect, buf: &mut Buffer) {
        let pos = match self.legend_position {
            Some(p) => p,
            None => return,
        };

        let named: Vec<&Dataset> = self.datasets.iter().filter(|d| d.name.is_some()).collect();
        if named.is_empty() {
            return;
        }

        let max_name_w = named
            .iter()
            .map(|d| d.name.as_ref().unwrap().content.len())
            .max()
            .unwrap_or(0) as u16;
        let legend_w = max_name_w + 4; // "● name"
        let legend_h = named.len() as u16;

        if legend_w > chart_area.width || legend_h > chart_area.height {
            return;
        }

        let (lx, ly) = match pos {
            LegendPosition::TopLeft => (chart_area.x + 1, chart_area.y),
            LegendPosition::TopRight => (
                chart_area.right().saturating_sub(legend_w + 1),
                chart_area.y,
            ),
            LegendPosition::BottomLeft => (
                chart_area.x + 1,
                chart_area.bottom().saturating_sub(legend_h),
            ),
            LegendPosition::BottomRight => (
                chart_area.right().saturating_sub(legend_w + 1),
                chart_area.bottom().saturating_sub(legend_h),
            ),
        };

        for (i, ds) in named.iter().enumerate() {
            let y = ly + i as u16;
            if y >= chart_area.bottom() {
                break;
            }
            buf.set_string(lx, y, "● ", ds.style);
            let name = ds.name.as_ref().unwrap();
            buf.set_string(lx + 2, y, &name.content, ds.style);
        }
    }
}

impl Default for Chart {
    fn default() -> Self {
        Self::new(Vec::<Dataset>::new())
    }
}

impl Widget for Chart {
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

        // Reserve space for axes
        let has_y_labels = !self.y_axis.labels.is_empty();
        let has_x_labels = !self.x_axis.labels.is_empty();
        let y_label_width = if has_y_labels {
            self.y_axis
                .labels
                .iter()
                .map(|l| l.content.len() as u16)
                .max()
                .unwrap_or(0)
                + 1
        } else {
            0
        };
        let x_label_height = has_x_labels as u16;

        let chart_area = Rect::new(
            inner.x + y_label_width,
            inner.y,
            inner.width.saturating_sub(y_label_width),
            inner.height.saturating_sub(x_label_height),
        );

        if chart_area.is_empty() {
            return;
        }

        // Draw Y axis labels
        if has_y_labels {
            let n_labels = self.y_axis.labels.len();
            for (i, label) in self.y_axis.labels.iter().enumerate() {
                let y = if n_labels <= 1 {
                    chart_area.y
                } else {
                    chart_area.bottom()
                        - 1
                        - ((i as f64 / (n_labels - 1) as f64)
                            * (chart_area.height.saturating_sub(1)) as f64)
                            .round() as u16
                };
                let text = &label.content;
                let tw = text.len() as u16;
                let lx = inner.x + y_label_width.saturating_sub(tw + 1);
                buf.set_string(lx, y, text, self.y_axis.style);
            }
        }

        // Draw X axis labels
        if has_x_labels {
            let label_y = chart_area.bottom();
            let n_labels = self.x_axis.labels.len();
            for (i, label) in self.x_axis.labels.iter().enumerate() {
                let x = if n_labels <= 1 {
                    chart_area.x
                } else {
                    chart_area.x
                        + ((i as f64 / (n_labels - 1) as f64)
                            * chart_area.width.saturating_sub(1) as f64)
                            .round() as u16
                };
                buf.set_string(x, label_y, &label.content, self.x_axis.style);
            }
        }

        // Render datasets
        for dataset in &self.datasets {
            match dataset.marker {
                Marker::Braille => self.render_braille_dataset(dataset, chart_area, buf),
                _ => self.render_block_dataset(dataset, chart_area, buf),
            }
        }

        // Render legend
        self.render_legend(chart_area, buf);
    }
}

impl Discoverable for Chart {
    fn schema() -> WidgetSchema {
        WidgetSchema {
            name: "Chart".into(),
            description: "An XY chart for line and scatter plots with multiple datasets.".into(),
            default_role: SemanticRole::DataVisualization,
            properties: vec![
                PropertySchema {
                    name: "datasets".into(),
                    description: "Data series to plot, each with points and styling.".into(),
                    property_type: PropertyType::Array(Box::new(PropertyType::Object(vec![]))),
                    required: true,
                    default_value: None,
                    constraints: vec![],
                },
                PropertySchema {
                    name: "x_axis".into(),
                    description: "X axis configuration (bounds, labels).".into(),
                    property_type: PropertyType::Object(vec![]),
                    required: true,
                    default_value: None,
                    constraints: vec![],
                },
                PropertySchema {
                    name: "y_axis".into(),
                    description: "Y axis configuration (bounds, labels).".into(),
                    property_type: PropertyType::Object(vec![]),
                    required: true,
                    default_value: None,
                    constraints: vec![],
                },
            ],
            usage_hint: Some(
                "Use for plotting numerical data trends. Braille markers give sub-cell resolution."
                    .into(),
            ),
            tags: vec![
                "chart".into(),
                "plot".into(),
                "line".into(),
                "scatter".into(),
                "data".into(),
                "visualization".into(),
            ],
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
        serde_json::json!({
            "dataset_count": self.datasets.len(),
            "x_bounds": self.x_axis.bounds,
            "y_bounds": self.y_axis.bounds,
        })
    }

    fn execute_action(
        &mut self,
        _action: &str,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        Err("Chart has no executable actions".into())
    }
}
