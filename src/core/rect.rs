use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub};

/// A rectangular area in terminal coordinates.
///
/// All positions are zero-indexed. The top-left of the terminal is (0, 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub const ZERO: Rect = Rect {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    };

    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Total number of cells in this rectangle.
    pub const fn area(&self) -> u32 {
        self.width as u32 * self.height as u32
    }

    /// Whether this rectangle has zero area.
    pub const fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// The leftmost column.
    pub const fn left(&self) -> u16 {
        self.x
    }

    /// The rightmost column (exclusive).
    pub const fn right(&self) -> u16 {
        self.x.saturating_add(self.width)
    }

    /// The topmost row.
    pub const fn top(&self) -> u16 {
        self.y
    }

    /// The bottommost row (exclusive).
    pub const fn bottom(&self) -> u16 {
        self.y.saturating_add(self.height)
    }

    /// The center position.
    pub const fn center(&self) -> Position {
        Position {
            x: self.x.saturating_add(self.width / 2),
            y: self.y.saturating_add(self.height / 2),
        }
    }

    /// Returns the intersection of two rectangles.
    pub fn intersection(&self, other: Rect) -> Rect {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        Rect {
            x,
            y,
            width: right.saturating_sub(x),
            height: bottom.saturating_sub(y),
        }
    }

    /// Returns the smallest rectangle that contains both.
    pub fn union(&self, other: Rect) -> Rect {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        Rect {
            x,
            y,
            width: right.saturating_sub(x),
            height: bottom.saturating_sub(y),
        }
    }

    /// Whether this rectangle contains a position.
    pub fn contains(&self, pos: Position) -> bool {
        pos.x >= self.x && pos.x < self.right() && pos.y >= self.y && pos.y < self.bottom()
    }

    /// Create a new rect with uniform inner margin applied.
    pub fn inner(&self, margin: Margin) -> Rect {
        let x = self.x.saturating_add(margin.left);
        let y = self.y.saturating_add(margin.top);
        let width = self
            .width
            .saturating_sub(margin.left.saturating_add(margin.right));
        let height = self
            .height
            .saturating_sub(margin.top.saturating_add(margin.bottom));
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    /// Iterator over all positions in this rectangle, row by row.
    pub fn positions(&self) -> impl Iterator<Item = Position> + '_ {
        (self.y..self.bottom())
            .flat_map(move |y| (self.x..self.right()).map(move |x| Position { x, y }))
    }

    /// Iterator over row indices.
    pub fn rows(&self) -> impl Iterator<Item = u16> {
        self.y..self.bottom()
    }

    /// Iterator over column indices.
    pub fn columns(&self) -> impl Iterator<Item = u16> {
        self.x..self.right()
    }
}

/// A position in terminal coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl Position {
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

impl Add<Offset> for Position {
    type Output = Position;
    fn add(self, rhs: Offset) -> Self::Output {
        Position {
            x: (self.x as i32 + rhs.x as i32).max(0) as u16,
            y: (self.y as i32 + rhs.y as i32).max(0) as u16,
        }
    }
}

impl Sub for Position {
    type Output = Offset;
    fn sub(self, rhs: Self) -> Self::Output {
        Offset {
            x: self.x as i16 - rhs.x as i16,
            y: self.y as i16 - rhs.y as i16,
        }
    }
}

/// A displacement in terminal coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Offset {
    pub x: i16,
    pub y: i16,
}

/// Margin/padding specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Margin {
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
    pub left: u16,
}

impl Margin {
    pub const ZERO: Margin = Margin {
        top: 0,
        right: 0,
        bottom: 0,
        left: 0,
    };

    /// Uniform margin on all sides.
    pub const fn uniform(value: u16) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Symmetric margin (vertical, horizontal).
    pub const fn symmetric(vertical: u16, horizontal: u16) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Individual sides.
    pub const fn new(top: u16, right: u16, bottom: u16, left: u16) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }
}

/// Size in columns and rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

impl Size {
    pub const fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }
}

impl From<Rect> for Size {
    fn from(rect: Rect) -> Self {
        Size {
            width: rect.width,
            height: rect.height,
        }
    }
}
