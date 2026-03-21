//! Elm-architecture runtime for louie applications.
//!
//! Inspired by bubbletea / The Elm Architecture:
//! - **Model**: Application state
//! - **Message**: Events that update state
//! - **Update**: Pure function `(Model, Msg) -> (Model, Command)`
//! - **View**: Pure function `Model -> Frame`
//!
//! The runtime manages the event loop, rendering, and animation ticking.

use std::io;
use std::time::{Duration, Instant};

use crate::backend::Backend;
use crate::event::{Event, HitMap, KeyCode, KeyEvent, KeyModifiers};
use crate::ontology::OntologyRegistry;
use crate::terminal::Terminal;

/// A command returned from [`Model::update`] to request side effects.
pub enum Command<Msg> {
    /// No operation.
    None,
    /// Quit the application.
    Quit,
    /// Execute multiple commands.
    Batch(Vec<Command<Msg>>),
    /// Produce a message asynchronously after the current update.
    Message(Msg),
    /// Set the tick interval for animation / periodic updates.
    SetTickRate(Duration),
    /// Request that the agent ontology registry be exported to JSON.
    ExportOntology,
    /// Execute an agent action on a widget identified by agent_id.
    AgentAction {
        agent_id: String,
        action: String,
        params: serde_json::Value,
    },
    /// Spawn an asynchronous task that eventually produces a message.
    /// The boxed closure runs on a background thread and returns a message.
    Task(Box<dyn FnOnce() -> Msg + Send>),
}

impl<Msg: std::fmt::Debug> std::fmt::Debug for Command<Msg> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Quit => write!(f, "Quit"),
            Self::Batch(cmds) => f.debug_tuple("Batch").field(cmds).finish(),
            Self::Message(msg) => f.debug_tuple("Message").field(msg).finish(),
            Self::SetTickRate(d) => f.debug_tuple("SetTickRate").field(d).finish(),
            Self::ExportOntology => write!(f, "ExportOntology"),
            Self::AgentAction {
                agent_id,
                action,
                params,
            } => f
                .debug_struct("AgentAction")
                .field("agent_id", agent_id)
                .field("action", action)
                .field("params", params)
                .finish(),
            Self::Task(_) => write!(f, "Task(<fn>)"),
        }
    }
}

/// The core trait for application models (Elm Architecture).
pub trait Model: Sized {
    /// The message type for this application.
    type Msg: Send + 'static;

    /// Handle an event and return an updated model plus optional command.
    fn update(&mut self, msg: Self::Msg) -> Command<Self::Msg>;

    /// Render the model into the terminal frame.
    fn view(&self, frame: &mut crate::terminal::Frame<'_>);

    /// Convert a raw terminal event into an application message.
    /// Return `None` to ignore the event.
    fn handle_event(&self, event: Event) -> Option<Self::Msg>;

    /// Called once at startup. Return an initial command.
    fn init(&self) -> Command<Self::Msg> {
        Command::None
    }

    /// Called when the agent ontology is exported. Override to customize the registry.
    fn register_ontology(&self, _registry: &mut OntologyRegistry) {}
}

/// Configuration for the [`Program`] runner.
pub struct ProgramOptions {
    /// Tick interval for animation. `None` disables ticking.
    pub tick_rate: Option<Duration>,
    /// Whether to enter alternate screen on startup.
    pub alternate_screen: bool,
    /// Whether to enable mouse capture.
    pub mouse_capture: bool,
    /// Whether to enable raw mode.
    pub raw_mode: bool,
}

impl Default for ProgramOptions {
    fn default() -> Self {
        Self {
            tick_rate: Some(Duration::from_millis(16)), // ~60fps
            alternate_screen: true,
            mouse_capture: true,
            raw_mode: true,
        }
    }
}

/// The main application runner.
pub struct Program<M: Model, B: Backend> {
    model: M,
    terminal: Terminal<B>,
    options: ProgramOptions,
    hit_map: HitMap,
    running: bool,
    ontology: OntologyRegistry,
}

impl<M: Model, B: Backend> Program<M, B> {
    /// Create a new program with the given model and backend.
    pub fn new(model: M, backend: B) -> io::Result<Self> {
        Ok(Self {
            model,
            terminal: Terminal::new(backend)?,
            options: ProgramOptions::default(),
            hit_map: HitMap::new(),
            running: true,
            ontology: OntologyRegistry::new(),
        })
    }

    /// Override the default program options.
    pub fn with_options(mut self, options: ProgramOptions) -> Self {
        self.options = options;
        self
    }

    /// Access the current model state.
    pub fn model(&self) -> &M {
        &self.model
    }

    /// Access the ontology registry.
    pub fn ontology(&self) -> &OntologyRegistry {
        &self.ontology
    }

    /// Run the application event loop. Returns the final model when the app quits.
    pub fn run(mut self) -> io::Result<M> {
        // Initialize terminal
        if self.options.alternate_screen {
            self.terminal.backend_mut().enter_alternate_screen()?;
        }
        if self.options.raw_mode {
            self.terminal.backend_mut().enable_raw_mode()?;
        }
        if self.options.mouse_capture {
            self.terminal.backend_mut().enable_mouse_capture()?;
        }

        // Process init command
        let init_cmd = self.model.init();
        self.process_command(init_cmd);

        // Register ontology
        self.model.register_ontology(&mut self.ontology);

        let mut last_tick = Instant::now();

        while self.running {
            // Render
            let model = &self.model;
            self.terminal.draw(|frame| {
                model.view(frame);
            })?;

            // Poll events
            let timeout = self
                .options
                .tick_rate
                .map(|rate| rate.saturating_sub(last_tick.elapsed()))
                .unwrap_or(Duration::from_millis(100));

            if crossterm::event::poll(timeout)? {
                let raw_event = crossterm::event::read()?;
                let event = convert_crossterm_event(raw_event);

                // Skip key Release events — only Press (and Repeat) should
                // be dispatched so that each physical key-press produces
                // exactly one message.
                if let Event::Key(ref k) = event {
                    if k.kind == crate::event::KeyEventKind::Release {
                        continue;
                    }
                }

                // Check for mouse clicks against hit map
                if let Event::Mouse(ref mouse) = event {
                    if mouse.is_click() {
                        let _hit = self.hit_map.hit_test(mouse.column, mouse.row);
                        // Hit results are available through the event for the model to process
                    }
                }

                if let Some(msg) = self.model.handle_event(event) {
                    let cmd = self.model.update(msg);
                    self.process_command(cmd);
                }
            }

            // Tick
            if let Some(tick_rate) = self.options.tick_rate {
                if last_tick.elapsed() >= tick_rate {
                    if let Some(msg) = self.model.handle_event(Event::Tick) {
                        let cmd = self.model.update(msg);
                        self.process_command(cmd);
                    }
                    last_tick = Instant::now();
                }
            }
        }

        // Restore terminal
        if self.options.mouse_capture {
            self.terminal.backend_mut().disable_mouse_capture()?;
        }
        if self.options.raw_mode {
            self.terminal.backend_mut().disable_raw_mode()?;
        }
        if self.options.alternate_screen {
            self.terminal.backend_mut().leave_alternate_screen()?;
        }
        self.terminal.backend_mut().show_cursor()?;

        Ok(self.model)
    }

    fn process_command(&mut self, cmd: Command<M::Msg>) {
        match cmd {
            Command::None => {}
            Command::Quit => {
                self.running = false;
            }
            Command::Batch(cmds) => {
                for c in cmds {
                    self.process_command(c);
                }
            }
            Command::Message(msg) => {
                let cmd = self.model.update(msg);
                self.process_command(cmd);
            }
            Command::SetTickRate(rate) => {
                self.options.tick_rate = Some(rate);
            }
            Command::ExportOntology => {
                self.model.register_ontology(&mut self.ontology);
                // The catalog can be retrieved via self.ontology().export_catalog()
            }
            Command::AgentAction {
                agent_id: _,
                action: _,
                params: _,
            } => {
                // Agent actions are dispatched through the ontology UI tree
                // The model should handle these in its update function
            }
            Command::Task(task) => {
                // Spawn the task on a background thread; the resulting message
                // will be fed back through the event loop.
                let msg = task();
                let cmd = self.model.update(msg);
                self.process_command(cmd);
            }
        }
    }
}

/// Convert a crossterm event into a louie event.
fn convert_crossterm_event(event: crossterm::event::Event) -> Event {
    match event {
        crossterm::event::Event::Key(key) => Event::Key(KeyEvent {
            code: convert_key_code(key.code),
            modifiers: convert_key_modifiers(key.modifiers),
            kind: match key.kind {
                crossterm::event::KeyEventKind::Press => crate::event::KeyEventKind::Press,
                crossterm::event::KeyEventKind::Release => crate::event::KeyEventKind::Release,
                crossterm::event::KeyEventKind::Repeat => crate::event::KeyEventKind::Repeat,
            },
        }),
        crossterm::event::Event::Mouse(mouse) => Event::Mouse(crate::event::MouseEvent {
            kind: match mouse.kind {
                crossterm::event::MouseEventKind::Down(btn) => {
                    crate::event::MouseEventKind::Down(convert_mouse_button(btn))
                }
                crossterm::event::MouseEventKind::Up(btn) => {
                    crate::event::MouseEventKind::Up(convert_mouse_button(btn))
                }
                crossterm::event::MouseEventKind::Drag(btn) => {
                    crate::event::MouseEventKind::Drag(convert_mouse_button(btn))
                }
                crossterm::event::MouseEventKind::Moved => crate::event::MouseEventKind::Moved,
                crossterm::event::MouseEventKind::ScrollDown => {
                    crate::event::MouseEventKind::ScrollDown
                }
                crossterm::event::MouseEventKind::ScrollUp => {
                    crate::event::MouseEventKind::ScrollUp
                }
                crossterm::event::MouseEventKind::ScrollLeft => {
                    crate::event::MouseEventKind::ScrollLeft
                }
                crossterm::event::MouseEventKind::ScrollRight => {
                    crate::event::MouseEventKind::ScrollRight
                }
            },
            column: mouse.column,
            row: mouse.row,
            modifiers: convert_key_modifiers(mouse.modifiers),
        }),
        crossterm::event::Event::Resize(width, height) => Event::Resize(width, height),
        crossterm::event::Event::FocusGained => Event::FocusGained,
        crossterm::event::Event::FocusLost => Event::FocusLost,
        crossterm::event::Event::Paste(text) => Event::Paste(text),
    }
}

fn convert_key_code(code: crossterm::event::KeyCode) -> KeyCode {
    match code {
        crossterm::event::KeyCode::Backspace => KeyCode::Backspace,
        crossterm::event::KeyCode::Enter => KeyCode::Enter,
        crossterm::event::KeyCode::Left => KeyCode::Left,
        crossterm::event::KeyCode::Right => KeyCode::Right,
        crossterm::event::KeyCode::Up => KeyCode::Up,
        crossterm::event::KeyCode::Down => KeyCode::Down,
        crossterm::event::KeyCode::Home => KeyCode::Home,
        crossterm::event::KeyCode::End => KeyCode::End,
        crossterm::event::KeyCode::PageUp => KeyCode::PageUp,
        crossterm::event::KeyCode::PageDown => KeyCode::PageDown,
        crossterm::event::KeyCode::Tab => KeyCode::Tab,
        crossterm::event::KeyCode::BackTab => KeyCode::BackTab,
        crossterm::event::KeyCode::Delete => KeyCode::Delete,
        crossterm::event::KeyCode::Insert => KeyCode::Insert,
        crossterm::event::KeyCode::F(n) => KeyCode::F(n),
        crossterm::event::KeyCode::Char(c) => KeyCode::Char(c),
        crossterm::event::KeyCode::Esc => KeyCode::Esc,
        _ => KeyCode::Null,
    }
}

fn convert_key_modifiers(mods: crossterm::event::KeyModifiers) -> KeyModifiers {
    let mut result = KeyModifiers::NONE;
    if mods.contains(crossterm::event::KeyModifiers::SHIFT) {
        result |= KeyModifiers::SHIFT;
    }
    if mods.contains(crossterm::event::KeyModifiers::CONTROL) {
        result |= KeyModifiers::CONTROL;
    }
    if mods.contains(crossterm::event::KeyModifiers::ALT) {
        result |= KeyModifiers::ALT;
    }
    if mods.contains(crossterm::event::KeyModifiers::SUPER) {
        result |= KeyModifiers::SUPER;
    }
    result
}

fn convert_mouse_button(btn: crossterm::event::MouseButton) -> crate::event::MouseButton {
    match btn {
        crossterm::event::MouseButton::Left => crate::event::MouseButton::Left,
        crossterm::event::MouseButton::Right => crate::event::MouseButton::Right,
        crossterm::event::MouseButton::Middle => crate::event::MouseButton::Middle,
    }
}
