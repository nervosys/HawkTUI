//! Every `usage_hint` in the ontology must compile.
//!
//! A hint is the one piece of the schema an agent is most likely to copy
//! verbatim. A hint that does not compile is worse than no hint at all: it is a
//! hallucination carrying the framework's own authority. Nothing checked them
//! before this test, and two were wrong.
//!
//! The hints are written out below as real code, so the compiler checks them.
//! `hints_in_the_schema_match_this_file` then reads this file back and asserts
//! that every code-shaped hint in the registry appears here verbatim, so a
//! schema edit cannot drift away from the code that proves it.

use hawktui::ontology::builtin_registry;
use hawktui::prelude::*;
use hawktui::widget::calendar::Calendar;
use hawktui::widget::cancellable_loader::CancellableLoader;
use hawktui::widget::canvas::{Canvas, CanvasLine};
use hawktui::widget::editor::Editor;
use hawktui::widget::input::Input;
use hawktui::widget::line_gauge::LineGauge;
use hawktui::widget::list::List;
use hawktui::widget::loader::{Loader, SpinnerStyle};
use hawktui::widget::markdown::Markdown;
use hawktui::widget::scrollbar::{Scrollbar, ScrollbarOrientation};
use hawktui::widget::select_list::{SelectItem, SelectList, SelectMode};
use hawktui::widget::sparkline::Sparkline;
use hawktui::widget::table::{Table, TableColumn, TableColumnWidth, TableRow};

/// Bindings the hints refer to. A hint is a fragment, not a whole program, so
/// it may name a value the caller is expected to have.
fn bindings() -> (usize, Vec<SelectItem>) {
    (
        0,
        vec![SelectItem::new("One", "one"), SelectItem::new("Two", "two")],
    )
}

#[test]
fn every_code_hint_compiles() {
    let (n, items) = bindings();

    let _ = Block::bordered().title("My Panel");
    let _ = Calendar::new(2026, 3).show_header(true);
    let _ = CancellableLoader::new("Processing...").tick(n);
    let _ = Canvas::new().x_bounds([0.0, 100.0]).line(CanvasLine {
        x1: 0.0,
        y1: 0.0,
        x2: 100.0,
        y2: 50.0,
        color: Color::White,
    });
    let _ = Editor::new().show_line_numbers(true);
    let _ = Gauge::new().percent(42).label("Loading...");
    let _ = Input::new().placeholder("Type here...");
    let _ = LineGauge::new().percent(65).label("Progress");
    let _ = List::new(["Item 1", "Item 2"]).highlight_symbol(">> ");
    let _ = Loader::new("Loading...")
        .spinner_style(SpinnerStyle::Braille)
        .tick(n);
    let _ = Markdown::new("# Hello\n\nSome **bold** text");
    let _ = Paragraph::new("Hello, world!").centered();
    let _ = Scrollbar::new(ScrollbarOrientation::Vertical);
    let _ = SelectList::new(items).mode(SelectMode::Multi);
    let _ = Sparkline::new(vec![0, 1, 3, 7, 5, 2]);
    let _ = Table::new(
        [TableColumn::new("Name", TableColumnWidth::Fill)],
        [TableRow::new(["Alice"])],
    );
    let _ = Tabs::new(["Tab 1", "Tab 2"]).select(0);
}

/// A hint that mentions a path is meant to be copied; one that reads as a
/// sentence is guidance. Only the former has to appear in this file.
fn looks_like_code(hint: &str) -> bool {
    hint.contains("::") && !hint.trim_end().ends_with('.')
}

/// Reduce Rust source to a form where formatting cannot cause a false mismatch:
/// drop all whitespace, then the trailing commas rustfmt inserts before a
/// closing delimiter but a one-line hint would never carry.
fn normalize(code: &str) -> String {
    let mut out: String = code.chars().filter(|c| !c.is_whitespace()).collect();
    loop {
        let shorter = out.replace(",}", "}").replace(",)", ")").replace(",]", "]");
        if shorter == out {
            return out;
        }
        out = shorter;
    }
}

#[test]
fn hints_in_the_schema_match_this_file() {
    let source = include_str!("usage_hints_compile.rs");
    let registry = builtin_registry();

    let mut missing = Vec::new();
    for name in registry.list_types() {
        let schema = registry.get_schema(name).expect("listed type has a schema");
        let Some(hint) = schema.usage_hint.as_deref() else {
            continue;
        };
        if !looks_like_code(hint) {
            continue;
        }
        // The hint is written across several lines here when rustfmt wraps it,
        // so compare the call chains in normalized form.
        let needle = normalize(hint);
        let haystack = normalize(source);
        if !haystack.contains(&needle) {
            missing.push(format!("{name}: {hint}"));
        }
    }

    assert!(
        missing.is_empty(),
        "these usage hints are not proven to compile — add them to \
         every_code_hint_compiles, or fix the schema:\n  {}",
        missing.join("\n  ")
    );
}

#[test]
fn hints_that_are_prose_are_deliberate() {
    // Four widgets describe when to use them rather than how to build them.
    // That is a legitimate choice, but it should be a known set rather than an
    // accident, so that a hint is never silently dropped.
    let registry = builtin_registry();
    let prose: Vec<&str> = registry
        .list_types()
        .into_iter()
        .filter(|name| {
            registry
                .get_schema(name)
                .and_then(|s| s.usage_hint.as_deref())
                .is_some_and(|h| !looks_like_code(h))
        })
        .collect();

    let mut prose = prose;
    prose.sort_unstable();
    assert_eq!(
        prose,
        ["BarChart", "Chart", "Image", "SettingsList"],
        "the set of widgets whose hint is prose rather than code has changed"
    );
}

#[test]
fn every_widget_has_a_usage_hint() {
    let registry = builtin_registry();
    let missing: Vec<&str> = registry
        .list_types()
        .into_iter()
        .filter(|name| {
            registry
                .get_schema(name)
                .and_then(|s| s.usage_hint.as_deref())
                .is_none_or(str::is_empty)
        })
        .collect();
    assert!(
        missing.is_empty(),
        "widgets with no usage hint: {missing:?}"
    );
}
