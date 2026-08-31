use crate::core::buffer::Buffer;
use crate::core::rect::Rect;
use crate::core::style::{Color, Style};
use crate::ontology::{
    AgentAction, AgentCapability, Discoverable, PropertySchema, PropertyType, SemanticRole,
    WidgetSchema,
};
use crate::widget::block::Block;
use crate::widget::Widget;
use std::rc::Rc;

/// A point on the canvas coordinate system.
#[derive(Debug, Clone, Copy)]
pub struct CanvasPoint {
    pub x: f64,
    pub y: f64,
}

/// Something that can be drawn on a [`Canvas`].
pub trait Shape {
    fn draw(&self, painter: &mut Painter);
}

/// A line between two points.
#[derive(Debug, Clone)]
pub struct CanvasLine {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub color: Color,
}

impl Shape for CanvasLine {
    fn draw(&self, painter: &mut Painter) {
        // Bresenham's line algorithm in float canvas coordinates
        let (mut x0, mut y0) = painter.canvas_to_grid(self.x1, self.y1);
        let (x1, y1) = painter.canvas_to_grid(self.x2, self.y2);

        let dx = (x1 as i32 - x0 as i32).abs();
        let dy = -(y1 as i32 - y0 as i32).abs();
        let sx: i32 = if x0 < x1 { 1 } else { -1 };
        let sy: i32 = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            painter.paint(x0, y0, self.color);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 = (x0 as i32 + sx) as usize;
            }
            if e2 <= dx {
                err += dx;
                y0 = (y0 as i32 + sy) as usize;
            }
        }
    }
}

/// A rectangle on the canvas.
#[derive(Debug, Clone)]
pub struct CanvasRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: Color,
}

impl Shape for CanvasRect {
    fn draw(&self, painter: &mut Painter) {
        let line_top = CanvasLine {
            x1: self.x,
            y1: self.y,
            x2: self.x + self.width,
            y2: self.y,
            color: self.color,
        };
        let line_bottom = CanvasLine {
            x1: self.x,
            y1: self.y + self.height,
            x2: self.x + self.width,
            y2: self.y + self.height,
            color: self.color,
        };
        let line_left = CanvasLine {
            x1: self.x,
            y1: self.y,
            x2: self.x,
            y2: self.y + self.height,
            color: self.color,
        };
        let line_right = CanvasLine {
            x1: self.x + self.width,
            y1: self.y,
            x2: self.x + self.width,
            y2: self.y + self.height,
            color: self.color,
        };
        line_top.draw(painter);
        line_bottom.draw(painter);
        line_left.draw(painter);
        line_right.draw(painter);
    }
}

/// A circle outline, drawn with the midpoint circle algorithm.
#[derive(Debug, Clone)]
pub struct CanvasCircle {
    /// Center x in canvas coordinates.
    pub x: f64,
    /// Center y in canvas coordinates.
    pub y: f64,
    /// Radius in canvas coordinates.
    pub radius: f64,
    pub color: Color,
}

impl Shape for CanvasCircle {
    fn draw(&self, painter: &mut Painter) {
        if self.radius <= 0.0 {
            return;
        }
        // Sample the circle in canvas space so the radius stays circular under
        // the canvas-to-grid aspect ratio, then let the painter map each point.
        let (cx, cy) = painter.canvas_to_grid(self.x, self.y);
        let (ex, _) = painter.canvas_to_grid(self.x + self.radius, self.y);
        let (_, ey) = painter.canvas_to_grid(self.x, self.y + self.radius);
        let rx = (ex as f64 - cx as f64).abs().max(1.0);
        let ry = (ey as f64 - cy as f64).abs().max(1.0);

        // One sample per grid step around the larger axis keeps the outline
        // gap-free without over-drawing.
        let steps = ((rx.max(ry) * 8.0) as usize).max(16);
        for i in 0..steps {
            let theta = (i as f64) * std::f64::consts::TAU / (steps as f64);
            let gx = cx as f64 + rx * theta.cos();
            let gy = cy as f64 + ry * theta.sin();
            if gx < 0.0 || gy < 0.0 {
                continue;
            }
            painter.paint(gx as usize, gy as usize, self.color);
        }
    }
}

/// A filled rectangle on the canvas.
#[derive(Debug, Clone)]
pub struct CanvasFilledRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: Color,
}

impl Shape for CanvasFilledRect {
    fn draw(&self, painter: &mut Painter) {
        let (x0, y0) = painter.canvas_to_grid(self.x, self.y);
        let (x1, y1) = painter.canvas_to_grid(self.x + self.width, self.y + self.height);
        let (left, right) = (x0.min(x1), x0.max(x1));
        let (top, bottom) = (y0.min(y1), y0.max(y1));
        for gy in top..=bottom {
            for gx in left..=right {
                painter.paint(gx, gy, self.color);
            }
        }
    }
}

/// How densely a [`CanvasMap`] samples its data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MapResolution {
    /// Draw every fourth vertex. Enough for a continent outline in a small
    /// pane, and four times cheaper.
    #[default]
    Low,
    /// Draw every vertex in the dataset.
    High,
}

impl MapResolution {
    /// Vertices to advance between samples.
    const fn stride(self) -> usize {
        match self {
            Self::Low => 4,
            Self::High => 1,
        }
    }
}

/// Geographic paths in `(longitude, latitude)` degrees.
///
/// Hawk TUI ships the renderer, not the cartography: point this at whatever
/// dataset your program already has. [`from_geojson`](Self::from_geojson)
/// reads Natural Earth and most other public coastline files directly, and
/// [`path`](Self::path) takes coordinates you build yourself.
///
/// ```
/// use hawktui::widget::canvas::MapData;
///
/// let data = MapData::new().path([(-9.5, 38.7), (2.3, 48.9), (13.4, 52.5)]);
/// assert_eq!(data.point_count(), 3);
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MapData {
    paths: Vec<Vec<(f64, f64)>>,
}

impl MapData {
    /// An empty dataset.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add one polyline.
    pub fn path(mut self, points: impl IntoIterator<Item = (f64, f64)>) -> Self {
        let points: Vec<(f64, f64)> = points.into_iter().collect();
        if !points.is_empty() {
            self.paths.push(points);
        }
        self
    }

    /// Build from many polylines at once.
    pub fn from_paths(paths: impl IntoIterator<Item = Vec<(f64, f64)>>) -> Self {
        Self {
            paths: paths.into_iter().filter(|p| !p.is_empty()).collect(),
        }
    }

    /// Read every line and polygon ring out of a GeoJSON document.
    ///
    /// `LineString`, `MultiLineString`, `Polygon`, and `MultiPolygon`
    /// geometries all become paths; points and unknown members are skipped.
    /// Feature properties are ignored — this is an outline reader, not a full
    /// GeoJSON implementation.
    ///
    /// Returns `None` if the input is not valid JSON.
    pub fn from_geojson(source: &str) -> Option<Self> {
        let value: serde_json::Value = serde_json::from_str(source).ok()?;
        let mut data = Self::new();
        collect_geojson(&value, &mut data.paths);
        Some(data)
    }

    /// The polylines, in insertion order.
    pub fn paths(&self) -> &[Vec<(f64, f64)>] {
        &self.paths
    }

    /// Total number of vertices across all paths.
    pub fn point_count(&self) -> usize {
        self.paths.iter().map(Vec::len).sum()
    }

    /// True when there is nothing to draw.
    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }
}

/// Walk a GeoJSON value, pushing every line or ring it contains.
fn collect_geojson(value: &serde_json::Value, out: &mut Vec<Vec<(f64, f64)>>) {
    let Some(object) = value.as_object() else {
        if let Some(array) = value.as_array() {
            for item in array {
                collect_geojson(item, out);
            }
        }
        return;
    };

    // Containers: recurse into whatever holds the geometries.
    for key in ["features", "geometries", "geometry"] {
        if let Some(inner) = object.get(key) {
            collect_geojson(inner, out);
        }
    }

    let Some(kind) = object.get("type").and_then(serde_json::Value::as_str) else {
        return;
    };
    let Some(coordinates) = object.get("coordinates") else {
        return;
    };
    // Nesting depth of the coordinate array before the [lon, lat] pairs.
    let depth = match kind {
        "LineString" => 1,
        "MultiLineString" | "Polygon" => 2,
        "MultiPolygon" => 3,
        _ => return,
    };
    push_rings(coordinates, depth, out);
}

/// Descend `depth` levels of array nesting, then read each innermost array as
/// a list of `[lon, lat]` pairs.
fn push_rings(value: &serde_json::Value, depth: usize, out: &mut Vec<Vec<(f64, f64)>>) {
    let Some(array) = value.as_array() else {
        return;
    };
    if depth > 1 {
        for item in array {
            push_rings(item, depth - 1, out);
        }
        return;
    }
    let ring: Vec<(f64, f64)> = array
        .iter()
        .filter_map(|pair| {
            let pair = pair.as_array()?;
            Some((pair.first()?.as_f64()?, pair.get(1)?.as_f64()?))
        })
        .collect();
    if !ring.is_empty() {
        out.push(ring);
    }
}

/// Coastlines, borders, or any other geographic paths drawn on a canvas.
///
/// Coordinates are plotted directly as `(longitude, latitude)`, so a canvas
/// with [`Canvas::geographic`] bounds shows an equirectangular projection.
#[derive(Debug, Clone)]
pub struct CanvasMap {
    pub data: MapData,
    pub color: Color,
    pub resolution: MapResolution,
    /// Join consecutive vertices with lines. With this off the map is drawn as
    /// a dot cloud, which reads better at very small sizes.
    pub connect: bool,
}

impl CanvasMap {
    /// Draw `data` in the default color, connecting vertices.
    pub fn new(data: MapData) -> Self {
        Self {
            data,
            color: Color::White,
            resolution: MapResolution::default(),
            connect: true,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn resolution(mut self, resolution: MapResolution) -> Self {
        self.resolution = resolution;
        self
    }

    pub fn connect(mut self, connect: bool) -> Self {
        self.connect = connect;
        self
    }
}

impl Shape for CanvasMap {
    fn draw(&self, painter: &mut Painter) {
        let stride = self.resolution.stride();
        for path in self.data.paths() {
            if path.is_empty() {
                continue;
            }
            let mut previous: Option<(f64, f64)> = None;
            // Always include the final vertex so a decimated ring still closes
            // where the data says it does.
            let indices = (0..path.len())
                .step_by(stride)
                .chain(std::iter::once(path.len() - 1));
            for index in indices {
                let (lon, lat) = path[index];
                if !lon.is_finite() || !lat.is_finite() {
                    previous = None;
                    continue;
                }
                if self.connect {
                    if let Some((plon, plat)) = previous {
                        // A jump across the antimeridian is a seam in the data,
                        // not a segment to draw across the whole map.
                        if (lon - plon).abs() <= 180.0 {
                            CanvasLine {
                                x1: plon,
                                y1: plat,
                                x2: lon,
                                y2: lat,
                                color: self.color,
                            }
                            .draw(painter);
                        }
                    }
                }
                let (gx, gy) = painter.canvas_to_grid(lon, lat);
                painter.paint(gx, gy, self.color);
                previous = Some((lon, lat));
            }
        }
    }
}

/// A scatter plot of points.
#[derive(Debug, Clone)]
pub struct Points {
    pub coords: Vec<(f64, f64)>,
    pub color: Color,
}

impl Shape for Points {
    fn draw(&self, painter: &mut Painter) {
        for &(x, y) in &self.coords {
            let (gx, gy) = painter.canvas_to_grid(x, y);
            painter.paint(gx, gy, self.color);
        }
    }
}

/// Braille dot grid for sub-cell resolution rendering.
///
/// Each terminal cell contains a 2x4 braille dot grid, giving 2x horizontal
/// and 4x vertical resolution relative to terminal cells.
#[derive(Debug, Clone)]
pub struct BrailleGrid {
    width: usize,
    height: usize,
    /// Dot data: 2 columns * 4 rows per terminal cell.
    dots: Vec<Vec<u8>>,
    colors: Vec<Vec<Color>>,
}

impl BrailleGrid {
    pub fn new(width: usize, height: usize) -> Self {
        let dot_width = width * 2;
        let dot_height = height * 4;
        Self {
            width,
            height,
            dots: vec![vec![0u8; dot_width]; dot_height],
            colors: vec![vec![Color::Reset; dot_width]; dot_height],
        }
    }

    pub fn set(&mut self, x: usize, y: usize, color: Color) {
        let dot_width = self.width * 2;
        let dot_height = self.height * 4;
        if x < dot_width && y < dot_height {
            self.dots[y][x] = 1;
            self.colors[y][x] = color;
        }
    }

    pub fn render_cell(&self, cx: usize, cy: usize) -> (char, Color) {
        // Braille base: U+2800
        // Dot positions (col, row):
        // (0,0)=0x01  (1,0)=0x08
        // (0,1)=0x02  (1,1)=0x10
        // (0,2)=0x04  (1,2)=0x20
        // (0,3)=0x40  (1,3)=0x80
        static DOT_MAP: [[u8; 2]; 4] = [[0x01, 0x08], [0x02, 0x10], [0x04, 0x20], [0x40, 0x80]];

        let mut pattern: u8 = 0;
        let mut first_color = Color::Reset;

        for (row, dot_row) in DOT_MAP.iter().enumerate() {
            for (col, &dot_val) in dot_row.iter().enumerate() {
                let dx = cx * 2 + col;
                let dy = cy * 4 + row;
                if dy < self.dots.len() && dx < self.dots[dy].len() && self.dots[dy][dx] != 0 {
                    pattern |= dot_val;
                    if first_color == Color::Reset {
                        first_color = self.colors[dy][dx];
                    }
                }
            }
        }

        (
            char::from_u32(0x2800 + pattern as u32).unwrap_or(' '),
            first_color,
        )
    }
}

/// Painter provides coordinate mapping and drawing operations for shapes.
pub struct Painter {
    grid: BrailleGrid,
    x_bounds: [f64; 2],
    y_bounds: [f64; 2],
}

impl Painter {
    pub fn new(width: usize, height: usize, x_bounds: [f64; 2], y_bounds: [f64; 2]) -> Self {
        Self {
            grid: BrailleGrid::new(width, height),
            x_bounds,
            y_bounds,
        }
    }

    /// Convert canvas coordinates to braille grid coordinates.
    pub fn canvas_to_grid(&self, x: f64, y: f64) -> (usize, usize) {
        let grid_width = self.grid.width * 2;
        let grid_height = self.grid.height * 4;
        let x_range = self.x_bounds[1] - self.x_bounds[0];
        let y_range = self.y_bounds[1] - self.y_bounds[0];

        let gx = if x_range == 0.0 {
            0
        } else {
            (((x - self.x_bounds[0]) / x_range * (grid_width as f64 - 1.0)).round() as usize)
                .min(grid_width.saturating_sub(1))
        };

        // Y is inverted: lower bound at bottom
        let gy = if y_range == 0.0 {
            0
        } else {
            let normalized = (y - self.y_bounds[0]) / y_range;
            ((1.0 - normalized) * (grid_height as f64 - 1.0)).round() as usize
        }
        .min(grid_height.saturating_sub(1));

        (gx, gy)
    }

    pub fn paint(&mut self, x: usize, y: usize, color: Color) {
        self.grid.set(x, y, color);
    }

    pub fn into_grid(self) -> BrailleGrid {
        self.grid
    }
}

/// A freeform drawing canvas using braille characters for sub-cell resolution.
#[derive(Debug, Clone)]
pub struct Canvas {
    block: Option<Block>,
    x_bounds: [f64; 2],
    y_bounds: [f64; 2],
    style: Style,
    shapes: Vec<CanvasShapeBox>,
}

/// Type-erased shape storage for the Canvas builder.
///
/// Built-in shapes are stored by value so the canvas stays `Clone`; user shapes
/// are shared behind an `Rc`, which keeps that property without requiring them
/// to be clonable themselves.
#[derive(Clone)]
enum CanvasShapeBox {
    Line(CanvasLine),
    Rect(CanvasRect),
    FilledRect(CanvasFilledRect),
    Circle(CanvasCircle),
    Points(Points),
    Map(CanvasMap),
    Custom(Rc<dyn Shape>),
}

impl CanvasShapeBox {
    fn draw(&self, painter: &mut Painter) {
        match self {
            CanvasShapeBox::Line(s) => s.draw(painter),
            CanvasShapeBox::Rect(s) => s.draw(painter),
            CanvasShapeBox::FilledRect(s) => s.draw(painter),
            CanvasShapeBox::Circle(s) => s.draw(painter),
            CanvasShapeBox::Points(s) => s.draw(painter),
            CanvasShapeBox::Map(s) => s.draw(painter),
            CanvasShapeBox::Custom(s) => s.draw(painter),
        }
    }
}

impl std::fmt::Debug for CanvasShapeBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CanvasShapeBox::Line(s) => f.debug_tuple("Line").field(s).finish(),
            CanvasShapeBox::Rect(s) => f.debug_tuple("Rect").field(s).finish(),
            CanvasShapeBox::FilledRect(s) => f.debug_tuple("FilledRect").field(s).finish(),
            CanvasShapeBox::Circle(s) => f.debug_tuple("Circle").field(s).finish(),
            CanvasShapeBox::Points(s) => f.debug_tuple("Points").field(s).finish(),
            CanvasShapeBox::Map(s) => f.debug_tuple("Map").field(s).finish(),
            CanvasShapeBox::Custom(_) => f.write_str("Custom(..)"),
        }
    }
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            block: None,
            x_bounds: [0.0, 100.0],
            y_bounds: [0.0, 100.0],
            style: Style::default(),
            shapes: Vec::new(),
        }
    }

    pub fn block(mut self, block: Block) -> Self {
        self.block = Some(block);
        self
    }

    pub fn x_bounds(mut self, bounds: [f64; 2]) -> Self {
        self.x_bounds = bounds;
        self
    }

    pub fn y_bounds(mut self, bounds: [f64; 2]) -> Self {
        self.y_bounds = bounds;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn line(mut self, line: CanvasLine) -> Self {
        self.shapes.push(CanvasShapeBox::Line(line));
        self
    }

    pub fn rect(mut self, rect: CanvasRect) -> Self {
        self.shapes.push(CanvasShapeBox::Rect(rect));
        self
    }

    /// Draw a circle outline.
    pub fn circle(mut self, circle: CanvasCircle) -> Self {
        self.shapes.push(CanvasShapeBox::Circle(circle));
        self
    }

    /// Draw a filled rectangle.
    pub fn filled_rect(mut self, rect: CanvasFilledRect) -> Self {
        self.shapes.push(CanvasShapeBox::FilledRect(rect));
        self
    }

    /// Draw any custom [`Shape`].
    ///
    /// Everything the built-in shapes do is available to your own types: paint
    /// into the [`Painter`] and the canvas composites the result the same way.
    pub fn shape(mut self, shape: impl Shape + 'static) -> Self {
        self.shapes.push(CanvasShapeBox::Custom(Rc::new(shape)));
        self
    }

    /// Draw geographic paths.
    ///
    /// Pair with [`geographic`](Self::geographic) unless you are showing a
    /// region and want to set the bounds yourself.
    pub fn map(mut self, map: CanvasMap) -> Self {
        self.shapes.push(CanvasShapeBox::Map(map));
        self
    }

    /// Set whole-world bounds: longitude -180..180, latitude -90..90.
    pub fn geographic(mut self) -> Self {
        self.x_bounds = [-180.0, 180.0];
        self.y_bounds = [-90.0, 90.0];
        self
    }

    pub fn points(mut self, points: Points) -> Self {
        self.shapes.push(CanvasShapeBox::Points(points));
        self
    }
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Canvas {
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

        let width = inner.width as usize;
        let height = inner.height as usize;

        let mut painter = Painter::new(width, height, self.x_bounds, self.y_bounds);

        for shape in &self.shapes {
            shape.draw(&mut painter);
        }

        let grid = painter.into_grid();

        for cy in 0..height {
            for cx in 0..width {
                let (ch, color) = grid.render_cell(cx, cy);
                let x = inner.x + cx as u16;
                let y = inner.y + cy as u16;
                if ch != '\u{2800}' {
                    // Not empty braille
                    buf[(x, y)].set_char(ch);
                    if color != Color::Reset {
                        buf[(x, y)].set_style(Style::default().fg(color));
                    }
                }
            }
        }
    }
}

impl Discoverable for Canvas {
    fn schema() -> WidgetSchema {
        WidgetSchema {
            name: "Canvas".into(),
            description:
                "A freeform drawing surface using braille characters for sub-cell resolution."
                    .into(),
            default_role: SemanticRole::DataVisualization,
            properties: vec![
                PropertySchema {
                    name: "x_bounds".into(),
                    description: "Horizontal coordinate range [min, max].".into(),
                    property_type: PropertyType::Array(Box::new(PropertyType::Float)),
                    required: false,
                    default_value: Some(serde_json::json!([0.0, 100.0])),
                    constraints: vec![],
                },
                PropertySchema {
                    name: "y_bounds".into(),
                    description: "Vertical coordinate range [min, max].".into(),
                    property_type: PropertyType::Array(Box::new(PropertyType::Float)),
                    required: false,
                    default_value: Some(serde_json::json!([0.0, 100.0])),
                    constraints: vec![],
                },
            ],
            actions: vec![],

            usage_hint: Some("Canvas::new().x_bounds([0.0, 100.0]).line(CanvasLine { .. })".into()),
            tags: vec![
                "canvas".into(),
                "draw".into(),
                "chart".into(),
                "braille".into(),
                "graph".into(),
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
            "x_bounds": self.x_bounds,
            "y_bounds": self.y_bounds,
            "shape_count": self.shapes.len(),
        })
    }

    fn execute_action(
        &mut self,
        _action: &str,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        Err("Canvas is a static rendering widget. Modify shapes through the builder.".into())
    }
}
