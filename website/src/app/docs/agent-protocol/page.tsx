export default function AgentProtocol() {
  return (
    <>
      <h1>Agent Protocol Reference</h1>
      <p>
        The Louie Agent Protocol enables AI systems to programmatically discover,
        inspect, and control terminal user interfaces via JSON Lines over
        stdin/stdout.
      </p>

      <div className="arch-diagram">{`┌──────────────────┐     stdin (JSONL)      ┌──────────────────┐
│                  │ ─────────────────────► │                  │
│   AI Agent       │                        │   louie-server   │
│   (Claude, GPT,  │ ◄───────────────────── │   (headless)     │
│    Codex, etc.)  │     stdout (JSONL)     │                  │
└──────────────────┘                        └──────────────────┘`}</div>

      <h2>Quick Start</h2>
      <pre><code>{`# Build the server
cargo build --release --bin louie-server --features bin

# Test connectivity
echo '{"type":"ping"}' | ./target/release/louie-server

# Discover all widget types
echo '{"type":"query_ontology"}' | ./target/release/louie-server`}</code></pre>

      <h2>Message Format</h2>

      <h3>Request</h3>
      <div className="protocol-msg">{`{"id": "req-1", "type": "<request_type>", ...fields}`}</div>
      <p>
        The <code>id</code> field is optional. If present, the server echoes it in
        the response for correlation.
      </p>

      <h3>Response</h3>
      <div className="protocol-msg">{`{"success": true, "id": "req-1", "data": {...}}`}</div>
      <div className="protocol-msg">{`{"success": false, "id": "req-1", "error": "Widget not found: editor-1"}`}</div>

      <h2>Request Types</h2>

      <table>
        <thead>
          <tr><th>Type</th><th>Description</th><th>Parameters</th></tr>
        </thead>
        <tbody>
          <tr>
            <td><code>ping</code></td>
            <td>Connection test</td>
            <td>None</td>
          </tr>
          <tr>
            <td><code>query_ontology</code></td>
            <td>Discover widget types</td>
            <td><code>query?</code>, <code>role?</code></td>
          </tr>
          <tr>
            <td><code>get_schema</code></td>
            <td>Get schema for a widget type</td>
            <td><code>widget_type</code></td>
          </tr>
          <tr>
            <td><code>get_tree</code></td>
            <td>Get live UI tree snapshot</td>
            <td>None</td>
          </tr>
          <tr>
            <td><code>get_state</code></td>
            <td>Get widget state</td>
            <td><code>agent_id</code></td>
          </tr>
          <tr>
            <td><code>execute_action</code></td>
            <td>Invoke a widget action</td>
            <td><code>agent_id</code>, <code>action</code>, <code>params?</code></td>
          </tr>
          <tr>
            <td><code>inject_event</code></td>
            <td>Send raw input</td>
            <td><code>event_type</code>, <code>key?</code>, <code>x?</code>, <code>y?</code></td>
          </tr>
          <tr>
            <td><code>subscribe</code></td>
            <td>Subscribe to events</td>
            <td><code>events</code> (array)</td>
          </tr>
          <tr>
            <td><code>unsubscribe</code></td>
            <td>Unsubscribe from events</td>
            <td><code>events</code> (array)</td>
          </tr>
          <tr>
            <td><code>quit</code></td>
            <td>Terminate the server</td>
            <td>None</td>
          </tr>
        </tbody>
      </table>

      <h2>Event Subscriptions</h2>
      <p>Subscribe to receive streamed events:</p>
      <table>
        <thead><tr><th>Event</th><th>Description</th></tr></thead>
        <tbody>
          <tr><td><code>state_changed</code></td><td>A widget&apos;s state changed</td></tr>
          <tr><td><code>render_update</code></td><td>A frame was rendered</td></tr>
          <tr><td><code>action_result</code></td><td>An action completed</td></tr>
          <tr><td><code>app_quit</code></td><td>The application is shutting down</td></tr>
          <tr><td><code>error</code></td><td>A runtime error occurred</td></tr>
        </tbody>
      </table>

      <h2>Typical Agent Workflow</h2>
      <ol>
        <li>Spawn <code>louie-server</code> as a child process.</li>
        <li>Send <code>ping</code> to verify connectivity.</li>
        <li>Send <code>query_ontology</code> to discover available widget types.</li>
        <li>Send <code>get_tree</code> to get the live UI tree with widget IDs.</li>
        <li>Loop: <code>get_state</code> → reason → <code>execute_action</code> / <code>inject_event</code> → observe.</li>
        <li>Send <code>quit</code> to terminate.</li>
      </ol>

      <h2>Security</h2>
      <div className="callout callout-info">
        <p>
          <strong>Rate limiting:</strong> 1,000 requests/second. <strong>Max request size:</strong> 1 MB.{" "}
          <strong>Max subscriptions:</strong> 100. <strong>Terminal size clamping:</strong> 1&ndash;1024.{" "}
          <strong>Auth:</strong> Optional <code>--auth-token</code> flag for handshake verification.
        </p>
      </div>

      <p>
        See the <a href="/docs/agent-integration">Integration Guide</a> for
        language-specific examples and the <a href="/docs/headless-driver">Headless
        Driver</a> for in-process agent operation.
      </p>
    </>
  );
}
