//! louie-server — Headless Louie RPC server for AI agents.
//!
//! A standalone binary that exposes a Louie application over the JSON Lines
//! protocol on stdin/stdout. AI agents (Claude, GPT, Codex, or any LLM-based
//! system) spawn this process and communicate through structured JSON messages.
//!
//! # Usage
//!
//! ```sh
//! louie-server [--width W] [--height H] [--help] [--version]
//! ```
//!
//! # Protocol
//!
//! Send one JSON object per line on stdin. Receive one JSON response per line
//! on stdout. See `docs/agent-protocol.md` for the full specification.
//!
//! # Examples
//!
//! ```sh
//! # Test connectivity
//! echo '{"type":"ping"}' | louie-server
//!
//! # Discover all widget types
//! echo '{"type":"query_ontology"}' | louie-server
//!
//! # Interactive session (pipe from agent)
//! my-agent | louie-server
//! ```

use std::io::{self, Write};
use std::time::Instant;

use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

use louie::agent::driver::HeadlessDriver;
use louie::agent::protocol::{AgentRequest, RequestEnvelope};
use louie::agent::read_capped_line;
// Security limits (see docs/SECURITY-AUDIT.md INP-1, INP-2, INP-4)
const MAX_LINE_BYTES: usize = 1_048_576; // 1 MB max request size
const MAX_WIDTH: u16 = 1024;
const MAX_HEIGHT: u16 = 512;
const MAX_REQUESTS_PER_SEC: u32 = 1000; // Rate limit (INP-4)
use louie::ontology::registry::OntologyRegistry;
use louie::prelude::*;
use louie::runtime::{Command, Model};
use louie::widget::gauge::Gauge;
use louie::widget::input::Input;
use louie::widget::list::{List, ListItem, ListState};
use louie::widget::select_list::SelectList;

// ---------------------------------------------------------------------------
// Demo application: a multi-widget showcase for agent exploration
// ---------------------------------------------------------------------------

struct DemoApp {
    counter: i64,
    items: Vec<String>,
    list_state: ListState,
    log: Vec<String>,
}

impl DemoApp {
    fn new() -> Self {
        Self {
            counter: 0,
            items: vec![
                "Task: Review PR #42".into(),
                "Task: Deploy staging".into(),
                "Task: Write unit tests".into(),
                "Task: Update docs".into(),
                "Bug: Fix login timeout".into(),
            ],
            list_state: ListState::default(),
            log: vec!["Server started. Waiting for agent commands.".into()],
        }
    }
}

#[derive(Debug)]
enum Msg {
    Increment,
    Decrement,
}

impl Model for DemoApp {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Increment => {
                self.counter += 1;
                self.log
                    .push(format!("Counter incremented to {}", self.counter));
            }
            Msg::Decrement => {
                self.counter -= 1;
                self.log
                    .push(format!("Counter decremented to {}", self.counter));
            }
        }
        Command::None
    }

    fn view(&self, frame: &mut Frame) {
        let area = frame.area();
        let main_layout = louie::layout::Layout::default()
            .direction(louie::layout::Direction::Vertical)
            .constraints([
                louie::layout::Constraint::Length(3),
                louie::layout::Constraint::Length(3),
                louie::layout::Constraint::Fill(1),
                louie::layout::Constraint::Length(5),
            ])
            .split(area);

        // Counter display
        let counter_text = Paragraph::new(Text::from(format!("Counter: {}", self.counter))).block(
            Block::default()
                .title("Counter (agent_id: counter)")
                .borders(Borders::ALL),
        );
        frame.render_widget(counter_text, main_layout[0]);

        // Gauge
        let ratio = (self.counter as f64 / 100.0).clamp(0.0, 1.0);
        let gauge = Gauge::new()
            .ratio(ratio)
            .label(format!("{:.0}%", ratio * 100.0))
            .block(
                Block::default()
                    .title("Progress (agent_id: gauge)")
                    .borders(Borders::ALL),
            );
        frame.render_widget(gauge, main_layout[1]);

        // Task list
        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|i| ListItem::new(i.clone()))
            .collect();
        let list = List::new(items).block(
            Block::default()
                .title("Tasks (agent_id: tasks)")
                .borders(Borders::ALL),
        );
        frame.render_stateful_widget(list, main_layout[2], &mut self.list_state.clone());

        // Log
        let log_text: Vec<String> = self.log.iter().rev().take(3).rev().cloned().collect();
        let log = Paragraph::new(Text::from(log_text.join("\n")))
            .block(Block::default().title("Agent Log").borders(Borders::ALL));
        frame.render_widget(log, main_layout[3]);
    }

    fn handle_event(&self, event: Event) -> Option<Msg> {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => Some(Msg::Increment),
                KeyCode::Down | KeyCode::Char('j') => Some(Msg::Decrement),
                _ => None,
            }
        } else {
            None
        }
    }

    fn register_ontology(&self, registry: &mut OntologyRegistry) {
        registry.register::<Paragraph>();
        registry.register::<Gauge>();
        registry.register::<List>();
        registry.register::<Block>();
        registry.register::<Input>();
        registry.register::<SelectList>();
    }
}

// ---------------------------------------------------------------------------
// CLI and main loop
// ---------------------------------------------------------------------------

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Extract a human-readable name for the request type.
fn request_type_name(req: &AgentRequest) -> &'static str {
    match req {
        AgentRequest::Ping => "ping",
        AgentRequest::Quit => "quit",
        AgentRequest::QueryOntology { .. } => "query_ontology",
        AgentRequest::GetSchema { .. } => "get_schema",
        AgentRequest::GetTree => "get_tree",
        AgentRequest::GetState { .. } => "get_state",
        AgentRequest::ExecuteAction { .. } => "execute_action",
        AgentRequest::InjectEvent { .. } => "inject_event",
        AgentRequest::Subscribe { .. } => "subscribe",
        AgentRequest::Unsubscribe { .. } => "unsubscribe",
    }
}

struct Args {
    width: u16,
    height: u16,
    show_help: bool,
    show_version: bool,
    auth_token: Option<String>,
}

fn parse_args() -> Args {
    let mut args = Args {
        width: 120,
        height: 40,
        show_help: false,
        show_version: false,
        auth_token: None,
    };

    let mut iter = std::env::args().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--help" | "-h" => args.show_help = true,
            "--version" | "-V" => args.show_version = true,
            "--width" | "-w" => {
                if let Some(val) = iter.next() {
                    args.width = val.parse().unwrap_or(120);
                }
            }
            "--height" | "-H" => {
                if let Some(val) = iter.next() {
                    args.height = val.parse().unwrap_or(40);
                }
            }
            "--auth-token" => {
                if let Some(val) = iter.next() {
                    args.auth_token = Some(val);
                }
            }
            _ => {
                eprintln!("Unknown argument: {arg}");
                eprintln!("Run with --help for usage.");
                std::process::exit(1);
            }
        }
    }

    // Clamp terminal dimensions to safe bounds (INP-2)
    args.width = args.width.clamp(1, MAX_WIDTH);
    args.height = args.height.clamp(1, MAX_HEIGHT);

    args
}

fn print_help() {
    eprintln!(
        r#"louie-server v{VERSION} — Headless Louie RPC server for AI agents

USAGE:
    louie-server [OPTIONS]

OPTIONS:
    -w, --width <W>     Virtual terminal width  [default: 120]
    -H, --height <H>    Virtual terminal height [default: 40]
    --auth-token <T>    Require agent to authenticate with this token
    -h, --help          Show this help message
    -V, --version       Show version

PROTOCOL:
    Send one JSON object per line on stdin. Receive one JSON response per line
    on stdout. Diagnostic messages are printed to stderr.

    Request types:
      {{"type":"ping"}}                                 Connection test
      {{"type":"query_ontology"}}                       List all widget types
      {{"type":"query_ontology","query":"Input"}}       Search by name/tag
      {{"type":"get_schema","widget_type":"Input"}}     Get widget schema
      {{"type":"get_tree"}}                             Get UI tree snapshot
      {{"type":"get_state","agent_id":"counter"}}       Get widget state
      {{"type":"execute_action","agent_id":"counter",
        "action":"increment","params":{{}}}}             Execute widget action
      {{"type":"inject_event","event":{{
        "kind":"key","code":"Up"}}}}                     Inject input event
      {{"type":"subscribe","events":["state_changed"]}} Subscribe to events
      {{"type":"quit"}}                                 Shut down server

    Responses:
      {{"success":true,"data":{{...}}}}                  Success
      {{"success":false,"error":"..."}}                  Error

    All requests support an optional "id" field for correlation.

EXAMPLES:
    # Test connectivity
    echo '{{"type":"ping"}}' | louie-server

    # Discover widget ontology
    echo '{{"type":"query_ontology"}}' | louie-server

    # Interactive session with an AI agent
    my-agent | louie-server

    # Python agent example
    import subprocess, json
    proc = subprocess.Popen(["louie-server"], stdin=PIPE, stdout=PIPE, text=True)
    proc.stdin.write(json.dumps({{"type":"ping"}}) + "\n")
    proc.stdin.flush()
    print(json.loads(proc.stdout.readline()))

For full protocol documentation, see docs/agent-protocol.md"#
    );
}

fn main() -> io::Result<()> {
    let args = parse_args();

    if args.show_version {
        eprintln!("louie-server v{VERSION}");
        return Ok(());
    }
    if args.show_help {
        print_help();
        return Ok(());
    }

    // Initialise tracing subscriber (writes to stderr, respects RUST_LOG)
    tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    info!(version = VERSION, "startup");
    info!(
        width = args.width,
        height = args.height,
        max_rps = MAX_REQUESTS_PER_SEC,
        "config"
    );
    info!("Listening on stdin for JSON Lines requests");

    let app = DemoApp::new();
    let mut driver = HeadlessDriver::new(app, args.width, args.height)?;
    driver.init();
    driver.render()?;

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut reader = stdin.lock();

    // Optional auth handshake (AUTH-1)
    if let Some(ref expected_token) = args.auth_token {
        info!("Waiting for auth handshake");
        let authenticated = loop {
            let (raw, oversized) = match read_capped_line(&mut reader, MAX_LINE_BYTES)? {
                Some(v) => v,
                None => break false,
            };
            // Cap buffering so an unauthenticated client cannot exhaust memory
            // with an unbounded line before the handshake completes.
            if oversized {
                let resp =
                    serde_json::json!({"success": false, "error": "Request too large"});
                writeln!(out, "{}", serde_json::to_string(&resp).unwrap())?;
                out.flush()?;
                warn!("oversized request during auth handshake");
                break false;
            }
            let line = String::from_utf8_lossy(&raw);
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            match serde_json::from_str::<serde_json::Value>(trimmed) {
                Ok(val) if val.get("type").and_then(|t| t.as_str()) == Some("auth") => {
                    if val.get("token").and_then(|t| t.as_str()) == Some(expected_token) {
                        let resp = serde_json::json!({"success": true, "data": {"status": "authenticated"}});
                        writeln!(out, "{}", serde_json::to_string(&resp).unwrap())?;
                        out.flush()?;
                        info!("authenticated");
                        break true;
                    } else {
                        let resp =
                            serde_json::json!({"success": false, "error": "Invalid auth token"});
                        writeln!(out, "{}", serde_json::to_string(&resp).unwrap())?;
                        out.flush()?;
                        warn!("invalid auth token");
                        break false;
                    }
                }
                _ => {
                    let resp = serde_json::json!({"success": false, "error": "Authentication required. Send {\"type\":\"auth\",\"token\":\"...\"}"});
                    writeln!(out, "{}", serde_json::to_string(&resp).unwrap())?;
                    out.flush()?;
                    warn!("non-auth message before handshake");
                    break false;
                }
            }
        };
        if !authenticated {
            error!("Authentication failed, exiting");
            return Ok(());
        }
    }

    // Rate limiter state (INP-4)
    let mut window_start = Instant::now();
    let mut request_count: u32 = 0;

    while let Some((raw, oversized)) = read_capped_line(&mut reader, MAX_LINE_BYTES)? {
        let line = String::from_utf8_lossy(&raw);
        let trimmed = line.trim();
        if !oversized && trimmed.is_empty() {
            continue;
        }

        // Rate limiting (INP-4): reset counter every second, reject excess
        let elapsed = window_start.elapsed();
        if elapsed.as_secs() >= 1 {
            window_start = Instant::now();
            request_count = 0;
        }
        request_count += 1;
        if request_count > MAX_REQUESTS_PER_SEC {
            warn!("rate limit exceeded");
            let err_resp = serde_json::json!({
                "success": false,
                "error": format!("Rate limit exceeded ({MAX_REQUESTS_PER_SEC} req/s)")
            });
            writeln!(out, "{}", serde_json::to_string(&err_resp).unwrap())?;
            out.flush()?;
            continue;
        }

        // Reject oversized requests (INP-1). The reader caps buffering at
        // MAX_LINE_BYTES, so an unbounded line can never exhaust memory before
        // this guard fires.
        if oversized {
            warn!("oversized request");
            let err_resp = serde_json::json!({
                "success": false,
                "error": format!("Request too large (max {MAX_LINE_BYTES} bytes)")
            });
            writeln!(out, "{}", serde_json::to_string(&err_resp).unwrap())?;
            out.flush()?;
            continue;
        }

        let envelope: RequestEnvelope = match serde_json::from_str(trimmed) {
            Ok(e) => e,
            Err(err) => {
                warn!(%err, "parse error");
                let err_resp = serde_json::json!({
                    "success": false,
                    "error": format!("parse error: {err}")
                });
                writeln!(out, "{}", serde_json::to_string(&err_resp).unwrap())?;
                out.flush()?;
                continue;
            }
        };

        // Log the request type and optional ID (LOG-1)
        let req_type = request_type_name(&envelope.request);
        let id_str = envelope.id.as_deref().unwrap_or("-");
        info!(r#type = req_type, id = id_str, "request");

        let response = driver.process_envelope(&envelope);
        let _ = driver.render();

        if !response.success {
            if let Some(ref e) = response.error {
                warn!(error = %e, "response error");
            }
        }

        writeln!(out, "{}", serde_json::to_string(&response).unwrap())?;
        out.flush()?;

        if !driver.is_running() {
            info!("Agent requested quit");
            break;
        }
    }

    info!("louie-server exiting");
    Ok(())
}
