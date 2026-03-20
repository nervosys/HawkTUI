use serde::{Deserialize, Serialize};

/// An action that an agent can invoke on a widget.
///
/// Actions are the primary way agents interact with widgets. Each action
/// has a name, description, typed parameters, and a return type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAction {
    /// Unique action name (e.g., "select_item", "scroll_to", "set_text").
    pub name: String,
    /// Human-readable description of what this action does.
    pub description: String,
    /// Parameters this action accepts.
    pub params: Vec<ActionParam>,
    /// Description of the return value.
    pub returns: Option<String>,
    /// Whether this action mutates the widget state.
    pub mutates: bool,
    /// Whether this action is idempotent (safe to retry).
    pub idempotent: bool,
    /// Keyboard shortcut, if any (e.g., "Ctrl+A").
    pub shortcut: Option<String>,
}

/// A parameter for an agent action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionParam {
    /// Parameter name.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// The type of this parameter.
    pub param_type: ActionParamType,
    /// Whether this parameter is required.
    pub required: bool,
    /// Default value as JSON, if optional.
    pub default_value: Option<serde_json::Value>,
}

/// Type of an action parameter.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionParamType {
    String,
    Integer,
    Float,
    Boolean,
    Index,
    Position { x: bool, y: bool },
    Enum(Vec<String>),
    Any,
}
