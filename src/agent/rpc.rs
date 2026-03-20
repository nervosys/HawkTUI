//! RPC transport: JSON Lines over stdin/stdout.
//!
//! This implements the stdio-based JSON Lines protocol used by AI coding agents
//! (like OpenCode/Pi, OpenClaw, etc.) to embed and control Louie applications.
//!
//! Protocol:
//! - Each line on stdin is a JSON [`RequestEnvelope`]
//! - Each line on stdout is a JSON [`AgentResponse`] or [`AgentEvent`]
//! - Lines are delimited by `\n`

use std::io::{self, BufRead, Write};
use std::time::Instant;

use super::protocol::{AgentResponse, RequestEnvelope};
use super::session::AgentSession;
use crate::backend::test::TestBackend;
use crate::ontology::OntologyRegistry;
use crate::runtime::{Command, Model};
use crate::terminal::Terminal;

/// Maximum allowed size for a single JSON request line (1 MB).
const MAX_LINE_BYTES: usize = 1_048_576;

/// Maximum requests per second before throttling (INP-4).
const MAX_REQUESTS_PER_SEC: u32 = 1000;

/// Runs a Louie application over stdin/stdout JSON Lines protocol.
///
/// The agent sends [`RequestEnvelope`] JSON objects (one per line) on stdin.
/// The transport responds with [`AgentResponse`] JSON objects on stdout.
pub struct RpcTransport<M: Model> {
    model: M,
    terminal: Terminal<TestBackend>,
    session: AgentSession,
    ontology: OntologyRegistry,
    running: bool,
}

impl<M: Model> RpcTransport<M> {
    /// Create a new RPC transport with the given model and virtual terminal size.
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

    /// Run the RPC loop, reading from stdin and writing to stdout.
    ///
    /// This blocks until the agent sends a `quit` request or stdin closes.
    pub fn run(mut self) -> io::Result<M> {
        // Initialize
        let init_cmd = self.model.init();
        self.process_command(init_cmd);
        self.model.register_ontology(&mut self.ontology);

        // Initial render
        let model = &self.model;
        self.terminal.draw(|frame| {
            model.view(frame);
        })?;

        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let reader = stdin.lock();

        // Rate limiter state (INP-4)
        let mut window_start = Instant::now();
        let mut request_count: u32 = 0;

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Rate limiting (INP-4)
            let elapsed = window_start.elapsed();
            if elapsed.as_secs() >= 1 {
                window_start = Instant::now();
                request_count = 0;
            }
            request_count += 1;
            if request_count > MAX_REQUESTS_PER_SEC {
                let resp = AgentResponse::err(format!(
                    "Rate limit exceeded ({MAX_REQUESTS_PER_SEC} req/s)"
                ));
                let json = serde_json::to_string(&resp).unwrap_or_default();
                writeln!(stdout, "{json}")?;
                stdout.flush()?;
                continue;
            }

            // Reject oversized requests (INP-1)
            if trimmed.len() > MAX_LINE_BYTES {
                let resp = AgentResponse::err(format!(
                    "Request too large ({} bytes, max {})",
                    trimmed.len(),
                    MAX_LINE_BYTES
                ));
                let json = serde_json::to_string(&resp).unwrap_or_default();
                writeln!(stdout, "{json}")?;
                stdout.flush()?;
                continue;
            }

            let envelope: RequestEnvelope = match serde_json::from_str(trimmed) {
                Ok(e) => e,
                Err(err) => {
                    let resp = AgentResponse::err(format!("Invalid JSON: {err}"));
                    let json = serde_json::to_string(&resp).unwrap_or_default();
                    writeln!(stdout, "{json}")?;
                    stdout.flush()?;
                    continue;
                }
            };

            let (mut response, should_quit) = self
                .session
                .process_request(&envelope.request, &self.ontology);

            // Handle side effects
            if let super::protocol::AgentRequest::ExecuteAction {
                agent_id,
                action,
                params,
            } = &envelope.request
            {
                let cmd = Command::AgentAction {
                    agent_id: agent_id.clone(),
                    action: action.clone(),
                    params: params.clone(),
                };
                self.process_command(cmd);
            }

            if let super::protocol::AgentRequest::InjectEvent { event } = &envelope.request {
                if let Some(ev) = AgentSession::convert_injected_event(event) {
                    if let Some(msg) = self.model.handle_event(ev) {
                        let cmd = self.model.update(msg);
                        self.process_command(cmd);
                    }
                }
            }

            // Re-render after processing
            let model = &self.model;
            self.terminal.draw(|frame| {
                model.view(frame);
            })?;

            // Set response ID
            if let Some(ref id) = envelope.id {
                response = response.with_id(id.clone());
            }

            let json = serde_json::to_string(&response).unwrap_or_default();
            writeln!(stdout, "{json}")?;
            stdout.flush()?;

            if should_quit {
                self.running = false;
                break;
            }
        }

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
            Command::SetTickRate(_) => {}
            Command::ExportOntology => {
                self.model.register_ontology(&mut self.ontology);
            }
            Command::AgentAction {
                agent_id: _,
                action: _,
                params: _,
            } => {}
            Command::Task(_) => {}
        }
    }
}
