//! Geographic paths on a canvas: data loading, projection, and decimation.

use hawktui::core::buffer::Buffer;
use hawktui::core::rect::Rect;
use hawktui::core::style::Color;
use hawktui::widget::canvas::{Canvas, CanvasMap, MapData, MapResolution};
use hawktui::widget::Widget;

/// Cells that ended up with a braille glyph on them.
fn painted(buf: &Buffer) -> usize {
    let area = buf.area;
    let mut count = 0;
    for y in area.y..area.bottom() {
        for x in area.x..area.right() {
            let s = buf[(x, y)].symbol();
            if !s.is_empty() && s != " " {
                count += 1;
            }
        }
    }
    count
}

fn render(map: CanvasMap) -> Buffer {
    let area = Rect::new(0, 0, 40, 12);
    let mut buf = Buffer::empty(area);
    Canvas::new().geographic().map(map).render(area, &mut buf);
    buf
}

#[test]
fn paths_are_collected_and_counted() {
    let data = MapData::new()
        .path([(0.0, 0.0), (10.0, 10.0)])
        .path([(20.0, 20.0)]);
    assert_eq!(data.paths().len(), 2);
    assert_eq!(data.point_count(), 3);
    assert!(!data.is_empty());
}

#[test]
fn empty_paths_are_dropped_rather_than_stored() {
    let data = MapData::new().path(Vec::<(f64, f64)>::new());
    assert!(data.is_empty());
    assert_eq!(
        MapData::from_paths(vec![vec![], vec![(1.0, 2.0)]])
            .paths()
            .len(),
        1
    );
}

#[test]
fn geojson_line_strings_become_paths() {
    let source = r#"{
        "type": "LineString",
        "coordinates": [[0, 0], [1, 1], [2, 2]]
    }"#;
    let data = MapData::from_geojson(source).unwrap();
    assert_eq!(data.paths().len(), 1);
    assert_eq!(data.paths()[0], vec![(0.0, 0.0), (1.0, 1.0), (2.0, 2.0)]);
}

#[test]
fn geojson_polygons_multi_polygons_and_feature_collections_all_load() {
    let source = r#"{
      "type": "FeatureCollection",
      "features": [
        {
          "type": "Feature",
          "properties": {"name": "ignored"},
          "geometry": {
            "type": "Polygon",
            "coordinates": [[[0, 0], [1, 0], [1, 1], [0, 0]]]
          }
        },
        {
          "type": "Feature",
          "geometry": {
            "type": "MultiPolygon",
            "coordinates": [
              [[[10, 10], [11, 10], [10, 11], [10, 10]]],
              [[[20, 20], [21, 20], [20, 21], [20, 20]]]
            ]
          }
        },
        {
          "type": "Feature",
          "geometry": {
            "type": "MultiLineString",
            "coordinates": [[[30, 30], [31, 31]], [[40, 40], [41, 41]]]
          }
        }
      ]
    }"#;
    let data = MapData::from_geojson(source).unwrap();
    // One polygon ring, two multi-polygon rings, two line strings.
    assert_eq!(data.paths().len(), 5);
    assert_eq!(data.point_count(), 4 + 4 + 4 + 2 + 2);
}

#[test]
fn geojson_points_and_unknown_geometries_are_skipped() {
    let source = r#"{
      "type": "GeometryCollection",
      "geometries": [
        {"type": "Point", "coordinates": [1, 2]},
        {"type": "Sphere"},
        {"type": "LineString", "coordinates": [[0, 0], [1, 1]]}
      ]
    }"#;
    let data = MapData::from_geojson(source).unwrap();
    assert_eq!(data.paths().len(), 1);
}

#[test]
fn malformed_json_is_reported_rather_than_panicking() {
    assert!(MapData::from_geojson("{not json").is_none());
    // Valid JSON with no geometry simply yields nothing to draw.
    assert!(MapData::from_geojson("[1, 2, 3]").unwrap().is_empty());
}

#[test]
fn coordinates_with_a_third_element_keep_their_longitude_and_latitude() {
    let data = MapData::from_geojson(r#"{"type":"LineString","coordinates":[[5,6,700]]}"#).unwrap();
    assert_eq!(data.paths()[0], vec![(5.0, 6.0)]);
}

#[test]
fn a_map_paints_where_its_coordinates_say_it_should() {
    // A short meridian segment near the prime meridian lands in the middle of
    // a whole-world canvas, not at an edge.
    let data = MapData::new().path([(0.0, -20.0), (0.0, 20.0)]);
    let buf = render(CanvasMap::new(data).resolution(MapResolution::High));
    assert!(painted(&buf) > 0);

    let mid_column = 20u16;
    let mut column_hits = 0;
    for y in 0..12u16 {
        let s = buf[(mid_column, y)].symbol();
        if !s.is_empty() && s != " " {
            column_hits += 1;
        }
    }
    assert!(
        column_hits >= 2,
        "a meridian should paint a vertical run in the middle column"
    );
    // Nothing at the far west edge.
    assert!((0..12u16).all(|y| {
        let s = buf[(0, y)].symbol();
        s.is_empty() || s == " "
    }));
}

#[test]
fn low_resolution_draws_fewer_dots_than_high_when_unconnected() {
    // One vertex per braille dot column across the whole world, so dropping
    // three vertices in four leaves visible gaps.
    let dense: Vec<(f64, f64)> = (0..80).map(|i| (-180.0 + i as f64 * 4.5, 0.0)).collect();
    let data = MapData::from_paths(vec![dense]);

    let high = render(
        CanvasMap::new(data.clone())
            .resolution(MapResolution::High)
            .connect(false),
    );
    let low = render(
        CanvasMap::new(data)
            .resolution(MapResolution::Low)
            .connect(false),
    );
    assert!(
        painted(&low) < painted(&high),
        "low resolution painted {} cells, high painted {}",
        painted(&low),
        painted(&high)
    );
}

#[test]
fn a_connected_low_resolution_map_still_covers_the_whole_path() {
    let dense: Vec<(f64, f64)> = (0..40).map(|i| (i as f64 * 4.0 - 80.0, 0.0)).collect();
    let data = MapData::from_paths(vec![dense]);
    let low = render(CanvasMap::new(data.clone()).resolution(MapResolution::Low));
    let high = render(CanvasMap::new(data).resolution(MapResolution::High));
    // Decimation removes vertices, not coverage: the lines between the kept
    // vertices fill the same cells.
    assert_eq!(painted(&low), painted(&high));
}

#[test]
fn a_seam_across_the_antimeridian_is_not_drawn_as_a_segment() {
    // Two points on either side of the date line. Connecting them naively
    // would stripe the entire map.
    let data = MapData::new().path([(179.0, 0.0), (-179.0, 0.0)]);
    let buf = render(CanvasMap::new(data).resolution(MapResolution::High));
    // Only the two endpoints, both near the edges — the middle stays clear.
    for x in 5..35u16 {
        for y in 0..12u16 {
            let s = buf[(x, y)].symbol();
            assert!(
                s.is_empty() || s == " ",
                "the seam was drawn across the map at ({x}, {y})"
            );
        }
    }
}

#[test]
fn map_color_reaches_the_cells() {
    let data = MapData::new().path([(-100.0, 0.0), (100.0, 0.0)]);
    let buf = render(CanvasMap::new(data).color(Color::LightRed));
    let mut found = false;
    for y in 0..12u16 {
        for x in 0..40u16 {
            let s = buf[(x, y)].symbol();
            if !s.is_empty() && s != " " {
                assert_eq!(buf[(x, y)].fg, Color::LightRed);
                found = true;
            }
        }
    }
    assert!(found, "the map painted nothing");
}

#[test]
fn an_empty_dataset_renders_an_empty_canvas() {
    let buf = render(CanvasMap::new(MapData::new()));
    assert_eq!(painted(&buf), 0);
}

#[test]
fn non_finite_coordinates_are_skipped_without_panicking() {
    let data = MapData::new().path([
        (0.0, 0.0),
        (f64::NAN, 10.0),
        (f64::INFINITY, 20.0),
        (30.0, 30.0),
    ]);
    let buf = render(CanvasMap::new(data).resolution(MapResolution::High));
    assert!(painted(&buf) > 0);
}
