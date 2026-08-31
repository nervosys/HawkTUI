//! Canvas shapes: circles, filled rectangles, and user-defined shapes.

use hawktui::core::buffer::Buffer;
use hawktui::core::rect::{Position, Rect};
use hawktui::core::style::Color;
use hawktui::widget::canvas::{Canvas, CanvasCircle, CanvasFilledRect, CanvasLine, Painter, Shape};
use hawktui::widget::Widget;

fn area() -> Rect {
    Rect::new(0, 0, 20, 10)
}

/// Cells that received any paint, as a count.
fn painted(buf: &Buffer) -> usize {
    (0..area().height)
        .flat_map(|y| (0..area().width).map(move |x| (x, y)))
        .filter(|(x, y)| {
            buf.cell(Position::new(*x, *y))
                .map(|c| c.symbol.as_str() != " ")
                .unwrap_or(false)
        })
        .count()
}

fn render(canvas: Canvas) -> Buffer {
    let mut buf = Buffer::empty(area());
    canvas.render(area(), &mut buf);
    buf
}

#[test]
fn a_circle_paints_an_outline_not_a_disc() {
    let outline = render(
        Canvas::new()
            .x_bounds([0.0, 100.0])
            .y_bounds([0.0, 100.0])
            .circle(CanvasCircle {
                x: 50.0,
                y: 50.0,
                radius: 30.0,
                color: Color::Red,
            }),
    );
    let disc = render(
        Canvas::new()
            .x_bounds([0.0, 100.0])
            .y_bounds([0.0, 100.0])
            .filled_rect(CanvasFilledRect {
                x: 20.0,
                y: 20.0,
                width: 60.0,
                height: 60.0,
                color: Color::Red,
            }),
    );

    assert!(painted(&outline) > 0, "circle painted nothing");
    assert!(
        painted(&outline) < painted(&disc),
        "an outline should touch fewer cells than a filled area of the same span"
    );
}

#[test]
fn a_zero_radius_circle_paints_nothing() {
    let buf = render(Canvas::new().circle(CanvasCircle {
        x: 50.0,
        y: 50.0,
        radius: 0.0,
        color: Color::Red,
    }));
    assert_eq!(painted(&buf), 0);
}

#[test]
fn a_circle_stays_inside_its_bounding_box() {
    // A small circle in the lower-left quadrant must not paint the top-right.
    let buf = render(
        Canvas::new()
            .x_bounds([0.0, 100.0])
            .y_bounds([0.0, 100.0])
            .circle(CanvasCircle {
                x: 20.0,
                y: 20.0,
                radius: 10.0,
                color: Color::Red,
            }),
    );
    let top_right_painted = (14..20)
        .flat_map(|x| (0..3).map(move |y| (x, y)))
        .any(|(x, y)| {
            buf.cell(Position::new(x, y))
                .map(|c| c.symbol.as_str() != " ")
                .unwrap_or(false)
        });
    assert!(!top_right_painted, "circle leaked outside its radius");
}

#[test]
fn a_filled_rectangle_covers_its_interior() {
    let filled = render(
        Canvas::new()
            .x_bounds([0.0, 100.0])
            .y_bounds([0.0, 100.0])
            .filled_rect(CanvasFilledRect {
                x: 10.0,
                y: 10.0,
                width: 80.0,
                height: 80.0,
                color: Color::Green,
            }),
    );
    let outlined = render(
        Canvas::new()
            .x_bounds([0.0, 100.0])
            .y_bounds([0.0, 100.0])
            .rect(hawktui::widget::canvas::CanvasRect {
                x: 10.0,
                y: 10.0,
                width: 80.0,
                height: 80.0,
                color: Color::Green,
            }),
    );
    assert!(
        painted(&filled) > painted(&outlined),
        "filled {} should exceed outline {}",
        painted(&filled),
        painted(&outlined)
    );
}

/// A user-defined shape: a horizontal tick mark.
struct Tick {
    y: f64,
}

impl Shape for Tick {
    fn draw(&self, painter: &mut Painter) {
        CanvasLine {
            x1: 0.0,
            y1: self.y,
            x2: 100.0,
            y2: self.y,
            color: Color::Cyan,
        }
        .draw(painter);
    }
}

#[test]
fn user_defined_shapes_render_like_built_in_ones() {
    let custom = render(
        Canvas::new()
            .x_bounds([0.0, 100.0])
            .y_bounds([0.0, 100.0])
            .shape(Tick { y: 50.0 }),
    );
    let equivalent = render(
        Canvas::new()
            .x_bounds([0.0, 100.0])
            .y_bounds([0.0, 100.0])
            .line(CanvasLine {
                x1: 0.0,
                y1: 50.0,
                x2: 100.0,
                y2: 50.0,
                color: Color::Cyan,
            }),
    );
    assert_eq!(painted(&custom), painted(&equivalent));
    assert!(painted(&custom) > 0);
}

#[test]
fn shapes_compose_on_one_canvas() {
    let buf = render(
        Canvas::new()
            .x_bounds([0.0, 100.0])
            .y_bounds([0.0, 100.0])
            .circle(CanvasCircle {
                x: 30.0,
                y: 50.0,
                radius: 20.0,
                color: Color::Red,
            })
            .circle(CanvasCircle {
                x: 70.0,
                y: 50.0,
                radius: 20.0,
                color: Color::Blue,
            }),
    );
    assert!(painted(&buf) > 0);
}
