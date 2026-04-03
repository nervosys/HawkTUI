export default function AgentIntegration() {
  return (
    <>
      <h1>Agent Integration Guide</h1>
      <p>
        This guide shows how to connect AI agents to a Louie application using
        the JSON Lines protocol. Examples in Python, TypeScript, and Rust.
      </p>

      <h2>Launching the Server</h2>
      <pre><code>{`# Build the headless server
cargo build --release --bin louie-server --features bin

# Start with custom terminal size
./target/release/louie-server --width 160 --height 50`}</code></pre>

      <h2>Python Example</h2>
      <pre><code>{`import subprocess, json

proc = subprocess.Popen(
    ["./target/release/louie-server"],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    text=True,
)

def send(request):
    proc.stdin.write(json.dumps(request) + "\\n")
    proc.stdin.flush()
    return json.loads(proc.stdout.readline())

# Ping
print(send({"type": "ping"}))

# Discover widgets
response = send({"type": "query_ontology"})
for widget in response["data"]["schemas"]:
    print(f"  {widget['name']} — {widget['description']}")

# Get UI tree
tree = send({"type": "get_tree"})
for node in tree["data"]["nodes"]:
    print(f"  [{node['role']}] {node['widget_type']} id={node['agent_id']}")

# Execute action
send({
    "type": "execute_action",
    "agent_id": "search-input",
    "action": "set_text",
    "params": {"text": "hello"}
})

# Quit
send({"type": "quit"})`}</code></pre>

      <h2>TypeScript / Node.js Example</h2>
      <pre><code>{`import { spawn } from "child_process";
import * as readline from "readline";

const proc = spawn("./target/release/louie-server", [], {
  stdio: ["pipe", "pipe", "pipe"],
});

const rl = readline.createInterface({ input: proc.stdout! });

function send(request: object): Promise<any> {
  return new Promise((resolve) => {
    rl.once("line", (line) => resolve(JSON.parse(line)));
    proc.stdin!.write(JSON.stringify(request) + "\\n");
  });
}

const ping = await send({ type: "ping" });
console.log("Protocol version:", ping.data.version);

const ontology = await send({ type: "query_ontology" });
console.log("Widget types:", ontology.data.schemas.length);

await send({ type: "quit" });`}</code></pre>

      <h2>Rust In-Process (HeadlessDriver)</h2>
      <p>
        For embedding Louie directly in a Rust agent, use the{" "}
        <code>HeadlessDriver</code> without spawning a separate process:
      </p>
      <pre><code>{`use louie::agent::HeadlessDriver;
use louie::backend::TestBackend;

let backend = TestBackend::new(80, 24);
let mut driver = HeadlessDriver::new(MyApp::new(), backend);

// Process a protocol request directly
let response = driver.handle_request(&AgentRequest::Ping);
let tree = driver.handle_request(&AgentRequest::GetTree);`}</code></pre>
      <p>
        See the <a href="/docs/headless-driver">Headless Driver</a> page for more
        details.
      </p>

      <h2>Widget Discoverability</h2>
      <p>
        Every widget in a Louie application exposes:
      </p>
      <table>
        <thead><tr><th>Attribute</th><th>Description</th><th>Protocol Request</th></tr></thead>
        <tbody>
          <tr><td>Schema</td><td>Type name, properties, constraints</td><td><code>get_schema</code></td></tr>
          <tr><td>Capabilities</td><td>What the widget can do (Focusable, TextInput, ...)</td><td><code>query_ontology</code></td></tr>
          <tr><td>Actions</td><td>Named operations with typed parameters</td><td><code>execute_action</code></td></tr>
          <tr><td>Semantic Role</td><td>Purpose category (Container, Input, ...)</td><td><code>query_ontology</code></td></tr>
          <tr><td>State</td><td>Current widget state as JSON</td><td><code>get_state</code></td></tr>
        </tbody>
      </table>

      <h2>Best Practices</h2>
      <ul>
        <li>Always start with <code>ping</code> to verify the server is ready.</li>
        <li>Use <code>query_ontology</code> to discover capabilities before assuming widget structure.</li>
        <li>Prefer <code>execute_action</code> over <code>inject_event</code> for reliability.</li>
        <li>Subscribe to <code>state_changed</code> events rather than polling <code>get_state</code>.</li>
        <li>Handle errors gracefully &mdash; widgets may be removed or renamed between versions.</li>
      </ul>
    </>
  );
}
