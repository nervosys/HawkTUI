# Louie Agent Protocol Specification

**Version**: 0.1.0  
**Transport**: JSON Lines (JSONL) over stdin/stdout  
**Encoding**: UTF-8

## Overview

The Louie Agent Protocol enables AI systems to programmatically discover, inspect, and control terminal user interfaces. Unlike screen-scraping approaches that interpret raw terminal output, this protocol provides structured access to every widget's type, capabilities, state, and available actions.

An agent spawns a Louie application as a child process and communicates entirely through JSON messages — one per line, delimited by `\n`.

## Architecture

```
┌──────────────────┐     stdin (JSONL)      ┌──────────────────┐
│                  │ ─────────────────────► │                  │
│   AI Agent       │                        │   louie-server   │
│   (Claude, GPT,  │ ◄───────────────────── │   (headless)     │
│    Codex, etc.)  │     stdout (JSONL)     │                  │
└──────────────────┘                        └──────────────────┘
                          stderr: diagnostics (human-readable)
```

- **Agent → Server**: `RequestEnvelope` JSON objects on stdin
- **Server → Agent**: `AgentResponse` JSON objects on stdout
- **Diagnostics**: Human-readable status messages on stderr (never JSON)

## Quick Start

```sh
# Install
cargo install --path .

# Test connectivity
echo '{"type":"ping"}' | louie-server

# Discover all widget types
echo '{"type":"query_ontology"}' | louie-server

# Start with custom terminal size
louie-server --width 160 --height 50
```

## Message Format

### Request Envelope

Every request is a single JSON object on one line:

```json
{"id": "req-1", "type": "<request_type>", ...fields}
```

The `id` field is optional. If present, the server echoes it back in the response for correlation.

### Response

```json
{"success": true, "id": "req-1", "data": {...}}
```

On error:

```json
{"success": false, "id": "req-1", "error": "Widget not found: editor-1"}
```

Fields:
| Field     | Type    | Required | Description                              |
| --------- | ------- | -------- | ---------------------------------------- |
| `success` | boolean | yes      | Whether the request succeeded            |
| `id`      | string  | no       | Echo of request ID (omitted if not sent) |
| `data`    | object  | no       | Result payload (on success)              |
| `error`   | string  | no       | Error message (on failure)               |

## Request Types

### `ping` — Connection Test

```json
{"type": "ping"}
```

**Response:**
```json
{"success": true, "data": {"status": "pong"}}
```

### `query_ontology` — Discover Widget Types

List all registered widget types in the application:

```json
{"type": "query_ontology"}
```

Filter by name, tag, or description:

```json
{"type": "query_ontology", "query": "Input"}
```

Filter by semantic role:

```json
{"type": "query_ontology", "role": "Input"}
```

**Response:**

```json
{
  "success": true,
  "data": [
    {
      "name": "Input",
      "description": "A single-line text input field with cursor navigation.",
      "default_role": "Input",
      "properties": [
        {
          "name": "placeholder",
          "description": "Hint text shown when empty.",
          "property_type": "String",
          "required": false
        }
      ],
      "tags": ["input", "text", "form", "editable"],
      "usage_hint": "Input::new().placeholder(\"Type here...\")"
    }
  ]
}
```

### `get_schema` — Get Single Widget Schema

```json
{"type": "get_schema", "widget_type": "Editor"}
```

Returns the full schema for a specific widget type, including all properties, constraints, capabilities, and usage hints.

### `get_tree` — Get UI Tree Snapshot

```json
{"type": "get_tree"}
```

Returns the current UI node tree. Each node contains:

| Field          | Type     | Description                                       |
| -------------- | -------- | ------------------------------------------------- |
| `agent_id`     | string   | Unique identifier for addressing this widget      |
| `widget_type`  | string   | Type name (e.g., "Editor", "List")                |
| `role`         | string   | Semantic role (Input, Display, Navigation, etc.)  |
| `state`        | object   | Current state snapshot as JSON                    |
| `label`        | string?  | Human-readable label                              |
| `bounds`       | object?  | Position and size {x, y, width, height}           |
| `capabilities` | string[] | Available capabilities (Scrollable, Focusable...) |

### `get_state` — Get Widget State

```json
{"type": "get_state", "agent_id": "editor-1"}
```

Returns the current state of a specific widget identified by its `agent_id`.

### `execute_action` — Invoke Widget Action

```json
{
  "type": "execute_action",
  "agent_id": "editor-1",
  "action": "insert_text",
  "params": {"text": "hello world"}
}
```

Action names and their parameters are defined in each widget's schema (discoverable via `query_ontology` or `get_schema`).

**Response:**
```json
{"success": true, "data": {"status": "dispatched", "agent_id": "editor-1", "action": "insert_text"}}
```

### `inject_event` — Send Raw Input

Inject keyboard, mouse, paste, or resize events directly:

**Keyboard:**
```json
{"type": "inject_event", "event": {"kind": "key", "code": "enter"}}
{"type": "inject_event", "event": {"kind": "key", "code": "a", "modifiers": ["ctrl"]}}
```

**Mouse:**
```json
{"type": "inject_event", "event": {"kind": "mouse_click", "x": 10, "y": 5, "button": "left"}}
```

**Paste:**
```json
{"type": "inject_event", "event": {"kind": "paste", "text": "pasted content"}}
```

**Resize:**
```json
{"type": "inject_event", "event": {"kind": "resize", "width": 120, "height": 40}}
```

**Key codes:** Single characters (`a`, `1`, `/`), named keys (`enter`, `tab`, `esc`, `backspace`, `delete`, `up`, `down`, `left`, `right`, `home`, `end`, `pageup`, `pagedown`), function keys (`f1`–`f12`).

**Modifiers:** `shift`, `ctrl`, `alt`, `super`.

### `subscribe` / `unsubscribe` — Event Streams

```json
{"type": "subscribe", "events": ["state_changed", "render_update"]}
{"type": "subscribe", "events": ["*"]}
{"type": "unsubscribe", "events": ["render_update"]}
```

**Event types:** `state_changed`, `render_update`, `action_result`, `app_quit`, `error`.

### `quit` — Terminate

```json
{"type": "quit"}
```

**Response:**
```json
{"success": true, "data": {"status": "quitting"}}
```

## Ontology Concepts

### Widget Schema

Every Louie widget exposes a typed schema describing its properties, constraints, and usage:

```json
{
  "name": "Gauge",
  "description": "A horizontal progress bar.",
  "default_role": "Progress",
  "properties": [
    {
      "name": "ratio",
      "property_type": "Float",
      "required": true,
      "constraints": [{"Min": 0.0}, {"Max": 1.0}]
    }
  ],
  "tags": ["gauge", "progress", "bar"]
}
```

### Semantic Roles

Widgets declare their purpose through semantic roles:

| Role                | Description            | Example Widgets            |
| ------------------- | ---------------------- | -------------------------- |
| `Container`         | Groups other widgets   | Block                      |
| `Display`           | Shows information      | Paragraph, Image, Calendar |
| `Input`             | Accepts user input     | Input, Editor, SelectList  |
| `Selection`         | Choose from options    | List, Tabs                 |
| `Progress`          | Shows completion       | Gauge, LineGauge           |
| `Navigation`        | Navigate between views | Tabs                       |
| `DataVisualization` | Charts and graphs      | BarChart, Chart, Sparkline |

### Capabilities

Widgets declare what they can do:

| Capability       | Description                     |
| ---------------- | ------------------------------- |
| `Focusable`      | Can receive keyboard focus      |
| `Scrollable`     | Has scrollable content          |
| `Selectable`     | Has selectable items            |
| `TextInput`      | Accepts text entry              |
| `RangeEditable`  | Has a numeric value in a range  |
| `Clickable`      | Responds to mouse clicks        |
| `Sortable`       | Can sort its contents           |
| `Searchable`     | Has search/filter functionality |
| `Copyable`       | Content can be copied           |
| `Animated`       | Has visual animation            |
| `HasKeyBindings` | Exposes keyboard shortcuts      |

### Actions

Widgets expose named operations that agents can invoke:

```json
{
  "name": "insert_text",
  "description": "Insert text at the cursor position.",
  "parameters": [
    {"name": "text", "param_type": "String", "required": true}
  ]
}
```

## Typical Agent Workflow

```
1. Spawn louie-server as child process
2. ping                    → verify connection is alive
3. query_ontology          → learn what widget types exist
4. get_tree                → understand current UI structure
5. Loop:
   a. get_state            → read relevant widget state
   b. (LLM reasoning)      → decide what to do
   c. execute_action        → perform the action
      OR inject_event       → send raw keyboard/mouse input
   d. get_state            → observe the result
6. quit                    → clean shutdown
```

## Integration Examples

### Python

```python
import subprocess
import json

proc = subprocess.Popen(
    ["louie-server"],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    text=True,
)

def send(request, req_id=None):
    """Send a request and read the response."""
    msg = {**request}
    if req_id:
        msg["id"] = req_id
    proc.stdin.write(json.dumps(msg) + "\n")
    proc.stdin.flush()
    return json.loads(proc.stdout.readline())

# Test connection
assert send({"type": "ping"})["success"]

# Discover what widgets exist
schemas = send({"type": "query_ontology"})
for schema in schemas["data"]:
    print(f"  {schema['name']} ({schema['default_role']})")

# Read current UI tree
tree = send({"type": "get_tree"})

# Execute an action
send({
    "type": "execute_action",
    "agent_id": "counter",
    "action": "increment",
    "params": {},
})

# Inject a key press
send({
    "type": "inject_event",
    "event": {"kind": "key", "code": "Up"},
})

# Clean shutdown
send({"type": "quit"})
proc.wait()
```

### TypeScript / Node.js

```typescript
import { spawn } from "child_process";
import * as readline from "readline";

const server = spawn("louie-server", [], {
  stdio: ["pipe", "pipe", "pipe"],
});

const rl = readline.createInterface({ input: server.stdout });

function send(request: object): Promise<any> {
  return new Promise((resolve) => {
    rl.once("line", (line) => resolve(JSON.parse(line)));
    server.stdin.write(JSON.stringify(request) + "\n");
  });
}

const pong = await send({ type: "ping" });
console.log(pong); // { success: true, data: { status: "pong" } }

const schemas = await send({ type: "query_ontology" });
console.log(`Found ${schemas.data.length} widget types`);

await send({ type: "quit" });
```

### Rust (In-Process)

```rust
use louie::agent::HeadlessDriver;
use louie::agent::protocol::AgentRequest;

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
```

## Comparison with Alternatives

| Approach           | Discovery         | Reliability | Latency | Maintenance |
| ------------------ | ----------------- | ----------- | ------- | ----------- |
| Screen scraping    | None              | Fragile     | High    | High        |
| Accessibility APIs | Partial           | Medium      | Medium  | Medium      |
| Custom RPC         | Hardcoded         | Good        | Low     | High        |
| **Louie Protocol** | **Full ontology** | **Robust**  | **Low** | **Low**     |

Screen-scraping requires the agent to parse terminal ANSI escape sequences, guess at widget boundaries, and break whenever the layout changes. The Louie protocol provides typed, versioned access to every widget's schema, state, and actions — the UI is self-describing.
