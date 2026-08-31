//! Agent session: processes agent requests against the ontology and model.

use std::collections::HashSet;

use crate::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use crate::ontology::registry::OntologyRegistry;

use super::protocol::{AgentEvent, AgentRequest, AgentResponse, InjectedEvent, PROTOCOL_VERSION};

/// Maximum number of event subscriptions per agent session (INP-3).
const MAX_SUBSCRIPTIONS: usize = 100;

/// Maximum terminal dimension allowed via injected resize events (INP-2).
const MAX_TERMINAL_DIM: u16 = 1024;

/// Manages an agent's connection to a Hawk TUI application.
///
/// The session processes incoming [`AgentRequest`]s, dispatches them against
/// the ontology registry and widget tree, and tracks event subscriptions.
pub struct AgentSession {
    /// Event types this agent has subscribed to.
    subscriptions: HashSet<String>,
}

impl AgentSession {
    pub fn new() -> Self {
        Self {
            subscriptions: HashSet::new(),
        }
    }

    /// Check if the agent is subscribed to a specific event type.
    pub fn is_subscribed(&self, event_type: &str) -> bool {
        self.subscriptions.contains(event_type) || self.subscriptions.contains("*")
    }

    /// Process an incoming agent request and return a response plus optional events.
    ///
    /// Returns `(response, should_quit)`.
    pub fn process_request(
        &mut self,
        request: &AgentRequest,
        registry: &OntologyRegistry,
    ) -> (AgentResponse, bool) {
        match request {
            AgentRequest::QueryOntology { query, role } => {
                let schemas = if let Some(q) = query {
                    registry.search(q)
                } else {
                    // Return all types
                    registry
                        .list_types()
                        .iter()
                        .filter_map(|name| registry.get_schema(name))
                        .collect()
                };

                // Filter by role if specified
                let schemas: Vec<_> = if let Some(role_str) = role {
                    schemas
                        .into_iter()
                        .filter(|s| s.default_role.to_string() == *role_str)
                        .collect()
                } else {
                    schemas
                };

                let data = serde_json::to_value(&schemas).unwrap_or_default();
                (AgentResponse::ok(data), false)
            }

            AgentRequest::GetSchema { widget_type } => match registry.get_schema(widget_type) {
                Some(schema) => {
                    let data = serde_json::to_value(schema).unwrap_or_default();
                    (AgentResponse::ok(data), false)
                }
                None => (
                    AgentResponse::err(format!("Unknown widget type: {widget_type}")),
                    false,
                ),
            },

            AgentRequest::GetTree => {
                let tree = registry.export_tree();
                (AgentResponse::ok(tree), false)
            }

            AgentRequest::GetState { agent_id } => match registry.find_node(agent_id) {
                Some(node) => {
                    let data = serde_json::json!({
                        "agent_id": node.agent_id,
                        "widget_type": node.widget_type,
                        "role": node.role,
                        "state": node.state,
                        "label": node.label,
                        "bounds": node.bounds,
                        "capabilities": node.capabilities,
                    });
                    (AgentResponse::ok(data), false)
                }
                None => (
                    AgentResponse::err(format!("Widget not found: {agent_id}")),
                    false,
                ),
            },

            AgentRequest::ExecuteAction {
                agent_id,
                action,
                params,
            } => {
                // The actual execution is delegated to the runtime/model.
                // The session validates the request against the ontology first (INJ-2).
                match registry.find_node(agent_id) {
                    Some(node) => {
                        // Validate params against the widget schema's declared action, if any.
                        if let Err(e) =
                            registry.validate_action_params(&node.widget_type, action, params)
                        {
                            return (
                                AgentResponse::err(format!(
                                    "Invalid params for {}.{}: {e}",
                                    node.widget_type, action
                                )),
                                false,
                            );
                        }

                        let data = serde_json::json!({
                            "status": "dispatched",
                            "agent_id": agent_id,
                            "action": action,
                            "params": params,
                        });
                        (AgentResponse::ok(data), false)
                    }
                    None => (
                        AgentResponse::err(format!("Widget not found: {agent_id}")),
                        false,
                    ),
                }
            }

            AgentRequest::InjectEvent { event: _ } => {
                // Events are converted and injected by the driver/runtime.
                (
                    AgentResponse::ok(serde_json::json!({"status": "injected"})),
                    false,
                )
            }

            AgentRequest::Subscribe { events } => {
                for ev in events {
                    if self.subscriptions.len() >= MAX_SUBSCRIPTIONS {
                        return (
                            AgentResponse::err(format!(
                                "Subscription limit reached (max {MAX_SUBSCRIPTIONS})"
                            )),
                            false,
                        );
                    }
                    // Reject oversized event type names
                    if ev.len() > 256 {
                        continue;
                    }
                    self.subscriptions.insert(ev.clone());
                }
                let data = serde_json::json!({
                    "subscriptions": self.subscriptions.iter().collect::<Vec<_>>()
                });
                (AgentResponse::ok(data), false)
            }

            AgentRequest::Unsubscribe { events } => {
                for ev in events {
                    self.subscriptions.remove(ev);
                }
                let data = serde_json::json!({
                    "subscriptions": self.subscriptions.iter().collect::<Vec<_>>()
                });
                (AgentResponse::ok(data), false)
            }

            AgentRequest::Ping => (
                AgentResponse::ok(serde_json::json!({
                    "status": "pong",
                    "protocol_version": PROTOCOL_VERSION,
                })),
                false,
            ),

            AgentRequest::Quit => (
                AgentResponse::ok(serde_json::json!({"status": "quitting"})),
                true,
            ),
        }
    }

    /// Convert an [`InjectedEvent`] into a hawktui [`Event`].
    pub fn convert_injected_event(injected: &InjectedEvent) -> Option<Event> {
        match injected {
            InjectedEvent::Key { code, modifiers } => {
                let key_code = parse_key_code(code)?;
                let mut mods = KeyModifiers::NONE;
                for m in modifiers {
                    match m.to_lowercase().as_str() {
                        "shift" => mods |= KeyModifiers::SHIFT,
                        "ctrl" | "control" => mods |= KeyModifiers::CONTROL,
                        "alt" => mods |= KeyModifiers::ALT,
                        "super" => mods |= KeyModifiers::SUPER,
                        _ => {}
                    }
                }
                Some(Event::Key(KeyEvent::new(key_code, mods)))
            }
            InjectedEvent::MouseClick { x, y, button } => {
                let btn = match button.to_lowercase().as_str() {
                    "left" => MouseButton::Left,
                    "right" => MouseButton::Right,
                    "middle" => MouseButton::Middle,
                    _ => MouseButton::Left,
                };
                Some(Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Down(btn),
                    column: *x,
                    row: *y,
                    modifiers: KeyModifiers::NONE,
                }))
            }
            InjectedEvent::Paste { text } => Some(Event::Paste(text.clone())),
            InjectedEvent::Resize { width, height } => {
                let w = (*width).clamp(1, MAX_TERMINAL_DIM);
                let h = (*height).clamp(1, MAX_TERMINAL_DIM);
                Some(Event::Resize(w, h))
            }
        }
    }

    /// Generate an [`AgentEvent`] for a state change, if the agent is subscribed.
    pub fn emit_state_changed(
        &self,
        agent_id: &str,
        state: serde_json::Value,
    ) -> Option<AgentEvent> {
        if self.is_subscribed("state_changed") || self.is_subscribed("*") {
            Some(AgentEvent::StateChanged {
                agent_id: agent_id.to_string(),
                state,
            })
        } else {
            None
        }
    }

    /// Generate an [`AgentEvent`] for a render update, if subscribed.
    pub fn emit_render_update(&self, tree: serde_json::Value) -> Option<AgentEvent> {
        if self.is_subscribed("render_update") || self.is_subscribed("*") {
            Some(AgentEvent::RenderUpdate { tree })
        } else {
            None
        }
    }
}

impl Default for AgentSession {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a key code string into a [`KeyCode`].
fn parse_key_code(s: &str) -> Option<KeyCode> {
    match s.to_lowercase().as_str() {
        "enter" | "return" => Some(KeyCode::Enter),
        "backspace" => Some(KeyCode::Backspace),
        "tab" => Some(KeyCode::Tab),
        "backtab" => Some(KeyCode::BackTab),
        "esc" | "escape" => Some(KeyCode::Esc),
        "left" => Some(KeyCode::Left),
        "right" => Some(KeyCode::Right),
        "up" => Some(KeyCode::Up),
        "down" => Some(KeyCode::Down),
        "home" => Some(KeyCode::Home),
        "end" => Some(KeyCode::End),
        "pageup" => Some(KeyCode::PageUp),
        "pagedown" => Some(KeyCode::PageDown),
        "insert" => Some(KeyCode::Insert),
        "delete" => Some(KeyCode::Delete),
        "space" => Some(KeyCode::Char(' ')),
        s if s.starts_with('f') && s.len() > 1 => s[1..].parse::<u8>().ok().map(KeyCode::F),
        s if s.chars().count() == 1 => s.chars().next().map(KeyCode::Char),
        _ => None,
    }
}
