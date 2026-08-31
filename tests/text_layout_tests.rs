//! Text placement cases where the ASCII fast path must not cut corners:
//! combining marks, wide graphemes, mixed scripts, and truncation boundaries.

use hawktui::core::buffer::Buffer;
use hawktui::core::rect::{Position, Rect};
use hawktui::core::style::{Color, Style};

fn buf(width: u16) -> Buffer {
    Buffer::empty(Rect::new(0, 0, width, 1))
}

/// The symbols of row 0, continuation cells included as empty strings.
fn cells(b: &Buffer, count: u16) -> Vec<String> {
    (0..count)
        .map(|x| {
            b.cell(Position::new(x, 0))
                .map(|c| c.symbol.as_str().to_string())
                .unwrap_or_default()
        })
        .collect()
}

#[test]
fn ascii_is_one_cell_per_byte() {
    let mut b = buf(10);
    let used = b.set_string(0, 0, "hello", Style::default());
    assert_eq!(used, 5);
    assert_eq!(cells(&b, 6), ["h", "e", "l", "l", "o", " "]);
}

#[test]
fn combining_mark_stays_with_its_base_character() {
    // "e" + U+0301 is one grapheme cluster occupying one cell — the ASCII run
    // must not claim the "e" and orphan the accent.
    let mut b = buf(10);
    let used = b.set_string(0, 0, "cafe\u{0301}!", Style::default());
    assert_eq!(used, 5, "café! is five columns");
    assert_eq!(cells(&b, 6), ["c", "a", "f", "e\u{0301}", "!", " "]);
}

#[test]
fn wide_graphemes_claim_two_cells() {
    let mut b = buf(10);
    let used = b.set_string(0, 0, "日本", Style::default());
    assert_eq!(used, 4);
    assert_eq!(cells(&b, 5), ["日", "", "本", "", " "]);
}

#[test]
fn mixed_ascii_and_wide_text_lands_in_the_right_columns() {
    let mut b = buf(20);
    let used = b.set_string(0, 0, "ab日cd", Style::default());
    assert_eq!(used, 6);
    assert_eq!(cells(&b, 7), ["a", "b", "日", "", "c", "d", " "]);
}

#[test]
fn emoji_round_trips_through_the_cell() {
    let mut b = buf(10);
    b.set_string(0, 0, "x🦀y", Style::default());
    let row = cells(&b, 5);
    assert_eq!(row[0], "x");
    assert_eq!(row[1], "🦀");
    assert_eq!(row[2], "", "wide emoji owns a continuation cell");
    assert_eq!(row[3], "y");
}

#[test]
fn long_cluster_survives_placement() {
    let family = "👨‍👩‍👧‍👦";
    let mut b = buf(10);
    b.set_string(0, 0, family, Style::default());
    assert_eq!(b.cell(Position::new(0, 0)).unwrap().symbol.as_str(), family);
}

#[test]
fn truncation_stops_at_the_limit_without_splitting_a_wide_grapheme() {
    let mut b = buf(10);
    // Three columns available, "日" needs two: it fits, the next one does not.
    let used = b.set_string_truncated(0, 0, "日本語", 3, Style::default());
    assert_eq!(used, 2, "a wide grapheme is never half-written");
    assert_eq!(cells(&b, 4), ["日", "", " ", " "]);
}

#[test]
fn ascii_truncation_stops_at_the_limit() {
    let mut b = buf(10);
    let used = b.set_string_truncated(0, 0, "abcdefgh", 4, Style::default());
    assert_eq!(used, 4);
    assert_eq!(cells(&b, 5), ["a", "b", "c", "d", " "]);
}

#[test]
fn writing_past_the_right_edge_is_clipped_not_wrapped() {
    let mut b = buf(5);
    let used = b.set_string(3, 0, "abcdef", Style::default());
    assert_eq!(used, 2);
    assert_eq!(cells(&b, 5), [" ", " ", " ", "a", "b"]);
}

#[test]
fn style_applies_to_every_cell_including_continuations() {
    let mut b = buf(10);
    let style = Style::default().fg(Color::Red);
    b.set_string(0, 0, "a日", style);
    for x in 0..3 {
        assert_eq!(
            b.cell(Position::new(x, 0)).unwrap().fg,
            Color::Red,
            "cell {x} lost its style"
        );
    }
}

#[test]
fn out_of_bounds_row_writes_nothing() {
    let mut b = buf(10);
    assert_eq!(b.set_string(0, 5, "hello", Style::default()), 0);
}

#[test]
fn mixed_script_line_matches_column_by_column() {
    // Latin, CJK, combining, and emoji in one string.
    let mut b = buf(30);
    let used = b.set_string(0, 0, "id: 42 名前 e\u{0301} 🦀", Style::default());
    let expected = [
        "i",
        "d",
        ":",
        " ",
        "4",
        "2",
        " ",
        "名",
        "",
        "前",
        "",
        " ",
        "e\u{0301}",
        " ",
        "🦀",
        "",
    ];
    assert_eq!(cells(&b, expected.len() as u16), expected);
    assert_eq!(used as usize, expected.len());
}

#[test]
fn alternating_ascii_and_unicode_runs_all_land() {
    // Exercises the hand-off between the ASCII fast path and the segmenter in
    // both directions, repeatedly, including runs shorter than the threshold.
    let mut b = Buffer::empty(Rect::new(0, 0, 40, 1));
    let used = b.set_string(0, 0, "ab日cdef語g hi語", Style::default());
    let expected = [
        "a", "b", "日", "", "c", "d", "e", "f", "語", "", "g", " ", "h", "i", "語", "",
    ];
    assert_eq!(cells(&b, expected.len() as u16), expected);
    assert_eq!(used as usize, expected.len());
}

#[test]
fn a_long_ascii_tail_after_unicode_is_not_dropped() {
    let mut b = Buffer::empty(Rect::new(0, 0, 40, 1));
    let used = b.set_string(0, 0, "日 the quick brown fox", Style::default());
    assert_eq!(used, 22);
    assert_eq!(
        cells(&b, 22).join(""),
        "日 the quick brown fox",
        "ASCII tail lost after the Unicode stretch"
    );
}

// ── Cases the scalar fast path must refuse to take ───────────────────────────

#[test]
fn hangul_syllable_with_trailing_jamo_stays_one_cluster() {
    // U+AC00 (가) followed by a conjoining trailing jamo composes into one
    // cluster; splitting them would render two glyphs.
    let mut b = Buffer::empty(Rect::new(0, 0, 10, 1));
    let text = "\u{AC00}\u{11A8}";
    b.set_string(0, 0, text, Style::default());
    assert_eq!(
        b.cell(Position::new(0, 0)).unwrap().symbol.as_str(),
        text,
        "jamo was split from its syllable"
    );
}

#[test]
fn regional_indicator_pair_is_one_flag() {
    let mut b = Buffer::empty(Rect::new(0, 0, 10, 1));
    let flag = "\u{1F1EF}\u{1F1F5}"; // 🇯🇵
    b.set_string(0, 0, flag, Style::default());
    assert_eq!(
        b.cell(Position::new(0, 0)).unwrap().symbol.as_str(),
        flag,
        "flag split into two regional indicators"
    );
}

#[test]
fn variation_selector_stays_with_its_base() {
    let mut b = Buffer::empty(Rect::new(0, 0, 10, 1));
    // A dingbat plus VS16 asks for emoji presentation; they are one cluster.
    let text = "\u{2764}\u{FE0F}";
    b.set_string(0, 0, text, Style::default());
    assert_eq!(
        b.cell(Position::new(0, 0)).unwrap().symbol.as_str(),
        text,
        "variation selector was orphaned"
    );
}

#[test]
fn kana_voiced_sound_mark_stays_with_its_base() {
    // U+304B + U+3099 (か + combining voiced mark) is one cluster.
    let mut b = Buffer::empty(Rect::new(0, 0, 10, 1));
    let text = "\u{304B}\u{3099}";
    b.set_string(0, 0, text, Style::default());
    assert_eq!(
        b.cell(Position::new(0, 0)).unwrap().symbol.as_str(),
        text,
        "voiced sound mark was split from its kana"
    );
}

#[test]
fn devanagari_cluster_is_not_split() {
    // क + ि — a consonant with a dependent vowel sign.
    let mut b = Buffer::empty(Rect::new(0, 0, 10, 1));
    let text = "\u{0915}\u{093F}";
    b.set_string(0, 0, text, Style::default());
    assert_eq!(
        b.cell(Position::new(0, 0)).unwrap().symbol.as_str(),
        text,
        "Devanagari cluster was split"
    );
}

#[test]
fn plain_cjk_and_kana_land_one_glyph_per_two_columns() {
    let mut b = Buffer::empty(Rect::new(0, 0, 20, 1));
    let used = b.set_string(0, 0, "漢字かなガナ", Style::default());
    assert_eq!(used, 12);
    assert_eq!(
        cells(&b, 12),
        ["漢", "", "字", "", "か", "", "な", "", "ガ", "", "ナ", ""]
    );
}

#[test]
fn greek_and_cyrillic_are_single_columns() {
    let mut b = Buffer::empty(Rect::new(0, 0, 20, 1));
    let used = b.set_string(0, 0, "αβγ дом", Style::default());
    assert_eq!(used, 7);
    assert_eq!(cells(&b, 7), ["α", "β", "γ", " ", "д", "о", "м"]);
}

#[test]
fn combining_accent_after_greek_is_not_split() {
    let mut b = Buffer::empty(Rect::new(0, 0, 10, 1));
    let text = "\u{03B1}\u{0301}"; // α + acute
    let used = b.set_string(0, 0, text, Style::default());
    assert_eq!(used, 1);
    assert_eq!(b.cell(Position::new(0, 0)).unwrap().symbol.as_str(), text);
}

#[test]
fn fast_and_slow_paths_agree_on_a_torture_string() {
    // Every case that decides which path is taken, in one line.
    let text = "ab αβ 漢字 e\u{0301} \u{AC00}\u{11A8} 🦀 \u{1F1EF}\u{1F1F5} ok";
    let mut wide = Buffer::empty(Rect::new(0, 0, 60, 1));
    let used = wide.set_string(0, 0, text, Style::default());

    // Writing the same text one grapheme at a time must produce the same cells.
    let mut reference = Buffer::empty(Rect::new(0, 0, 60, 1));
    let mut col = 0u16;
    for g in unicode_segmentation::UnicodeSegmentation::graphemes(text, true) {
        col += reference.set_string(col, 0, g, Style::default());
    }

    assert_eq!(used, col, "column count differs between paths");
    assert_eq!(
        cells(&wide, 40),
        cells(&reference, 40),
        "bulk write disagrees with per-grapheme write"
    );
}

#[test]
fn fast_path_widths_agree_with_the_unicode_tables() {
    // Sweep every code point the scalar fast path might classify itself and
    // check the column count against unicode-width's answer.
    use unicode_width::UnicodeWidthChar;

    let ranges = [
        (0x20u32, 0x7Eu32),
        (0xA1, 0x2FF),
        (0x370, 0x482),
        (0x2010, 0x2027),
        (0x2030, 0x205E),
        (0x2070, 0x209F),
        (0x2190, 0x2BFF),
        (0x3001, 0x3029),
        (0x303B, 0x303F),
        (0x3041, 0x3098),
        (0x309B, 0x30FF),
        (0x3400, 0x3500),
        (0x4E00, 0x4F00),
        (0xAC00, 0xAD00),
        (0xF900, 0xFA00),
        (0xFF01, 0xFF60),
    ];

    let mut checked = 0usize;
    for (lo, hi) in ranges {
        for cp in lo..=hi {
            let Some(ch) = char::from_u32(cp) else {
                continue;
            };
            let Some(expected) = UnicodeWidthChar::width(ch) else {
                continue;
            };
            if expected == 0 {
                continue;
            }
            // Two of the same character, so the fast path's "next is also
            // standalone" precondition holds for the first one.
            let mut b = Buffer::empty(Rect::new(0, 0, 8, 1));
            let used = b.set_string(0, 0, &format!("{ch}{ch}"), Style::default());
            assert_eq!(
                used as usize,
                expected * 2,
                "width mismatch for U+{cp:04X} ({ch:?}): wrote {used} columns, tables say {}",
                expected * 2
            );
            assert_eq!(
                b.cell(Position::new(0, 0)).unwrap().symbol.as_str(),
                ch.to_string(),
                "wrong glyph stored for U+{cp:04X}"
            );
            checked += 1;
        }
    }
    assert!(checked > 2000, "sweep covered only {checked} code points");
}
