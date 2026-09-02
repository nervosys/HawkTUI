//! hawktui-mcp — expose the Hawk TUI widget ontology over the Model Context
//! Protocol.
//!
//! JSON-RPC 2.0 over stdio, one message per line: the transport every MCP
//! client already speaks. Register it with an agent that supports MCP servers
//! and the ontology becomes a set of callable tools rather than a protocol the
//! integrator has to implement.
//!
//! ```sh
//! # Tools it offers
//! echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | hawktui-mcp
//!
//! # One widget in full
//! echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"get_widget_schema","arguments":{"name":"Gauge"}}}' | hawktui-mcp
//! ```
//!
//! Example client configuration:
//!
//! ```json
//! { "mcpServers": { "hawktui": { "command": "hawktui-mcp" } } }
//! ```

use std::io::{self, BufRead, Read, Write};

use hawktui::agent::mcp::McpServer;

/// Refuse absurd input rather than allocating for it. Matches the cap
/// `hawktui-server` applies to its own line protocol.
const MAX_LINE_BYTES: usize = 1_048_576;

fn main() -> io::Result<()> {
    let mut server = McpServer::new();
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut stdout = io::stdout().lock();
    let mut line = String::new();

    loop {
        line.clear();
        // Cap the read so a client cannot make the server allocate without
        // bound, matching the limit `hawktui-server` applies to its protocol.
        let read = (&mut reader)
            .take(MAX_LINE_BYTES as u64 + 1)
            .read_line(&mut line)?;
        if read == 0 {
            return Ok(()); // client closed the pipe
        }
        if read > MAX_LINE_BYTES {
            eprintln!("hawktui-mcp: message exceeds {MAX_LINE_BYTES} bytes; ignoring");
            continue;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(response) = server.handle(trimmed) {
            // A closed stdout means the client is gone; that is a normal exit,
            // not a crash worth a panic.
            if writeln!(stdout, "{response}").is_err() || stdout.flush().is_err() {
                return Ok(());
            }
        }
    }
}
