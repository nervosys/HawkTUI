export default function Changelog() {
  return (
    <>
      <h1>Changelog</h1>
      <p>
        All notable changes to Hawk TUI are documented here. This project adheres to{" "}
        <a href="https://semver.org/spec/v2.0.0.html">Semantic Versioning</a> and
        follows <a href="https://keepachangelog.com/en/1.1.0/">Keep a Changelog</a>.
      </p>

      <h2 id="v1.0.0">1.0.0 — 2025-07-17</h2>

      <h3>Added</h3>
      <ul>
        <li><strong>Property-Based Tests</strong>: 10 proptest-driven tests for agent protocol round-tripping and fuzz deserialization</li>
        <li><strong>Structured Logging</strong>: Replaced <code>eprintln</code> in <code>hawktui-server</code> with <code>tracing</code> + <code>tracing-subscriber</code></li>
        <li><strong>CONTRIBUTING.md</strong>: Contributor guide with development workflow, commit conventions, and widget addition guide</li>
        <li><strong>SECURITY.md</strong>: Security policy with responsible disclosure process and hardening summary</li>
        <li><strong>cargo-deny</strong>: License auditing and advisory checks via <code>deny.toml</code> + CI job</li>
        <li><strong>CI Coverage</strong>: <code>cargo-tarpaulin</code> coverage reporting with Codecov upload</li>
        <li><strong>CI Benchmarks</strong>: <code>criterion</code> benchmark regression detection via <code>github-action-benchmark</code></li>
        <li><strong>README Install Section</strong>: <code>cargo add hawktui</code> instructions and MSRV note</li>
      </ul>

      <h3>Changed</h3>
      <ul>
        <li><strong>API Stability</strong>: <code>util</code> module marked <code>#[doc(hidden)]</code> — internal utilities are no longer part of the public API</li>
        <li><strong>Buffer Safety</strong>: <code>Buffer::IndexMut</code> no longer panics on out-of-bounds writes; uses a scratch cell for defense-in-depth (MEM-1)</li>
        <li><strong>Dependencies</strong>: Added <code>tracing</code> 0.1, <code>tracing-subscriber</code> 0.3; added <code>proptest</code> 1 (dev)</li>
      </ul>

      <h3>Security</h3>
      <ul>
        <li>Comprehensive doc comments added to all agent protocol types, event types, animation API, overlay system, runtime, terminal, focus, and widget traits</li>
      </ul>

      <h2 id="v0.1.0">0.1.0 — 2025-07-16</h2>

      <h3>Added</h3>
      <ul>
        <li><strong>Elm Architecture Runtime</strong>: <code>Model</code>, <code>Command</code>, <code>Program</code> with async task support, tick rates, and event loop</li>
        <li><strong>Agent Protocol</strong>: JSON Lines over stdin/stdout — 10 request types</li>
        <li><strong>Ontology System</strong>: <code>Discoverable</code> trait, <code>WidgetSchema</code>, <code>AgentCapability</code> (18 variants), <code>SemanticRole</code> (15 variants), <code>AgentAction</code></li>
        <li><strong>Ontology Registry</strong>: Type catalog + live UI tree with search, role-based discovery, and action parameter validation</li>
        <li><strong>Headless Driver</strong>: <code>HeadlessDriver</code> for agent-only operation, automated testing, and CI/CD</li>
        <li><strong>RPC Transport</strong>: <code>RpcTransport</code> with stdin/stdout JSON Lines, rate limiting (1000 req/s), and 1 MB line size cap</li>
        <li><strong>21 Widgets</strong>: Paragraph, List, Block, Tabs, Gauge, LineGauge, Scrollbar, Table, Input, Editor, SelectList, SettingsList, Loader, CancellableLoader, Sparkline, BarChart, Calendar, Chart, Image, Toast, Canvas</li>
        <li><strong>Focus System</strong>: <code>FocusManager</code> with focus ring and programmatic focus control</li>
        <li><strong>Overlay System</strong>: <code>OverlayStack</code> with focus capture and <code>ModalBox</code></li>
        <li><strong>Animation</strong>: Tweens, springs, 25 easing curves, and timeline sequencing</li>
        <li><strong>Layout Engine</strong>: Constraint-based layout with 6 constraint types</li>
        <li><strong>Core Primitives</strong>: <code>Buffer</code>, <code>Cell</code>, <code>Rect</code>, <code>Style</code>, <code>Text</code>, <code>Color</code> (16 + RGB + indexed)</li>
        <li><strong>Theme System</strong>: <code>Theme</code> with token-based styling, built-in dark/light themes</li>
        <li><strong>Error Types</strong>: Unified <code>hawktui::Error</code> enum with 6 variants</li>
        <li><strong>Backend Abstraction</strong>: <code>Backend</code> trait with crossterm (optional) and test backends</li>
      </ul>

      <h3>Security</h3>
      <ul>
        <li>Input sanitization on all agent protocol fields</li>
        <li>Subscription limit (100 max) to prevent resource exhaustion</li>
        <li>Terminal dimension clamping (1&ndash;1024) on injected resize events</li>
        <li>Rate limiting on RPC transport (1,000 requests/second)</li>
        <li>Action parameter schema validation before dispatch</li>
        <li>Structured logging with redacted sensitive fields</li>
        <li>Auth handshake support in agent sessions</li>
        <li>Binary path validation for external commands</li>
      </ul>
    </>
  );
}
