//! Sixel encoding for terminals that speak the DEC graphics protocol.
//!
//! Sixel encodes six vertical pixels at a time: each character carries one
//! column of six pixels as a bitmask, offset by `?` (0x3F). An image is emitted
//! band by band, and within a band once per palette color, so a color's pixels
//! are laid down in a single pass over the band's width.
//!
//! The encoder is self-contained — no image or color crates — and quantizes to
//! the 6×6×6 color cube plus a short grayscale ramp, which is what fits in a
//! Sixel palette without a full median-cut pass.

/// Number of pixel rows encoded by one sixel character.
const BAND: usize = 6;

/// Levels per channel in the color cube.
const LEVELS: usize = 6;

/// Palette entries: the 6×6×6 cube followed by a grayscale ramp.
const CUBE_COLORS: usize = LEVELS * LEVELS * LEVELS;
const GRAY_COLORS: usize = 24;
const PALETTE_SIZE: usize = CUBE_COLORS + GRAY_COLORS;

/// Quantize one channel to its cube level.
#[inline]
fn level(value: u8) -> usize {
    (value as usize * (LEVELS - 1) + 127) / 255
}

/// Palette index for an RGB triple.
///
/// Near-neutral colors go to the grayscale ramp, which has far finer steps than
/// the cube's diagonal, so gradients and anti-aliased text stay smooth.
fn palette_index(r: u8, g: u8, b: u8) -> usize {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    if max - min <= 8 {
        let gray = ((r as u16 + g as u16 + b as u16) / 3) as usize;
        let step = (gray * (GRAY_COLORS - 1) + 127) / 255;
        return CUBE_COLORS + step;
    }
    level(r) * LEVELS * LEVELS + level(g) * LEVELS + level(b)
}

/// The RGB value a palette index represents, in percent per channel — the unit
/// Sixel palette definitions use.
fn palette_percent(index: usize) -> (u8, u8, u8) {
    if index >= CUBE_COLORS {
        let step = index - CUBE_COLORS;
        let value = (step * 100 / (GRAY_COLORS - 1)) as u8;
        return (value, value, value);
    }
    let r = index / (LEVELS * LEVELS);
    let g = (index / LEVELS) % LEVELS;
    let b = index % LEVELS;
    let scale = |v: usize| (v * 100 / (LEVELS - 1)) as u8;
    (scale(r), scale(g), scale(b))
}

/// Append a decimal number without invoking the formatting machinery.
fn push_num(out: &mut String, mut value: usize) {
    if value == 0 {
        out.push('0');
        return;
    }
    let mut buf = [0u8; 10];
    let mut i = buf.len();
    while value > 0 {
        i -= 1;
        buf[i] = b'0' + (value % 10) as u8;
        value /= 10;
    }
    for &byte in &buf[i..] {
        out.push(byte as char);
    }
}

/// Emit a run of identical sixel characters, using the repeat introducer when
/// that is shorter than writing them out.
fn push_run(out: &mut String, ch: char, count: usize) {
    if count == 0 {
        return;
    }
    if count > 3 {
        out.push('!');
        push_num(out, count);
        out.push(ch);
    } else {
        for _ in 0..count {
            out.push(ch);
        }
    }
}

/// Encode RGBA8 pixels as a Sixel escape sequence.
///
/// `width` and `height` are in pixels; `rgba` is `width * height * 4` bytes.
/// Pixels with alpha below 128 are treated as transparent and simply left out
/// of every color's mask, so the terminal's background shows through.
///
/// Returns `None` if the buffer length does not match the dimensions.
pub fn encode_rgba(width: u32, height: u32, rgba: &[u8]) -> Option<String> {
    let (w, h) = (width as usize, height as usize);
    if w == 0 || h == 0 || rgba.len() != w * h * 4 {
        return None;
    }

    // Map every pixel to a palette index once; -1 marks transparent.
    let mut indexed: Vec<i16> = Vec::with_capacity(w * h);
    let mut used = [false; PALETTE_SIZE];
    for px in rgba.chunks_exact(4) {
        if px[3] < 128 {
            indexed.push(-1);
            continue;
        }
        let idx = palette_index(px[0], px[1], px[2]);
        used[idx] = true;
        indexed.push(idx as i16);
    }

    let mut out = String::with_capacity(w * h / 4 + 256);
    // Device Control String: P1=0 (pixels default to transparent), P2=1
    // (background untouched), P3=0, then `q` to enter sixel mode.
    out.push_str("\x1bP0;1;0q");
    // Raster attributes: 1:1 pixel aspect ratio, then the image size.
    out.push_str("\"1;1;");
    push_num(&mut out, w);
    out.push(';');
    push_num(&mut out, h);

    for (idx, is_used) in used.iter().enumerate() {
        if !is_used {
            continue;
        }
        let (r, g, b) = palette_percent(idx);
        out.push('#');
        push_num(&mut out, idx);
        out.push_str(";2;");
        push_num(&mut out, r as usize);
        out.push(';');
        push_num(&mut out, g as usize);
        out.push(';');
        push_num(&mut out, b as usize);
    }

    let bands = h.div_ceil(BAND);
    for band in 0..bands {
        let top = band * BAND;
        let rows = BAND.min(h - top);

        // Which colors appear anywhere in this band.
        let mut band_used = [false; PALETTE_SIZE];
        for row in 0..rows {
            for x in 0..w {
                let v = indexed[(top + row) * w + x];
                if v >= 0 {
                    band_used[v as usize] = true;
                }
            }
        }

        let mut first = true;
        for (color, is_used) in band_used.iter().enumerate() {
            if !is_used {
                continue;
            }
            if !first {
                // Carriage return: overlay the next color on the same band.
                out.push('$');
            }
            first = false;
            out.push('#');
            push_num(&mut out, color);

            // One sixel character per column, run-length encoded.
            let mut run_char = '\0';
            let mut run_len = 0usize;
            for x in 0..w {
                let mut bits = 0u8;
                for row in 0..rows {
                    if indexed[(top + row) * w + x] == color as i16 {
                        bits |= 1 << row;
                    }
                }
                let ch = (b'?' + bits) as char;
                if ch == run_char {
                    run_len += 1;
                } else {
                    push_run(&mut out, run_char, run_len);
                    run_char = ch;
                    run_len = 1;
                }
            }
            push_run(&mut out, run_char, run_len);
        }

        if band + 1 < bands {
            // Graphics newline: advance to the next band.
            out.push('-');
        }
    }

    out.push_str("\x1b\\");
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(w: u32, h: u32, color: [u8; 4]) -> Vec<u8> {
        color
            .iter()
            .copied()
            .cycle()
            .take((w * h * 4) as usize)
            .collect()
    }

    #[test]
    fn rejects_mismatched_buffers() {
        assert!(encode_rgba(2, 2, &[0; 15]).is_none());
        assert!(encode_rgba(0, 2, &[]).is_none());
        assert!(encode_rgba(2, 2, &[0; 16]).is_some());
    }

    #[test]
    fn wraps_the_payload_in_a_device_control_string() {
        let out = encode_rgba(4, 6, &solid(4, 6, [255, 0, 0, 255])).unwrap();
        assert!(out.starts_with("\x1bP0;1;0q"), "missing DCS: {out:?}");
        assert!(out.ends_with("\x1b\\"), "missing terminator: {out:?}");
        assert!(
            out.contains("\"1;1;4;6"),
            "missing raster attributes: {out:?}"
        );
    }

    #[test]
    fn a_solid_image_defines_exactly_one_color() {
        let out = encode_rgba(8, 6, &solid(8, 6, [255, 0, 0, 255])).unwrap();
        let definitions = out.matches(";2;").count();
        assert_eq!(definitions, 1, "expected one palette entry: {out:?}");
        // Pure red is the cube's maximum red with no green or blue.
        assert!(out.contains(";2;100;0;0"), "wrong color: {out:?}");
    }

    #[test]
    fn a_full_band_of_one_color_is_run_length_encoded() {
        // Six rows of one color: every column is the all-bits sixel, `~`.
        let out = encode_rgba(64, 6, &solid(64, 6, [0, 0, 255, 255])).unwrap();
        assert!(out.contains("!64~"), "run not compressed: {out:?}");
    }

    #[test]
    fn taller_images_emit_one_band_per_six_rows() {
        let out = encode_rgba(4, 18, &solid(4, 18, [0, 255, 0, 255])).unwrap();
        // Three bands means two graphics newlines between them.
        assert_eq!(out.matches('-').count(), 2, "band count wrong: {out:?}");
    }

    #[test]
    fn transparent_pixels_are_left_out() {
        let out = encode_rgba(4, 6, &solid(4, 6, [255, 0, 0, 0])).unwrap();
        assert_eq!(
            out.matches(";2;").count(),
            0,
            "transparent image drew: {out:?}"
        );
    }

    #[test]
    fn two_colors_share_a_band_with_a_carriage_return() {
        let mut data = Vec::new();
        for y in 0..6 {
            for _ in 0..4 {
                if y < 3 {
                    data.extend_from_slice(&[255, 0, 0, 255]);
                } else {
                    data.extend_from_slice(&[0, 0, 255, 255]);
                }
            }
        }
        let out = encode_rgba(4, 6, &data).unwrap();
        assert_eq!(
            out.matches(";2;").count(),
            2,
            "expected two colors: {out:?}"
        );
        assert!(
            out.contains('$'),
            "colors not overlaid in one band: {out:?}"
        );
    }

    #[test]
    fn grays_use_the_ramp_not_the_cube_diagonal() {
        // Mid gray is far from any cube diagonal step, so it must land in the
        // grayscale ramp to render accurately.
        let idx = palette_index(128, 128, 128);
        assert!(idx >= CUBE_COLORS, "gray fell into the color cube");
        let (r, g, b) = palette_percent(idx);
        assert_eq!((r, g, b), (52, 52, 52), "gray ramp step is off");
    }

    #[test]
    fn every_sixel_character_is_printable() {
        let out = encode_rgba(16, 12, &solid(16, 12, [10, 200, 40, 255])).unwrap();
        let payload = out
            .trim_start_matches("\x1bP0;1;0q")
            .trim_end_matches("\x1b\\");
        assert!(
            payload.chars().all(|c| c.is_ascii() && !c.is_control()),
            "payload contains a control character"
        );
    }
}
