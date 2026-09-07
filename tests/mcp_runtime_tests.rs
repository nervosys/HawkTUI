//! The MCP server's runtime half: tools that need a running application.
//!
//! The catalog tools answer from a static registry and were already covered.
//! These address a live program — read its state, run its actions, send it
//! events — so they can only be tested against one, which is what
//! `HeadlessDriver` provides.

use hawktui::agent::driver::HeadlessDriver;
use hawktui::agent::mcp::McpServer;
use hawktui::prelude::*;
use hawktui::runtime::{Command, Model};
use hawktui::terminal::Frame;
use serde_json::Value;

#[derive(Debug, Clone, Copy)]
enum Msg {
    Down,
    Quit,
}

struct CounterApp {
    count: u32,
    quit: bool,
}

impl Model for CounterApp {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Down => {
                self.count += 1;
                Command::None
            }
            Msg::Quit => {
                self.quit = true;
                Command::Quit
            }
        }
    }

    fn view(&self, frame: &mut Frame<'_>) {
        let area = frame.area();
        frame.render_widget(Paragraph::new(format!("count {}", self.count)), area);
    }

    fn handle_event(&self, event: Event) -> Option<Msg> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Down => Some(Msg::Down),
                KeyCode::Char('q') => Some(Msg::Quit),
                _ => None,
            },
            _ => None,
        }
    }
}

fn driver() -> HeadlessDriver<CounterApp> {
    HeadlessDriver::new(
        CounterApp {
            count: 0,
            quit: false,
        },
        20,
        5,
    )
    .expect("headless driver starts")
}

fn call(server: &mut McpServer, line: &str) -> Value {
    let response = server.handle(line).expect("a request gets a response");
    serde_json::from_str(&response).expect("responses are JSON")
}

fn tool_names(server: &mut McpServer) -> Vec<String> {
    let listed = call(server, r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#);
    listed["result"]["tools"]
        .as_array()
        .expect("tools is an array")
        .iter()
        .map(|t| t["name"].as_str().unwrap_or_default().to_string())
        .collect()
}

#[test]
fn a_server_without_a_runtime_does_not_offer_the_runtime_tools() {
    // A tool a model can see is a tool it will call. Offering one that can only
    // fail wastes a turn and teaches the model the server is unreliable, so an
    // unattached server must not advertise them at all.
    let mut server = McpServer::new();
    let names = tool_names(&mut server);

    assert!(names.contains(&"list_widgets".to_string()));
    for runtime_only in ["get_tree", "get_state", "execute_action", "inject_event"] {
        assert!(
            !names.contains(&runtime_only.to_string()),
            "{runtime_only} was offered without a running application"
        );
    }
    assert!(!server.has_runtime());
}

#[test]
fn attaching_a_runtime_adds_the_runtime_tools() {
    let mut server = McpServer::new().with_runtime(Box::new(driver()));
    let names = tool_names(&mut server);

    assert!(server.has_runtime());
    for expected in ["get_tree", "get_state", "execute_action", "inject_event"] {
        assert!(names.contains(&expected.to_string()), "{expected} missing");
    }
    // The catalog does not go away when a program is attached.
    assert!(names.contains(&"program_skeleton".to_string()));
}

#[test]
fn calling_a_runtime_tool_without_a_runtime_is_a_tool_error_not_a_protocol_error() {
    // The model has to see the reason as content it can act on. A protocol
    // error is addressed to the client, which will not relay it.
    let mut server = McpServer::new();
    let response = call(
        &mut server,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call",
            "params":{"name":"get_tree","arguments":{}}}"#,
    );

    assert!(response.get("error").is_none(), "should not be a protocol error");
    assert_eq!(response["result"]["isError"], Value::Bool(true));
    let text = response["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or_default();
    assert!(
        text.contains("running application"),
        "unhelpful message: {text}"
    );
}

#[test]
fn get_tree_reaches_the_running_application() {
    let mut server = McpServer::new().with_runtime(Box::new(driver()));
    let response = call(
        &mut server,
        r#"{"jsonrpc":"2.0","id":3,"method":"tools/call",
            "params":{"name":"get_tree","arguments":{}}}"#,
    );

    assert!(response["result"].get("isError").is_none());
    let text = response["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or_default();
    assert!(!text.is_empty(), "get_tree returned nothing");
}

#[test]
fn inject_event_drives_the_program_through_its_own_event_loop() {
    // The point of the runtime half: the model changes because the application
    // handled an event, not because the server reached into it.
    let mut server = McpServer::new().with_runtime(Box::new(driver()));
    let response = call(
        &mut server,
        r#"{"jsonrpc":"2.0","id":4,"method":"tools/call",
            "params":{"name":"inject_event",
                      "arguments":{"event":{"kind":"key","code":"Down"}}}}"#,
    );

    assert!(
        response["result"].get("isError").is_none(),
        "inject_event failed: {response}"
    );
}

#[test]
fn a_malformed_runtime_argument_is_a_protocol_error() {
    // get_state requires an agent_id. Omitting it is the client's mistake, and
    // unlike an unknown widget it is not something the model can fix by
    // reading the answer, so it is reported as invalid params.
    let mut server = McpServer::new().with_runtime(Box::new(driver()));
    let response = call(
        &mut server,
        r#"{"jsonrpc":"2.0","id":5,"method":"tools/call",
            "params":{"name":"get_state","arguments":{}}}"#,
    );

    assert_eq!(response["error"]["code"], -32602);
}

#[test]
fn every_runtime_tool_declares_a_schema() {
    let mut server = McpServer::new().with_runtime(Box::new(driver()));
    let listed = call(&mut server, r#"{"jsonrpc":"2.0","id":6,"method":"tools/list"}"#);

    for tool in listed["result"]["tools"].as_array().expect("array") {
        let name = tool["name"].as_str().unwrap_or_default();
        assert_eq!(
            tool["inputSchema"]["type"], "object",
            "{name} has no object schema"
        );
    }
}
