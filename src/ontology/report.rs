//! Human- and agent-readable renderings of the widget catalog.
//!
//! These return `String` rather than printing, so they can be tested, embedded
//! in a document, or served over a protocol. `hawktui-ontology` and
//! `examples/ontology_query.rs` are both thin wrappers around them.
//!
//! Every rendering is derived from the registered schemas. None of it is
//! hand-written prose about the API.

use super::registry::OntologyRegistry;
use super::{PropertyType, WidgetSchema};
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// Schemas in a stable, name-sorted order.
fn sorted(registry: &OntologyRegistry) -> Vec<&WidgetSchema> {
    let mut all: Vec<_> = registry
        .list_types()
        .into_iter()
        .filter_map(|n| registry.get_schema(n))
        .collect();
    all.sort_by(|a, b| a.name.cmp(&b.name));
    all
}

/// A short, readable name for a property type.
pub fn type_name(ty: &PropertyType) -> String {
    match ty {
        PropertyType::String => "string".into(),
        PropertyType::Integer => "int".into(),
        PropertyType::Float => "float".into(),
        PropertyType::Boolean => "bool".into(),
        PropertyType::Color => "color".into(),
        PropertyType::Style => "style".into(),
        PropertyType::Rect => "rect".into(),
        PropertyType::Enum(variants) => format!("enum({})", variants.join("|")),
        PropertyType::Array(inner) => format!("[{}]", type_name(inner)),
        PropertyType::Object(_) => "object".into(),
        PropertyType::Widget => "widget".into(),
        PropertyType::Any => "any".into(),
    }
}

/// One line per widget: name, semantic role, description.
pub fn list(registry: &OntologyRegistry) -> String {
    let mut out = String::new();
    for schema in sorted(registry) {
        let _ = writeln!(
            out,
            "{:<18} {:<18} {}",
            schema.name,
            schema.default_role.to_string(),
            schema.description
        );
    }
    out
}

/// Widgets matching a name, description or tag, in the same shape as [`list`].
pub fn search(registry: &OntologyRegistry, query: &str) -> String {
    let mut hits = registry.search(query);
    hits.sort_by(|a, b| a.name.cmp(&b.name));
    let mut out = String::new();
    for schema in hits {
        let _ = writeln!(
            out,
            "{:<18} {:<18} {}",
            schema.name,
            schema.default_role.to_string(),
            schema.description
        );
    }
    out
}

/// A full schema: properties with types and constraints, actions, usage hint.
///
/// Returns `None` when the registry has no such widget.
pub fn schema(registry: &OntologyRegistry, name: &str) -> Option<String> {
    let schema = registry.get_schema(name)?;
    let mut out = String::new();

    let _ = writeln!(out, "{} — {}", schema.name, schema.description);
    let _ = writeln!(out, "role: {}", schema.default_role);
    if !schema.tags.is_empty() {
        let _ = writeln!(out, "tags: {}", schema.tags.join(", "));
    }

    let _ = writeln!(out, "\nproperties:");
    for p in &schema.properties {
        let req = if p.required { " (required)" } else { "" };
        let _ = writeln!(
            out,
            "  {:<16} {:<14}{} {}",
            p.name,
            type_name(&p.property_type),
            req,
            p.description
        );
        for c in &p.constraints {
            let _ = writeln!(out, "      constraint: {c:?}");
        }
        if let Some(default) = &p.default_value {
            let _ = writeln!(out, "      default: {default}");
        }
    }

    if !schema.actions.is_empty() {
        let _ = writeln!(out, "\nactions:");
        for a in &schema.actions {
            let params: Vec<String> = a
                .params
                .iter()
                .map(|p| format!("{}: {:?}", p.name, p.param_type))
                .collect();
            let _ = writeln!(
                out,
                "  {}({}) — {}",
                a.name,
                params.join(", "),
                a.description
            );
        }
    }

    if let Some(hint) = &schema.usage_hint {
        let _ = writeln!(out, "\nusage:\n  {hint}");
    }
    Some(out)
}

/// Widgets grouped by semantic role, for finding one by what it does.
pub fn roles(registry: &OntologyRegistry) -> String {
    let mut by_role: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    for schema in sorted(registry) {
        by_role
            .entry(schema.default_role.to_string())
            .or_default()
            .push(schema.name.as_str());
    }
    let mut out = String::new();
    for (role, names) in by_role {
        let _ = writeln!(out, "{:<20} {}", role, names.join(", "));
    }
    out
}

/// A compact Markdown cheatsheet, dense enough to paste into an agent's context.
pub fn digest(registry: &OntologyRegistry) -> String {
    let schemas = sorted(registry);
    let mut out = String::from("# Hawk TUI widget ontology (generated)\n\n");
    let _ = writeln!(
        out,
        "{} widget types. Format: `property: type` per line.\n",
        schemas.len()
    );
    for schema in schemas {
        let _ = writeln!(
            out,
            "## {} — {} ({})",
            schema.name, schema.description, schema.default_role
        );
        for p in &schema.properties {
            let req = if p.required { "*" } else { "" };
            let _ = writeln!(out, "- {}{}: {}", p.name, req, type_name(&p.property_type));
        }
        if let Some(hint) = &schema.usage_hint {
            let _ = writeln!(out, "\n```rust\n{hint}\n```");
        }
        out.push('\n');
    }
    out
}

/// The whole catalog as pretty-printed JSON.
pub fn export(registry: &OntologyRegistry) -> String {
    serde_json::to_string_pretty(&registry.export_catalog()).unwrap_or_else(|_| "{}".into())
}

/// The authoring view of one type: how to construct it, what to chain onto it,
/// and — for a widget — the exact line that renders it.
///
/// This answers the questions the widget schema cannot: which state type a
/// `StatefulWidget` needs, what a builder method actually takes, and how an
/// enum's variants are spelled.
pub fn api(name: &str) -> Option<String> {
    let ty = super::api::get(name)?;
    let mut out = String::new();

    let kind = match &ty.kind {
        super::api::ApiKind::Widget => "widget".to_string(),
        super::api::ApiKind::StatefulWidget { state } => {
            format!("stateful widget (state: {state})")
        }
        super::api::ApiKind::Enum => "enum".to_string(),
        super::api::ApiKind::Trait => "trait".to_string(),
        super::api::ApiKind::Struct => "struct".to_string(),
    };
    let _ = writeln!(out, "{} — {}", ty.name, kind);
    let _ = writeln!(out, "use {}::{};", ty.module, ty.name);
    if !ty.summary.is_empty() {
        let _ = writeln!(out, "{}", ty.summary);
    }
    if !ty.variants.is_empty() {
        let _ = writeln!(
            out,
            "
variants: {}",
            ty.variants.join(", ")
        );
    }

    for (label, mut fns) in [
        (
            "required methods",
            ty.functions
                .iter()
                .filter(|f| f.role == "required")
                .collect::<Vec<_>>(),
        ),
        (
            "provided methods",
            ty.functions
                .iter()
                .filter(|f| f.role == "provided")
                .collect::<Vec<_>>(),
        ),
        ("constructors", ty.constructors().collect::<Vec<_>>()),
        ("builders", ty.builders().collect::<Vec<_>>()),
        ("methods", ty.methods().collect::<Vec<_>>()),
    ] {
        if fns.is_empty() {
            continue;
        }
        fns.sort_by_key(|f| f.name);
        let _ = writeln!(
            out,
            "
{label}:"
        );
        for f in fns {
            let _ = writeln!(out, "  {}", f.signature);
            if !f.summary.is_empty() {
                let _ = writeln!(out, "      {}", f.summary);
            }
        }
    }

    if let Some(call) = ty.render_call() {
        let _ = writeln!(
            out,
            "
render:
  {call}"
        );
    }
    Some(out)
}

/// Types matching a query, with the shape of each, for finding the right one.
pub fn api_search(query: &str) -> String {
    let mut hits = super::api::search(query);
    hits.sort_by_key(|t| t.name);
    let mut out = String::new();
    for t in hits {
        let shape = match &t.kind {
            super::api::ApiKind::Widget => "widget".to_string(),
            super::api::ApiKind::StatefulWidget { state } => format!("stateful/{state}"),
            super::api::ApiKind::Enum => "enum".to_string(),
            super::api::ApiKind::Trait => "trait".to_string(),
            super::api::ApiKind::Struct => "struct".to_string(),
        };
        let _ = writeln!(out, "{:<18} {:<20} {}", t.name, shape, t.module);
    }
    out
}

/// Every widget that needs a companion state value.
pub fn stateful() -> String {
    let mut out = String::from(
        "widget -> state type (render_stateful_widget)
",
    );
    for (widget, state) in super::api::stateful_widgets() {
        let _ = writeln!(out, "  {widget:<16} {state}");
    }
    out
}

/// The minimal complete program, served verbatim from `examples/skeleton.rs`.
///
/// `include_str!` of a compiled example rather than a string literal, so the
/// skeleton handed to an agent is exactly the one CI builds. Transcripts showed
/// agents reading `examples/counter.rs`, `runtime/mod.rs` and `terminal.rs` to
/// work out how a program is assembled; this answers that in one call.
pub fn skeleton() -> String {
    include_str!("../../examples/skeleton.rs").to_string()
}

/// What `use hawktui::prelude::*` brings into scope.
///
/// `src/lib.rs` is among the files agents read most, and this is the only
/// reason they need to.
pub fn prelude() -> String {
    let mut out = String::from(
        "use hawktui::prelude::*; brings these into scope.
         Program, Model, Command and ProgramOptions are NOT included; import          them from hawktui::runtime.

",
    );
    for item in super::api::PRELUDE {
        let _ = writeln!(out, "  {item}");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ontology::builtin_registry;

    #[test]
    fn list_has_one_line_per_widget() {
        let registry = builtin_registry();
        let lines = list(&registry).lines().count();
        assert_eq!(lines, registry.list_types().len());
    }

    #[test]
    fn schema_of_an_unknown_widget_is_none() {
        assert!(schema(&builtin_registry(), "Nonexistent").is_none());
    }

    #[test]
    fn schema_shows_constraints_and_the_usage_hint() {
        let text = schema(&builtin_registry(), "Gauge").expect("Gauge is built in");
        assert!(text.contains("ratio"), "{text}");
        assert!(text.contains("constraint: Max(1.0)"), "{text}");
        assert!(text.contains("Gauge::new().percent(42)"), "{text}");
    }

    #[test]
    fn search_finds_by_tag() {
        let text = search(&builtin_registry(), "scroll");
        assert!(text.contains("Scrollbar"), "{text}");
    }

    #[test]
    fn export_is_valid_json_covering_every_widget() {
        let registry = builtin_registry();
        let value: serde_json::Value =
            serde_json::from_str(&export(&registry)).expect("export is JSON");
        assert_eq!(
            value.as_object().map(|o| o.len()),
            Some(registry.list_types().len())
        );
    }

    #[test]
    fn api_shows_how_to_construct_and_render() {
        let text = api("List").expect("List is in the authoring catalog");
        assert!(
            text.contains("stateful widget (state: ListState)"),
            "{text}"
        );
        assert!(text.contains("use hawktui::widget::list::List;"), "{text}");
        assert!(text.contains("constructors:"), "{text}");
        assert!(text.contains("pub fn new("), "{text}");
        assert!(text.contains("render_stateful_widget"), "{text}");
    }

    #[test]
    fn api_covers_the_layout_system() {
        let text = api("Constraint").expect("Constraint is in the catalog");
        assert!(text.contains("enum"), "{text}");
        for v in ["Length", "Percentage", "Fill"] {
            assert!(text.contains(v), "missing variant {v} in {text}");
        }
        // Not a widget, so it has no render call to offer.
        assert!(!text.contains("render:"), "{text}");
    }

    #[test]
    fn api_of_a_plain_widget_uses_the_simple_render_call() {
        let text = api("Paragraph").expect("Paragraph is in the catalog");
        assert!(text.contains("render_widget(paragraph, area)"), "{text}");
        assert!(!text.contains("render_stateful_widget"), "{text}");
    }

    #[test]
    fn api_of_an_unknown_type_is_none() {
        assert!(api("NoSuchType").is_none());
    }

    #[test]
    fn api_search_finds_by_method_name() {
        let text = api_search("highlight_symbol");
        assert!(text.contains("List"), "{text}");
    }

    #[test]
    fn api_search_reports_the_shape_of_each_hit() {
        let text = api_search("Constraint");
        assert!(text.contains("enum"), "{text}");
        assert!(text.contains("hawktui::layout"), "{text}");
    }

    #[test]
    fn stateful_lists_every_widget_needing_state() {
        let text = stateful();
        for (widget, state) in [
            ("List", "ListState"),
            ("Table", "TableState"),
            ("SettingsList", "SettingsListState"),
        ] {
            assert!(text.contains(widget), "{widget} missing from {text}");
            assert!(text.contains(state), "{state} missing from {text}");
        }
    }

    #[test]
    fn digest_names_every_widget() {
        let registry = builtin_registry();
        let text = digest(&registry);
        for name in registry.list_types() {
            assert!(text.contains(&format!("## {name} ")), "{name} missing");
        }
    }
}
