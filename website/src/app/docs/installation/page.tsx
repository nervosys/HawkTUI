export default function Installation() {
  return (
    <>
      <h1>Installation</h1>

      <h2>Requirements</h2>
      <ul>
        <li><strong>Rust</strong> 1.80 or later (MSRV)</li>
        <li>A terminal emulator with ANSI support (any modern terminal works)</li>
      </ul>

      <h2>Add to Your Project</h2>
      <p>The quickest way to add Louie:</p>
      <pre><code>cargo add louie</code></pre>

      <p>Or add it manually to your <code>Cargo.toml</code>:</p>
      <pre><code>{`[dependencies]
louie = "1"`}</code></pre>

      <h2>Feature Flags</h2>
      <table>
        <thead>
          <tr>
            <th>Feature</th>
            <th>Default</th>
            <th>Description</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td><code>crossterm</code></td>
            <td>Yes</td>
            <td>Crossterm terminal backend. Disable for headless/agent-only use.</td>
          </tr>
          <tr>
            <td><code>bin</code></td>
            <td>No</td>
            <td>Enables <code>louie-server</code> and <code>louie-demo</code> binaries (pulls in <code>tracing</code>).</td>
          </tr>
        </tbody>
      </table>

      <h3>Headless Mode (No Terminal)</h3>
      <p>
        If you only need the agent protocol and don&apos;t need a real terminal, disable
        the default crossterm backend:
      </p>
      <pre><code>{`[dependencies]
louie = { version = "1", default-features = false }`}</code></pre>

      <h3>Building the Server Binary</h3>
      <p>
        The standalone <code>louie-server</code> binary lets AI agents spawn and
        control a headless Louie application:
      </p>
      <pre><code>cargo build --release --bin louie-server --features bin</code></pre>

      <h2>Verify Installation</h2>
      <pre><code>{`cargo build
cargo test`}</code></pre>
      <p>
        If both commands succeed, Louie is ready to use. Continue to the{" "}
        <a href="/docs/quick-start">Quick Start</a> guide.
      </p>
    </>
  );
}
