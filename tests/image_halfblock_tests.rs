//! Half-block image rendering: two pixels per cell, no graphics protocol.

use hawktui::core::buffer::Buffer;
use hawktui::core::rect::{Position, Rect};
use hawktui::core::style::Color;
use hawktui::widget::image::{Image, ImageProtocol, Pixels};
use hawktui::widget::Widget;

/// A `width × height` image where every pixel is `color`.
fn solid(width: u32, height: u32, color: [u8; 4]) -> Vec<u8> {
    color
        .iter()
        .copied()
        .cycle()
        .take((width * height * 4) as usize)
        .collect()
}

/// Two horizontal bands: the top half `top`, the bottom half `bottom`.
fn banded(width: u32, height: u32, top: [u8; 4], bottom: [u8; 4]) -> Vec<u8> {
    let mut out = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        let c = if y < height / 2 { top } else { bottom };
        for _ in 0..width {
            out.extend_from_slice(&c);
        }
    }
    out
}

#[test]
fn rgba_input_must_match_its_dimensions() {
    assert!(Pixels::from_rgba(2, 2, vec![0; 16]).is_some());
    assert!(Pixels::from_rgba(2, 2, vec![0; 15]).is_none());
    assert!(Pixels::from_rgba(0, 2, vec![]).is_none());
}

#[test]
fn rgb_input_is_promoted_to_opaque_rgba() {
    let pixels = Pixels::from_rgb(2, 1, &[1, 2, 3, 4, 5, 6]).expect("valid rgb");
    assert_eq!(pixels.width(), 2);
    assert_eq!(pixels.height(), 1);
    assert!(Pixels::from_rgb(2, 1, &[1, 2, 3]).is_none());
}

#[test]
fn a_solid_image_fills_cells_with_the_half_block_glyph() {
    let mut buf = Buffer::empty(Rect::new(0, 0, 4, 2));
    let img = Image::from_rgba(4, 4, solid(4, 4, [10, 20, 30, 255])).expect("valid image");
    img.render(Rect::new(0, 0, 4, 2), &mut buf);

    for y in 0..2 {
        for x in 0..4 {
            let cell = buf.cell(Position::new(x, y)).unwrap();
            assert_eq!(cell.symbol.as_str(), "\u{2580}", "cell ({x},{y})");
            assert_eq!(cell.fg, Color::Rgb(10, 20, 30), "top pixel at ({x},{y})");
            assert_eq!(cell.bg, Color::Rgb(10, 20, 30), "bottom pixel at ({x},{y})");
        }
    }
}

#[test]
fn each_cell_carries_two_stacked_pixels() {
    // A 1×2 image in a 1×1 cell area: top pixel red, bottom pixel blue.
    let mut buf = Buffer::empty(Rect::new(0, 0, 1, 1));
    let data = banded(1, 2, [255, 0, 0, 255], [0, 0, 255, 255]);
    let img = Image::from_rgba(1, 2, data).expect("valid image");
    img.render(Rect::new(0, 0, 1, 1), &mut buf);

    let cell = buf.cell(Position::new(0, 0)).unwrap();
    assert_eq!(cell.fg, Color::Rgb(255, 0, 0), "top half is the foreground");
    assert_eq!(
        cell.bg,
        Color::Rgb(0, 0, 255),
        "bottom half is the background"
    );
}

#[test]
fn images_scale_to_the_area_they_are_given() {
    // An 8×8 image drawn into 2×2 cells (4 pixel rows) still resolves its bands.
    let mut buf = Buffer::empty(Rect::new(0, 0, 2, 2));
    let data = banded(8, 8, [255, 255, 255, 255], [0, 0, 0, 255]);
    let img = Image::from_rgba(8, 8, data).expect("valid image");
    img.render(Rect::new(0, 0, 2, 2), &mut buf);

    assert_eq!(
        buf.cell(Position::new(0, 0)).unwrap().fg,
        Color::Rgb(255, 255, 255),
        "first row samples the white band"
    );
    assert_eq!(
        buf.cell(Position::new(0, 1)).unwrap().bg,
        Color::Rgb(0, 0, 0),
        "last row samples the black band"
    );
}

#[test]
fn transparent_pixels_leave_the_cell_untouched() {
    let mut buf = Buffer::empty(Rect::new(0, 0, 2, 1));
    buf.set_string(0, 0, "ab", hawktui::core::style::Style::default());

    let img = Image::from_rgba(2, 2, solid(2, 2, [9, 9, 9, 0])).expect("valid image");
    img.render(Rect::new(0, 0, 2, 1), &mut buf);

    assert_eq!(buf.cell(Position::new(0, 0)).unwrap().symbol.as_str(), "a");
    assert_eq!(buf.cell(Position::new(1, 0)).unwrap().symbol.as_str(), "b");
}

#[test]
fn half_block_is_the_default_protocol_for_pixel_images() {
    let img = Image::from_rgba(1, 1, solid(1, 1, [0, 0, 0, 255])).expect("valid image");
    // Rendering without setting a protocol must still paint pixels.
    let mut buf = Buffer::empty(Rect::new(0, 0, 1, 1));
    img.render(Rect::new(0, 0, 1, 1), &mut buf);
    assert_eq!(
        buf.cell(Position::new(0, 0)).unwrap().symbol.as_str(),
        "\u{2580}"
    );
}

#[test]
fn encoded_images_can_carry_pixels_as_a_fallback() {
    let pixels = Pixels::from_rgba(1, 2, banded(1, 2, [1, 2, 3, 255], [4, 5, 6, 255]))
        .expect("valid pixels");
    let img = Image::new(vec![0x89, 0x50], "image/png")
        .pixels(pixels)
        .protocol(ImageProtocol::HalfBlock);

    let mut buf = Buffer::empty(Rect::new(0, 0, 1, 1));
    img.render(Rect::new(0, 0, 1, 1), &mut buf);
    assert_eq!(
        buf.cell(Position::new(0, 0)).unwrap().fg,
        Color::Rgb(1, 2, 3)
    );
}

#[test]
fn an_empty_area_renders_nothing_and_does_not_panic() {
    let mut buf = Buffer::empty(Rect::new(0, 0, 4, 2));
    let img = Image::from_rgba(4, 4, solid(4, 4, [1, 1, 1, 255])).expect("valid image");
    img.render(Rect::new(0, 0, 0, 0), &mut buf);
    assert_eq!(buf.cell(Position::new(0, 0)).unwrap().symbol.as_str(), " ");
}

// ── Sixel ────────────────────────────────────────────────────────────────────

#[test]
fn sixel_protocol_writes_a_sequence_into_the_first_cell() {
    let mut buf = Buffer::empty(Rect::new(0, 0, 8, 4));
    let img = Image::from_rgba(8, 12, solid(8, 12, [0, 128, 255, 255]))
        .expect("valid image")
        .protocol(ImageProtocol::Sixel);
    img.render(Rect::new(0, 0, 8, 4), &mut buf);

    let seq = buf
        .cell(Position::new(0, 0))
        .unwrap()
        .symbol
        .as_str()
        .to_string();
    assert!(seq.starts_with("\x1bP"), "not a sixel sequence: {seq:?}");
    assert!(seq.ends_with("\x1b\\"), "sequence not terminated");
    assert!(seq.contains("\"1;1;8;12"), "raster attributes missing");
}

#[test]
fn pixels_expose_their_own_sixel_encoding() {
    let pixels = Pixels::from_rgba(4, 6, solid(4, 6, [255, 255, 0, 255])).expect("valid pixels");
    let seq = pixels.to_sixel().expect("encodes");
    assert!(
        seq.contains(";2;100;100;0"),
        "yellow not in palette: {seq:?}"
    );
}

#[test]
fn sixel_without_pixel_data_renders_nothing() {
    let mut buf = Buffer::empty(Rect::new(0, 0, 8, 4));
    let img = Image::new(vec![0x89, 0x50], "image/png").protocol(ImageProtocol::Sixel);
    img.render(Rect::new(0, 0, 8, 4), &mut buf);
    assert_eq!(buf.cell(Position::new(0, 0)).unwrap().symbol.as_str(), " ");
}
