//! hawktui-ontology — query the Hawk TUI widget ontology from the command line.
//!
//! The same catalog `hawktui-server` serves over the `query_ontology` RPC, but
//! available while *writing* a Hawk TUI program rather than while driving one.
//!
//! ```sh
//! hawktui-ontology list              # every widget with its role
//! hawktui-ontology search scroll     # widgets matching a name, tag or description
//! hawktui-ontology schema Gauge      # one widget in full
//! hawktui-ontology roles             # grouped by what they do
//! hawktui-ontology digest            # a compact cheatsheet
//! hawktui-ontology export            # the whole catalog as JSON
//! ```
//!
//! Scope, stated plainly: the ontology describes a widget's runtime state, not
//! its builder API. Use it to choose a widget and learn what it holds; read the
//! rustdoc for the methods that construct it.

use std::io::{self, Write};

use hawktui::ontology::{builtin_registry, report};

const USAGE: &str = "\
usage: hawktui-ontology <command>

  list            every widget type with its role and description
  search QUERY    widget types matching a name, description or tag
  schema NAME     full schema for one widget type
  roles           widget types grouped by semantic role
  digest          a compact generated cheatsheet
  export          the whole catalog as JSON
";

fn main() {
    let registry = builtin_registry();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = args.first().map(String::as_str).unwrap_or("list");

    let output = match command {
        "list" => report::list(&registry),
        "roles" => report::roles(&registry),
        "digest" => report::digest(&registry),
        "export" => report::export(&registry),
        "search" => match args.get(1) {
            Some(query) => {
                let hits = report::search(&registry, query);
                if hits.is_empty() {
                    eprintln!("no widget matches {query:?}");
                    std::process::exit(1);
                }
                hits
            }
            None => {
                eprintln!("search needs a query\n\n{USAGE}");
                std::process::exit(2);
            }
        },
        "schema" => match args.get(1) {
            Some(name) => match report::schema(&registry, name) {
                Some(text) => text,
                None => {
                    eprintln!("unknown widget {name:?}; try `hawktui-ontology list`");
                    std::process::exit(1);
                }
            },
            None => {
                eprintln!("schema needs a widget name\n\n{USAGE}");
                std::process::exit(2);
            }
        },
        "--help" | "-h" | "help" => {
            print!("{USAGE}");
            return;
        }
        other => {
            eprintln!("unknown command {other:?}\n\n{USAGE}");
            std::process::exit(2);
        }
    };

    // A closed pipe (`| head`) is not an error worth a panic.
    if io::stdout().write_all(output.as_bytes()).is_err() {
        std::process::exit(0);
    }
}
