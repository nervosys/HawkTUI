use super::cell::Cell;
use super::rect::{Position, Rect};
use super::style::Style;
use super::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

/// A two-dimensional grid of terminal cells.
///
/// The buffer is the primary rendering target. Widgets write into a buffer,
/// and the terminal backend diffs the current buffer against the previous
/// frame to compute minimal screen updates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Buffer {
    pub area: Rect,
    pub content: Vec<Cell>,
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            area: Rect::ZERO,
            content: Vec::new(),
        }
    }
}

impl Buffer {
    /// Create a new buffer filled with empty cells.
    pub fn empty(area: Rect) -> Self {
        let size = area.area() as usize;
        Self {
            area,
            content: vec![Cell::default(); size],
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
    pub fn reset(&mut self) {
        for cell in &mut self.content {
            cell.reset();
        }
    }

    /// Resize the buffer (discards content).
    pub fn resize(&mut self, area: Rect) {
        let size = area.area() as usize;
        self.area = area;
        self.content.clear();
        self.content.resize(size, Cell::default());
    }

    /// Get the cell at (x, y), if within bounds.
    pub fn cell(&self, pos: Position) -> Option<&Cell> {
        self.index_of(pos.x, pos.y).map(|i| &self.content[i])
    }

    /// Get a mutable reference to the cell at (x, y).
    pub fn cell_mut(&mut self, pos: Position) -> Option<&mut Cell> {
        self.index_of(pos.x, pos.y).map(|i| &mut self.content[i])
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
    pub fn set_string_truncated(
        &mut self,
        x: u16,
        y: u16,
        string: &str,
        max_width: u16,
        style: Style,
    ) -> u16 {
        let mut col = 0u16;
        for grapheme in unicode_segmentation::UnicodeSegmentation::graphemes(string, true) {
            let w = grapheme.width() as u16;
            if col + w > max_width {
                break;
            }
            if let Some(idx) = self.index_of(x + col, y) {
                self.content[idx].set_symbol(grapheme).set_style(style);
                // For wide characters, set continuation cells
                for i in 1..w {
                    if let Some(idx2) = self.index_of(x + col + i, y) {
                        self.content[idx2].set_symbol("").set_style(style);
                    }
                }
            }
            col += w;
        }
        col
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
            for x in area.x..area.right() {
                if let Some(idx) = self.index_of(x, y) {
                    self.content[idx].set_style(style);
                }
            }
        }
    }

    /// Fill an area with a character and style.
    pub fn fill(&mut self, area: Rect, symbol: &str, style: Style) {
        let area = self.area.intersection(area);
        for y in area.y..area.bottom() {
            for x in area.x..area.right() {
                if let Some(idx) = self.index_of(x, y) {
                    self.content[idx].set_symbol(symbol).set_style(style);
                }
            }
        }
    }

    /// Compute the diff between this buffer and another.
    /// Returns an iterator of (x, y, &Cell) for cells that differ.
    pub fn diff<'a>(&'a self, other: &'a Buffer) -> Vec<(u16, u16, &'a Cell)> {
        let mut changes = Vec::new();
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
        let area = self.area.intersection(other.area);
        for y in area.y..area.bottom() {
            for x in area.x..area.right() {
                if let (Some(dst), Some(src)) = (self.index_of(x, y), other.index_of(x, y)) {
                    self.content[dst] = other.content[src].clone();
                }
            }
        }
    }
}

impl std::ops::Index<(u16, u16)> for Buffer {
    type Output = Cell;
    fn index(&self, (x, y): (u16, u16)) -> &Self::Output {
        let idx = self.index_of(x, y).expect("position out of bounds");
        &self.content[idx]
    }
}

impl std::ops::IndexMut<(u16, u16)> for Buffer {
    fn index_mut(&mut self, (x, y): (u16, u16)) -> &mut Self::Output {
        let idx = self.index_of(x, y).expect("position out of bounds");
        &mut self.content[idx]
    }
}
