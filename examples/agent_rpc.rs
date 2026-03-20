//! Agent RPC example: headless Louie app driven via JSON Lines on stdin/stdout.
//!
//! Run with: `echo '{"type":"ping"}' | cargo run --example agent_rpc`
//!
//! Send JSON objects (one per line) on stdin. Each is parsed as a
//! `RequestEnvelope` and processed by a `HeadlessDriver`. Responses are
//! written as JSON lines on stdout.
//!
//! Supported requests:
//!   {"type":"ping"}
//!   {"type":"query_ontology"}
//!   {"type":"get_tree"}
//!   {"type":"get_state", "agent_id":"counter"}
//!   {"type":"inject_event","event":{"kind":"key","code":"Up"}}
//!   {"id":"r1","type":"execute_action","agent_id":"counter","action":"increment","params":{}}
//!   {"type":"quit"}

use std::io::{self, BufRead, Write};

use louie::agent::driver::HeadlessDriver;
use louie::agent::protocol::RequestEnvelope;
use louie::ontology::registry::OntologyRegistry;
use louie::prelude::*;
use louie::runtime::{Command, Model};
use louie::widget::gauge::Gauge;

// ---------------------------------------------------------------------------
// Application model
// ---------------------------------------------------------------------------

struct CounterApp {
    count: i64,
}

#[derive(Debug)]
enum Msg {
    Increment,
    Decrement,
    Tick,
}

impl Model for CounterApp {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Increment => self.count += 1,
            Msg::Decrement => self.count -= 1,
            Msg::Tick => {}
        }
        Command::None
    }

    fn view(&self, frame: &mut Frame) {
        let area = frame.area();
        let chunks = louie::layout::Layout::default()
            .direction(louie::layout::Direction::Vertical)
            .constraints([
                louie::layout::Constraint::Length(3),
                louie::layout::Constraint::Length(3),
                louie::layout::Constraint::Fill(1),
            ])
            .split(area);

        let title = Paragraph::new(Text::from(format!("Counter: {}", self.count)));
        frame.render_widget(title, chunks[0]);

        let ratio = (self.count as f64 / 100.0).clamp(0.0, 1.0);
        let gauge = Gauge::new()
            .ratio(ratio)
            .label(format!("{:.0}%", ratio * 100.0));
        frame.render_widget(gauge, chunks[1]);
    }

    fn handle_event(&self, event: Event) -> Option<Msg> {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => Some(Msg::Increment),
                KeyCode::Down | KeyCode::Char('j') => Some(Msg::Decrement),
                _ => None,
            }
        } else if matches!(event, Event::Tick) {
            Some(Msg::Tick)
        } else {
            None
        }
    }

    fn register_ontology(&self, registry: &mut OntologyRegistry) {
        registry.register::<Paragraph>();
        registry.register::<Gauge>();
        registry.register::<Block>();
    }
}

// ---------------------------------------------------------------------------
// Main: read JSON Lines from stdin, respond on stdout
// ---------------------------------------------------------------------------

fn main() -> io::Result<()> {
    let app = CounterApp { count: 0 };
    let mut driver = HeadlessDriver::new(app, 80, 24)?;
    driver.init();
    driver.render()?;

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let envelope: RequestEnvelope = match serde_json::from_str(trimmed) {
            Ok(e) => e,
            Err(err) => {
                let err_resp = serde_json::json!({
                    "success": false,
                    "error": format!("parse error: {err}")
                });
                writeln!(out, "{}", serde_json::to_string(&err_resp).unwrap())?;
                out.flush()?;
                continue;
            }
        };

        let response = driver.process_envelope(&envelope);

        // Re-render after each request so subsequent queries see updated state
        let _ = driver.render();

        writeln!(out, "{}", serde_json::to_string(&response).unwrap())?;
        out.flush()?;

        if !driver.is_running() {
            break;
        }
    }

    Ok(())
}
