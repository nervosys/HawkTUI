//! Layout engine with constraint-based space allocation.
//!
//! Inspired by ratatui's layout system with flexbox-like semantics.

pub use crate::core::rect::Margin;
use crate::core::rect::Rect;

pub use crate::core::text::Alignment;

/// Layouting direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Direction {
    #[default]
    Vertical,
    Horizontal,
}

/// Size constraint for layout segments.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Constraint {
    /// Fixed length in cells.
    Length(u16),
    /// Percentage of available space (0-100).
    Percentage(u16),
    /// Minimum size.
    Min(u16),
    /// Maximum size.
    Max(u16),
    /// Ratio of available space (numerator/denominator).
    Ratio(u32, u32),
    /// Fill remaining space proportionally (weight relative to other fills).
    Fill(u16),
}

/// How to distribute excess space after constraints are satisfied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Flex {
    /// Pack segments at the start.
    #[default]
    Start,
    /// Center segments.
    Center,
    /// Pack segments at the end.
    End,
    /// Distribute space evenly between segments.
    SpaceBetween,
    /// Distribute space evenly around segments.
    SpaceAround,
}

/// Layout builder.
#[derive(Debug, Clone)]
pub struct Layout {
    direction: Direction,
    constraints: Vec<Constraint>,
    margin: Margin,
    flex: Flex,
    spacing: i16,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            direction: Direction::Vertical,
            constraints: Vec::new(),
            margin: Margin::ZERO,
            flex: Flex::Start,
            spacing: 0,
        }
    }
}

impl Layout {
    pub fn new(direction: Direction, constraints: impl Into<Vec<Constraint>>) -> Self {
        Self {
            direction,
            constraints: constraints.into(),
            ..Default::default()
        }
    }

    pub fn vertical(constraints: impl Into<Vec<Constraint>>) -> Self {
        Self::new(Direction::Vertical, constraints)
    }

    pub fn horizontal(constraints: impl Into<Vec<Constraint>>) -> Self {
        Self::new(Direction::Horizontal, constraints)
    }

    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub fn constraints(mut self, constraints: impl Into<Vec<Constraint>>) -> Self {
        self.constraints = constraints.into();
        self
    }

    pub fn margin(mut self, margin: Margin) -> Self {
        self.margin = margin;
        self
    }

    pub fn flex(mut self, flex: Flex) -> Self {
        self.flex = flex;
        self
    }

    pub fn spacing(mut self, spacing: i16) -> Self {
        self.spacing = spacing;
        self
    }

    /// Split the given area into segments according to constraints.
    pub fn split(&self, area: Rect) -> Vec<Rect> {
        let inner = area.inner(self.margin);
        if self.constraints.is_empty() || inner.is_empty() {
            return vec![inner];
        }

        let total_space = match self.direction {
            Direction::Vertical => inner.height,
            Direction::Horizontal => inner.width,
        };

        let n = self.constraints.len();
        let total_spacing = if n > 1 {
            (n as i32 - 1) * self.spacing as i32
        } else {
            0
        };
        let available = (total_space as i32 - total_spacing).max(0) as u16;

        // Phase 1: compute initial sizes
        let mut sizes: Vec<u16> = self
            .constraints
            .iter()
            .map(|c| match c {
                Constraint::Length(l) => (*l).min(available),
                Constraint::Percentage(p) => ((available as u32 * *p as u32) / 100) as u16,
                Constraint::Min(m) => *m,
                Constraint::Max(m) => (*m).min(available),
                Constraint::Ratio(num, den) => {
                    (available as u32 * *num).checked_div(*den).unwrap_or(0) as u16
                }
                Constraint::Fill(_) => 0,
            })
            .collect();

        // Phase 2: distribute remaining space to Fill constraints
        let fixed_total: u16 = sizes.iter().sum();
        let remaining = available.saturating_sub(fixed_total);

        let fill_total_weight: u16 = self
            .constraints
            .iter()
            .filter_map(|c| match c {
                Constraint::Fill(w) => Some(*w),
                _ => None,
            })
            .sum();

        if fill_total_weight > 0 && remaining > 0 {
            let mut distributed = 0u16;
            let fill_count = self
                .constraints
                .iter()
                .filter(|c| matches!(c, Constraint::Fill(_)))
                .count();
            let mut fill_idx = 0;

            for (i, c) in self.constraints.iter().enumerate() {
                if let Constraint::Fill(w) = c {
                    fill_idx += 1;
                    let share = if fill_idx == fill_count {
                        // Last fill gets the remainder to avoid rounding errors
                        remaining - distributed
                    } else {
                        ((remaining as u32 * *w as u32) / fill_total_weight as u32) as u16
                    };
                    sizes[i] = share;
                    distributed += share;
                }
            }
        }

        // Phase 2b: distribute remaining space to Min constraints.
        // Min(m) means "at least m, but grow to fill available space" — this
        // matches ratatui semantics where Min(0) acts as a flexible fill.
        {
            let used: u16 = sizes.iter().sum();
            let leftover = available.saturating_sub(used);
            let min_count = self
                .constraints
                .iter()
                .filter(|c| matches!(c, Constraint::Min(_)))
                .count();
            if min_count > 0 && leftover > 0 {
                let share = leftover / min_count as u16;
                let mut distributed = 0u16;
                let mut idx = 0;
                for (i, c) in self.constraints.iter().enumerate() {
                    if let Constraint::Min(_) = c {
                        idx += 1;
                        let extra = if idx == min_count {
                            leftover - distributed
                        } else {
                            share
                        };
                        sizes[i] += extra;
                        distributed += extra;
                    }
                }
            }
        }

        // Phase 3: apply Min/Max constraint adjustments
        for (i, c) in self.constraints.iter().enumerate() {
            match c {
                Constraint::Min(m) => sizes[i] = sizes[i].max(*m),
                Constraint::Max(m) => sizes[i] = sizes[i].min(*m),
                _ => {}
            }
        }

        // Phase 4: clamp total to available space
        let total_used: u16 = sizes.iter().sum();
        if total_used > available {
            // Proportionally shrink all segments
            let scale = available as f64 / total_used as f64;
            let mut shrunk_total = 0u16;
            for (i, size) in sizes.iter_mut().enumerate() {
                if i == n - 1 {
                    *size = available - shrunk_total;
                } else {
                    *size = (*size as f64 * scale) as u16;
                    shrunk_total += *size;
                }
            }
        }

        // Phase 5: compute positions and emit Rects
        let mut rects = Vec::with_capacity(n);
        let actual_total: u16 = sizes.iter().sum();
        let excess = available.saturating_sub(actual_total);

        let start_offset = match self.flex {
            Flex::Start | Flex::SpaceBetween => 0,
            Flex::Center | Flex::SpaceAround => excess / 2,
            Flex::End => excess,
        };

        let mut pos = match self.direction {
            Direction::Vertical => inner.y + start_offset,
            Direction::Horizontal => inner.x + start_offset,
        };

        for (i, size) in sizes.iter().enumerate() {
            let rect = match self.direction {
                Direction::Vertical => Rect::new(inner.x, pos, inner.width, *size),
                Direction::Horizontal => Rect::new(pos, inner.y, *size, inner.height),
            };
            rects.push(rect);
            pos = pos.saturating_add(*size);
            if i < n - 1 {
                pos = (pos as i32 + self.spacing as i32).max(0) as u16;
            }
        }

        rects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertical_fixed_lengths() {
        let area = Rect::new(0, 0, 80, 24);
        let rects =
            Layout::vertical(vec![Constraint::Length(3), Constraint::Length(5)]).split(area);
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0], Rect::new(0, 0, 80, 3));
        assert_eq!(rects[1], Rect::new(0, 3, 80, 5));
    }

    #[test]
    fn fill_distributes_remaining() {
        let area = Rect::new(0, 0, 80, 24);
        let rects = Layout::vertical(vec![Constraint::Length(4), Constraint::Fill(1)]).split(area);
        assert_eq!(rects[0].height, 4);
        assert_eq!(rects[1].height, 20);
    }

    #[test]
    fn horizontal_with_margin() {
        let area = Rect::new(0, 0, 80, 24);
        let rects =
            Layout::horizontal(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
                .margin(Margin::uniform(1))
                .split(area);
        assert_eq!(rects[0].x, 1);
        assert_eq!(rects[0].width + rects[1].width, 78);
    }

    #[test]
    fn min_absorbs_remaining_space() {
        let area = Rect::new(0, 0, 80, 24);
        let rects = Layout::vertical(vec![Constraint::Min(0), Constraint::Length(3)]).split(area);
        assert_eq!(rects[0].height, 21);
        assert_eq!(rects[1].height, 3);
        assert_eq!(rects[1].y, 21);
    }
}
