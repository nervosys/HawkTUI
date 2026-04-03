export default function Examples() {
  return (
    <>
      <h1>Examples</h1>
      <p>
        Louie ships several examples demonstrating different features and
        complexity levels. Run any example with:
      </p>
      <pre><code>cargo run --example &lt;name&gt;</code></pre>

      <h2>hello</h2>
      <p>
        Minimal &quot;Hello, Louie!&quot; application. Demonstrates the basic Elm
        architecture: a model struct, quit message, bordered paragraph, and the
        program runtime.
      </p>
      <pre><code>cargo run --example hello</code></pre>

      <h2>counter</h2>
      <p>
        Increment/decrement counter with an animated gauge. Shows message handling,
        keyboard events, layout splitting, and the Gauge widget with real-time
        animation.
      </p>
      <pre><code>cargo run --example counter</code></pre>

      <h2>agent_demo</h2>
      <p>
        Browse the widget ontology interactively. Displays all registered widget
        schemas, their capabilities, actions, and semantic roles. Demonstrates the
        ontology registry and the Discoverable trait.
      </p>
      <pre><code>cargo run --example agent_demo</code></pre>

      <h2>agent_rpc</h2>
      <p>
        Headless RPC server that reads JSON commands from stdin and writes events
        to stdout. This is the foundation of the agent protocol &mdash; use it to
        test agent integration without a terminal.
      </p>
      <pre><code>{`cargo run --example agent_rpc

# Then pipe JSON:
echo '{"type":"ping"}' | cargo run --example agent_rpc`}</code></pre>

      <h2>opencode</h2>
      <p>
        OpenCode-style AI chat assistant interface. Features a multi-pane layout
        with a chat history panel, input editor, file tree, and status bar.
        Demonstrates complex layout composition, multiple widget types, and focus
        management.
      </p>
      <pre><code>cargo run --example opencode</code></pre>

      <h2>lazygit</h2>
      <p>
        Lazygit-style Git client interface. Features a diff viewer, file status
        list, commit log, and branch selector. Demonstrates Table, List, Tabs,
        and split layouts.
      </p>
      <pre><code>cargo run --example lazygit</code></pre>

      <h2>btop</h2>
      <p>
        btop-style system resource monitor interface. Features CPU/memory gauges,
        sparklines, charts, and bar charts. Demonstrates data visualization widgets
        and the animation system.
      </p>
      <pre><code>cargo run --example btop</code></pre>

      <h2>Running the Demo Script</h2>
      <p>
        A Python demo script exercises the agent protocol against{" "}
        <code>louie-server</code>:
      </p>
      <pre><code>{`cargo build --release --bin louie-server --features bin
python3 scripts/louie-demo.py`}</code></pre>
    </>
  );
}
