# Widget ontology (queryable)

This project ships a machine-readable ontology describing every Hawk TUI
widget: its properties with types and constraints, its semantic role, the
actions it supports, and a usage hint. Query it instead of guessing at the API.

```sh
cargo run --quiet --manifest-path C:/Users/adamm/dev/nervosys/utilities/HawkTUI/Cargo.toml --example ontology_query -- list
cargo run --quiet --manifest-path C:/Users/adamm/dev/nervosys/utilities/HawkTUI/Cargo.toml --example ontology_query -- search scroll
cargo run --quiet --manifest-path C:/Users/adamm/dev/nervosys/utilities/HawkTUI/Cargo.toml --example ontology_query -- schema Gauge
cargo run --quiet --manifest-path C:/Users/adamm/dev/nervosys/utilities/HawkTUI/Cargo.toml --example ontology_query -- roles
```

- `list` — every widget type with its semantic role and description
- `search QUERY` — widget types matching a name, description or tag
- `schema NAME` — full schema for one widget type, including constraints
- `roles` — widget types grouped by semantic role, for finding the right widget
  when you know what it should *do* but not what it is called
