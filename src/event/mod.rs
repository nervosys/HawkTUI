//! Event system with keyboard, mouse, and resize events.
//!
//! Provides a unified event type for the runtime, with full mouse support
//! including click, drag, scroll, and hover tracking.

use crate::core::rect::Position;
use serde::{Deserialize, Serialize};

/// A terminal event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// Keyboard event.
    Key(KeyEvent),
    /// Mouse event.
    Mouse(MouseEvent),
    /// Terminal was resized to (columns, rows).
    Resize(u16, u16),
    /// Terminal gained focus.
    FocusGained,
    /// Terminal lost focus.
    FocusLost,
    /// Text was pasted (bracketed paste mode).
    Paste(String),
    /// A scheduled tick from the runtime (for animations).
    Tick,
}

/// A keyboard event.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
    pub kind: KeyEventKind,
}

impl KeyEvent {
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self {
            code,
            modifiers,
            kind: KeyEventKind::Press,
        }
    }

    /// Convenience: is this a Ctrl+<key> press?
    pub fn is_ctrl(&self, code: KeyCode) -> bool {
        self.code == code && self.modifiers.contains(KeyModifiers::CONTROL)
    }

    /// Convenience: matches a key code with no modifiers.
    pub fn is_key(&self, code: KeyCode) -> bool {
        self.code == code && self.modifiers.is_empty()
    }
}

/// The kind of key event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyEventKind {
    Press,
    Release,
    Repeat,
}

/// Key code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyCode {
    Char(char),
    F(u8),
    Backspace,
    Enter,
    Tab,
    BackTab,
    Esc,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    Delete,
    Null,
}

/// Key modifier flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct KeyModifiers(u8);

impl KeyModifiers {
    pub const NONE: Self = Self(0);
    pub const SHIFT: Self = Self(1 << 0);
    pub const CONTROL: Self = Self(1 << 1);
    pub const ALT: Self = Self(1 << 2);
    pub const SUPER: Self = Self(1 << 3);
    pub const HYPER: Self = Self(1 << 4);
    pub const META: Self = Self(1 << 5);

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl std::ops::BitOr for KeyModifiers {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for KeyModifiers {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// A mouse event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MouseEvent {
    pub kind: MouseEventKind,
    pub column: u16,
    pub row: u16,
    pub modifiers: KeyModifiers,
}

impl MouseEvent {
    /// Whether this is a click (button down) event.
    pub fn is_click(&self) -> bool {
        matches!(self.kind, MouseEventKind::Down(_))
    }

    /// Whether this is a drag event.
    pub fn is_drag(&self) -> bool {
        matches!(self.kind, MouseEventKind::Drag(_))
    }

    /// Whether this is a scroll event.
    pub fn is_scroll(&self) -> bool {
        matches!(
            self.kind,
            MouseEventKind::ScrollUp | MouseEventKind::ScrollDown
        )
    }

    /// Whether the given rect was clicked.
    pub fn clicked_in(&self, area: crate::core::rect::Rect) -> bool {
        self.is_click()
            && area.contains(Position::new(self.column, self.row))
    }
}

/// The kind of mouse event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseEventKind {
    Down(MouseButton),
    Up(MouseButton),
    Drag(MouseButton),
    Moved,
    ScrollUp,
    ScrollDown,
    ScrollLeft,
    ScrollRight,
}

/// Mouse button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    None,
}

/// Hit-test result: which widget (by agent_id) was targeted.
#[derive(Debug, Clone)]
pub struct HitTestResult {
    pub agent_id: String,
    pub local_position: Position,
}

/// Manages hit-testing for clickable regions.
///
/// Widgets register their clickable areas during rendering, and the event
/// system resolves mouse positions to specific widget instances.
#[derive(Debug, Default)]
pub struct HitMap {
    regions: Vec<HitRegion>,
}

#[derive(Debug, Clone)]
struct HitRegion {
    agent_id: String,
    area: crate::core::rect::Rect,
    z_index: u16,
}

impl HitMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all registered regions (called before each frame).
    pub fn clear(&mut self) {
        self.regions.clear();
    }

    /// Register a clickable region.
    pub fn register(
        &mut self,
        agent_id: impl Into<String>,
        area: crate::core::rect::Rect,
        z_index: u16,
    ) {
        self.regions.push(HitRegion {
            agent_id: agent_id.into(),
            area,
            z_index,
        });
    }

    /// Resolve a position to the topmost widget that contains it.
    pub fn hit_test(&self, x: u16, y: u16) -> Option<HitTestResult> {
        let pos = Position::new(x, y);
        self.regions
            .iter()
            .filter(|r| r.area.contains(pos))
            .max_by_key(|r| r.z_index)
            .map(|r| HitTestResult {
                agent_id: r.agent_id.clone(),
                local_position: Position::new(
                    pos.x.saturating_sub(r.area.x),
                    pos.y.saturating_sub(r.area.y),
                ),
            })
    }
}
