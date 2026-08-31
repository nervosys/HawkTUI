//! OSC 8 hyperlink support: attachment, diffing, compositing, and emission.

use hawktui::backend::crossterm_backend::CrosstermBackend;
use hawktui::backend::Backend;
use hawktui::core::buffer::Buffer;
use hawktui::core::rect::Rect;
use hawktui::core::style::Style;

const URL: &str = "https://example.com/docs";

fn area() -> Rect {
    Rect::new(0, 0, 30, 3)
}

/// Emit the diff between two buffers, hyperlinks included.
fn emit(front: &Buffer, back: &Buffer) -> String {
    let changes = front.diff(back);
    let linked = back.attach_hyperlinks(&changes);
    let mut sink: Vec<u8> = Vec::new();
    {
        let mut backend = CrosstermBackend::new(&mut sink);
        backend.draw_linked(linked.into_iter()).unwrap();
    }
    String::from_utf8(sink).unwrap()
}

#[test]
fn a_buffer_without_links_reports_none() {
    let mut b = Buffer::empty(area());
    b.set_string(0, 0, "plain", Style::default());
    assert!(!b.has_hyperlinks());
    assert_eq!(b.hyperlink_at(0, 0), None);
}

#[test]
fn linked_text_reports_its_target_per_cell() {
    let mut b = Buffer::empty(area());
    b.set_string_linked(2, 1, "docs", Style::default(), URL);

    assert!(b.has_hyperlinks());
    for x in 2..6 {
        assert_eq!(b.hyperlink_at(x, 1), Some(URL), "cell {x} lost its link");
    }
    assert_eq!(b.hyperlink_at(1, 1), None, "link leaked left");
    assert_eq!(b.hyperlink_at(6, 1), None, "link leaked right");
    assert_eq!(b.hyperlink_at(2, 0), None, "link leaked to another row");
}

#[test]
fn links_are_emitted_around_the_text_they_cover() {
    let front = Buffer::empty(area());
    let mut back = Buffer::empty(area());
    back.set_string_linked(0, 0, "docs", Style::default(), URL);

    let out = emit(&front, &back);
    let open = format!("\x1b]8;;{URL}\x1b\\");
    assert!(out.contains(&open), "no opening sequence: {out:?}");
    assert!(out.contains("docs"), "text missing: {out:?}");
    // Opened once for the run, and closed before the stream ends.
    assert_eq!(out.matches(&open).count(), 1, "link re-opened: {out:?}");
    assert!(out.contains("\x1b]8;;\x1b\\"), "never closed: {out:?}");
}

#[test]
fn adjacent_different_links_each_get_their_own_sequence() {
    let front = Buffer::empty(area());
    let mut back = Buffer::empty(area());
    back.set_string_linked(0, 0, "aa", Style::default(), "https://a.example");
    back.set_string_linked(2, 0, "bb", Style::default(), "https://b.example");

    let out = emit(&front, &back);
    assert!(out.contains("\x1b]8;;https://a.example\x1b\\"));
    assert!(out.contains("\x1b]8;;https://b.example\x1b\\"));
}

#[test]
fn unlinked_text_after_a_link_closes_it() {
    let front = Buffer::empty(area());
    let mut back = Buffer::empty(area());
    back.set_string_linked(0, 0, "link", Style::default(), URL);
    back.set_string(4, 0, "plain", Style::default());

    let out = emit(&front, &back);
    let close_index = out.find("\x1b]8;;\x1b\\").expect("link never closed");
    let plain_index = out.find("plain").expect("plain text missing");
    assert!(
        close_index < plain_index,
        "plain text was swallowed by the link: {out:?}"
    );
}

#[test]
fn changing_only_the_link_still_redraws_the_cells() {
    let mut front = Buffer::empty(area());
    front.set_string_linked(0, 0, "docs", Style::default(), URL);
    let mut back = Buffer::empty(area());
    back.set_string_linked(0, 0, "docs", Style::default(), "https://example.com/other");

    // Same glyphs, same style — only the target moved.
    assert_eq!(front.diff(&back).len(), 4, "link change went unnoticed");
}

#[test]
fn removing_a_link_redraws_the_cells() {
    let mut front = Buffer::empty(area());
    front.set_string_linked(0, 0, "docs", Style::default(), URL);
    let mut back = Buffer::empty(area());
    back.set_string(0, 0, "docs", Style::default());

    assert_eq!(front.diff(&back).len(), 4);
}

#[test]
fn identical_linked_frames_produce_no_changes() {
    let mut front = Buffer::empty(area());
    front.set_string_linked(0, 0, "docs", Style::default(), URL);
    let mut back = Buffer::empty(area());
    back.set_string_linked(0, 0, "docs", Style::default(), URL);

    assert!(front.diff(&back).is_empty());
}

#[test]
fn reset_clears_links_with_content() {
    let mut b = Buffer::empty(area());
    b.set_string_linked(0, 0, "docs", Style::default(), URL);
    b.reset();
    assert!(!b.has_hyperlinks());
    assert_eq!(b.hyperlink_at(0, 0), None);
}

#[test]
fn merging_an_overlay_carries_its_links() {
    let mut base = Buffer::empty(area());
    base.set_string(0, 0, "..............", Style::default());

    let mut overlay = Buffer::empty(Rect::new(2, 0, 4, 1));
    overlay.set_string_linked(2, 0, "docs", Style::default(), URL);

    base.merge(&overlay);
    assert_eq!(base.hyperlink_at(2, 0), Some(URL));
    assert_eq!(base.hyperlink_at(5, 0), Some(URL));
    assert_eq!(base.hyperlink_at(6, 0), None);
}

#[test]
fn links_out_of_bounds_are_ignored() {
    let mut b = Buffer::empty(area());
    b.set_hyperlink(0, 9, 4, URL);
    b.set_hyperlink(0, 0, 0, URL);
    b.set_hyperlink(0, 0, 4, "");
    assert!(!b.has_hyperlinks());
}

#[test]
fn a_link_never_runs_past_the_end_of_its_row() {
    let mut b = Buffer::empty(area());
    b.set_hyperlink(28, 0, 10, URL);
    assert_eq!(b.hyperlink_at(29, 0), Some(URL));
    // Row 1 starts where row 0 ends; the link must not bleed into it.
    assert_eq!(b.hyperlink_at(0, 1), None);
}
