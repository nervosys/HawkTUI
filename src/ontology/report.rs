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
    fn digest_names_every_widget() {
        let registry = builtin_registry();
        let text = digest(&registry);
        for name in registry.list_types() {
            assert!(text.contains(&format!("## {name} ")), "{name} missing");
        }
    }
}
