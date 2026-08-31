//! Direct ANSI/SGR escape-sequence encoding.
//!
//! The draw path is the hottest code in a TUI: it runs once per changed cell,
//! every frame. Going through a generic command layer means a `Display` impl
//! and a formatting machine invocation per escape sequence — several per cell
//! in the worst case.
//!
//! This module encodes the same sequences straight into a byte buffer with
//! hand-rolled integer formatting, so a full frame is one contiguous write and
//! no formatting machinery runs at all.

use crate::core::style::{Color, Modifier};

/// Every modifier that has an SGR "on" code, paired with that code.
const MODIFIER_CODES: &[(Modifier, u8)] = &[
    (Modifier::BOLD, 1),
    (Modifier::DIM, 2),
    (Modifier::ITALIC, 3),
    (Modifier::UNDERLINED, 4),
    (Modifier::SLOW_BLINK, 5),
    (Modifier::RAPID_BLINK, 6),
    (Modifier::REVERSED, 7),
    (Modifier::HIDDEN, 8),
    (Modifier::CROSSED_OUT, 9),
    (Modifier::DOUBLE_UNDERLINED, 21),
    (Modifier::OVERLINED, 53),
    (Modifier::SUPERSCRIPT, 73),
    (Modifier::SUBSCRIPT, 74),
];

/// Append a decimal integer without invoking the formatting machinery.
#[inline]
pub fn push_u16(out: &mut Vec<u8>, mut value: u16) {
    if value < 10 {
        out.push(b'0' + value as u8);
        return;
    }
    let mut buf = [0u8; 5];
    let mut i = buf.len();
    while value > 0 {
        i -= 1;
        buf[i] = b'0' + (value % 10) as u8;
        value /= 10;
    }
    out.extend_from_slice(&buf[i..]);
}

/// `CSI y+1 ; x+1 H` — move the cursor to a 0-indexed position.
#[inline]
pub fn move_to(out: &mut Vec<u8>, x: u16, y: u16) {
    out.extend_from_slice(b"\x1b[");
    push_u16(out, y.saturating_add(1));
    out.push(b';');
    push_u16(out, x.saturating_add(1));
    out.push(b'H');
}

/// SGR base codes for a color, as `(foreground, background)` pairs.
const fn base_codes(color: Color) -> Option<(u8, u8)> {
    Some(match color {
        Color::Black => (30, 40),
        Color::Red => (31, 41),
        Color::Green => (32, 42),
        Color::Yellow => (33, 43),
        Color::Blue => (34, 44),
        Color::Magenta => (35, 45),
        Color::Cyan => (36, 46),
        Color::Gray => (37, 47),
        Color::DarkGray => (90, 100),
        Color::LightRed => (91, 101),
        Color::LightGreen => (92, 102),
        Color::LightYellow => (93, 103),
        Color::LightBlue => (94, 104),
        Color::LightMagenta => (95, 105),
        Color::LightCyan => (96, 106),
        Color::White => (97, 107),
        _ => return None,
    })
}

/// Write the SGR sequence selecting `color` for the foreground or background.
pub fn set_color(out: &mut Vec<u8>, color: Color, foreground: bool) {
    out.extend_from_slice(b"\x1b[");
    match color {
        Color::Reset => out.extend_from_slice(if foreground { b"39" } else { b"49" }),
        Color::Indexed(i) => {
            out.extend_from_slice(if foreground { b"38;5;" } else { b"48;5;" });
            push_u16(out, i as u16);
        }
        Color::Rgb(r, g, b) => {
            out.extend_from_slice(if foreground { b"38;2;" } else { b"48;2;" });
            push_u16(out, r as u16);
            out.push(b';');
            push_u16(out, g as u16);
            out.push(b';');
            push_u16(out, b as u16);
        }
        other => {
            // Every remaining variant is a named color with a base code.
            let (fg, bg) = base_codes(other).unwrap_or((39, 49));
            push_u16(out, if foreground { fg as u16 } else { bg as u16 });
        }
    }
    out.push(b'm');
}

/// Write the extended SGR sequence selecting an underline color (`58;…`).
pub fn set_underline_color(out: &mut Vec<u8>, color: Color) {
    out.extend_from_slice(b"\x1b[");
    match color {
        Color::Reset => out.extend_from_slice(b"59"),
        Color::Indexed(i) => {
            out.extend_from_slice(b"58;5;");
            push_u16(out, i as u16);
        }
        Color::Rgb(r, g, b) => {
            out.extend_from_slice(b"58;2;");
            push_u16(out, r as u16);
            out.push(b';');
            push_u16(out, g as u16);
            out.push(b';');
            push_u16(out, b as u16);
        }
        other => {
            // Named colors have no direct underline code; map through the
            // 256-color palette so the intent survives.
            out.extend_from_slice(b"58;5;");
            push_u16(out, named_palette_index(other) as u16);
        }
    }
    out.push(b'm');
}

/// Palette index for a named color, used where only indexed colors are allowed.
const fn named_palette_index(color: Color) -> u8 {
    match color {
        Color::Black => 0,
        Color::Red => 1,
        Color::Green => 2,
        Color::Yellow => 3,
        Color::Blue => 4,
        Color::Magenta => 5,
        Color::Cyan => 6,
        Color::Gray => 7,
        Color::DarkGray => 8,
        Color::LightRed => 9,
        Color::LightGreen => 10,
        Color::LightYellow => 11,
        Color::LightBlue => 12,
        Color::LightMagenta => 13,
        Color::LightCyan => 14,
        Color::White => 15,
        _ => 7,
    }
}

/// `OSC 8 ; ; URL ST` — open a hyperlink that covers the following cells.
///
/// Terminals without OSC 8 support ignore the sequence and render the text
/// unchanged, so this is always safe to emit.
pub fn open_hyperlink(out: &mut Vec<u8>, url: &str) {
    out.extend_from_slice(b"\x1b]8;;");
    // Control characters would terminate the sequence early; drop them rather
    // than emit a malformed escape.
    out.extend(
        url.as_bytes()
            .iter()
            .copied()
            .filter(|b| *b >= 0x20 && *b != 0x7F),
    );
    out.extend_from_slice(b"\x1b\\");
}

/// `OSC 8 ; ; ST` — close the currently open hyperlink.
#[inline]
pub fn close_hyperlink(out: &mut Vec<u8>) {
    out.extend_from_slice(b"\x1b]8;;\x1b\\");
}

/// `CSI 0 m` — reset all attributes and colors.
#[inline]
pub fn reset_attributes(out: &mut Vec<u8>) {
    out.extend_from_slice(b"\x1b[0m");
}

/// Modifiers that can be switched off individually, with their "off" SGR code.
///
/// `BOLD` and `DIM` share code 22, so turning one off turns off both; the
/// diffing path re-asserts whichever should remain on.
const MODIFIER_OFF_CODES: &[(Modifier, u8)] = &[
    (Modifier::BOLD, 22),
    (Modifier::DIM, 22),
    (Modifier::ITALIC, 23),
    (Modifier::UNDERLINED, 24),
    (Modifier::DOUBLE_UNDERLINED, 24),
    (Modifier::SLOW_BLINK, 25),
    (Modifier::RAPID_BLINK, 25),
    (Modifier::REVERSED, 27),
    (Modifier::HIDDEN, 28),
    (Modifier::CROSSED_OUT, 29),
    (Modifier::OVERLINED, 55),
    (Modifier::SUPERSCRIPT, 75),
    (Modifier::SUBSCRIPT, 75),
];

/// Underline styles that are expressed as `CSI 4:Nm` rather than a plain code.
const UNDERLINE_STYLES: &[(Modifier, &[u8])] = &[
    (Modifier::UNDERCURLED, b"\x1b[4:3m"),
    (Modifier::UNDERDOTTED, b"\x1b[4:4m"),
    (Modifier::UNDERDASHED, b"\x1b[4:5m"),
];

/// Write the minimal SGR sequence moving from `from` to `to`.
///
/// Only the flags that actually changed are emitted, so a run of cells that
/// share their attributes costs nothing, and a single attribute flip costs one
/// short sequence instead of a full reset and re-assert.
pub fn diff_modifiers(out: &mut Vec<u8>, from: Modifier, to: Modifier) {
    if from == to {
        return;
    }
    let removed = from.difference(to);
    let added = to.difference(from);

    if !removed.is_empty() {
        let mut emitted_off: [u8; 16] = [0; 16];
        let mut n = 0usize;
        for (flag, code) in MODIFIER_OFF_CODES {
            if !removed.contains(*flag) {
                continue;
            }
            // Codes are shared between flags (22 for bold/dim, 25 for blinks);
            // emit each distinct code once.
            if emitted_off[..n].contains(code) {
                continue;
            }
            if n < emitted_off.len() {
                emitted_off[n] = *code;
                n += 1;
            }
            out.extend_from_slice(b"\x1b[");
            push_u16(out, *code as u16);
            out.push(b'm');
        }
        // A shared "off" code may have cleared a flag that should stay on.
        for (flag, code) in MODIFIER_OFF_CODES {
            if to.contains(*flag) && emitted_off[..n].contains(code) {
                set_modifiers(out, *flag);
            }
        }
        if UNDERLINE_STYLES.iter().any(|(f, _)| removed.contains(*f)) {
            out.extend_from_slice(b"\x1b[24m");
        }
    }

    set_modifiers(out, added);
}

/// Write the SGR codes turning on every modifier in `modifier`.
pub fn set_modifiers(out: &mut Vec<u8>, modifier: Modifier) {
    if modifier.is_empty() {
        return;
    }
    for (flag, code) in MODIFIER_CODES {
        if modifier.contains(*flag) {
            out.extend_from_slice(b"\x1b[");
            push_u16(out, *code as u16);
            out.push(b'm');
        }
    }
    // Curly, dotted, and dashed underlines use the extended underline style
    // sequence rather than a plain SGR code.
    for (flag, seq) in UNDERLINE_STYLES {
        if modifier.contains(*flag) {
            out.extend_from_slice(seq);
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(f: impl FnOnce(&mut Vec<u8>)) -> String {
        let mut out = Vec::new();
        f(&mut out);
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn integers_round_trip() {
        for v in [0u16, 1, 9, 10, 99, 100, 1234, 9999, 65535] {
            assert_eq!(s(|o| push_u16(o, v)), v.to_string());
        }
    }

    #[test]
    fn move_to_is_one_indexed() {
        assert_eq!(s(|o| move_to(o, 0, 0)), "\x1b[1;1H");
        assert_eq!(s(|o| move_to(o, 79, 23)), "\x1b[24;80H");
    }

    #[test]
    fn colors_match_ansi_codes() {
        assert_eq!(s(|o| set_color(o, Color::Red, true)), "\x1b[31m");
        assert_eq!(s(|o| set_color(o, Color::Red, false)), "\x1b[41m");
        assert_eq!(s(|o| set_color(o, Color::Reset, true)), "\x1b[39m");
        assert_eq!(s(|o| set_color(o, Color::Reset, false)), "\x1b[49m");
        assert_eq!(
            s(|o| set_color(o, Color::Indexed(200), true)),
            "\x1b[38;5;200m"
        );
        assert_eq!(
            s(|o| set_color(o, Color::Rgb(1, 22, 255), false)),
            "\x1b[48;2;1;22;255m"
        );
        assert_eq!(s(|o| set_color(o, Color::White, true)), "\x1b[97m");
    }

    #[test]
    fn hyperlinks_open_and_close() {
        assert_eq!(
            s(|o| open_hyperlink(o, "https://example.com/a?b=1")),
            "\x1b]8;;https://example.com/a?b=1\x1b\\"
        );
        assert_eq!(s(close_hyperlink), "\x1b]8;;\x1b\\");
    }

    #[test]
    fn hyperlink_strips_control_characters() {
        let out = s(|o| open_hyperlink(o, "https://example.com/\x07evil\n"));
        assert_eq!(out, "\x1b]8;;https://example.com/evil\x1b\\");
    }

    #[test]
    fn modifier_diff_emits_nothing_when_unchanged() {
        assert_eq!(s(|o| diff_modifiers(o, Modifier::BOLD, Modifier::BOLD)), "");
    }

    #[test]
    fn modifier_diff_turns_single_flag_on_and_off() {
        assert_eq!(
            s(|o| diff_modifiers(o, Modifier::NONE, Modifier::ITALIC)),
            "\x1b[3m"
        );
        assert_eq!(
            s(|o| diff_modifiers(o, Modifier::ITALIC, Modifier::NONE)),
            "\x1b[23m"
        );
    }

    #[test]
    fn modifier_diff_reasserts_flags_sharing_an_off_code() {
        // 22 clears bold and dim together, so dropping bold while keeping dim
        // must turn dim back on.
        let from = Modifier::BOLD.union(Modifier::DIM);
        let out = s(|o| diff_modifiers(o, from, Modifier::DIM));
        assert_eq!(out, "\x1b[22m\x1b[2m");
    }

    #[test]
    fn modifier_diff_keeps_untouched_flags_alone() {
        let from = Modifier::BOLD;
        let to = Modifier::BOLD.union(Modifier::UNDERLINED);
        assert_eq!(s(|o| diff_modifiers(o, from, to)), "\x1b[4m");
    }

    #[test]
    fn modifier_diff_clears_extended_underline_styles() {
        let out = s(|o| diff_modifiers(o, Modifier::UNDERCURLED, Modifier::NONE));
        assert!(
            out.contains("\x1b[24m"),
            "expected underline reset, got {out:?}"
        );
    }

    #[test]
    fn modifiers_emit_each_code() {
        assert_eq!(s(|o| set_modifiers(o, Modifier::NONE)), "");
        assert_eq!(s(|o| set_modifiers(o, Modifier::BOLD)), "\x1b[1m");
        assert_eq!(
            s(|o| set_modifiers(o, Modifier::BOLD.union(Modifier::ITALIC))),
            "\x1b[1m\x1b[3m"
        );
        assert_eq!(s(|o| set_modifiers(o, Modifier::UNDERCURLED)), "\x1b[4:3m");
    }
}
