//! The built-in registry must stay complete.
//!
//! Rust cannot enumerate the implementors of a trait at runtime, so this scans
//! the widget sources for `impl Discoverable for X` and checks that every `X`
//! is registered. Without it the catalog drifts silently: before
//! `register_builtin_widgets` existed, `hawktui-server` registered six of
//! twenty-one discoverable widgets and nothing failed.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use hawktui::ontology::builtin_registry;

/// Every type with an `impl Discoverable for` block under `src/widget/`.
fn discoverable_widgets_in_source() -> BTreeSet<String> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/widget");
    let mut found = BTreeSet::new();

    for entry in fs::read_dir(&dir).expect("src/widget is readable") {
        let path = entry.expect("directory entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let source = fs::read_to_string(&path).expect("widget source is readable");
        for line in source.lines() {
            let line = line.trim();
            let Some(rest) = line.strip_prefix("impl Discoverable for ") else {
                continue;
            };
            let name = rest.trim_end_matches(" {").trim();
            // Skip anything generic; the registry takes concrete types only and
            // a generic impl would need a decision about which instantiation to
            // register, which is not this test's job to guess.
            if !name.is_empty() && !name.contains('<') {
                found.insert(name.to_string());
            }
        }
    }

    assert!(
        !found.is_empty(),
        "found no `impl Discoverable for` blocks under src/widget — the scan \
         is broken, not the registry"
    );
    found
}

#[test]
fn every_discoverable_widget_is_registered() {
    let expected = discoverable_widgets_in_source();
    let registry = builtin_registry();
    let registered: BTreeSet<String> = registry
        .list_types()
        .into_iter()
        .map(String::from)
        .collect();

    let missing: Vec<_> = expected.difference(&registered).cloned().collect();
    assert!(
        missing.is_empty(),
        "these widgets implement Discoverable but are not registered in \
         src/ontology/builtin.rs: {missing:?}\n\
         An agent querying the ontology would never learn they exist."
    );
}

#[test]
fn registry_holds_no_widget_that_is_not_discoverable() {
    let expected = discoverable_widgets_in_source();
    let registry = builtin_registry();
    let registered: BTreeSet<String> = registry
        .list_types()
        .into_iter()
        .map(String::from)
        .collect();

    let extra: Vec<_> = registered.difference(&expected).cloned().collect();
    assert!(
        extra.is_empty(),
        "registered but not found in src/widget sources: {extra:?}"
    );
}

#[test]
fn every_registered_schema_is_usefully_populated() {
    let registry = builtin_registry();
    for name in registry.list_types() {
        let schema = registry.get_schema(name).expect("listed type has a schema");
        assert!(
            !schema.description.trim().is_empty(),
            "{name} has an empty description; an agent choosing a widget by \
             description would skip it"
        );
        assert!(
            !schema.tags.is_empty(),
            "{name} has no tags, so `search` can never surface it"
        );
    }
}
