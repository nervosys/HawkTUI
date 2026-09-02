use super::cell::Cell;
use super::rect::{Position, Rect};
use super::style::Style;
use super::symbol::Symbol;
use super::text::{Line, Span};
use std::collections::HashMap;
use std::sync::Arc;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// A two-dimensional grid of terminal cells.
///
/// The buffer is the primary rendering target. Widgets write into a buffer,
/// and the terminal backend diffs the current buffer against the previous
/// frame to compute minimal screen updates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Buffer {
    pub area: Rect,
    pub content: Vec<Cell>,
    /// Sparse map of cell index → hyperlink target.
    ///
    /// Hyperlinks are rare and usually cover a handful of cells, so they live
    /// beside the grid rather than inside [`Cell`]. A buffer with no links
    /// allocates nothing for this and every hot path skips it entirely.
    links: HashMap<u32, Arc<str>>,
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            area: Rect::ZERO,
            content: Vec::new(),
            links: HashMap::new(),
        }
    }
}

impl Buffer {
    /// Create a new buffer filled with empty cells.
    pub fn empty(area: Rect) -> Self {
        let size = area.area() as usize;
        Self {
            area,
            content: vec![Cell::EMPTY; size],
            links: HashMap::new(),
        }
    }

    /// Create a buffer filled with a specific string (for testing).
    pub fn with_lines<'a>(lines: impl IntoIterator<Item = &'a str>) -> Self {
        let lines: Vec<&str> = lines.into_iter().collect();
        let height = lines.len() as u16;
        let width = lines.iter().map(|l| l.width() as u16).max().unwrap_or(0);
        let area = Rect::new(0, 0, width, height);
        let mut buf = Self::empty(area);
        for (y, line) in lines.iter().enumerate() {
            buf.set_string(0, y as u16, line, Style::default());
        }
        buf
    }

    /// Reset all cells to empty.
    ///
    /// `Cell` is `Copy`, so this compiles to a straight memory fill rather than
    /// a per-cell method call.
    pub fn reset(&mut self) {
        self.content.fill(Cell::EMPTY);
        self.links.clear();
    }

    /// Resize the buffer (discards content).
    pub fn resize(&mut self, area: Rect) {
        let size = area.area() as usize;
        self.area = area;
        self.links.clear();
        self.content.clear();
        self.content.resize(size, Cell::EMPTY);
    }

    /// Get the cell at (x, y), if within bounds.
    pub fn cell(&self, pos: Position) -> Option<&Cell> {
        self.index_of(pos.x, pos.y).map(|i| &self.content[i])
    }

    /// Get a mutable reference to the cell at (x, y).
    pub fn cell_mut(&mut self, pos: Position) -> Option<&mut Cell> {
        self.index_of(pos.x, pos.y).map(|i| &mut self.content[i])
    }

    /// The text of one row, without styling.
    ///
    /// Trailing cells are included, so the string is as wide as the buffer
    /// except where a double-width grapheme covers two cells: it contributes
    /// one character, and its trailing cell contributes nothing.
    ///
    /// Returns an empty string when `y` is outside the buffer.
    pub fn row_text(&self, y: u16) -> String {
        let mut out = String::with_capacity(self.area.width as usize);
        for x in self.area.x..self.area.right() {
            let Some(cell) = self.cell(Position::new(x, y)) else {
                continue;
            };
            // A wide grapheme owns its trailing cell, which stays empty so the
            // backend emits nothing there. The same rule keeps this text one
            // character per glyph rather than one per cell.
            if !cell.symbol.is_empty() {
                out.push_str(cell.symbol.as_str());
            }
        }
        out
    }

    /// The whole buffer as plain text, one line per row, joined by `\n`.
    ///
    /// This is the readback half of rendering: draw a frame into a
    /// [`Buffer`](Self) and compare what came out against what was expected,
    /// without a terminal and without parsing escape sequences.
    ///
    /// ```
    /// use hawktui::core::buffer::Buffer;
    /// use hawktui::core::rect::Rect;
    /// use hawktui::core::style::Style;
    ///
    /// let mut buffer = Buffer::empty(Rect::new(0, 0, 5, 2));
    /// buffer.set_string(0, 0, "hi", Style::default());
    /// assert_eq!(buffer.to_text(), "hi   \n     ");
    /// ```
    pub fn to_text(&self) -> String {
        let mut out =
            String::with_capacity((self.area.width as usize + 1) * self.area.height as usize);
        for y in self.area.y..self.area.bottom() {
            if y != self.area.y {
                out.push('\n');
            }
            out.push_str(&self.row_text(y));
        }
        out
    }

    /// Copy the other buffer's hyperlinks into the overlapping region.
    fn merge_links(&mut self, other: &Buffer) {
        let area = self.area.intersection(other.area);
        for y in area.y..area.bottom() {
            for x in area.x..area.right() {
                let (Some(dst), Some(src)) = (self.index_of(x, y), other.index_of(x, y)) else {
                    continue;
                };
                match other.links.get(&(src as u32)) {
                    Some(url) => {
                        self.links.insert(dst as u32, Arc::clone(url));
                    }
                    None => {
                        self.links.remove(&(dst as u32));
                    }
                }
            }
        }
    }

    /// Index one past the last cell of row `y`, used to clamp row slices.
    fn row_end(&self, y: u16) -> usize {
        let row = y.saturating_sub(self.area.y) as usize;
        (row + 1) * self.area.width as usize
    }

    /// Slice range covering columns `[left, right)` of row `y`.
    fn row_range(&self, left: u16, right: u16, y: u16) -> Option<std::ops::Range<usize>> {
        if right <= left {
            return None;
        }
        let start = self.index_of(left, y)?;
        let end = (start + (right - left) as usize).min(self.row_end(y));
        Some(start..end)
    }

    fn index_of(&self, x: u16, y: u16) -> Option<usize> {
        if x >= self.area.x && x < self.area.right() && y >= self.area.y && y < self.area.bottom() {
            Some(
                ((y - self.area.y) as usize) * (self.area.width as usize)
                    + ((x - self.area.x) as usize),
            )
        } else {
            None
        }
    }

    /// Set a string starting at (x, y) with the given style.
    /// Returns the number of columns consumed.
    pub fn set_string(&mut self, x: u16, y: u16, string: &str, style: Style) -> u16 {
        self.set_string_truncated(x, y, string, self.area.right().saturating_sub(x), style)
    }

    /// Set a string with a maximum width, truncating if necessary.
    ///
    /// Runs of printable ASCII are written one cell per byte with no grapheme
    /// segmentation and no width lookup; the segmenter is only invoked for the
    /// stretches that actually need it. Mixed text pays the Unicode cost only
    /// for its Unicode parts.
    pub fn set_string_truncated(
        &mut self,
        x: u16,
        y: u16,
        string: &str,
        max_width: u16,
        style: Style,
    ) -> u16 {
        let Some(row_start) = self.index_of(x, y) else {
            return 0;
        };
        let limit = (row_start + max_width as usize).min(self.row_end(y));
        let mut idx = row_start;
        let mut col = 0u16;
        let mut rest = string;

        while !rest.is_empty() && idx < limit {
            let ascii_run = ascii_prefix_len(rest);

            // A trailing ASCII byte may be the base of a grapheme cluster whose
            // combining marks follow (`e` + U+0301). Leave that last byte to the
            // segmenter so the cluster stays intact.
            let ascii_take = if ascii_run == rest.len() {
                ascii_run
            } else {
                ascii_run.saturating_sub(1)
            };

            if ascii_take > 0 {
                let take = ascii_take.min(limit - idx);
                for (cell, byte) in self.content[idx..idx + take]
                    .iter_mut()
                    .zip(rest.as_bytes())
                {
                    cell.symbol = Symbol::from_ascii(*byte);
                    cell.set_style(style);
                }
                idx += take;
                col += take as u16;
                if take < ascii_take {
                    break;
                }
                rest = &rest[ascii_take..];
                continue;
            }

            // Scalar fast path: a character that cannot be extended, followed
            // by another that cannot extend it, is a grapheme cluster on its
            // own. That covers Latin, Greek, Cyrillic, CJK, kana, and Hangul
            // syllables — the bulk of non-ASCII UI text — with no segmenter at
            // all. Anything else (emoji, Indic, Arabic, Hebrew, jamo) falls
            // through to the real thing below.
            let mut chars = rest.chars();
            let mut standalone = 0usize;
            let mut current = chars.next();
            while let Some(ch) = current {
                let Some(w) = standalone_width(ch) else {
                    break;
                };
                let next = chars.next();
                let next_is_safe = match next {
                    None => true,
                    Some(n) => is_standalone_scalar(n),
                };
                if !next_is_safe {
                    break;
                }
                if w == 0 || col + w > max_width || idx + w as usize > limit {
                    current = None;
                    standalone = usize::MAX; // signal "stop entirely"
                    break;
                }

                self.content[idx].symbol = Symbol::from_char(ch);
                self.content[idx].set_style(style);
                for cell in &mut self.content[idx + 1..idx + w as usize] {
                    cell.symbol = Symbol::EMPTY;
                    cell.set_style(style);
                }
                idx += w as usize;
                col += w;
                standalone += ch.len_utf8();
                current = next;
            }
            if standalone == usize::MAX {
                break;
            }
            if standalone > 0 {
                rest = &rest[standalone..];
                continue;
            }

            // Unicode stretch: build the segmenter once and consume clusters
            // from it until a long ASCII run starts again. Re-creating the
            // iterator per grapheme would cost more than the segmentation.
            let mut iter = unicode_segmentation::UnicodeSegmentation::graphemes(rest, true);
            let mut consumed = 0usize;
            loop {
                if idx >= limit {
                    break;
                }
                let Some(grapheme) = iter.next() else { break };

                // Back to text the fast paths can handle: hand the run back
                // rather than segmenting everything that follows.
                let tail = &rest[consumed..];
                if (grapheme.len() == 1 && ascii_prefix_len(tail) >= ASCII_RUN_MIN)
                    || scalar_run_starts(tail)
                {
                    break;
                }

                let w = grapheme_width(grapheme);
                if w == 0 {
                    // A lone joiner or control: nothing occupies a cell.
                    consumed += grapheme.len();
                    continue;
                }
                if col + w > max_width || idx + w as usize > limit {
                    consumed = rest.len();
                    break;
                }
                consumed += grapheme.len();

                self.content[idx].symbol = Symbol::new(grapheme);
                self.content[idx].set_style(style);
                // Wide graphemes own their trailing cell, which stays empty so
                // the backend knows not to emit anything there.
                for cell in &mut self.content[idx + 1..idx + w as usize] {
                    cell.symbol = Symbol::EMPTY;
                    cell.set_style(style);
                }
                idx += w as usize;
                col += w;
            }
            if consumed == 0 {
                break;
            }
            rest = &rest[consumed.min(rest.len())..];
        }

        col
    }

    /// Write one grapheme at `(x, y)`, returning the columns it consumed.
    ///
    /// This is the single-cell counterpart to [`Buffer::fill`]: the caller has
    /// already decided what the glyph is, so nothing is scanned or segmented.
    pub fn set_grapheme(&mut self, x: u16, y: u16, symbol: &str, style: Style) -> u16 {
        self.set_symbol(x, y, Symbol::new(symbol), style)
    }

    /// Write an already-converted [`Symbol`] at `(x, y)`.
    ///
    /// Wide glyphs claim the following cell, which is left empty so the backend
    /// knows not to emit anything for it.
    pub fn set_symbol(&mut self, x: u16, y: u16, symbol: Symbol, style: Style) -> u16 {
        let width = grapheme_width(symbol.as_str());
        if width == 0 {
            return 0;
        }
        let Some(idx) = self.index_of(x, y) else {
            return 0;
        };
        let end = (idx + width as usize).min(self.row_end(y));
        if end <= idx {
            return 0;
        }
        self.content[idx].symbol = symbol;
        self.content[idx].set_style(style);
        for cell in &mut self.content[idx + 1..end] {
            cell.symbol = Symbol::EMPTY;
            cell.set_style(style);
        }
        (end - idx) as u16
    }

    /// Set a styled line at position.
    pub fn set_line(&mut self, x: u16, y: u16, line: &Line, max_width: u16) -> u16 {
        let mut col = 0u16;
        for span in &line.spans {
            if col >= max_width {
                break;
            }
            let remaining = max_width - col;
            let written =
                self.set_string_truncated(x + col, y, &span.content, remaining, span.style);
            col += written;
        }
        col
    }

    /// Set a single span at position with a maximum width.
    pub fn set_span(&mut self, x: u16, y: u16, span: &Span, max_width: u16) -> u16 {
        self.set_string_truncated(x, y, &span.content, max_width, span.style)
    }

    /// Fill an area with a style (without changing symbols).
    pub fn set_style(&mut self, area: Rect, style: Style) {
        let area = self.area.intersection(area);
        for y in area.y..area.bottom() {
            let Some(range) = self.row_range(area.x, area.right(), y) else {
                continue;
            };
            for cell in &mut self.content[range] {
                cell.set_style(style);
            }
        }
    }

    /// Fill an area with a character and style.
    pub fn fill(&mut self, area: Rect, symbol: &str, style: Style) {
        let area = self.area.intersection(area);
        // Convert the grapheme once, then stamp it across each row.
        let symbol = Symbol::new(symbol);
        for y in area.y..area.bottom() {
            let Some(range) = self.row_range(area.x, area.right(), y) else {
                continue;
            };
            for cell in &mut self.content[range] {
                cell.symbol = symbol;
                cell.set_style(style);
            }
        }
    }

    /// Attach a hyperlink to the cells `[x, x + width)` of row `y`.
    ///
    /// Terminals that support OSC 8 render those cells as a clickable link;
    /// everywhere else the text is unaffected.
    pub fn set_hyperlink(&mut self, x: u16, y: u16, width: u16, url: &str) {
        if width == 0 || url.is_empty() {
            return;
        }
        let Some(range) = self.row_range(x, x.saturating_add(width), y) else {
            return;
        };
        let url: Arc<str> = Arc::from(url);
        for idx in range {
            self.links.insert(idx as u32, Arc::clone(&url));
        }
    }

    /// Write a string and make it a hyperlink in one step.
    ///
    /// Returns the number of columns consumed, like [`Buffer::set_string`].
    pub fn set_string_linked(
        &mut self,
        x: u16,
        y: u16,
        string: &str,
        style: Style,
        url: &str,
    ) -> u16 {
        let width = self.set_string(x, y, string, style);
        self.set_hyperlink(x, y, width, url);
        width
    }

    /// The hyperlink attached to the cell at `(x, y)`, if any.
    pub fn hyperlink_at(&self, x: u16, y: u16) -> Option<&str> {
        if self.links.is_empty() {
            return None;
        }
        let idx = self.index_of(x, y)? as u32;
        self.links.get(&idx).map(|u| &**u)
    }

    /// Pair each changed cell with the hyperlink covering it, for the backend.
    pub fn attach_hyperlinks<'a>(
        &'a self,
        changes: &[(u16, u16, &'a Cell)],
    ) -> Vec<(u16, u16, &'a Cell, Option<&'a str>)> {
        changes
            .iter()
            .map(|(x, y, cell)| (*x, *y, *cell, self.hyperlink_at(*x, *y)))
            .collect()
    }

    /// Remove every hyperlink in this buffer.
    pub fn clear_hyperlinks(&mut self) {
        self.links.clear();
    }

    /// Whether any cell in this buffer carries a hyperlink.
    pub fn has_hyperlinks(&self) -> bool {
        !self.links.is_empty()
    }

    /// Compute the diff between this buffer and another.
    /// Returns an iterator of (x, y, &Cell) for cells that differ.
    pub fn diff<'a>(&'a self, other: &'a Buffer) -> Vec<(u16, u16, &'a Cell)> {
        let mut changes = Vec::new();

        // Fast path: identically positioned buffers of the same size compare as
        // two flat slices, so the inner loop is a straight zip with no per-cell
        // bounds arithmetic.
        if self.area == other.area {
            let width = self.area.width as usize;
            if width == 0 {
                return changes;
            }
            // Only consult the link tables when at least one buffer has links;
            // the overwhelmingly common case pays nothing for this.
            let check_links = !self.links.is_empty() || !other.links.is_empty();

            // Most frames change a small fraction of the screen, and the
            // unchanged parts are contiguous. Comparing a block of cells at a
            // time lets an untouched region be skipped with one comparison
            // instead of one per cell; only blocks that differ are walked.
            const BLOCK: usize = 32;
            for (block, (a_block, b_block)) in self
                .content
                .chunks(BLOCK)
                .zip(other.content.chunks(BLOCK))
                .enumerate()
            {
                let base = block * BLOCK;
                if !check_links && a_block == b_block {
                    continue;
                }
                for (offset, (a, b)) in a_block.iter().zip(b_block.iter()).enumerate() {
                    let i = base + offset;
                    if a != b
                        || (check_links
                            && self.links.get(&(i as u32)) != other.links.get(&(i as u32)))
                    {
                        changes.push((
                            self.area.x + (i % width) as u16,
                            self.area.y + (i / width) as u16,
                            b,
                        ));
                    }
                }
            }
            return changes;
        }

        let area = self.area.intersection(other.area);
        for y in area.y..area.bottom() {
            for x in area.x..area.right() {
                if let (Some(a), Some(b)) = (self.index_of(x, y), other.index_of(x, y)) {
                    if self.content[a] != other.content[b] {
                        changes.push((x, y, &other.content[b]));
                    }
                }
            }
        }
        changes
    }

    /// Merge another buffer on top of this one at its area position.
    pub fn merge(&mut self, other: &Buffer) {
        if !other.links.is_empty() {
            self.merge_links(other);
        }
        let area = self.area.intersection(other.area);
        let len = area.width as usize;
        for y in area.y..area.bottom() {
            let (Some(dst), Some(src)) = (self.index_of(area.x, y), other.index_of(area.x, y))
            else {
                continue;
            };
            self.content[dst..dst + len].copy_from_slice(&other.content[src..src + len]);
        }
    }
}

impl std::ops::Index<(u16, u16)> for Buffer {
    type Output = Cell;
    fn index(&self, (x, y): (u16, u16)) -> &Self::Output {
        /// Sentinel cell returned when indexing out of bounds, preventing panics (MEM-1).
        static OOB_CELL: std::sync::LazyLock<Cell> = std::sync::LazyLock::new(Cell::default);
        match self.index_of(x, y) {
            Some(i) => &self.content[i],
            None => &OOB_CELL,
        }
    }
}

impl std::ops::IndexMut<(u16, u16)> for Buffer {
    fn index_mut(&mut self, (x, y): (u16, u16)) -> &mut Self::Output {
        // Return a writable scratch cell for out-of-bounds writes instead of
        // panicking (MEM-1 hardening).  The scratch cell is appended once
        // and reused for subsequent OOB accesses within the same frame.
        match self.index_of(x, y) {
            Some(i) => &mut self.content[i],
            None => {
                self.content.push(Cell::default());
                let last = self.content.len() - 1;
                &mut self.content[last]
            }
        }
    }
}

/// Length of the leading run of printable ASCII, each byte exactly one cell.
#[inline]
fn ascii_prefix_len(s: &str) -> usize {
    s.as_bytes()
        .iter()
        .take_while(|b| (0x20..0x7F).contains(*b))
        .count()
}

/// Shortest ASCII run worth leaving to the byte-per-cell fast path.
///
/// Below this, switching loops costs more than simply letting the segmenter
/// walk the characters.
const ASCII_RUN_MIN: usize = 4;

/// Whether `s` begins with a character the scalar fast path can take.
///
/// Requires the character after it to be standalone too, since a cluster is
/// only guaranteed to end where nothing can extend it.
#[inline]
fn scalar_run_starts(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !is_standalone_scalar(first) {
        return false;
    }
    match chars.next() {
        None => true,
        Some(second) => is_standalone_scalar(second),
    }
}

/// Column width of `ch` if it is always a grapheme cluster by itself, given
/// that the character after it is too.
///
/// The ranges are deliberately conservative: they exclude every block that
/// contains combining marks, joiners, variation selectors, conjoining jamo, or
/// emoji modifiers, so a character that could be extended never takes this
/// path. Being wrong here would merge or split a cluster, so the rule is
/// "prove it cannot be extended", not "probably fine".
///
/// Widths come from the range itself wherever a block is uniformly narrow or
/// wide, which saves the width-table lookup for Latin, Greek, Cyrillic, CJK,
/// kana, Hangul, and fullwidth text. Blocks that mix widths defer to the table.
#[inline]
fn standalone_width(ch: char) -> Option<u16> {
    Some(match ch as u32 {
        // Printable ASCII and Latin-1 through Latin Extended-B, IPA, and
        // spacing modifier letters — everything below the combining block.
        0x20..=0x7E | 0xA1..=0x2FF => 1,
        // Greek and Coptic, and Cyrillic up to its combining marks (0x483).
        0x370..=0x482 => 1,
        // Punctuation, superscripts, and currency, skipping the invisible
        // formatting characters at 0x2028..0x202F and the joiner at 0x200D.
        0x2010..=0x2027 | 0x2030..=0x205E | 0x2070..=0x209F => 1,
        // Arrows, math operators, box drawing, block elements, geometric
        // shapes, and misc symbols. Widths are mixed here — ⌚ and ⬛ are wide
        // while → is narrow — so this block asks the width tables. These can
        // also take a variation selector, but the selector is not standalone,
        // so such a pair falls through to the segmenter.
        0x2190..=0x2BFF => UnicodeWidthChar::width(ch)? as u16,
        // CJK punctuation, skipping the combining marks at 0x302A..0x302F and
        // the voiced sound marks at 0x3099..0x309A.
        0x3001..=0x3029 | 0x303B..=0x303E => 2,
        // The ideographic half fill space is the one narrow character here.
        0x303F => 1,
        // Kana. The block ends at 0x3096; 0x3097 and 0x3098 are unassigned and
        // must not be claimed as wide.
        0x3041..=0x3096 | 0x309B..=0x30FF => 2,
        // CJK ideographs, extension A, and compatibility ideographs.
        0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF => 2,
        // Precomposed Hangul syllables. Trailing jamo (0x11A8..) are excluded,
        // so a syllable followed by one falls through to the segmenter.
        0xAC00..=0xD7A3 => 2,
        // Fullwidth forms.
        0xFF01..=0xFF60 => 2,
        _ => return None,
    })
}

/// Whether `ch` is always a grapheme cluster by itself, given that the
/// character after it is also one of these.
#[inline]
fn is_standalone_scalar(ch: char) -> bool {
    standalone_width(ch).is_some()
}

/// Display width of a grapheme cluster.
///
/// ASCII skips the tables entirely, and single-scalar clusters — nearly all CJK
/// and emoji — use the per-`char` table rather than summing over a string.
#[inline]
fn grapheme_width(g: &str) -> u16 {
    let b = g.as_bytes();
    if b.len() == 1 && (0x20..0x7F).contains(&b[0]) {
        return 1;
    }
    let mut chars = g.chars();
    match (chars.next(), chars.next()) {
        (Some(ch), None) => UnicodeWidthChar::width(ch).unwrap_or(0) as u16,
        _ => g.width() as u16,
    }
}
