//! Model Context Protocol server over the widget ontology.
//!
//! `hawktui-server` speaks a hand-rolled JSON Lines protocol that an integrator
//! has to implement before an agent can use it. Agent platforms already speak
//! MCP, so this exposes the same catalog as MCP tools and turns Hawk TUI from
//! "a protocol you must implement" into "a server your agent already knows how
//! to call".
//!
//! Transport is JSON-RPC 2.0 over stdio, one message per line. [`McpServer`] is
//! transport-free — hand it a line, get a response line back — so it can be
//! unit-tested without spawning a process.
//!
//! ```
//! use hawktui::agent::mcp::McpServer;
//!
//! let mut server = McpServer::new();
//! let response = server
//!     .handle(r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#)
//!     .expect("a request gets a response");
//! assert!(response.contains("get_widget_schema"));
//!
//! // Notifications get no reply, per JSON-RPC.
//! assert!(server
//!     .handle(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#)
//!     .is_none());
//! ```

use serde_json::{json, Value};

use crate::ontology::{builtin_registry, registry::OntologyRegistry, report};

/// The MCP revision this server implements when a client does not name one.
pub const DEFAULT_PROTOCOL_VERSION: &str = "2025-06-18";

/// Protocol revisions this server will accept if a client asks for them.
const SUPPORTED_PROTOCOL_VERSIONS: &[&str] = &["2025-06-18", "2025-03-26", "2024-11-05"];

const PARSE_ERROR: i64 = -32700;
const INVALID_REQUEST: i64 = -32600;
const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_PARAMS: i64 = -32602;

/// A tool this server exposes, and the arguments it takes.
struct Tool {
    name: &'static str,
    description: &'static str,
    /// Name of the single required string argument, if the tool takes one.
    argument: Option<(&'static str, &'static str)>,
}

const TOOLS: &[Tool] = &[
    Tool {
        name: "list_widgets",
        description: "List every widget type with its semantic role and description.",
        argument: None,
    },
    Tool {
        name: "get_widget_schema",
        description: "Full schema for one widget type: properties with types and \
                      constraints, actions, and a usage hint.",
        argument: Some(("name", "Widget type name, e.g. \"Gauge\".")),
    },
    Tool {
        name: "search_widgets",
        description: "Find widget types matching a name, description or tag.",
        argument: Some(("query", "Search term, e.g. \"scroll\".")),
    },
    Tool {
        name: "widget_roles",
        description: "Widget types grouped by semantic role, for finding a widget when \
                      you know what it should do but not what it is called.",
        argument: None,
    },
    Tool {
        name: "program_skeleton",
        description: "Call this FIRST, before writing any Hawk TUI program. Returns a complete minimal program that compiles and passes its own test: the Model trait's three required methods, how a Program is started, how a stateful widget is rendered with its state, and how to drive it headlessly. Start from this rather than reconstructing the skeleton from the source.",
        argument: None,
    },
    Tool {
        name: "prelude",
        description: "What `use hawktui::prelude::*` brings into scope, and what it does not — Program, Model, Command and ProgramOptions come from hawktui::runtime instead. Call this when an import will not resolve.",
        argument: None,
    },
    Tool {
        name: "widget_api",
        description: "Call this BEFORE writing the first line of code that uses a Hawk TUI type, and again whenever a build fails on a missing method. Returns its constructors and builder methods with full signatures, its enum variants, whether it renders as Widget or StatefulWidget and with which state type, and the exact render call. Guessing a signature costs a build cycle; this does not.",
        argument: Some(("name", "Type name, e.g. \"List\", \"Layout\", \"Constraint\".")),
    },
    Tool {
        name: "api_search",
        description: "Call this when you know what a thing should DO but not what Hawk TUI calls it, before assuming a name from another framework. Searches names, modules, summaries and method names across widgets and the core types a program is built from (Layout, Constraint, Rect, Style, Text).",
        argument: Some(("query", "Search term, e.g. \"constraint\" or \"highlight\".")),
    },
    Tool {
        name: "stateful_widgets",
        description: "Call this before rendering any list, table, editor or scrollbar. Six widgets need a companion state value and must be drawn with render_stateful_widget rather than render_widget; using the wrong one is the most common mistake made against this framework. Returns each widget with its state type.",
        argument: None,
    },
    Tool {
        name: "ontology_digest",
        description: "A compact cheatsheet of every widget and its properties.",
        argument: None,
    },
];

fn tool_schema(tool: &Tool) -> Value {
    match tool.argument {
        Some((name, description)) => json!({
            "type": "object",
            "properties": { name: { "type": "string", "description": description } },
            "required": [name],
        }),
        None => json!({ "type": "object", "properties": {} }),
    }
}

/// An MCP server over the Hawk TUI widget ontology.
pub struct McpServer {
    registry: OntologyRegistry,
    initialized: bool,
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

impl McpServer {
    /// A server over every built-in widget.
    pub fn new() -> Self {
        Self {
            registry: builtin_registry(),
            initialized: false,
        }
    }

    /// A server over a registry you assembled yourself.
    pub fn with_registry(registry: OntologyRegistry) -> Self {
        Self {
            registry,
            initialized: false,
        }
    }

    /// Whether the client has completed the initialize handshake.
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Handle one JSON-RPC line.
    ///
    /// Returns the response line, or `None` for a notification — which by
    /// JSON-RPC must not be answered.
    pub fn handle(&mut self, line: &str) -> Option<String> {
        let message: Value = match serde_json::from_str(line) {
            Ok(value) => value,
            // No id is recoverable from unparseable input, so the error is
            // reported against a null id, as JSON-RPC requires.
            Err(e) => return Some(error_response(Value::Null, PARSE_ERROR, &e.to_string())),
        };

        let id = message.get("id").cloned();
        let Some(method) = message.get("method").and_then(Value::as_str) else {
            return Some(error_response(
                id.unwrap_or(Value::Null),
                INVALID_REQUEST,
                "missing \"method\"",
            ));
        };
        let params = message.get("params").cloned().unwrap_or(json!({}));

        // A message with no id is a notification: act on it, answer nothing.
        let Some(id) = id else {
            if method == "notifications/initialized" {
                self.initialized = true;
            }
            return None;
        };

        Some(match method {
            "initialize" => success(id, self.initialize(&params)),
            "ping" => success(id, json!({})),
            "tools/list" => success(id, self.tools_list()),
            "tools/call" => match self.tools_call(&params) {
                Ok(result) => success(id, result),
                Err(McpError::InvalidParams(message)) => {
                    error_response(id, INVALID_PARAMS, &message)
                }
                // A tool that ran and failed is a successful call reporting an
                // error, not a protocol error — the model needs to see the text.
                Err(McpError::Tool(message)) => success(
                    id,
                    json!({
                        "content": [{ "type": "text", "text": message }],
                        "isError": true,
                    }),
                ),
            },
            other => error_response(id, METHOD_NOT_FOUND, &format!("unknown method {other:?}")),
        })
    }

    fn initialize(&mut self, params: &Value) -> Value {
        let requested = params.get("protocolVersion").and_then(Value::as_str);
        let version = match requested {
            Some(v) if SUPPORTED_PROTOCOL_VERSIONS.contains(&v) => v,
            _ => DEFAULT_PROTOCOL_VERSION,
        };
        json!({
            "protocolVersion": version,
            "capabilities": { "tools": { "listChanged": false } },
            "serverInfo": {
                "name": "hawktui-ontology",
                "version": env!("CARGO_PKG_VERSION"),
            },
            "instructions": "The Hawk TUI ontology. Start any new program with program_skeleton, which returns a complete compiling example. To *write* code, use widget_api, \n api_search and stateful_widgets: they give constructors, \n builder signatures, and which widgets need a companion \n state value. To inspect a *running* application, use \n list_widgets, get_widget_schema and widget_roles, which \n describe runtime state and semantic roles.",
        })
    }

    fn tools_list(&self) -> Value {
        let tools: Vec<Value> = TOOLS
            .iter()
            .map(|tool| {
                json!({
                    "name": tool.name,
                    "description": tool.description,
                    "inputSchema": tool_schema(tool),
                })
            })
            .collect();
        json!({ "tools": tools })
    }

    fn tools_call(&self, params: &Value) -> Result<Value, McpError> {
        let name = params
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| McpError::InvalidParams("missing tool \"name\"".into()))?;
        let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

        let tool = TOOLS
            .iter()
            .find(|t| t.name == name)
            .ok_or_else(|| McpError::Tool(format!("unknown tool {name:?}")))?;

        let argument = match tool.argument {
            Some((key, _)) => Some(
                arguments
                    .get(key)
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        McpError::InvalidParams(format!("{name} requires a string {key:?}"))
                    })?
                    .to_string(),
            ),
            None => None,
        };

        let text = match name {
            "list_widgets" => report::list(&self.registry),
            "stateful_widgets" => report::stateful(),
            "program_skeleton" => report::skeleton(),
            "prelude" => report::prelude(),
            "widget_api" => {
                let name = argument.expect("widget_api declares an argument");
                report::api(&name).ok_or_else(|| {
                    McpError::Tool(format!("unknown type {name:?}; try api_search"))
                })?
            }
            "api_search" => {
                let query = argument.expect("api_search declares an argument");
                let hits = report::api_search(&query);
                if hits.is_empty() {
                    return Err(McpError::Tool(format!("nothing matches {query:?}")));
                }
                hits
            }
            "widget_roles" => report::roles(&self.registry),
            "ontology_digest" => report::digest(&self.registry),
            "search_widgets" => {
                let query = argument.expect("search_widgets declares an argument");
                let hits = report::search(&self.registry, &query);
                if hits.is_empty() {
                    return Err(McpError::Tool(format!(
                        "no widget matches {query:?}; try list_widgets"
                    )));
                }
                hits
            }
            "get_widget_schema" => {
                let widget = argument.expect("get_widget_schema declares an argument");
                report::schema(&self.registry, &widget).ok_or_else(|| {
                    McpError::Tool(format!("unknown widget {widget:?}; try list_widgets"))
                })?
            }
            _ => unreachable!("tool was looked up in TOOLS"),
        };

        Ok(json!({ "content": [{ "type": "text", "text": text }] }))
    }
}

enum McpError {
    /// The request was malformed — a protocol-level error.
    InvalidParams(String),
    /// The tool ran and could not answer — reported to the model as content.
    Tool(String),
}

fn success(id: Value, result: Value) -> String {
    json!({ "jsonrpc": "2.0", "id": id, "result": result }).to_string()
}

fn error_response(id: Value, code: i64, message: &str) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message },
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(server: &mut McpServer, line: &str) -> Value {
        let raw = server.handle(line).expect("request gets a response");
        serde_json::from_str(&raw).expect("response is JSON")
    }

    fn text_of(response: &Value) -> String {
        response["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .to_string()
    }

    #[test]
    fn initialize_reports_tool_capability_and_echoes_a_known_version() {
        let mut server = McpServer::new();
        let response = call(
            &mut server,
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize",
                "params":{"protocolVersion":"2024-11-05"}}"#,
        );
        assert_eq!(response["result"]["protocolVersion"], "2024-11-05");
        assert!(response["result"]["capabilities"]["tools"].is_object());
        assert_eq!(response["result"]["serverInfo"]["name"], "hawktui-ontology");
    }

    #[test]
    fn an_unknown_protocol_version_falls_back_to_ours() {
        let mut server = McpServer::new();
        let response = call(
            &mut server,
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize",
                "params":{"protocolVersion":"1999-01-01"}}"#,
        );
        assert_eq!(
            response["result"]["protocolVersion"],
            DEFAULT_PROTOCOL_VERSION
        );
    }

    #[test]
    fn notifications_get_no_response_but_are_acted_on() {
        let mut server = McpServer::new();
        assert!(!server.is_initialized());
        assert!(server
            .handle(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#)
            .is_none());
        assert!(server.is_initialized());
    }

    #[test]
    fn tools_list_declares_every_tool_with_a_schema() {
        let mut server = McpServer::new();
        let response = call(
            &mut server,
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
        );
        let tools = response["result"]["tools"].as_array().expect("tools array");
        assert_eq!(tools.len(), TOOLS.len());
        for tool in tools {
            assert!(tool["name"].is_string());
            assert!(!tool["description"].as_str().unwrap_or_default().is_empty());
            assert_eq!(tool["inputSchema"]["type"], "object");
        }
    }

    #[test]
    fn list_widgets_returns_the_catalog() {
        let mut server = McpServer::new();
        let response = call(
            &mut server,
            r#"{"jsonrpc":"2.0","id":3,"method":"tools/call",
                "params":{"name":"list_widgets","arguments":{}}}"#,
        );
        let text = text_of(&response);
        assert!(text.contains("Gauge"), "{text}");
        assert_eq!(text.lines().count(), 21);
    }

    #[test]
    fn get_widget_schema_returns_one_widget() {
        let mut server = McpServer::new();
        let response = call(
            &mut server,
            r#"{"jsonrpc":"2.0","id":4,"method":"tools/call",
                "params":{"name":"get_widget_schema","arguments":{"name":"Gauge"}}}"#,
        );
        let text = text_of(&response);
        assert!(text.contains("constraint: Max(1.0)"), "{text}");
    }

    #[test]
    fn an_unknown_widget_is_a_tool_error_not_a_protocol_error() {
        let mut server = McpServer::new();
        let response = call(
            &mut server,
            r#"{"jsonrpc":"2.0","id":5,"method":"tools/call",
                "params":{"name":"get_widget_schema","arguments":{"name":"Nope"}}}"#,
        );
        assert!(
            response["error"].is_null(),
            "should not be a protocol error"
        );
        assert_eq!(response["result"]["isError"], true);
        assert!(text_of(&response).contains("unknown widget"));
    }

    #[test]
    fn a_missing_argument_is_a_protocol_error() {
        let mut server = McpServer::new();
        let response = call(
            &mut server,
            r#"{"jsonrpc":"2.0","id":6,"method":"tools/call",
                "params":{"name":"search_widgets","arguments":{}}}"#,
        );
        assert_eq!(response["error"]["code"], INVALID_PARAMS);
    }

    #[test]
    fn an_unknown_method_is_method_not_found() {
        let mut server = McpServer::new();
        let response = call(
            &mut server,
            r#"{"jsonrpc":"2.0","id":7,"method":"resources/list"}"#,
        );
        assert_eq!(response["error"]["code"], METHOD_NOT_FOUND);
    }

    #[test]
    fn malformed_json_is_a_parse_error_against_a_null_id() {
        let mut server = McpServer::new();
        let response = call(&mut server, "{not json");
        assert_eq!(response["error"]["code"], PARSE_ERROR);
        assert!(response["id"].is_null());
    }

    #[test]
    fn search_that_matches_nothing_reports_a_tool_error() {
        let mut server = McpServer::new();
        let response = call(
            &mut server,
            r#"{"jsonrpc":"2.0","id":8,"method":"tools/call",
                "params":{"name":"search_widgets","arguments":{"query":"zzzznope"}}}"#,
        );
        assert_eq!(response["result"]["isError"], true);
    }

    #[test]
    fn every_response_carries_the_request_id() {
        let mut server = McpServer::new();
        let response = call(
            &mut server,
            r#"{"jsonrpc":"2.0","id":"abc","method":"ping"}"#,
        );
        assert_eq!(response["id"], "abc");
        assert_eq!(response["jsonrpc"], "2.0");
    }
}
