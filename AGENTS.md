# Writing Hawk TUI programs

A guide for coding agents and for people in a hurry. It covers the shape of a
program, the four things that most often go wrong, and how to check your work
without a terminal.

Everything here is verified against the crate it ships with.

## The whole program

```rust
use hawktui::prelude::*;
use hawktui::runtime::{Command, Model, Program, ProgramOptions};

struct App {
    count: i64,
}

enum Msg {
    Increment,
    Quit,
}

impl Model for App {
    type Msg = Msg;

    // 1. Fold a message into the model.
    fn update(&mut self, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Increment => {
                self.count += 1;
                Command::None
            }
            Msg::Quit => Command::Quit,
        }
    }

    // 2. Draw the model. Takes &self — never mutate here.
    fn view(&self, frame: &mut Frame<'_>) {
        let block = Block::default().title("Counter").borders(Borders::ALL);
        let text = Paragraph::new(Text::from(format!("Count: {}", self.count)))
            .block(block);
        frame.render_widget(text, frame.area());
    }

    // 3. Translate a terminal event into a message, or ignore it.
    fn handle_event(&self, event: Event) -> Option<Msg> {
        let Event::Key(key) = event else { return None };
        match key.code {
            KeyCode::Char('+') => Some(Msg::Increment),
            KeyCode::Char('q') | KeyCode::Esc => Some(Msg::Quit),
            _ => None,
        }
    }
}

fn main() -> std::io::Result<()> {
    let backend = CrosstermBackend::new(std::io::stdout());
    Program::new(App { count: 0 }, backend)?.run()?;
    Ok(())
}
```

`Model` has exactly three required methods. `init` (an initial `Command`) and
`register_ontology` have defaults you can ignore.

## The four things that go wrong

### 1. `ProgramOptions` is not in the prelude

`use hawktui::prelude::*;` does **not** bring in `Program`, `Model`, `Command`
or `ProgramOptions`. Import them from `hawktui::runtime`:

```rust
use hawktui::runtime::{Command, Model, Program, ProgramOptions};
```

`Frame` *is* in the prelude, but it is generic over a lifetime — write
`Frame<'_>`, not `Frame`.

### 2. Six widgets are `StatefulWidget`, not `Widget`

These take a separate mutable state value and must be drawn with
`frame.render_stateful_widget(widget, area, &mut state)`:

| Widget | State |
|---|---|
| `List` | `ListState` |
| `Table` | `TableState` |
| `SelectList` | `SelectListState` |
| `SettingsList` | `SettingsListState` |
| `Scrollbar` | `ScrollbarState` |
| `Editor` | `EditorState` |

Everything else is a plain `Widget` and uses `frame.render_widget(widget, area)`.

The state holds the selection and scroll offset, and it lives in **your model**,
not in the widget — the widget is rebuilt from scratch every frame. Because
`view` takes `&self`, keep the state in the model and clone it into the call, or
hold it behind a `Cell`/`RefCell`, or move the mutation into `update`.

### 3. Layout: `split` returns `Rc<[Rect]>`

```rust
// Destructure a known number of areas — usually what you want:
let [header, body, footer] = Layout::vertical([
    Constraint::Length(3),
    Constraint::Min(0),
    Constraint::Length(1),
])
.areas(area);

// Or index into the shared slice:
let chunks = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
    .split(area);
let top = chunks[0];
```

`split` is memoised per thread and hands back `Rc<[Rect]>`; indexing and
iteration work as they would on a `Vec`. Use `Layout::solve` for an uncached
`Vec<Rect>`.

Constraints: `Length(u16)`, `Percentage(u16)`, `Min(u16)`, `Max(u16)`,
`Ratio(u32, u32)`, `Fill(u16)`. `Fill` distributes leftover space by weight.

Nest layouts by splitting an area you already got:

```rust
let [left, right] = Layout::horizontal([Constraint::Percentage(50); 2]).areas(body);
```

### 4. `Program` grabs the terminal by default

`ProgramOptions::default()` enables raw mode, the alternate screen, mouse
capture, and a 16 ms tick. For a program that must not touch the terminal —
a test, a headless run, anything with redirected stdout — turn them off:

```rust
let options = ProgramOptions {
    tick_rate: None,
    alternate_screen: false,
    mouse_capture: false,
    raw_mode: false,
};
Program::new(app, backend)?.with_options(options).run()?;
```

## Checking your work without a terminal

`hawktui::testing::Harness` owns your model and a `TestBackend`, applies keys,
and hands back each frame as plain text. No real terminal, no raw mode, no ANSI
parsing.

```rust
use hawktui::testing::Harness;

let mut harness = Harness::new(App { count: 0 }, 20, 1)?;
let frames = harness.run_script("+,+,q")?;

// One frame for the initial render, one per key — and none for `q`, because
// quitting ends the run. The frame count is how you test that quit works.
assert_eq!(frames.len(), 3);
assert!(frames[0].contains("Count: 0"));
assert!(frames[2].contains("Count: 2"));
assert!(!harness.is_running());
```

Script keys are comma-separated: `Up` `Down` `Left` `Right` `Enter` `Esc` `Tab`
`BackTab` `Backspace` `Delete` `Home` `End` `PageUp` `PageDown` `Insert`
`Space`, `F1`–`F12`, and any single character. An unrecognised name is an error
rather than a silent skip.

Every line is padded to the screen width, so you can assert on columns and on
the last row:

```rust
assert!(frames[3].lines().last().unwrap().contains("4 / 20"));
```

`Harness` folds `Command::Message` and `Command::Batch` back in exactly as the
real runtime does, and stops on `Command::Quit`. It deliberately does **not**
run `Command::Task`, which spawns a thread — a harness that sometimes ran it
would not be deterministic.

For a single frame with no model behind it, render into a buffer and read it
back:

```rust
use hawktui::backend::test::TestBackend;
use hawktui::terminal::Terminal;

let mut terminal = Terminal::new(TestBackend::new(20, 3))?;
terminal.draw(|frame| {
    let block = Block::default().title("Hi").borders(Borders::ALL);
    frame.render_widget(block, frame.area());
})?;
assert!(terminal.backend().to_text().contains("Hi"));
```

`Buffer::to_text()` and `Buffer::row_text()` do the same for a bare buffer. A
double-width grapheme contributes one character and its trailing cell
contributes none, so the text lines up with what a terminal shows.

To compare a whole screen, `assert_frame!` reports the first row and column that
differ instead of dumping two screens and leaving you to spot it. Trailing
padding is ignored on both sides, so the expected screen can be written without
padding every line:

```rust
use hawktui::assert_frame;

assert_frame!(frames[0], "> Item 01
  Item 02
1 / 2");
```

## The ontology, and what it is not

Every widget publishes a machine-readable schema — properties with types and
constraints, a semantic role, invocable actions, tags:

```sh
hawktui-ontology list
hawktui-ontology schema Gauge
hawktui-ontology search scroll
```

From a checkout without installing: `cargo run --example ontology_query -- list`.

**Read this honestly:** the ontology describes a widget's *runtime state*, for
an agent driving a running application over `hawktui-server`. It is not a
catalog of builder methods, and today it names about an eighth of the public
API. If you are writing code, use it to find *which widget* you want and what it
conceptually holds — then read the rustdoc or the source for the methods that
build it. Expecting constructor signatures there will waste your time.

`docs/agent-integration.md` and `docs/agent-protocol.md` cover driving a running
app, which is the job the ontology was built for.

## Style

Widgets are builders that consume and return `self`:

```rust
let gauge = Gauge::new()
    .percent(42)                       // or .ratio(0.42)
    .label("Loading…")
    .block(Block::default().borders(Borders::ALL));
```

`Style` composes the same way, and `Stylize` gives shorthands on strings and
spans:

```rust
Style::default().fg(Color::Cyan).bold()
Span::styled("warning", Style::default().fg(Color::Yellow))
```

## Where to look next

- `examples/` — `hello`, `counter`, and the larger `lazygit`, `btop`, `opencode`
  reproductions, which are the best reference for real layout composition
- `docs/FEATURES.md` — what exists, and where Hawk TUI is behind alternatives
- `docs/agent-protocol.md` — the RPC an agent uses to drive a running app
