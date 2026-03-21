//! JSON-based agent protocol messages.
//!
//! Defines the request/response/event types that AI agents use to communicate
//! with a Louie application. Modeled after pi-mono's RPC protocol.

use serde::{Deserialize, Serialize};

/// The current protocol version.
///
/// Agents should check this value in the ping response to verify compatibility.
/// Incremented when breaking changes are made to the protocol format.
pub const PROTOCOL_VERSION: u32 = 1;

/// A request from an AI agent to the Louie application.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AgentRequest {
    /// Query the ontology type catalog.
    #[serde(rename = "query_ontology")]
    QueryOntology {
        /// Optional filter: search by name, role, or tag.
        #[serde(default)]
        query: Option<String>,
        /// Optional filter: match by semantic role.
        #[serde(default)]
        role: Option<String>,
    },

    /// Get the schema for a specific widget type.
    #[serde(rename = "get_schema")]
    GetSchema {
        /// Widget type name (e.g., "Input", "List").
        widget_type: String,
    },

    /// Get the current UI tree snapshot.
    #[serde(rename = "get_tree")]
    GetTree,

    /// Get the state of a specific widget by its agent ID.
    #[serde(rename = "get_state")]
    GetState {
        /// The agent_id of the widget to query.
        agent_id: String,
    },

    /// Execute an action on a widget.
    #[serde(rename = "execute_action")]
    ExecuteAction {
        /// The agent_id of the target widget.
        agent_id: String,
        /// The action name to execute.
        action: String,
        /// Action parameters.
        #[serde(default)]
        params: serde_json::Value,
    },

    /// Inject an event into the application (keyboard, mouse, etc.).
    #[serde(rename = "inject_event")]
    InjectEvent {
        /// The event to inject, serialized.
        event: InjectedEvent,
    },

    /// Subscribe to application events.
    #[serde(rename = "subscribe")]
    Subscribe {
        /// Event types to subscribe to.
        events: Vec<String>,
    },

    /// Unsubscribe from application events.
    #[serde(rename = "unsubscribe")]
    Unsubscribe {
        /// Event types to unsubscribe from.
        events: Vec<String>,
    },

    /// Ping the application (for keepalive / connection testing).
    #[serde(rename = "ping")]
    Ping,

    /// Request the application to quit.
    #[serde(rename = "quit")]
    Quit,
}

/// An event injected by an agent into the application.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum InjectedEvent {
    /// A key press event.
    #[serde(rename = "key")]
    Key {
        code: String,
        #[serde(default)]
        modifiers: Vec<String>,
    },
    /// A mouse click event.
    #[serde(rename = "mouse_click")]
    MouseClick { x: u16, y: u16, button: String },
    /// A paste event.
    #[serde(rename = "paste")]
    Paste { text: String },
    /// A terminal resize event.
    #[serde(rename = "resize")]
    Resize { width: u16, height: u16 },
}

/// A response from the Louie application to an agent request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    /// Whether the request was successful.
    pub success: bool,
    /// The request ID this response corresponds to (echo-back).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The result payload (on success).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    /// Error message (on failure).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl AgentResponse {
    /// Create a successful response with the given data payload.
    pub fn ok(data: serde_json::Value) -> Self {
        Self {
            success: true,
            id: None,
            data: Some(data),
            error: None,
        }
    }

    /// Create an error response with the given message.
    pub fn err(message: impl Into<String>) -> Self {
        Self {
            success: false,
            id: None,
            data: None,
            error: Some(message.into()),
        }
    }

    /// Attach a request ID for correlation (echo-back to the agent).
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
}

/// An event streamed from the Louie application to a subscribed agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AgentEvent {
    /// Application state changed (a widget was updated).
    #[serde(rename = "state_changed")]
    StateChanged {
        /// The agent_id of the widget whose state changed.
        agent_id: String,
        /// The new state snapshot.
        state: serde_json::Value,
    },

    /// An action was executed and produced a result.
    #[serde(rename = "action_result")]
    ActionResult {
        agent_id: String,
        action: String,
        result: serde_json::Value,
    },

    /// The UI tree was updated (new render frame).
    #[serde(rename = "render_update")]
    RenderUpdate {
        /// The updated UI tree snapshot.
        tree: serde_json::Value,
    },

    /// The application is about to quit.
    #[serde(rename = "app_quit")]
    AppQuit,

    /// A pong response to a ping.
    #[serde(rename = "pong")]
    Pong,

    /// An error occurred processing a subscription.
    #[serde(rename = "error")]
    Error { message: String },
}

/// A framed message with an optional ID for request/response correlation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestEnvelope {
    /// Optional request ID for correlation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The request payload.
    #[serde(flatten)]
    pub request: AgentRequest,
}
