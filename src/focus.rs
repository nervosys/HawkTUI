//! Focus management system.
//!
//! Provides a focus ring for ordered keyboard navigation between widgets,
//! and programmatic focus control for agents.

/// Manages keyboard focus across a set of focusable widgets.
///
/// Widgets register themselves by agent_id. The focus ring maintains an
/// ordered list for Tab/Shift+Tab navigation. Agents can also focus
/// any widget directly by ID.
#[derive(Debug, Clone, Default)]
pub struct FocusManager {
    /// Ordered list of focusable widget IDs.
    ring: Vec<String>,
    /// Index of the currently focused widget (None = nothing focused).
    focused: Option<usize>,
}

impl FocusManager {
    /// Create a new empty focus manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all registered widgets (call before each frame rebuild).
    pub fn clear(&mut self) {
        self.ring.clear();
        self.focused = None;
    }

    /// Register a focusable widget. Order of registration determines tab order.
    pub fn register(&mut self, agent_id: impl Into<String>) {
        self.ring.push(agent_id.into());
    }

    /// The agent_id of the currently focused widget, if any.
    pub fn focused_id(&self) -> Option<&str> {
        self.focused
            .and_then(|i| self.ring.get(i))
            .map(|s| s.as_str())
    }

    /// Whether the given widget ID is currently focused.
    pub fn is_focused(&self, agent_id: &str) -> bool {
        self.focused_id() == Some(agent_id)
    }

    /// Move focus to the next widget in the ring (Tab).
    pub fn focus_next(&mut self) {
        if self.ring.is_empty() {
            return;
        }
        self.focused = Some(match self.focused {
            Some(i) => (i + 1) % self.ring.len(),
            None => 0,
        });
    }

    /// Move focus to the previous widget in the ring (Shift+Tab).
    pub fn focus_previous(&mut self) {
        if self.ring.is_empty() {
            return;
        }
        self.focused = Some(match self.focused {
            Some(0) => self.ring.len() - 1,
            Some(i) => i - 1,
            None => self.ring.len() - 1,
        });
    }

    /// Focus a specific widget by agent_id. Returns true if found.
    pub fn focus_id(&mut self, agent_id: &str) -> bool {
        if let Some(i) = self.ring.iter().position(|id| id == agent_id) {
            self.focused = Some(i);
            true
        } else {
            false
        }
    }

    /// Remove focus entirely.
    pub fn blur(&mut self) {
        self.focused = None;
    }

    /// Number of registered focusable widgets.
    pub fn len(&self) -> usize {
        self.ring.len()
    }

    /// Whether the focus ring is empty.
    pub fn is_empty(&self) -> bool {
        self.ring.is_empty()
    }

    /// Get all registered widget IDs in tab order.
    pub fn ids(&self) -> &[String] {
        &self.ring
    }
}
