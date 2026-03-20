use crate::core::buffer::Buffer;
use crate::core::rect::Rect;
use crate::core::style::{Color, Style};
use crate::ontology::{
    AgentAction, AgentCapability, Discoverable, PropertySchema, PropertyType, SemanticRole,
    WidgetSchema,
};
use crate::widget::block::Block;
use crate::widget::Widget;

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
        static DOT_MAP: [[u8; 2]; 4] = [
            [0x01, 0x08],
            [0x02, 0x10],
            [0x04, 0x20],
            [0x40, 0x80],
        ];

        let mut pattern: u8 = 0;
        let mut first_color = Color::Reset;

        for row in 0..4 {
            for col in 0..2 {
                let dx = cx * 2 + col;
                let dy = cy * 4 + row;
                if dy < self.dots.len() && dx < self.dots[dy].len() && self.dots[dy][dx] != 0 {
                    pattern |= DOT_MAP[row][col];
                    if first_color == Color::Reset {
                        first_color = self.colors[dy][dx];
                    }
                }
            }
        }

        (char::from_u32(0x2800 + pattern as u32).unwrap_or(' '), first_color)
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
#[derive(Debug, Clone)]
enum CanvasShapeBox {
    Line(CanvasLine),
    Rect(CanvasRect),
    Points(Points),
}

impl CanvasShapeBox {
    fn draw(&self, painter: &mut Painter) {
        match self {
            CanvasShapeBox::Line(s) => s.draw(painter),
            CanvasShapeBox::Rect(s) => s.draw(painter),
            CanvasShapeBox::Points(s) => s.draw(painter),
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

            usage_hint: Some(
                "Canvas::new().x_bounds([0.0, 100.0]).line(CanvasLine { .. })".into(),
            ),
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
