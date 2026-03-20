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

impl AgentAction {
    /// Validate a JSON params object against this action's declared parameters.
    ///
    /// Returns `Ok(())` if the params are valid, or `Err(description)` on the
    /// first validation failure.
    pub fn validate_params(&self, params: &serde_json::Value) -> Result<(), String> {
        for param in &self.params {
            let val = params.get(&param.name);
            match val {
                None | Some(serde_json::Value::Null) => {
                    if param.required {
                        return Err(format!("Missing required parameter '{}'", param.name));
                    }
                }
                Some(v) => {
                    param
                        .param_type
                        .check(v)
                        .map_err(|e| format!("Parameter '{}': {}", param.name, e))?;
                }
            }
        }
        Ok(())
    }
}

impl ActionParamType {
    /// Check that a JSON value matches this expected type.
    fn check(&self, value: &serde_json::Value) -> Result<(), String> {
        match self {
            ActionParamType::String => {
                if !value.is_string() {
                    return Err(format!("expected string, got {}", json_type_name(value)));
                }
            }
            ActionParamType::Integer => {
                if !value.is_i64() && !value.is_u64() {
                    return Err(format!("expected integer, got {}", json_type_name(value)));
                }
            }
            ActionParamType::Float => {
                if !value.is_number() {
                    return Err(format!("expected number, got {}", json_type_name(value)));
                }
            }
            ActionParamType::Boolean => {
                if !value.is_boolean() {
                    return Err(format!("expected boolean, got {}", json_type_name(value)));
                }
            }
            ActionParamType::Index => {
                if !value.is_u64() {
                    return Err(format!(
                        "expected non-negative integer, got {}",
                        json_type_name(value)
                    ));
                }
            }
            ActionParamType::Position { .. } => {
                if !value.is_object() {
                    return Err(format!("expected object, got {}", json_type_name(value)));
                }
            }
            ActionParamType::Enum(variants) => {
                if let Some(s) = value.as_str() {
                    if !variants.iter().any(|v| v == s) {
                        return Err(format!(
                            "expected one of [{}], got {:?}",
                            variants.join(", "),
                            s
                        ));
                    }
                } else {
                    return Err(format!(
                        "expected string enum, got {}",
                        json_type_name(value)
                    ));
                }
            }
            ActionParamType::Any => {}
        }
        Ok(())
    }
}

fn json_type_name(v: &serde_json::Value) -> &'static str {
    match v {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}
