//! End-to-end checks on the escape-sequence stream the terminal actually sees.
//!
//! The unit tests in `backend::ansi` cover each sequence in isolation; these
//! drive a real buffer diff through the crossterm backend and assert on the
//! bytes that come out the other side.

use hawktui::backend::crossterm_backend::CrosstermBackend;
use hawktui::backend::Backend;
use hawktui::core::buffer::Buffer;
use hawktui::core::rect::Rect;
use hawktui::core::style::{Color, Modifier, Style};

/// Render `back` over `front` and return the emitted byte stream.
fn emit(front: &Buffer, back: &Buffer) -> String {
    let changes = front.diff(back);
    let mut sink: Vec<u8> = Vec::new();
    {
        let mut backend = CrosstermBackend::new(&mut sink);
        backend
            .draw(changes.iter().map(|(x, y, c)| (*x, *y, *c)))
            .unwrap();
    }
    String::from_utf8(sink).expect("escape stream must be valid UTF-8")
}

fn area() -> Rect {
    Rect::new(0, 0, 20, 3)
}

#[test]
fn unchanged_frame_emits_only_the_trailing_reset() {
    let front = Buffer::empty(area());
    let back = Buffer::empty(area());
    assert_eq!(emit(&front, &back), "\x1b[0m");
}

#[test]
fn text_is_positioned_once_for_a_contiguous_run() {
    let front = Buffer::empty(area());
    let mut back = Buffer::empty(area());
    back.set_string(2, 1, "hello", Style::default());

    let out = emit(&front, &back);
    // One cursor move for the run, then the characters themselves.
    assert_eq!(
        out.matches("\x1b[").count() - 1,
        3,
        "unexpected stream: {out:?}"
    );
    assert!(out.contains("\x1b[2;3H"), "missing cursor move: {out:?}");
    assert!(out.contains("hello"), "missing text: {out:?}");
}

#[test]
fn separate_runs_each_get_a_cursor_move() {
    let front = Buffer::empty(area());
    let mut back = Buffer::empty(area());
    back.set_string(0, 0, "ab", Style::default());
    back.set_string(10, 2, "cd", Style::default());

    let out = emit(&front, &back);
    assert!(out.contains("\x1b[1;1H"), "missing first move: {out:?}");
    assert!(out.contains("\x1b[3;11H"), "missing second move: {out:?}");
}

#[test]
fn colors_are_set_once_per_run_not_per_cell() {
    let front = Buffer::empty(area());
    let mut back = Buffer::empty(area());
    back.set_string(0, 0, "green", Style::default().fg(Color::Green));

    let out = emit(&front, &back);
    assert_eq!(out.matches("\x1b[32m").count(), 1, "stream: {out:?}");
    assert!(out.contains("green"));
}

#[test]
fn attribute_change_does_not_re_emit_colors() {
    let front = Buffer::empty(area());
    let mut back = Buffer::empty(area());
    let base = Style::default().fg(Color::Red);
    let mut bold = base;
    bold.add_modifier = Modifier::BOLD;
    back.set_string(0, 0, "aa", base);
    back.set_string(2, 0, "bb", bold);
    back.set_string(4, 0, "cc", base);

    let out = emit(&front, &back);
    // The color is asserted once at the start of the run and never re-sent,
    // because turning bold off no longer resets the whole SGR state.
    assert_eq!(out.matches("\x1b[31m").count(), 1, "stream: {out:?}");
    assert!(out.contains("\x1b[1m"), "bold never turned on: {out:?}");
    assert!(out.contains("\x1b[22m"), "bold never turned off: {out:?}");
}

#[test]
fn wide_graphemes_emit_once_and_skip_their_continuation_cell() {
    let front = Buffer::empty(area());
    let mut back = Buffer::empty(area());
    back.set_string(0, 0, "🦀x", Style::default());

    let out = emit(&front, &back);
    assert_eq!(out.matches('🦀').count(), 1, "stream: {out:?}");
    // The continuation cell is skipped, so `x` needs a fresh cursor move.
    assert!(
        out.contains("\x1b[1;3H"),
        "missing move past wide char: {out:?}"
    );
}

#[test]
fn long_grapheme_clusters_survive_the_round_trip() {
    let front = Buffer::empty(area());
    let mut back = Buffer::empty(area());
    let family = "👨‍👩‍👧‍👦";
    back.set_string(0, 0, family, Style::default());

    let out = emit(&front, &back);
    assert!(out.contains(family), "interned cluster lost: {out:?}");
}

#[test]
fn stream_always_ends_with_a_reset() {
    let front = Buffer::empty(area());
    let mut back = Buffer::empty(area());
    back.set_string(0, 0, "x", Style::default().fg(Color::Blue));
    assert!(emit(&front, &back).ends_with("\x1b[0m"));
}
