//! # Ontology — Agent Discoverability System
//!
//! The ontology module provides structured metadata for every component in a
//! hawktui application. AI agents can introspect widget schemas, discover
//! available actions, query capabilities, and navigate the UI tree
//! programmatically.
//!
//! ## Design
//!
//! Every widget that implements [`Discoverable`] exposes:
//! - **Schema**: Type name, description, properties with types and constraints
//! - **Capabilities**: What the widget can do (scrollable, editable, focusable, etc.)
//! - **Actions**: Named operations with typed parameters that agents can invoke
//! - **Semantic Role**: The widget's purpose (input, navigation, display, container, etc.)
//!
//! The [`OntologyRegistry`] maintains a global catalog of all registered widget
//! types, enabling agents to list, search, and instantiate widgets by name or role.

mod action;
mod capability;
pub mod registry;
mod schema;

pub use action::{ActionParam, ActionParamType, AgentAction};
pub use capability::AgentCapability;
pub use registry::{OntologyRegistry, UiNode, UiTree};
pub use schema::{PropertyConstraint, PropertySchema, PropertyType, SemanticRole, WidgetSchema};

/// Trait for widgets that expose metadata to agents.
///
/// Any widget implementing this trait becomes discoverable: agents can introspect
/// its schema, invoke actions, and read its state as structured data.
pub trait Discoverable {
    /// Returns the static schema describing this widget type.
    fn schema() -> WidgetSchema
    where
        Self: Sized;

    /// Returns the capabilities of this specific widget instance.
    /// Returns the capabilities of this specific widget instance.
    fn capabilities(&self) -> Vec<AgentCapability>;

    /// Returns the actions available on this specific widget instance.
    fn actions(&self) -> Vec<AgentAction>;

    /// Returns the semantic role of this widget (e.g., TextInput, Display, Navigation).
    fn semantic_role(&self) -> SemanticRole;

    /// Returns the current state as a JSON value for agent inspection.
    fn agent_state(&self) -> serde_json::Value;

    /// Attempt to execute a named action with the given JSON parameters.
    /// Returns Ok with a result value, or Err with a description of the failure.
    fn execute_action(
        &mut self,
        action: &str,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, String>;

    /// An optional unique identifier for this widget instance in the UI tree.
    fn agent_id(&self) -> Option<&str> {
        None
    }

    /// An optional accessibility label for screen readers and agents.
    fn accessibility_label(&self) -> Option<String> {
        None
    }
}
