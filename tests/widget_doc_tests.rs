//! The ontology in the doc comments must describe the widgets that exist.
//!
//! Agents read the framework's source in every run and reach it by turn 3; a
//! separately delivered ontology is consulted in a minority of runs and, across
//! eight grids, changed no outcome. So the semantics belong in the doc comment
//! of the type, where the reader was already going to be.
//!
//! That is a second rendering of one source of truth, and a second rendering is
//! a drift risk. `scripts/gen_widget_docs.py --check` compares every generated
//! block against the registry it came from; this test is what makes a stale
//! block fail the build rather than quietly mislead a reader.

use std::path::PathBuf;
use std::process::Command;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn widget_doc_ontology_is_in_sync() {
    // Two traps, one on each platform. Ubuntu runners ship `python3` and not
    // always `python`, so trying a single name can skip the check on the
    // machine that matters most. Windows ships a `python3` *shim* that spawns
    // happily and prints "Python was not found" instead of running anything, so
    // trusting the first name that spawns picks a stub. Probe with --version and
    // require it to actually answer.
    let interpreter = ["python3", "python"].into_iter().find(|exe| {
        Command::new(exe)
            .arg("--version")
            .output()
            .map(|o| o.status.success() && o.stdout.starts_with(b"Python"))
            .unwrap_or(false)
    });
    let Some(interpreter) = interpreter else {
        panic!("no working python interpreter; the widget doc ontology cannot be checked");
    };

    let out = Command::new(interpreter)
        .arg("scripts/gen_widget_docs.py")
        .arg("--check")
        .current_dir(repo())
        .output()
        .expect("generator runs");

    assert!(
        out.status.success(),
        "widget doc ontology is stale:\n{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
}
