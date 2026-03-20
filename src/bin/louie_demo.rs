//! louie-demo — Self-running demo of the Louie Agent Protocol.
//!
//! Spawns `louie-server` as a child process and walks through the full agent
//! workflow: ping → discover → inspect → control → verify → quit.
//!
//! Produces color-coded terminal output with dramatic timing, suitable for
//! recording with asciinema, Windows Terminal recording, or Screen Studio.
//!
//! Usage:
//!     cargo build --release --bin louie-server --bin louie-demo
//!     cargo run --bin louie-demo

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

// ANSI color constants
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[32m";
const CYAN: &str = "\x1b[36m";
const YELLOW: &str = "\x1b[33m";
const MAGENTA: &str = "\x1b[35m";
const RESET: &str = "\x1b[0m";

fn sleep_ms(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}

fn slow_print(text: &str, delay_ms: u64) {
    for ch in text.chars() {
        print!("{ch}");
        std::io::stdout().flush().unwrap();
        sleep_ms(delay_ms);
    }
    println!();
}

fn section(title: &str) {
    println!();
    println!("{BOLD}{CYAN}{}{RESET}", "─".repeat(64));
    slow_print(&format!("{BOLD}{CYAN}  {title}{RESET}"), 20);
    println!("{BOLD}{CYAN}{}{RESET}", "─".repeat(64));
    println!();
    sleep_ms(500);
}

fn show_request(json: &str) {
    println!("  {DIM}agent  →{RESET}  {YELLOW}{json}{RESET}");
    sleep_ms(300);
}

fn show_response(json: &str) {
    // Try to pretty-print, fall back to compact
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(json) {
        if let Ok(pretty) = serde_json::to_string_pretty(&val) {
            for line in pretty.lines() {
                println!("  {DIM}server →{RESET}  {GREEN}{line}{RESET}");
            }
            sleep_ms(200);
            return;
        }
    }
    println!("  {DIM}server →{RESET}  {GREEN}{json}{RESET}");
    sleep_ms(200);
}

fn show_response_compact(json: &str) {
    // Truncate if very long
    let display = if json.len() > 120 {
        format!("{}...", &json[..120])
    } else {
        json.to_string()
    };
    println!("  {DIM}server →{RESET}  {GREEN}{display}{RESET}");
    sleep_ms(200);
}

fn main() {
    // Resolve the server binary relative to our own executable (BIN-1).
    let server_path = {
        let self_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()));

        let sibling = self_dir.as_ref().map(|d| {
            let name = if cfg!(target_os = "windows") {
                "louie-server.exe"
            } else {
                "louie-server"
            };
            d.join(name)
        });

        if let Some(ref p) = sibling {
            if p.exists() {
                p.to_string_lossy().to_string()
            } else {
                eprintln!(
                    "{BOLD}Error:{RESET} louie-server not found at {}",
                    p.display()
                );
                eprintln!("Run: cargo build --bin louie-server");
                std::process::exit(1);
            }
        } else {
            eprintln!("{BOLD}Error:{RESET} Could not determine executable directory.");
            std::process::exit(1);
        }
    };

    // Title
    println!();
    slow_print(
        &format!("{BOLD}{MAGENTA}  Louie Agent Protocol — Live Demo{RESET}"),
        40,
    );
    println!();
    println!("  {DIM}Showing how an AI agent discovers and controls a TUI application{RESET}");
    println!("  {DIM}through structured JSON messages — no screen-scraping needed.{RESET}");
    sleep_ms(2000);

    // Step 1 — Spawn the server
    section("1. Spawn the server");
    slow_print(&format!("  {DIM}$ louie-server{RESET}"), 50);
    sleep_ms(500);

    let mut child = Command::new(&server_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to spawn louie-server");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    let stdout = child.stdout.take().expect("Failed to open stdout");
    let mut reader = BufReader::new(stdout);

    let send = |stdin: &mut std::process::ChildStdin,
                reader: &mut BufReader<std::process::ChildStdout>,
                request: &str|
     -> String {
        stdin
            .write_all(request.as_bytes())
            .expect("Failed to write");
        stdin.write_all(b"\n").expect("Failed to write newline");
        stdin.flush().expect("Failed to flush");

        let mut line = String::new();
        reader.read_line(&mut line).expect("Failed to read line");
        line.trim().to_string()
    };

    println!("  {GREEN}✓ Server running (PID {}){RESET}", child.id());
    sleep_ms(1000);

    // Step 2 — Ping
    section("2. Test connectivity");
    let req = r#"{"type":"ping"}"#;
    show_request(req);
    let resp = send(&mut stdin, &mut reader, req);
    show_response_compact(&resp);
    println!();
    println!("  {GREEN}✓ Connection alive{RESET}");
    sleep_ms(1000);

    // Step 3 — Discover ontology
    section("3. Discover the UI — query_ontology");
    slow_print(
        &format!("{DIM}  The agent asks: \"What widgets exist in this app?\"{RESET}"),
        20,
    );
    println!();

    let req = r#"{"id":"discover-1","type":"query_ontology"}"#;
    show_request(req);
    let resp = send(&mut stdin, &mut reader, req);

    // Parse and display widget list nicely
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&resp) {
        if let Some(data) = val.get("data").and_then(|d| d.as_array()) {
            println!();
            for schema in data {
                let name = schema.get("name").and_then(|n| n.as_str()).unwrap_or("?");
                let role = schema
                    .get("default_role")
                    .and_then(|r| r.as_str())
                    .unwrap_or("?");
                let tags: Vec<&str> = schema
                    .get("tags")
                    .and_then(|t| t.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<&str>>())
                    .unwrap_or_default();
                println!(
                    "  {BOLD}{name:15}{RESET} {DIM}role={role:12} tags=[{}]{RESET}",
                    tags.join(", ")
                );
                sleep_ms(150);
            }
            println!();
            println!(
                "  {GREEN}✓ Found {} widget types with full schemas{RESET}",
                data.len()
            );
        }
    }
    sleep_ms(1500);

    // Step 4 — Get a specific schema
    section("4. Inspect a widget — get_schema");
    slow_print(
        &format!("{DIM}  \"Tell me everything about the Gauge widget.\"{RESET}"),
        20,
    );
    println!();

    let req = r#"{"id":"schema-1","type":"get_schema","widget_type":"Gauge"}"#;
    show_request(req);
    let resp = send(&mut stdin, &mut reader, req);
    show_response(&resp);
    sleep_ms(1500);

    // Step 5 — Get UI tree
    section("5. Read the UI tree — get_tree");
    slow_print(
        &format!("{DIM}  \"What's on screen right now?\"{RESET}"),
        20,
    );
    println!();

    let req = r#"{"id":"tree-1","type":"get_tree"}"#;
    show_request(req);
    let resp = send(&mut stdin, &mut reader, req);
    show_response_compact(&resp);
    sleep_ms(1500);

    // Step 6 — Inject events (drive the counter)
    section("6. Control the app — inject_event");
    slow_print(
        &format!("{DIM}  The agent presses Up 3 times to increment the counter.{RESET}"),
        20,
    );
    println!();

    for i in 0..3 {
        let req = format!(
            r#"{{"id":"key-{i}","type":"inject_event","event":{{"kind":"key","code":"Up"}}}}"#
        );
        show_request(&req);
        let resp = send(&mut stdin, &mut reader, &req);
        show_response_compact(&resp);
        sleep_ms(400);
    }
    sleep_ms(500);

    // Step 7 — Observe result
    section("7. Observe the result — get_tree");
    slow_print(&format!("{DIM}  \"Did the counter change?\"{RESET}"), 20);
    println!();

    let req = r#"{"id":"verify-1","type":"get_tree"}"#;
    show_request(req);
    let resp = send(&mut stdin, &mut reader, req);
    show_response_compact(&resp);
    sleep_ms(1500);

    // Step 8 — Clean shutdown
    section("8. Clean shutdown — quit");
    let req = r#"{"type":"quit"}"#;
    show_request(req);
    let resp = send(&mut stdin, &mut reader, req);
    show_response_compact(&resp);
    drop(stdin);
    let _ = child.wait();
    println!();
    println!("  {GREEN}✓ Server exited cleanly{RESET}");
    sleep_ms(1000);

    // Summary
    println!();
    println!("{BOLD}{CYAN}{}{RESET}", "─".repeat(64));
    println!();
    slow_print(&format!("{BOLD}  What just happened:{RESET}"), 30);
    println!();
    sleep_ms(300);
    println!("  1. Agent spawned a Louie TUI app as a headless process");
    sleep_ms(200);
    println!("  2. Agent discovered all widgets via {YELLOW}query_ontology{RESET}");
    sleep_ms(200);
    println!("  3. Agent inspected widget schemas — types, constraints, actions");
    sleep_ms(200);
    println!("  4. Agent read the live UI tree — positions, state, capabilities");
    sleep_ms(200);
    println!("  5. Agent controlled the app via {YELLOW}inject_event{RESET}");
    sleep_ms(200);
    println!("  6. Agent verified the result via {YELLOW}get_tree{RESET}");
    sleep_ms(200);
    println!("  7. Agent shut down cleanly via {YELLOW}quit{RESET}");
    println!();
    sleep_ms(500);
    slow_print(
        &format!("{BOLD}  No screen-scraping. No hardcoded selectors. No brittleness.{RESET}"),
        30,
    );
    slow_print(
        &format!("{BOLD}  Just a self-describing UI ontology.{RESET}"),
        30,
    );
    println!();
    println!("  {DIM}github.com/nervosys/louie{RESET}");
    println!();
}
