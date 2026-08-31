//! Headless agent driver: run a Hawk TUI app without a real terminal.
//!
//! The driver uses a [`TestBackend`] to render frames in memory and exposes
//! the full agent protocol for programmatic control. This enables:
//! - Automated testing of Hawk TUI apps
//! - Agent-only operation (no human at the terminal)
//! - CI/CD pipeline integration

use std::io;

use crate::backend::test::TestBackend;
use crate::ontology::OntologyRegistry;
use crate::runtime::{Command, Model};
use crate::terminal::Terminal;

use super::protocol::{AgentRequest, AgentResponse, RequestEnvelope};
use super::session::AgentSession;

/// Run a Hawk TUI application headlessly, driven entirely by agent protocol messages.
pub struct HeadlessDriver<M: Model> {
    model: M,
    terminal: Terminal<TestBackend>,
    session: AgentSession,
    ontology: OntologyRegistry,
    running: bool,
}

impl<M: Model> HeadlessDriver<M> {
    /// Create a new headless driver with the given model and virtual terminal size.
    pub fn new(model: M, width: u16, height: u16) -> io::Result<Self> {
        let backend = TestBackend::new(width, height);
        let terminal = Terminal::new(backend)?;
        let mut ontology = OntologyRegistry::new();
        model.register_ontology(&mut ontology);

        Ok(Self {
            model,
            terminal,
            session: AgentSession::new(),
            ontology,
            running: true,
        })
    }

    /// Whether the application is still running.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Access the current model.
    pub fn model(&self) -> &M {
        &self.model
    }

    /// Access the ontology registry.
    pub fn ontology(&self) -> &OntologyRegistry {
        &self.ontology
    }

    /// Access the agent session.
    pub fn session(&self) -> &AgentSession {
        &self.session
    }

    /// Get the rendered text content of a specific row.
    pub fn row_text(&self, y: u16) -> String {
        self.terminal.backend().row_text(y)
    }

    /// Render the current model state into the virtual terminal.
    pub fn render(&mut self) -> io::Result<()> {
        let model = &self.model;
        self.terminal.draw(|frame| {
            model.view(frame);
        })
    }

    /// Process a single agent request and return the response.
    pub fn process_request(&mut self, request: &AgentRequest) -> AgentResponse {
        let (response, should_quit) = self.session.process_request(request, &self.ontology);

        // Handle execute_action by dispatching through the model
        if let AgentRequest::ExecuteAction {
            agent_id,
            action,
            params,
        } = request
        {
            let cmd = Command::AgentAction {
                agent_id: agent_id.clone(),
                action: action.clone(),
                params: params.clone(),
            };
            self.process_command(cmd);
        }

        // Handle injected events
        if let AgentRequest::InjectEvent { event } = request {
            if let Some(ev) = AgentSession::convert_injected_event(event) {
                if let Some(msg) = self.model.handle_event(ev) {
                    let cmd = self.model.update(msg);
                    self.process_command(cmd);
                }
            }
        }

        if should_quit {
            self.running = false;
        }

        response
    }

    /// Process a framed request envelope.
    pub fn process_envelope(&mut self, envelope: &RequestEnvelope) -> AgentResponse {
        let mut response = self.process_request(&envelope.request);
        if let Some(ref id) = envelope.id {
            response = response.with_id(id.clone());
        }
        response
    }

    /// Inject a tick event (advance animations / periodic updates).
    pub fn tick(&mut self) {
        if let Some(msg) = self.model.handle_event(crate::event::Event::Tick) {
            let cmd = self.model.update(msg);
            self.process_command(cmd);
        }
    }

    /// Run the init command for the model.
    pub fn init(&mut self) {
        let cmd = self.model.init();
        self.process_command(cmd);
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
            Command::SetTickRate(_) => {
                // Ignored in headless mode; the caller controls tick timing.
            }
            Command::ExportOntology => {
                self.model.register_ontology(&mut self.ontology);
            }
            Command::AgentAction {
                agent_id: _,
                action: _,
                params: _,
            } => {
                // Agent actions flow through the model's update as messages.
                // In headless mode the model is expected to handle them.
            }
            Command::Task(_) => {
                // Async tasks are not executed in the synchronous headless driver.
                // Use RpcTransport for async execution.
            }
        }
    }
}
