use serde::{Deserialize, Serialize};

/// A capability that a widget instance advertises.
///
/// Agents query capabilities to understand what interactions are possible
/// with a given widget without trial-and-error.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentCapability {
    /// Widget can receive keyboard focus.
    Focusable,
    /// Widget responds to mouse clicks.
    Clickable,
    /// Widget can be scrolled (vertical, horizontal, or both).
    Scrollable { vertical: bool, horizontal: bool },
    /// Widget accepts text input.
    TextInput {
        multiline: bool,
        max_length: Option<usize>,
    },
    /// Widget supports item selection (single or multi).
    Selectable {
        multi_select: bool,
        item_count: usize,
    },
    /// Widget can be expanded or collapsed.
    Expandable { expanded: bool },
    /// Widget supports drag-and-drop.
    Draggable,
    /// Widget acts as a drop target.
    DropTarget,
    /// Widget supports resizing.
    Resizable {
        min_width: Option<u16>,
        min_height: Option<u16>,
    },
    /// Widget has an animation in progress.
    Animated { playing: bool },
    /// Widget supports value editing within a range.
    RangeEditable {
        min: f64,
        max: f64,
        step: Option<f64>,
    },
    /// Widget can be toggled on/off.
    Toggleable { state: bool },
    /// Widget supports sorting.
    Sortable { columns: Vec<String> },
    /// Widget supports filtering.
    Filterable,
    /// Widget supports search.
    Searchable,
    /// Widget supports copy-to-clipboard.
    Copyable,
    /// Widget provides hover tooltips.
    HasTooltip,
    /// Widget supports keyboard shortcuts.
    HasKeyBindings { bindings: Vec<(String, String)> },
    /// Custom capability with a name tag.
    Custom(String),
}

impl AgentCapability {
    /// Returns the canonical name of this capability for querying.
    pub fn name(&self) -> &str {
        match self {
            Self::Focusable => "focusable",
            Self::Clickable => "clickable",
            Self::Scrollable { .. } => "scrollable",
            Self::TextInput { .. } => "text-input",
            Self::Selectable { .. } => "selectable",
            Self::Expandable { .. } => "expandable",
            Self::Draggable => "draggable",
            Self::DropTarget => "drop-target",
            Self::Resizable { .. } => "resizable",
            Self::Animated { .. } => "animated",
            Self::RangeEditable { .. } => "range-editable",
            Self::Toggleable { .. } => "toggleable",
            Self::Sortable { .. } => "sortable",
            Self::Filterable => "filterable",
            Self::Searchable => "searchable",
            Self::Copyable => "copyable",
            Self::HasTooltip => "has-tooltip",
            Self::HasKeyBindings { .. } => "has-key-bindings",
            Self::Custom(name) => name.as_str(),
        }
    }
}
