export default function HeadlessDriver() {
  return (
    <>
      <h1>Headless Driver</h1>
      <p>
        The <code>HeadlessDriver</code> runs a Louie application without a real
        terminal, enabling automated testing, CI/CD pipelines, and in-process agent
        integration.
      </p>

      <h2>Use Cases</h2>
      <ul>
        <li><strong>Automated testing</strong> &mdash; Verify widget behavior without a terminal.</li>
        <li><strong>CI/CD</strong> &mdash; Run UI tests in headless environments (GitHub Actions, etc.).</li>
        <li><strong>In-process agents</strong> &mdash; Embed Louie directly in a Rust agent without stdio.</li>
        <li><strong>Benchmarking</strong> &mdash; Measure rendering performance without terminal overhead.</li>
      </ul>

      <h2>Basic Usage</h2>
      <pre><code>{`use louie::agent::HeadlessDriver;
use louie::backend::TestBackend;

// Create a headless app with an 80×24 virtual terminal
let backend = TestBackend::new(80, 24);
let mut driver = HeadlessDriver::new(MyApp::new(), backend);

// Process protocol requests in-process
let ping_response = driver.handle_request(&AgentRequest::Ping);
assert!(ping_response.success);

// Get the UI tree
let tree_response = driver.handle_request(&AgentRequest::GetTree);
let nodes = &tree_response.data["nodes"];

// Execute an action
let action_response = driver.handle_request(&AgentRequest::ExecuteAction {
    agent_id: "my-input".into(),
    action: "set_text".into(),
    params: serde_json::json!({"text": "hello"}),
});`}</code></pre>

      <h2>TestBackend</h2>
      <p>
        The <code>TestBackend</code> captures rendered output in an in-memory buffer
        rather than writing to a terminal:
      </p>
      <pre><code>{`use louie::backend::TestBackend;

let backend = TestBackend::new(120, 40);

// After rendering, inspect the buffer:
let buffer = backend.buffer();
let cell = &buffer[(5, 3)];  // Column 5, row 3
assert_eq!(cell.symbol(), "H");`}</code></pre>

      <h2>Testing Widgets</h2>
      <pre><code>{`#[test]
fn test_paragraph_renders() {
    let backend = TestBackend::new(20, 3);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| {
        let p = Paragraph::new("Hello!");
        frame.render_widget(p, frame.area());
    }).unwrap();

    let buf = terminal.backend().buffer();
    assert_eq!(buf[(0, 0)].symbol(), "H");
    assert_eq!(buf[(1, 0)].symbol(), "e");
}`}</code></pre>

      <h2>vs. louie-server</h2>
      <table>
        <thead>
          <tr><th>Feature</th><th>HeadlessDriver</th><th>louie-server</th></tr>
        </thead>
        <tbody>
          <tr><td>Transport</td><td>In-process function calls</td><td>JSON Lines over stdin/stdout</td></tr>
          <tr><td>Language</td><td>Rust only</td><td>Any (Python, TS, Go, ...)</td></tr>
          <tr><td>Latency</td><td>Minimal (no serialization)</td><td>~1ms per request</td></tr>
          <tr><td>Use case</td><td>Tests, Rust agents, benchmarks</td><td>Multi-language agents, spawned processes</td></tr>
        </tbody>
      </table>
    </>
  );
}
