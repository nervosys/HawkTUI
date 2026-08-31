//! Behavior of the widget options added for parity with other frameworks:
//! sparkline direction and list scroll padding.

use hawktui::core::buffer::Buffer;
use hawktui::core::rect::{Position, Rect};
use hawktui::core::style::Style;
use hawktui::widget::list::{List, ListItem, ListState};
use hawktui::widget::sparkline::{Sparkline, SparklineDirection};
use hawktui::widget::{StatefulWidget, Widget};

fn row_text(buf: &Buffer, y: u16, width: u16) -> String {
    (0..width)
        .map(|x| {
            buf.cell(Position::new(x, y))
                .map(|c| c.symbol.as_str().to_string())
                .unwrap_or_default()
        })
        .collect()
}

#[test]
fn sparkline_defaults_to_oldest_sample_on_the_left() {
    let area = Rect::new(0, 0, 4, 1);
    let mut buf = Buffer::empty(area);
    // A rising ramp: the tallest bar belongs on the right.
    Sparkline::new(vec![0u64, 1, 2, 8]).render(area, &mut buf);

    let left = buf
        .cell(Position::new(0, 0))
        .unwrap()
        .symbol
        .as_str()
        .to_string();
    let right = buf
        .cell(Position::new(3, 0))
        .unwrap()
        .symbol
        .as_str()
        .to_string();
    assert_ne!(left, right, "ramp rendered flat: {left:?} vs {right:?}");
    assert_eq!(
        right, "\u{2588}",
        "newest (largest) sample should be full height"
    );
}

#[test]
fn sparkline_right_to_left_mirrors_the_series() {
    let area = Rect::new(0, 0, 4, 1);
    let mut ltr = Buffer::empty(area);
    Sparkline::new(vec![0u64, 1, 2, 8]).render(area, &mut ltr);

    let mut rtl = Buffer::empty(area);
    Sparkline::new(vec![0u64, 1, 2, 8])
        .direction(SparklineDirection::RightToLeft)
        .render(area, &mut rtl);

    let ltr_row: Vec<String> = (0..4)
        .map(|x| {
            ltr.cell(Position::new(x, 0))
                .unwrap()
                .symbol
                .as_str()
                .to_string()
        })
        .collect();
    let mut rtl_row: Vec<String> = (0..4)
        .map(|x| {
            rtl.cell(Position::new(x, 0))
                .unwrap()
                .symbol
                .as_str()
                .to_string()
        })
        .collect();
    rtl_row.reverse();
    assert_eq!(ltr_row, rtl_row, "right-to-left is not the mirror image");
}

/// A list of `count` numbered items rendered into `height` rows.
fn render_list(count: usize, height: u16, selected: usize, padding: usize) -> ListState {
    let area = Rect::new(0, 0, 12, height);
    let mut buf = Buffer::empty(area);
    let items: Vec<ListItem> = (0..count)
        .map(|i| ListItem::new(format!("item{i:02}")))
        .collect();
    let mut state = ListState {
        selected: Some(selected),
        ..ListState::default()
    };
    StatefulWidget::render(
        List::new(items).scroll_padding(padding),
        area,
        &mut buf,
        &mut state,
    );
    state
}

#[test]
fn without_padding_the_selection_may_sit_on_the_edge() {
    let state = render_list(50, 5, 10, 0);
    // Selection lands on the last visible row.
    assert_eq!(state.offset, 6);
}

#[test]
fn scroll_padding_keeps_rows_visible_below_the_selection() {
    let state = render_list(50, 5, 10, 2);
    // Two rows of context after the selection: offset shifts up accordingly.
    assert_eq!(state.offset, 8);
}

#[test]
fn scroll_padding_keeps_rows_visible_above_the_selection() {
    let area = Rect::new(0, 0, 12, 5);
    let mut buf = Buffer::empty(area);
    let items: Vec<ListItem> = (0..50)
        .map(|i| ListItem::new(format!("item{i:02}")))
        .collect();
    let mut state = ListState {
        offset: 20,
        selected: Some(21),
    };
    StatefulWidget::render(
        List::new(items).scroll_padding(2),
        area,
        &mut buf,
        &mut state,
    );
    assert_eq!(state.offset, 19, "selection should keep two rows above it");
}

#[test]
fn scroll_padding_never_exceeds_the_viewport() {
    // Padding larger than the visible height must not wedge the offset.
    let state = render_list(50, 3, 25, 99);
    assert!(state.offset <= 25, "offset ran away: {}", state.offset);
    assert!(state.offset + 3 > 25, "selection scrolled out of view");
}

#[test]
fn scroll_padding_does_not_scroll_past_the_last_item() {
    let state = render_list(10, 5, 9, 3);
    assert_eq!(state.offset, 5, "offset must stop at the final page");
}

#[test]
fn selection_stays_rendered_with_padding_applied() {
    let area = Rect::new(0, 0, 12, 5);
    let mut buf = Buffer::empty(area);
    let items: Vec<ListItem> = (0..50)
        .map(|i| ListItem::new(format!("item{i:02}")))
        .collect();
    let mut state = ListState {
        selected: Some(10),
        ..ListState::default()
    };
    StatefulWidget::render(
        List::new(items).scroll_padding(2),
        area,
        &mut buf,
        &mut state,
    );

    let rows: Vec<String> = (0..5).map(|y| row_text(&buf, y, 12)).collect();
    assert!(
        rows.iter().any(|r| r.starts_with("item10")),
        "selected item is not on screen: {rows:?}"
    );
}

#[test]
fn a_short_list_never_scrolls() {
    let state = render_list(3, 10, 2, 2);
    assert_eq!(state.offset, 0);
}

#[test]
fn sparkline_direction_mirrors_within_a_block_border() {
    use hawktui::widget::block::{Block, Borders};
    let area = Rect::new(0, 0, 6, 3);
    // Series runs oldest → newest: 8 is the oldest sample, 1 the newest.
    let data = vec![8u64, 1, 0, 1];

    let mut buf = Buffer::empty(area);
    Sparkline::new(data.clone())
        .direction(SparklineDirection::RightToLeft)
        .block(Block::default().borders(Borders::ALL))
        .render(area, &mut buf);

    // The inner area is columns 1..5. Reversed, the oldest (tallest) sample
    // lands on its right edge, and the mirroring respects the border.
    assert_eq!(
        buf.cell(Position::new(4, 1)).unwrap().symbol.as_str(),
        "\u{2588}",
        "oldest, tallest sample should be at the right edge of the inner area"
    );
    assert_eq!(
        buf.cell(Position::new(0, 1)).unwrap().symbol.as_str(),
        "\u{2502}",
        "the border must not be overwritten"
    );
    let _ = Style::default();
}

#[test]
fn layout_areas_destructures_into_named_regions() {
    use hawktui::layout::{Constraint, Layout};

    let [header, body, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(Rect::new(0, 0, 80, 24));

    assert_eq!(header.height, 3);
    assert_eq!(body.height, 20);
    assert_eq!(footer.y, 23);
    assert_eq!(header.width, 80);
}

#[test]
fn layout_areas_agrees_with_split() {
    use hawktui::layout::{Constraint, Layout};

    let area = Rect::new(2, 1, 40, 12);
    let layout = Layout::horizontal([Constraint::Percentage(30), Constraint::Fill(1)]);
    let split = layout.split(area);
    let [left, right] = layout.areas(area);
    assert_eq!(left, split[0]);
    assert_eq!(right, split[1]);
}

#[test]
fn asking_for_more_areas_than_exist_does_not_panic() {
    use hawktui::layout::{Constraint, Layout};

    let [a, b, c, d] =
        Layout::vertical([Constraint::Length(2), Constraint::Min(0)]).areas(Rect::new(0, 0, 10, 6));
    assert_eq!(a.height, 2);
    assert_eq!(b.height, 4);
    assert_eq!(c.height, 0, "surplus areas are empty");
    assert_eq!(d.height, 0);
}
