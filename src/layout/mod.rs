//! Layout engine with constraint-based space allocation.
//!
//! Inspired by ratatui's layout system with flexbox-like semantics.

pub use crate::core::rect::Margin;
use crate::core::rect::Rect;
use std::rc::Rc;

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
    /// Excess space becomes equal gaps *between* segments; the outer edges
    /// stay flush.
    SpaceBetween,
    /// Every segment is wrapped in an equal half-gap, so the outer edges get
    /// half of what falls between two segments.
    SpaceAround,
    /// Gaps between segments and at both edges are all equal.
    SpaceEvenly,
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

/// Number of constraints a split can solve without touching the allocator.
/// Deeper layouts still work; they spill their working set to the heap.
const INLINE_CONSTRAINTS: usize = 16;

impl Layout {
    pub fn new(direction: Direction, constraints: impl IntoIterator<Item = Constraint>) -> Self {
        Self {
            direction,
            constraints: constraints.into_iter().collect(),
            ..Default::default()
        }
    }

    pub fn vertical(constraints: impl IntoIterator<Item = Constraint>) -> Self {
        Self::new(Direction::Vertical, constraints)
    }

    pub fn horizontal(constraints: impl IntoIterator<Item = Constraint>) -> Self {
        Self::new(Direction::Horizontal, constraints)
    }

    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub fn constraints(mut self, constraints: impl IntoIterator<Item = Constraint>) -> Self {
        self.constraints = constraints.into_iter().collect();
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

    /// Fixed space between adjacent segments — the flexbox `gap`.
    ///
    /// The same thing [`spacing`](Self::spacing) sets, named the way layout
    /// code usually says it and restricted to non-negative values.
    pub fn gap(mut self, gap: u16) -> Self {
        self.spacing = gap as i16;
        self
    }

    /// Inset the area on all four sides before splitting — the CSS `padding`.
    pub fn padding(mut self, padding: u16) -> Self {
        self.margin = Margin::uniform(padding);
        self
    }

    /// Inset the left and right edges before splitting.
    pub fn horizontal_margin(mut self, margin: u16) -> Self {
        self.margin.left = margin;
        self.margin.right = margin;
        self
    }

    /// Inset the top and bottom edges before splitting.
    pub fn vertical_margin(mut self, margin: u16) -> Self {
        self.margin.top = margin;
        self.margin.bottom = margin;
        self
    }

    /// Split the given area into segments according to constraints.
    ///
    /// Results are memoized per thread: splitting the same area with the same
    /// constraints again — which is what every redraw does — returns a shared
    /// handle without re-solving or allocating.
    pub fn split(&self, area: Rect) -> Rc<[Rect]> {
        cache::get_or_solve(self, area)
    }

    /// Split into a fixed number of areas, destructurable at the call site.
    ///
    /// ```
    /// use hawktui::layout::{Constraint, Layout};
    /// use hawktui::core::rect::Rect;
    ///
    /// let [header, body, footer] = Layout::vertical([
    ///     Constraint::Length(3),
    ///     Constraint::Min(0),
    ///     Constraint::Length(1),
    /// ])
    /// .areas(Rect::new(0, 0, 80, 24));
    /// assert_eq!(header.height, 3);
    /// assert_eq!(footer.y, 23);
    /// ```
    ///
    /// Asking for more areas than the layout produces fills the remainder with
    /// empty rects at the layout's origin, so the destructuring never panics.
    pub fn areas<const N: usize>(&self, area: Rect) -> [Rect; N] {
        let solved = self.split(area);
        let mut out = [Rect::new(area.x, area.y, 0, 0); N];
        for (slot, rect) in out.iter_mut().zip(solved.iter()) {
            *slot = *rect;
        }
        out
    }

    /// Solve the layout from scratch, bypassing the memo cache.
    pub fn solve(&self, area: Rect) -> Vec<Rect> {
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

        // Phase 1: compute initial sizes.
        //
        // Layouts almost never exceed a handful of constraints, so the working
        // set lives in a stack buffer; only pathological layouts spill to the
        // heap. This keeps `split` allocation-free apart from the result.
        let mut inline = [0u16; INLINE_CONSTRAINTS];
        let mut spilled: Vec<u16>;
        let sizes: &mut [u16] = if n <= INLINE_CONSTRAINTS {
            &mut inline[..n]
        } else {
            spilled = vec![0; n];
            &mut spilled
        };
        for (slot, c) in sizes.iter_mut().zip(self.constraints.iter()) {
            *slot = match c {
                Constraint::Length(l) => (*l).min(available),
                Constraint::Percentage(p) => ((available as u32 * *p as u32) / 100) as u16,
                Constraint::Min(m) => *m,
                Constraint::Max(m) => (*m).min(available),
                Constraint::Ratio(num, den) => {
                    (available as u32 * *num).checked_div(*den).unwrap_or(0) as u16
                }
                Constraint::Fill(_) => 0,
            };
        }

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

        // Leftover space either sits at one end, or is dealt out as gaps.
        // `gap_base` goes between every adjacent pair; `gap_extra` spreads the
        // integer-division remainder over the leading gaps so the segments
        // still end flush with the area.
        let gap_count = n.saturating_sub(1) as u16;
        let (start_offset, gap_base, gap_extra) = match self.flex {
            Flex::Start => (0, 0, 0),
            Flex::Center => (excess / 2, 0, 0),
            Flex::End => (excess, 0, 0),
            Flex::SpaceBetween => match excess.checked_div(gap_count) {
                Some(base) => (0, base, excess % gap_count),
                None => (0, 0, 0),
            },
            Flex::SpaceAround => {
                let half = excess / (n as u16 * 2);
                (half, half * 2, 0)
            }
            Flex::SpaceEvenly => {
                let unit = excess / (n as u16 + 1);
                (unit, unit, 0)
            }
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
                let extra = if (i as u16) < gap_extra { 1 } else { 0 };
                pos = (pos as i32 + self.spacing as i32).max(0) as u16;
                pos = pos.saturating_add(gap_base + extra);
            }
        }

        rects
    }
}

/// Per-thread memoization of layout results.
///
/// A TUI re-splits the same areas with the same constraints on every frame, so
/// the same handful of layouts repeat indefinitely. Keeping the last few
/// results keyed on the full input makes a redraw's layout pass allocation-free
/// and comparison-cheap; a miss simply solves as before.
mod cache {
    use super::{Direction, Flex, Layout, Margin};
    use crate::core::rect::Rect;
    use std::cell::RefCell;
    use std::rc::Rc;

    /// Entries retained per thread. Small enough to scan linearly, large enough
    /// to cover the nested splits of a busy screen.
    const CAPACITY: usize = 32;

    struct Entry {
        area: Rect,
        direction: Direction,
        margin: Margin,
        flex: Flex,
        spacing: i16,
        constraints: Box<[super::Constraint]>,
        result: Rc<[Rect]>,
    }

    impl Entry {
        fn matches(&self, layout: &Layout, area: Rect) -> bool {
            self.area == area
                && self.direction == layout.direction
                && self.spacing == layout.spacing
                && self.flex == layout.flex
                && self.margin == layout.margin
                && *self.constraints == *layout.constraints
        }
    }

    thread_local! {
        static CACHE: RefCell<Vec<Entry>> = const { RefCell::new(Vec::new()) };
    }

    pub(super) fn get_or_solve(layout: &Layout, area: Rect) -> Rc<[Rect]> {
        if let Some(hit) = CACHE.with(|c| {
            let mut cache = c.borrow_mut();
            let found = cache.iter().position(|e| e.matches(layout, area));
            found.map(|i| {
                // Move the hit to the front so hot layouts stay cheap to find.
                if i != 0 {
                    cache.swap(0, i);
                }
                Rc::clone(&cache[0].result)
            })
        }) {
            return hit;
        }

        let result: Rc<[Rect]> = Rc::from(layout.solve(area));
        CACHE.with(|c| {
            let mut cache = c.borrow_mut();
            if cache.len() >= CAPACITY {
                cache.pop();
            }
            cache.insert(
                0,
                Entry {
                    area,
                    direction: layout.direction,
                    margin: layout.margin,
                    flex: layout.flex,
                    spacing: layout.spacing,
                    constraints: layout.constraints.clone().into_boxed_slice(),
                    result: Rc::clone(&result),
                },
            );
        });
        result
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

    #[test]
    fn cached_split_matches_uncached_solve() {
        let layout = Layout::vertical(vec![
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Percentage(25),
        ]);
        for area in [
            Rect::new(0, 0, 80, 24),
            Rect::new(0, 0, 200, 50),
            Rect::new(4, 2, 37, 19),
        ] {
            // Twice, so the second call is served from the memo cache.
            assert_eq!(*layout.split(area), *layout.solve(area));
            assert_eq!(*layout.split(area), *layout.solve(area));
        }
    }

    #[test]
    fn cache_distinguishes_layouts_that_differ_only_slightly() {
        let area = Rect::new(0, 0, 80, 24);
        let a = Layout::vertical(vec![Constraint::Length(3), Constraint::Min(0)]);
        let b = Layout::vertical(vec![Constraint::Length(4), Constraint::Min(0)]);
        let c = Layout::horizontal(vec![Constraint::Length(3), Constraint::Min(0)]);
        let d = Layout::vertical(vec![Constraint::Length(3), Constraint::Min(0)]).spacing(1);

        assert_ne!(a.split(area)[0], b.split(area)[0]);
        assert_ne!(a.split(area)[0], c.split(area)[0]);
        assert_ne!(a.split(area)[1], d.split(area)[1]);
        // Re-splitting after the neighbours were cached still gives `a`'s answer.
        assert_eq!(a.split(area)[0].height, 3);
    }

    #[test]
    fn cache_survives_more_distinct_layouts_than_it_holds() {
        // Push well past the cache capacity, then re-check an early entry.
        let first = Layout::vertical(vec![Constraint::Length(1), Constraint::Min(0)]);
        let area = Rect::new(0, 0, 80, 24);
        let expected = first.solve(area);
        for i in 0..64u16 {
            let l = Layout::vertical(vec![Constraint::Length(i), Constraint::Min(0)]);
            let _ = l.split(Rect::new(0, 0, 80 + i, 24));
        }
        assert_eq!(*first.split(area), *expected);
    }
}
