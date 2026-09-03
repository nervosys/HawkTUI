//! The authoring ontology must describe the API that actually exists.
//!
//! `src/ontology/api_generated.rs` is produced by `scripts/gen_api_ontology.py`
//! from the signatures themselves. That only stays true if something fails when
//! it does not: the registry in §1.4 of the DX plan drifted to 29% populated
//! precisely because nothing checked it.
//!
//! These tests are the check. They cover both directions — the catalog must not
//! fall behind the source, and it must not describe things the source no longer
//! has.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use hawktui::ontology::api;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Public functions declared in one source file.
fn pub_fns(path: &Path) -> BTreeSet<String> {
    let source = fs::read_to_string(path).expect("source file is readable");
    let mut names = BTreeSet::new();
    for line in source.lines() {
        let line = line.trim_start();
        if let Some(rest) = line.strip_prefix("pub fn ") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                names.insert(name);
            }
        }
    }
    names
}

/// Trait methods and derived impls an author never calls directly.
const NOT_AUTHORING_API: &[&str] = &[
    "fmt",
    "clone",
    "default",
    "eq",
    "ne",
    "hash",
    "cmp",
    "partial_cmp",
    "draw",
    "schema",
    "capabilities",
    "actions",
    "semantic_role",
    "agent_state",
    "execute_action",
    "agent_id",
    "accessibility_label",
];

#[test]
fn the_generated_catalog_is_not_stale() {
    let output = Command::new("python")
        .arg("scripts/gen_api_ontology.py")
        .arg("--check")
        .current_dir(repo())
        .output();

    let Ok(output) = output else {
        // Python is not a build requirement; skip rather than fail the suite on
        // a machine that lacks it. CI has it, and CI is where this must hold.
        eprintln!("python unavailable; skipping the staleness check");
        return;
    };

    assert!(
        output.status.success(),
        "the authoring ontology is stale — run `python scripts/gen_api_ontology.py`\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
fn every_widget_function_is_described() {
    let widgets = repo().join("src/widget");
    let mut missing: Vec<String> = Vec::new();

    for entry in fs::read_dir(&widgets).expect("src/widget is readable") {
        let path = entry.expect("directory entry").path();
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        // `mod` declares nothing; the highlighter and the Sixel encoder are
        // support code rather than widgets an author composes.
        if path.extension().and_then(|e| e.to_str()) != Some("rs")
            || matches!(stem, "mod" | "highlight" | "sixel")
        {
            continue;
        }

        let described: BTreeSet<String> = api::API
            .iter()
            .filter(|t| t.module.ends_with(stem))
            .flat_map(|t| t.functions.iter().map(|f| f.name.to_string()))
            .collect();

        for name in pub_fns(&path) {
            if NOT_AUTHORING_API.contains(&name.as_str()) || described.contains(&name) {
                continue;
            }
            missing.push(format!("{stem}::{name}"));
        }
    }

    assert!(
        missing.is_empty(),
        "{} public widget function(s) the authoring ontology does not describe: {:?}\n\
         Run `python scripts/gen_api_ontology.py`.",
        missing.len(),
        missing,
    );
}

#[test]
fn the_core_types_a_program_is_built_from_are_present() {
    // The old ontology described none of these, which is why it could not help
    // an agent write a program: every task above the simplest needs them.
    for name in ["Layout", "Constraint", "Rect", "Buffer", "Style", "Text"] {
        assert!(
            api::get(name).is_some(),
            "{name} is missing from the authoring ontology"
        );
    }
}

#[test]
fn stateful_widgets_are_paired_with_their_state() {
    let pairs = api::stateful_widgets();
    assert!(
        pairs.len() >= 6,
        "expected at least six stateful widgets, found {pairs:?}"
    );
    for (widget, state) in &pairs {
        assert!(
            state.ends_with("State"),
            "{widget} is paired with {state}, which is not a state type"
        );
        assert!(
            api::get(state).is_some(),
            "{widget} names {state}, which the catalog does not describe"
        );
    }
}

#[test]
fn signatures_are_complete_enough_to_call() {
    // A signature without a parameter list cannot tell an author what to pass,
    // which was the whole complaint about the property-based schema.
    for ty in api::API {
        for f in ty.functions {
            assert!(
                f.signature.contains('(') && f.signature.contains(')'),
                "{}::{} has an unusable signature: {:?}",
                ty.name,
                f.name,
                f.signature
            );
            assert!(
                f.signature.starts_with("pub fn "),
                "{}::{} signature is malformed: {:?}",
                ty.name,
                f.name,
                f.signature
            );
        }
    }
}

#[test]
fn coverage_is_far_better_than_the_property_schema() {
    // The widget schema described 40 properties against 334 public functions:
    // 12%. This exists to be better than that, and to say so in numbers.
    let described = api::function_count();
    assert!(
        described >= 300,
        "the authoring ontology describes only {described} functions"
    );
}
