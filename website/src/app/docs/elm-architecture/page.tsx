export default function ElmArchitecture() {
  return (
    <>
      <h1>Elm Architecture</h1>
      <p>
        Louie uses <strong>The Elm Architecture</strong> (TEA), the same pattern
        used by Elm, bubbletea, and Iced. Your application is defined by three
        functions and a message type.
      </p>

      <h2>The Model Trait</h2>
      <pre><code>{`pub trait Model: Sized {
    type Msg: Send + 'static;

    fn update(&mut self, msg: Self::Msg) -> Command<Self::Msg>;
    fn view(&self, frame: &mut Frame<'_>);
    fn handle_event(&self, event: Event) -> Option<Self::Msg>;

    // Optional hooks:
    fn init(&mut self) -> Command<Self::Msg> { Command::None }
    fn register_ontology(&self, _registry: &mut OntologyRegistry) {}
}`}</code></pre>

      <h3>update()</h3>
      <p>
        Receives a message, mutates state, and returns a <code>Command</code>.
        This is the only place state changes happen.
      </p>

      <h3>view()</h3>
      <p>
        Renders the current state into a <code>Frame</code>. Called every tick.
        Widgets are placed by calling <code>frame.render_widget(widget, area)</code>.
      </p>

      <h3>handle_event()</h3>
      <p>
        Converts raw terminal events (key presses, mouse clicks, resize) into your
        application&apos;s message type. Return <code>None</code> to ignore an event.
      </p>

      <h2>Commands</h2>
      <p>
        Commands are returned from <code>update()</code> and <code>init()</code> to
        trigger side effects:
      </p>
      <pre><code>{`pub enum Command<Msg> {
    None,                              // No effect
    Quit,                              // Exit the program
    Batch(Vec<Command<Msg>>),          // Multiple commands
    Message(Msg),                      // Re-dispatch a message
    SetTickRate(Duration),             // Change the tick rate
    ExportOntology,                    // Dump ontology to stderr
    AgentAction {                      // Programmatic agent action
        agent_id: String,
        action: String,
        params: serde_json::Value,
    },
    Task(Box<dyn FnOnce() -> Msg + Send>),  // Async background work
}`}</code></pre>

      <h3>Async Tasks</h3>
      <p>
        <code>Command::Task</code> spawns a closure on a separate thread. When it
        completes, the returned message is dispatched back into <code>update()</code>.
        This enables network I/O, file reads, and LLM streaming without blocking the
        UI.
      </p>

      <h2>Program</h2>
      <p>
        The <code>Program</code> type ties everything together:
      </p>
      <pre><code>{`use louie::runtime::{Program, ProgramOptions};
use louie::backend::CrosstermBackend;

let backend = CrosstermBackend::new(std::io::stdout());
let options = ProgramOptions::default()
    .tick_rate(std::time::Duration::from_millis(16));  // 60 FPS

Program::with_options(MyApp::new(), backend, options)?.run()`}</code></pre>
      <p>
        The runtime loop: setup terminal &rarr; <code>init()</code> &rarr; loop
        (render &rarr; poll events &rarr; dispatch messages &rarr; tick) &rarr;
        teardown terminal.
      </p>

      <h2>ProgramOptions</h2>
      <table>
        <thead>
          <tr><th>Option</th><th>Default</th><th>Description</th></tr>
        </thead>
        <tbody>
          <tr><td><code>tick_rate</code></td><td>16ms (60 FPS)</td><td>How often the event loop ticks</td></tr>
          <tr><td><code>alternate_screen</code></td><td><code>true</code></td><td>Use alternate screen buffer</td></tr>
          <tr><td><code>mouse_capture</code></td><td><code>true</code></td><td>Enable mouse event capture</td></tr>
          <tr><td><code>raw_mode</code></td><td><code>true</code></td><td>Enable terminal raw mode</td></tr>
        </tbody>
      </table>

      <h2>Ontology Registration</h2>
      <p>
        Override <code>register_ontology()</code> to register your widget types
        so agents can discover them:
      </p>
      <pre><code>{`fn register_ontology(&self, registry: &mut OntologyRegistry) {
    registry.register::<Block>();
    registry.register::<Paragraph>();
    registry.register::<Input>();
    registry.register::<List>();
}`}</code></pre>
    </>
  );
}
