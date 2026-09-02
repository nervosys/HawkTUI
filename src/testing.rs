//! Drive a [`Model`] headlessly and read back what it rendered.
//!
//! A TUI is hard to test because the thing you want to assert on — the screen —
//! normally only exists as escape sequences on a real terminal. [`Harness`]
//! removes that: it owns a model and a [`TestBackend`], applies keys, and hands
//! back each frame as plain text.
//!
//! ```
//! use hawktui::prelude::*;
//! use hawktui::runtime::{Command, Model};
//! use hawktui::testing::Harness;
//!
//! struct Counter { n: i32 }
//! enum Msg { Inc, Quit }
//!
//! impl Model for Counter {
//!     type Msg = Msg;
//!     fn update(&mut self, msg: Msg) -> Command<Msg> {
//!         match msg {
//!             Msg::Inc => { self.n += 1; Command::None }
//!             Msg::Quit => Command::Quit,
//!         }
//!     }
//!     fn view(&self, frame: &mut Frame<'_>) {
//!         let text = Paragraph::new(Text::from(format!("Count: {}", self.n)));
//!         frame.render_widget(text, frame.area());
//!     }
//!     fn handle_event(&self, event: Event) -> Option<Msg> {
//!         let Event::Key(key) = event else { return None };
//!         match key.code {
//!             KeyCode::Char('+') => Some(Msg::Inc),
//!             KeyCode::Char('q') => Some(Msg::Quit),
//!             _ => None,
//!         }
//!     }
//! }
//!
//! let mut harness = Harness::new(Counter { n: 0 }, 20, 1).unwrap();
//! let frames = harness.run_script("+,+,q").unwrap();
//!
//! // One frame for the initial render, one per key — and none for `q`,
//! // because quitting ends the run.
//! assert_eq!(frames.len(), 3);
//! assert!(frames[0].contains("Count: 0"));
//! assert!(frames[2].contains("Count: 2"));
//! assert!(!harness.is_running());
//! ```

use std::io;

use crate::backend::test::TestBackend;
use crate::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crate::runtime::{Command, Model};
use crate::terminal::Terminal;

/// A model, a screen, and the loop between them — without a terminal.
pub struct Harness<M: Model> {
    model: M,
    terminal: Terminal<TestBackend>,
    running: bool,
}

impl<M: Model> Harness<M> {
    /// Build a harness around `model` with a `width` × `height` screen.
    ///
    /// The model's [`init`](Model::init) command runs immediately, exactly as
    /// [`Program`](crate::runtime::Program) would run it.
    pub fn new(model: M, width: u16, height: u16) -> io::Result<Self> {
        let terminal = Terminal::new(TestBackend::new(width, height))?;
        let mut harness = Self {
            model,
            terminal,
            running: true,
        };
        let cmd = harness.model.init();
        harness.process(cmd);
        Ok(harness)
    }

    /// The model, for assertions that are easier against state than pixels.
    pub fn model(&self) -> &M {
        &self.model
    }

    /// Whether the model is still running, i.e. has not returned
    /// [`Command::Quit`].
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Render a frame and return it as plain text.
    ///
    /// Lines are padded to the screen width and joined with `\n`.
    pub fn render(&mut self) -> io::Result<String> {
        let Self {
            model, terminal, ..
        } = self;
        terminal.draw(|frame| model.view(frame))?;
        Ok(self.text())
    }

    /// The most recently rendered frame, without drawing a new one.
    pub fn text(&self) -> String {
        self.terminal.backend().to_text()
    }

    /// Feed an event through [`handle_event`](Model::handle_event) and, if it
    /// produced a message, through [`update`](Model::update).
    pub fn send(&mut self, event: Event) {
        if let Some(msg) = self.model.handle_event(event) {
            let cmd = self.model.update(msg);
            self.process(cmd);
        }
    }

    /// Send a key press with no modifiers.
    pub fn key(&mut self, code: KeyCode) {
        self.send(Event::Key(KeyEvent::new(code, KeyModifiers::NONE)));
    }

    /// Send a character key press.
    pub fn char(&mut self, c: char) {
        self.key(KeyCode::Char(c));
    }

    /// Render the initial frame, then apply each key in `script` and render
    /// after every one, returning all the frames.
    ///
    /// `script` is a comma-separated list of [key names](parse_key), e.g.
    /// `"Down,Down,Enter,q"`. When a key quits the model, the run stops
    /// **without** a frame for it — so the frame count tells you whether the
    /// quit key works.
    pub fn run_script(&mut self, script: &str) -> io::Result<Vec<String>> {
        let mut frames = vec![self.render()?];
        for name in script.split(',') {
            let name = name.trim();
            if name.is_empty() {
                continue;
            }
            let code = parse_key(name).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, format!("unknown key {name:?}"))
            })?;
            self.key(code);
            if !self.running {
                break;
            }
            frames.push(self.render()?);
        }
        Ok(frames)
    }

    /// Apply a command the way [`Program`](crate::runtime::Program) does.
    ///
    /// [`Command::Task`] is deliberately not executed: it runs on a background
    /// thread, and a test harness that sometimes ran it would be
    /// non-deterministic. Drive the resulting message yourself with
    /// [`send`](Self::send) if the behaviour matters.
    fn process(&mut self, cmd: Command<M::Msg>) {
        match cmd {
            Command::Quit => self.running = false,
            Command::Batch(cmds) => {
                for c in cmds {
                    self.process(c);
                }
            }
            Command::Message(msg) => {
                let next = self.model.update(msg);
                self.process(next);
            }
            Command::None
            | Command::SetTickRate(_)
            | Command::ExportOntology
            | Command::AgentAction { .. }
            | Command::Task(_) => {}
        }
    }
}

/// Compare a rendered frame against an expected screen.
///
/// Returns `None` when they match, or a report naming the first row that
/// differs and the column where it diverges. Trailing spaces are ignored on
/// both sides, so an expected screen can be written without padding every line
/// out to the full width.
///
/// ```
/// use hawktui::testing::frame_diff;
///
/// assert!(frame_diff("ab\ncd", "ab\ncd").is_none());
/// let report = frame_diff("ab\ncd", "ab\ncX").unwrap();
/// assert!(report.contains("row 1"));
/// ```
pub fn frame_diff(actual: &str, expected: &str) -> Option<String> {
    let a: Vec<&str> = actual.lines().map(str::trim_end).collect();
    let b: Vec<&str> = expected.lines().map(str::trim_end).collect();

    let differing: Vec<usize> = (0..a.len().max(b.len()))
        .filter(|&i| a.get(i).unwrap_or(&"") != b.get(i).unwrap_or(&""))
        .collect();
    let first = *differing.first()?;

    let got = a.get(first).unwrap_or(&"<missing>");
    let want = b.get(first).unwrap_or(&"<missing>");
    let column = got
        .chars()
        .zip(want.chars())
        .position(|(x, y)| x != y)
        .unwrap_or_else(|| got.chars().count().min(want.chars().count()));

    let mut report = String::from("rendered frame does not match\n");
    if a.len() != b.len() {
        report.push_str(&format!(
            "  height: rendered {} rows, expected {}\n",
            a.len(),
            b.len()
        ));
    }
    report.push_str(&format!(
        "  {} row(s) differ; first at row {}, column {}\n",
        differing.len(),
        first,
        column
    ));
    report.push_str(&format!("    expected: {want:?}\n"));
    report.push_str(&format!("    rendered: {got:?}\n"));
    report.push_str(&format!("              {}^\n", " ".repeat(column + 1)));
    Some(report)
}

/// Assert that a rendered frame matches an expected screen.
///
/// On failure the panic names the first differing row and column rather than
/// dumping two screens and leaving you to find the difference.
///
/// ```
/// use hawktui::assert_frame;
///
/// let rendered = "ab   \ncd   ";
/// assert_frame!(rendered, "ab\ncd");
/// ```
#[macro_export]
macro_rules! assert_frame {
    ($actual:expr, $expected:expr $(,)?) => {
        if let Some(report) = $crate::testing::frame_diff(&$actual, $expected) {
            panic!("{}", report);
        }
    };
}

/// Parse a key name used by [`Harness::run_script`].
///
/// Named keys are `Up`, `Down`, `Left`, `Right`, `Enter`, `Esc`, `Tab`,
/// `BackTab`, `Backspace`, `Delete`, `Home`, `End`, `PageUp`, `PageDown`,
/// `Insert` and `Space`. Function keys are `F1`–`F12`. Any single character is
/// itself, so `a`, `+` and `q` all work.
///
/// ```
/// use hawktui::event::KeyCode;
/// use hawktui::testing::parse_key;
///
/// assert_eq!(parse_key("Down"), Some(KeyCode::Down));
/// assert_eq!(parse_key("q"), Some(KeyCode::Char('q')));
/// assert_eq!(parse_key("Space"), Some(KeyCode::Char(' ')));
/// assert_eq!(parse_key("nope"), None);
/// ```
pub fn parse_key(name: &str) -> Option<KeyCode> {
    let code = match name {
        "Up" => KeyCode::Up,
        "Down" => KeyCode::Down,
        "Left" => KeyCode::Left,
        "Right" => KeyCode::Right,
        "Enter" => KeyCode::Enter,
        "Esc" => KeyCode::Esc,
        "Tab" => KeyCode::Tab,
        "BackTab" => KeyCode::BackTab,
        "Backspace" => KeyCode::Backspace,
        "Delete" => KeyCode::Delete,
        "Home" => KeyCode::Home,
        "End" => KeyCode::End,
        "PageUp" => KeyCode::PageUp,
        "PageDown" => KeyCode::PageDown,
        "Insert" => KeyCode::Insert,
        "Space" => KeyCode::Char(' '),
        other => {
            if let Some(n) = other.strip_prefix('F').and_then(|d| d.parse::<u8>().ok()) {
                if (1..=12).contains(&n) {
                    return Some(KeyCode::F(n));
                }
            }
            let mut chars = other.chars();
            let first = chars.next()?;
            if chars.next().is_some() {
                return None;
            }
            KeyCode::Char(first)
        }
    };
    Some(code)
}
