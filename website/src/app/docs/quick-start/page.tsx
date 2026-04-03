export default function QuickStart() {
  return (
    <>
      <h1>Quick Start</h1>
      <p>Build a minimal Louie application in under 5 minutes.</p>

      <h2>1. Create a New Project</h2>
      <pre><code>{`cargo new my-louie-app
cd my-louie-app
cargo add louie`}</code></pre>

      <h2>2. Write Your App</h2>
      <p>
        Replace <code>src/main.rs</code> with:
      </p>
      <pre><code>{`use louie::prelude::*;
use louie::runtime::{Command, Model, Program};

struct App;

#[derive(Debug)]
enum Msg {
    Quit,
}

impl Model for App {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Quit => Command::Quit,
        }
    }

    fn view(&self, frame: &mut Frame<'_>) {
        let greeting = Paragraph::new("Hello, Louie!")
            .block(Block::default().title("Demo").borders(Borders::ALL));
        frame.render_widget(greeting, frame.area());
    }

    fn handle_event(&self, event: Event) -> Option<Msg> {
        if let Event::Key(key) = event {
            if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                return Some(Msg::Quit);
            }
        }
        None
    }
}

fn main() -> std::io::Result<()> {
    let backend = CrosstermBackend::new(std::io::stdout());
    Program::new(App, backend)?.run()
}`}</code></pre>

      <h2>3. Run It</h2>
      <pre><code>cargo run</code></pre>
      <p>
        Press <code>q</code> or <code>Esc</code> to exit. You should see a bordered
        box with &quot;Hello, Louie!&quot; rendered in your terminal.
      </p>

      <h2>How It Works</h2>
      <p>
        Louie uses <strong>The Elm Architecture</strong> (TEA):
      </p>
      <ol>
        <li>
          <strong>Model</strong> &mdash; Your application state (here, a simple unit struct).
        </li>
        <li>
          <strong>Update</strong> &mdash; <code>update()</code> receives messages and returns commands.
        </li>
        <li>
          <strong>View</strong> &mdash; <code>view()</code> renders the current state to a frame.
        </li>
        <li>
          <strong>Events</strong> &mdash; <code>handle_event()</code> converts terminal
          events (key presses, mouse clicks, resize) into your message type.
        </li>
      </ol>
      <p>
        The <code>Program</code> runtime manages the event loop: render &rarr; poll
        events &rarr; dispatch messages &rarr; repeat. Double-buffered differential
        rendering ensures only changed cells are written to the terminal.
      </p>

      <h2>Next Steps</h2>
      <ul>
        <li><a href="/docs/elm-architecture">Elm Architecture</a> &mdash; Deep dive into Model, Update, View, and Commands.</li>
        <li><a href="/docs/widgets">Widget Catalog</a> &mdash; Explore all 22 widgets.</li>
        <li><a href="/docs/examples">Examples</a> &mdash; See full demos (counter, OpenCode clone, lazygit clone).</li>
        <li><a href="/docs/agent-protocol">Agent Protocol</a> &mdash; Connect an AI agent to your app.</li>
      </ul>
    </>
  );
}
