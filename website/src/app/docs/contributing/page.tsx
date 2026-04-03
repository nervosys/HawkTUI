export default function Contributing() {
  return (
    <>
      <h1>Contributing to Louie</h1>
      <p>
        Contributions are welcome! This guide covers the development workflow, code
        style, and how to add new widgets.
      </p>

      <h2>Development Setup</h2>
      <pre><code>{`git clone https://github.com/nervosys/louie.git
cd louie
cargo build
cargo test`}</code></pre>
      <p>
        <strong>Minimum supported Rust version (MSRV):</strong> 1.80
      </p>

      <h2>Workflow</h2>
      <ol>
        <li>Fork the repository and create a feature branch.</li>
        <li>Make your changes.</li>
        <li>Run the full check suite:</li>
      </ol>
      <pre><code>{`cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --lib --tests
cargo doc --no-deps`}</code></pre>
      <ol start={4}>
        <li>Submit a pull request against <code>master</code>.</li>
      </ol>

      <h2>Code Style</h2>
      <ul>
        <li>Run <code>cargo fmt</code> before committing.</li>
        <li>No <code>clippy::allow</code> without a comment explaining why.</li>
        <li>Doc comments on all public items (<code>#![warn(missing_docs)]</code>).</li>
        <li>Use <strong>Conventional Commits</strong> format: <code>feat:</code>, <code>fix:</code>, <code>docs:</code>, <code>test:</code>, <code>refactor:</code>, <code>chore:</code>.</li>
      </ul>

      <h2>Adding a Widget</h2>
      <ol>
        <li>
          Create <code>src/widget/my_widget.rs</code> implementing both{" "}
          <code>Widget</code> (or <code>StatefulWidget</code>) and <code>Discoverable</code>.
        </li>
        <li>
          Re-export from <code>src/widget/mod.rs</code>.
        </li>
        <li>
          Add rendering tests in <code>tests/widget_render_tests.rs</code> using <code>TestBackend</code>.
        </li>
        <li>
          Update the widget catalog in the README and website.
        </li>
      </ol>

      <h3>Widget Checklist</h3>
      <ul>
        <li>Implements <code>Widget</code> or <code>StatefulWidget</code></li>
        <li>Implements <code>Discoverable</code> with schema, capabilities, actions, role, and state</li>
        <li>Has builder methods for configuration</li>
        <li>Has at least 3 render tests with buffer assertions</li>
        <li>Has a discoverable schema test</li>
      </ul>

      <h2>Running Benchmarks</h2>
      <pre><code>cargo bench --bench core_bench</code></pre>

      <h2>License</h2>
      <p>
        By contributing, you agree that your contributions will be licensed under{" "}
        <strong>AGPL-3.0-or-later</strong>. See the <a href="https://github.com/nervosys/Louie/blob/master/LICENSE">LICENSE</a> file.
      </p>
    </>
  );
}
