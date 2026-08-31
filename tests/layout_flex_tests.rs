//! Flex distribution and the layout shorthands.

use hawktui::core::rect::Rect;
use hawktui::layout::{Constraint, Flex, Layout, Margin};

const AREA: Rect = Rect {
    x: 0,
    y: 0,
    width: 30,
    height: 10,
};

fn xs(rects: &[Rect]) -> Vec<(u16, u16)> {
    rects.iter().map(|r| (r.x, r.width)).collect()
}

#[test]
fn space_between_puts_all_slack_between_the_segments() {
    let layout = Layout::horizontal([Constraint::Length(4); 3]).flex(Flex::SpaceBetween);
    let rects = layout.split(AREA);
    // 30 - 12 = 18 of slack over two gaps.
    assert_eq!(xs(&rects), vec![(0, 4), (13, 4), (26, 4)]);
    // The outer edges stay flush with the area.
    assert_eq!(rects[0].x, AREA.x);
    assert_eq!(rects[2].right(), AREA.right());
}

#[test]
fn space_between_spreads_an_odd_remainder_over_the_leading_gaps() {
    // 30 - 4*3 = 18 of slack over 3 gaps for 4 segments: 6 each, no remainder.
    let even = Layout::horizontal([Constraint::Length(3); 4]).flex(Flex::SpaceBetween);
    assert_eq!(
        xs(&even.split(AREA)),
        vec![(0, 3), (9, 3), (18, 3), (27, 3)]
    );

    // 29 wide: 17 of slack over 3 gaps is 5 remainder 2 — the first two gaps
    // take the extra so the last segment still ends flush.
    let odd_area = Rect::new(0, 0, 29, 10);
    let odd = Layout::horizontal([Constraint::Length(3); 4]).flex(Flex::SpaceBetween);
    let rects = odd.split(odd_area);
    assert_eq!(rects[3].right(), odd_area.right());
    let gaps: Vec<u16> = rects.windows(2).map(|w| w[1].x - w[0].right()).collect();
    assert_eq!(gaps, vec![6, 6, 5]);
}

#[test]
fn space_evenly_makes_the_edges_match_the_inner_gaps() {
    // 30 - 3*6 = 12 of slack over 4 equal gaps.
    let rects = Layout::horizontal([Constraint::Length(6); 3])
        .flex(Flex::SpaceEvenly)
        .split(AREA);
    assert_eq!(rects[0].x - AREA.x, 3);
    assert_eq!(rects[1].x - rects[0].right(), 3);
    assert_eq!(rects[2].x - rects[1].right(), 3);
}

#[test]
fn space_around_gives_the_edges_half_of_an_inner_gap() {
    // 30 - 3*6 = 12 of slack: 2 at each edge, 4 between segments.
    let rects = Layout::horizontal([Constraint::Length(6); 3])
        .flex(Flex::SpaceAround)
        .split(AREA);
    assert_eq!(rects[0].x - AREA.x, 2);
    assert_eq!(rects[1].x - rects[0].right(), 4);
    assert_eq!(rects[2].x - rects[1].right(), 4);
}

#[test]
fn start_center_and_end_still_pack_without_gaps() {
    let sizes = [Constraint::Length(6); 3];
    for flex in [Flex::Start, Flex::Center, Flex::End] {
        let rects = Layout::horizontal(sizes).flex(flex).split(AREA);
        for pair in rects.windows(2) {
            assert_eq!(pair[1].x, pair[0].right(), "{flex:?} should not add gaps");
        }
    }
    assert_eq!(
        Layout::horizontal(sizes).flex(Flex::Start).split(AREA)[0].x,
        0
    );
    assert_eq!(
        Layout::horizontal(sizes).flex(Flex::End).split(AREA)[0].x,
        12
    );
    assert_eq!(
        Layout::horizontal(sizes).flex(Flex::Center).split(AREA)[0].x,
        6
    );
}

#[test]
fn flex_gaps_stack_on_top_of_explicit_spacing() {
    let rects = Layout::horizontal([Constraint::Length(6); 3])
        .flex(Flex::SpaceBetween)
        .gap(2)
        .split(AREA);
    // Spacing eats 4 of the 30 before the slack is computed, leaving 8 over
    // two gaps — 4 each — on top of the 2 of spacing.
    let gaps: Vec<u16> = rects.windows(2).map(|w| w[1].x - w[0].right()).collect();
    assert_eq!(gaps, vec![6, 6]);
    assert_eq!(rects[2].right(), AREA.right());
}

#[test]
fn a_single_segment_is_unaffected_by_any_flex_mode() {
    for flex in [
        Flex::Start,
        Flex::SpaceBetween,
        Flex::SpaceAround,
        Flex::SpaceEvenly,
    ] {
        let rects = Layout::horizontal([Constraint::Length(30)])
            .flex(flex)
            .split(AREA);
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0], AREA);
    }
}

#[test]
fn gap_is_spacing_under_another_name() {
    let with_gap = Layout::vertical([Constraint::Length(2); 3]).gap(1);
    let with_spacing = Layout::vertical([Constraint::Length(2); 3]).spacing(1);
    assert_eq!(*with_gap.split(AREA), *with_spacing.split(AREA));
}

#[test]
fn padding_insets_all_four_sides() {
    let padded = Layout::vertical([Constraint::Min(0)]).padding(2);
    let explicit = Layout::vertical([Constraint::Min(0)]).margin(Margin::uniform(2));
    assert_eq!(*padded.split(AREA), *explicit.split(AREA));
    assert_eq!(padded.split(AREA)[0], Rect::new(2, 2, 26, 6));
}

#[test]
fn horizontal_and_vertical_margins_inset_one_axis_each() {
    let horizontal = Layout::vertical([Constraint::Min(0)]).horizontal_margin(3);
    assert_eq!(horizontal.split(AREA)[0], Rect::new(3, 0, 24, 10));

    let vertical = Layout::vertical([Constraint::Min(0)]).vertical_margin(1);
    assert_eq!(vertical.split(AREA)[0], Rect::new(0, 1, 30, 8));

    // They compose, and neither disturbs the other axis.
    let both = Layout::vertical([Constraint::Min(0)])
        .horizontal_margin(3)
        .vertical_margin(1);
    assert_eq!(both.split(AREA)[0], Rect::new(3, 1, 24, 8));
}

#[test]
fn flex_mode_is_part_of_the_memo_key() {
    let start = Layout::horizontal([Constraint::Length(6); 3]).flex(Flex::Start);
    let between = Layout::horizontal([Constraint::Length(6); 3]).flex(Flex::SpaceBetween);
    let evenly = Layout::horizontal([Constraint::Length(6); 3]).flex(Flex::SpaceEvenly);
    // Solving all three back to back must not serve one from another's entry.
    let a = start.split(AREA);
    let b = between.split(AREA);
    let c = evenly.split(AREA);
    assert_ne!(*a, *b);
    assert_ne!(*b, *c);
    assert_eq!(*start.split(AREA), *a);
}

#[test]
fn segments_never_leave_the_area_in_any_flex_mode() {
    for flex in [
        Flex::Start,
        Flex::Center,
        Flex::End,
        Flex::SpaceBetween,
        Flex::SpaceAround,
        Flex::SpaceEvenly,
    ] {
        for width in 0..40u16 {
            for count in 1..5usize {
                let area = Rect::new(0, 0, width, 4);
                let constraints = vec![Constraint::Length(3); count];
                for rect in Layout::horizontal(constraints)
                    .flex(flex)
                    .split(area)
                    .iter()
                {
                    assert!(
                        rect.right() <= area.right(),
                        "{flex:?} overflowed at width {width} with {count} segments: {rect:?}"
                    );
                    assert!(rect.x >= area.x);
                }
            }
        }
    }
}
