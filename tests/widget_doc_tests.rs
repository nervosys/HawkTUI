//! The ontology in the comments must describe the code that exists.
//!
//! Agents read the framework's source, so that is where the semantics belong:
//! each widget's doc comment carries its role, properties and actions, and the
//! crate-level docs carry the import map. Both are generated from the registry
//! rather than written by hand.
//!
//! Generated prose is a drift risk, and this repository has watched a registry
//! drift to 29% populated because nothing checked it. These run each generator
//! with `--check`, so a stale block fails the build instead of quietly
//! misleading a reader.

use std::path::PathBuf;
use std::process::Command;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The interpreter to run the generators with.
///
/// Two traps, one on each platform. Ubuntu runners ship `python3` and not
/// always `python`, so probing a single name can skip the check on the machine
/// that matters most. Windows ships a `python3` *shim* that spawns happily and
/// prints "Python was not found" instead of running anything, so trusting the
/// first name that spawns picks a stub. Probe with `--version` and require an
/// answer.
fn interpreter() -> &'static str {
    ["python3", "python"]
        .into_iter()
        .find(|exe| {
            Command::new(exe)
                .arg("--version")
                .output()
                .map(|o| o.status.success() && o.stdout.starts_with(b"Python"))
                .unwrap_or(false)
        })
        .expect("a working python interpreter; the generated ontology cannot be checked")
}

fn check(script: &str) {
    let out = Command::new(interpreter())
        .arg(script)
        .arg("--check")
        .current_dir(repo())
        .output()
        .expect("generator runs");
    assert!(
        out.status.success(),
        "{script} reports the generated ontology is stale:\n{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
}

#[test]
fn widget_doc_ontology_is_in_sync() {
    check("scripts/gen_widget_docs.py");
}

#[test]
fn import_map_is_in_sync() {
    check("scripts/gen_api_map.py");
}
