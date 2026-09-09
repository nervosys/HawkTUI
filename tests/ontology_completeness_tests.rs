//! The catalog an agent queries must describe what the widgets actually do.
//!
//! `schema()` is static, so the registry never sees an instance and cannot ask
//! it what it can do. The consequence was a catalog that reported no actions for
//! eighteen of twenty-one widgets and no capabilities for any, while the code
//! implemented five actions and four capabilities for `List` alone. Every
//! ontology grid in `benchmarks/agentic` was run against that emptier catalog.
//!
//! `action_schema()` and `capability_kinds()` close it, and these tests keep
//! them honest: the type-level declaration must match what an instance reports,
//! or the catalog is lying in a new way instead of an old one.

use hawktui::ontology::{builtin_registry, Discoverable};
use hawktui::widget::gauge::Gauge;
use hawktui::widget::input::Input;
use hawktui::widget::list::List;
use hawktui::widget::select_list::{SelectItem, SelectList};

/// Declared actions must be the actions an instance offers.
fn assert_actions_match<W: Discoverable>(instance: &W, name: &str) {
    let declared: Vec<String> = W::action_schema().iter().map(|a| a.name.clone()).collect();
    let actual: Vec<String> = instance.actions().iter().map(|a| a.name.clone()).collect();
    assert_eq!(declared, actual, "{name}: action_schema() disagrees with actions()");
}

/// Declared capability kinds must be the kinds an instance exhibits.
fn assert_capabilities_match<W: Discoverable>(instance: &W, name: &str) {
    let declared = W::capability_kinds();
    let mut actual: Vec<String> = instance
        .capabilities()
        .iter()
        .map(|c| c.name().to_string())
        .collect();
    actual.dedup();
    assert_eq!(declared, actual, "{name}: capability_kinds() disagrees with capabilities()");
}

#[test]
fn declarations_match_instances() {
    assert_actions_match(&List::new(["a", "b"]), "List");
    assert_capabilities_match(&List::new(["a", "b"]), "List");
    assert_actions_match(&Input::default(), "Input");
    assert_capabilities_match(&Input::default(), "Input");
    assert_actions_match(&Gauge::new(), "Gauge");
    assert_capabilities_match(&Gauge::new(), "Gauge");
    assert_actions_match(&SelectList::new(vec![SelectItem::new("a", "a")]), "SelectList");
    assert_capabilities_match(&SelectList::new(vec![SelectItem::new("a", "a")]), "SelectList");
}

/// The registry must serve the merged schema, not the bare literal.
#[test]
fn registry_serves_actions_and_capabilities() {
    let registry = builtin_registry();
    let list = registry.get_schema("List").expect("List is registered");
    assert!(
        !list.actions.is_empty(),
        "List reached the registry with no actions; the merge in register() is not running"
    );
    assert!(
        !list.capabilities.is_empty(),
        "List reached the registry with no capabilities"
    );

    // The regression this whole change exists to prevent: a catalog that is
    // mostly empty while the code is not.
    let with_actions = registry
        .list_types()
        .iter()
        .filter(|n| {
            registry
                .get_schema(n)
                .map(|s| !s.actions.is_empty())
                .unwrap_or(false)
        })
        .count();
    assert!(
        with_actions >= 13,
        "only {with_actions} widgets expose actions; 13 implement them"
    );
}
