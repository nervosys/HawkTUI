# Agent Integration Guide

How to connect an AI agent (OpenCode, Pi, OpenClaw, or any LLM-based system) to a
Hawk TUI application via the JSON Lines RPC protocol.

## Overview

Hawk TUI exposes a JSON Lines (JSONL) protocol over stdin/stdout that lets agents:

1. **Discover** the UI — query the ontology for widget types, schemas, capabilities
2. **Inspect** state — read widget state and the UI tree snapshot
3. **Act** — execute actions on widgets, inject keyboard/mouse events
4. **Subscribe** — receive notifications when state changes

No terminal is needed. The agent spawns the Hawk TUI app as a child process and
communicates entirely through structured JSON messages.

## Transport

- **Protocol**: One JSON object per line, delimited by `\n`
- **Direction**:
  - Agent → App: `RequestEnvelope` on stdin
  - App → Agent: `AgentResponse` on stdout
- **Encoding**: UTF-8
- **Correlation**: Requests include an optional `"id"` field; responses echo it back

## Message Format

### Request Envelope

Every request is wrapped in an envelope with an optional ID for correlation:

```json
{
  "id": "req-1",
  "type": "<request_type>",
  ...fields...
}
```

### Response

```json
{
  "success": true,
  "id": "req-1",
  "data": { ... }
}
```

On error:

```json
{
  "success": false,
  "id": "req-1",
  "error": "Widget not found: editor-1"
}
```

## Request Types

### `ping` — Connection Test

```json
{"type": "ping"}
```

Response: `{"success": true, "data": {"status": "pong"}}`

### `query_ontology` — Discover Widget Types

List all registered widget types, optionally filtered:

```json
{"type": "query_ontology"}
{"type": "query_ontology", "query": "Input"}
{"type": "query_ontology", "role": "Input"}
```

Response contains an array of `WidgetSchema` objects with type name, description,
properties (name/type/constraints), capabilities, and default semantic role.

### `get_schema` — Get Single Widget Schema

```json
{"type": "get_schema", "widget_type": "Editor"}
```

### `get_tree` — Get UI Tree Snapshot

```json
{"type": "get_tree"}
```

Returns the full UI node tree with each widget's agent_id, type, role, state,
label, bounds, and capabilities.

### `get_state` — Get Widget State

```json
{"type": "get_state", "agent_id": "editor-1"}
```

Returns the widget's current state, bounds, capabilities, and label.

### `execute_action` — Invoke Widget Action

```json
{
  "type": "execute_action",
  "agent_id": "editor-1",
  "action": "insert_text",
  "params": {"text": "hello world"}
}
```

Action names and parameters are defined in each widget's `AgentAction` list
(discoverable via `query_ontology` or `get_schema`).

### `inject_event` — Send Raw Input

Inject keyboard, mouse, paste, or resize events:

```json
{"type": "inject_event", "event": {"kind": "key", "code": "enter"}}
{"type": "inject_event", "event": {"kind": "key", "code": "a", "modifiers": ["ctrl"]}}
{"type": "inject_event", "event": {"kind": "mouse_click", "x": 10, "y": 5, "button": "left"}}
{"type": "inject_event", "event": {"kind": "paste", "text": "pasted content"}}
{"type": "inject_event", "event": {"kind": "resize", "width": 120, "height": 40}}
```

Key codes: single characters (`a`, `1`, `/`), named keys (`enter`, `tab`, `esc`,
`backspace`, `delete`, `up`, `down`, `left`, `right`, `home`, `end`, `pageup`,
`pagedown`), function keys (`f1`–`f12`).

Modifiers: `shift`, `ctrl`, `alt`, `super`.

### `subscribe` / `unsubscribe` — Event Streams

```json
{"type": "subscribe", "events": ["state_changed", "render_update"]}
{"type": "subscribe", "events": ["*"]}
{"type": "unsubscribe", "events": ["render_update"]}
```

Event types: `state_changed`, `render_update`, `action_result`, `app_quit`, `error`.

### `quit` — Terminate

```json
{"type": "quit"}
```

## Launching a Hawk TUI App

### Option 1: RPC Transport (stdin/stdout)

Build a Hawk TUI app that uses `RpcTransport`:

```rust
use hawktui::agent::rpc::RpcTransport;

fn main() -> std::io::Result<()> {
    let app = MyApp::default();
    let transport = RpcTransport::new(app, 120, 40)?;
    let _final_state = transport.run()?;
    Ok(())
}
```

The agent spawns this binary and communicates via stdio:

```python
import subprocess, json

proc = subprocess.Popen(
    ["./target/release/my_app"],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    text=True
)

def send(request, req_id=None):
    msg = {**request}
    if req_id:
        msg["id"] = req_id
    proc.stdin.write(json.dumps(msg) + "\n")
    proc.stdin.flush()
    return json.loads(proc.stdout.readline())

# Test connection
print(send({"type": "ping"}))

# Discover widgets
schemas = send({"type": "query_ontology"})

# Execute an action
send({"type": "execute_action",
      "agent_id": "counter",
      "action": "increment"})

# Read state
state = send({"type": "get_state", "agent_id": "counter"})

# Clean shutdown
send({"type": "quit"})
proc.wait()
```

### Option 2: Headless Driver (In-Process)

For Rust-based agents or testing, use `HeadlessDriver` directly:

```rust
use hawktui::agent::HeadlessDriver;
use hawktui::agent::protocol::AgentRequest;

let mut driver = HeadlessDriver::new(MyApp::default(), 120, 40)?;
driver.init();
driver.render()?;

// Query ontology
let resp = driver.process_request(&AgentRequest::Ping);
assert!(resp.success);

// Execute action
let resp = driver.process_request(&AgentRequest::ExecuteAction {
    agent_id: "editor-1".into(),
    action: "insert_text".into(),
    params: serde_json::json!({"text": "hello"}),
});
driver.render()?;

// Read rendered output
let line = driver.row_text(0);
```

## Typical Agent Workflow

```
1. spawn process / create driver
2. ping  →  verify connection
3. query_ontology  →  learn what widgets exist
4. get_tree  →  understand current UI structure
5. loop:
   a. get_state for relevant widgets
   b. decide on action (LLM reasoning)
   c. execute_action or inject_event
   d. get_state to observe result
6. quit
```

## Widget Discoverability

Every Hawk TUI widget implements `Discoverable`, which exposes:

| Attribute       | Description                                                       |
| --------------- | ----------------------------------------------------------------- |
| `agent_id`      | Unique identifier for addressing the widget                       |
| `widget_type`   | Type name (e.g., "Editor", "SelectList")                          |
| `semantic_role` | Purpose: Input, Display, Navigation, Container, etc.              |
| `capabilities`  | What it can do: Scrollable, Editable, Focusable, Searchable, etc. |
| `actions`       | Named operations with typed parameters                            |
| `properties`    | Current state with types and constraints                          |

Agents can use this metadata to operate any Hawk TUI app without hard-coded knowledge
of the specific UI. The ontology acts as a self-describing API.

## Compatibility with Pi/OpenCode

Hawk TUI's RPC protocol is modeled after the pi-mono JSONL protocol used by OpenCode
and Pi. Key differences:

| Feature        | pi-mono                                | Hawk TUI                      |
| -------------- | -------------------------------------- | -------------------------- |
| Transport      | JSONL stdin/stdout                     | JSONL stdin/stdout         |
| Framing        | `{type, id?}`                          | `{type, id?}` (compatible) |
| Discovery      | Client asks for capabilities           | Full ontology with schemas |
| Actions        | Command-based (`prompt`, `bash`, etc.) | Widget-action based        |
| Events         | `AgentEvent` stream                    | `AgentEvent` stream        |
| Error handling | `{success: false, error}`              | `{success: false, error}`  |

An adapter layer can bridge the two protocols — Hawk TUI's richer ontology is a
superset of pi-mono's capability model.
