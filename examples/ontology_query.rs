//! Query the Hawk TUI widget ontology from a checkout.
//!
//! Identical to the shipped `hawktui-ontology` binary, but runnable without
//! installing anything:
//!
//! ```sh
//! cargo run --example ontology_query -- list
//! cargo run --example ontology_query -- search scroll
//! cargo run --example ontology_query -- schema Gauge
//! cargo run --example ontology_query -- roles
//! cargo run --example ontology_query -- digest
//! cargo run --example ontology_query -- export
//! ```
//!
//! Both are thin wrappers over [`hawktui::ontology::report`], so there is one
//! implementation to keep correct.

use hawktui::ontology::{builtin_registry, report};

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
            Some(query) => report::search(&registry, query),
            None => {
                eprintln!("search needs a query");
                std::process::exit(2);
            }
        },
        "schema" => match args.get(1) {
            Some(name) => report::schema(&registry, name).unwrap_or_else(|| {
                eprintln!("unknown widget {name:?}; try `list`");
                std::process::exit(1);
            }),
            None => {
                eprintln!("schema needs a widget name");
                std::process::exit(2);
            }
        },
        other => {
            eprintln!(
                "unknown command {other:?}\n\
                 usage: ontology_query <list|search QUERY|schema NAME|roles|digest|export>"
            );
            std::process::exit(2);
        }
    };
    print!("{output}");
}
