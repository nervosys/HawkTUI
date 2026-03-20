use super::{AgentCapability, Discoverable, SemanticRole, WidgetSchema};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Global registry of widget types and the live UI tree.
///
/// The registry serves two purposes:
/// 1. **Type catalog**: Agents can list all available widget types, search by
///    name/role/tag, and read schemas before instantiating widgets.
/// 2. **UI tree**: The current widget hierarchy is exposed as a navigable tree
///    so agents can inspect the live application state.
#[derive(Debug, Default)]
pub struct OntologyRegistry {
    schemas: HashMap<String, WidgetSchema>,
    tree: Option<UiTree>,
}

impl OntologyRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    // ── Type Catalog ─────────────────────────────────────────────────

    /// Register a widget type's schema.
    pub fn register_schema(&mut self, schema: WidgetSchema) {
        self.schemas.insert(schema.name.clone(), schema);
    }

    /// Register a discoverable widget type (convenience).
    pub fn register<W: Discoverable>(&mut self) {
        self.register_schema(W::schema());
    }

    /// List all registered widget type names.
    pub fn list_types(&self) -> Vec<&str> {
        self.schemas.keys().map(|s| s.as_str()).collect()
    }

    /// Get the schema for a widget type by name.
    pub fn get_schema(&self, name: &str) -> Option<&WidgetSchema> {
        self.schemas.get(name)
    }

    /// Find widget types matching a semantic role.
    pub fn find_by_role(&self, role: SemanticRole) -> Vec<&WidgetSchema> {
        self.schemas
            .values()
            .filter(|s| s.default_role == role)
            .collect()
    }

    /// Search widget types by tag (case-insensitive substring match).
    pub fn search(&self, query: &str) -> Vec<&WidgetSchema> {
        let query_lower = query.to_lowercase();
        self.schemas
            .values()
            .filter(|s| {
                s.name.to_lowercase().contains(&query_lower)
                    || s.description.to_lowercase().contains(&query_lower)
                    || s.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    /// Export the full type catalog as JSON.
    pub fn export_catalog(&self) -> serde_json::Value {
        serde_json::to_value(&self.schemas).unwrap_or_default()
    }

    // ── Live UI Tree ─────────────────────────────────────────────────

    /// Set the current UI tree snapshot.
    pub fn set_tree(&mut self, tree: UiTree) {
        self.tree = Some(tree);
    }

    /// Get the current UI tree.
    pub fn tree(&self) -> Option<&UiTree> {
        self.tree.as_ref()
    }

    /// Find a node in the UI tree by its agent ID.
    pub fn find_node(&self, agent_id: &str) -> Option<&UiNode> {
        self.tree.as_ref().and_then(|t| t.find(agent_id))
    }

    /// Export the UI tree as JSON for agent consumption.
    pub fn export_tree(&self) -> serde_json::Value {
        match &self.tree {
            Some(tree) => serde_json::to_value(tree).unwrap_or_default(),
            None => serde_json::Value::Null,
        }
    }
}

/// A snapshot of the live UI widget tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiTree {
    pub root: UiNode,
}

impl UiTree {
    pub fn new(root: UiNode) -> Self {
        Self { root }
    }

    /// Depth-first search for a node by agent_id.
    pub fn find(&self, agent_id: &str) -> Option<&UiNode> {
        self.root.find(agent_id)
    }

    /// Collect all nodes matching a role.
    pub fn find_by_role(&self, role: SemanticRole) -> Vec<&UiNode> {
        let mut results = Vec::new();
        self.root.collect_by_role(role, &mut results);
        results
    }

    /// Collect all focusable nodes.
    pub fn focusable_nodes(&self) -> Vec<&UiNode> {
        let mut results = Vec::new();
        self.root.collect_by_capability("focusable", &mut results);
        results
    }
}

/// A node in the UI tree representing a single widget instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiNode {
    /// Optional unique agent-addressable ID.
    pub agent_id: Option<String>,
    /// The widget type name (matches a registered schema).
    pub widget_type: String,
    /// Semantic role of this instance.
    pub role: SemanticRole,
    /// Capabilities of this instance.
    pub capabilities: Vec<AgentCapability>,
    /// Current state snapshot as JSON.
    pub state: serde_json::Value,
    /// Accessibility label.
    pub label: Option<String>,
    /// Bounding rectangle in terminal coordinates.
    pub bounds: Option<NodeBounds>,
    /// Child nodes.
    pub children: Vec<UiNode>,
}

/// Bounding rectangle of a UI node in terminal coordinates.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NodeBounds {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl UiNode {
    pub fn new(widget_type: impl Into<String>, role: SemanticRole) -> Self {
        Self {
            agent_id: None,
            widget_type: widget_type.into(),
            role,
            capabilities: Vec::new(),
            state: serde_json::Value::Null,
            label: None,
            bounds: None,
            children: Vec::new(),
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.agent_id = Some(id.into());
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_bounds(mut self, x: u16, y: u16, width: u16, height: u16) -> Self {
        self.bounds = Some(NodeBounds {
            x,
            y,
            width,
            height,
        });
        self
    }

    pub fn with_capability(mut self, cap: AgentCapability) -> Self {
        self.capabilities.push(cap);
        self
    }

    pub fn with_state(mut self, state: serde_json::Value) -> Self {
        self.state = state;
        self
    }

    pub fn with_child(mut self, child: UiNode) -> Self {
        self.children.push(child);
        self
    }

    fn find(&self, agent_id: &str) -> Option<&UiNode> {
        if self.agent_id.as_deref() == Some(agent_id) {
            return Some(self);
        }
        for child in &self.children {
            if let Some(node) = child.find(agent_id) {
                return Some(node);
            }
        }
        None
    }

    fn collect_by_role<'a>(&'a self, role: SemanticRole, results: &mut Vec<&'a UiNode>) {
        if self.role == role {
            results.push(self);
        }
        for child in &self.children {
            child.collect_by_role(role, results);
        }
    }

    fn collect_by_capability<'a>(&'a self, cap_name: &str, results: &mut Vec<&'a UiNode>) {
        if self.capabilities.iter().any(|c| c.name() == cap_name) {
            results.push(self);
        }
        for child in &self.children {
            child.collect_by_capability(cap_name, results);
        }
    }
}
