//! The authoring ontology: what an agent needs to *write* a Hawk TUI program.
//!
//! The widget ontology in [`schema`](super::schema) describes runtime state —
//! a gauge's ratio, a list's selection — which is what an agent *driving* a
//! running application needs. An agent writing one needs different facts:
//! which constructor to call, what a builder method takes, whether a widget
//! renders as `Widget` or `StatefulWidget` and with which state type, and what
//! the layout constraints are called.
//!
//! Those facts live in the signatures, so they are parsed out of the source by
//! `scripts/gen_api_ontology.py` rather than transcribed. Hand-maintained
//! copies of an API drift; this one fails CI when it does.
//!
//! ```
//! use hawktui::ontology::api;
//!
//! // Which state type does a List need?
//! let list = api::get("List").expect("List is a widget");
//! assert_eq!(list.state_type(), Some("ListState"));
//!
//! // What can I call on a Gauge?
//! let gauge = api::get("Gauge").expect("Gauge is a widget");
//! assert!(gauge.builders().any(|f| f.name == "percent"));
//!
//! // What are the layout constraints called?
//! let c = api::get("Constraint").expect("Constraint is in the layout module");
//! assert!(c.variants.contains(&"Percentage"));
//! ```

use serde::Serialize;

pub use super::api_generated::API;

/// How a type participates in rendering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ApiKind {
    /// Renders with `frame.render_widget(widget, area)`.
    Widget,
    /// Renders with `frame.render_stateful_widget(widget, area, &mut state)`,
    /// where the state is the named type.
    StatefulWidget { state: &'static str },
    /// A plain struct: a builder, a state value, a helper.
    Struct,
    /// An enum. Its `variants` are the spellings an author needs.
    Enum,
}

/// One public function, with the signature an author has to satisfy.
#[derive(Debug, Clone, Serialize)]
pub struct ApiFn {
    pub name: &'static str,
    /// The full signature as written, including argument and return types.
    pub signature: &'static str,
    /// `constructor` (no receiver), `builder` (takes `self`), or `method`.
    pub role: &'static str,
    /// First line of the doc comment, empty when undocumented.
    pub summary: &'static str,
}

/// A type an author builds a program out of.
#[derive(Debug, Clone, Serialize)]
pub struct ApiType {
    pub name: &'static str,
    /// The `use` path, e.g. `hawktui::widget::gauge`.
    pub module: &'static str,
    pub kind: ApiKind,
    pub summary: &'static str,
    /// Enum variant names; empty for non-enums.
    pub variants: &'static [&'static str],
    pub functions: &'static [ApiFn],
}

impl ApiType {
    /// The companion state type, when this renders as a `StatefulWidget`.
    ///
    /// Getting this wrong is the single most common mistake when writing
    /// against this framework, so it is a first-class field rather than
    /// something to infer.
    pub fn state_type(&self) -> Option<&'static str> {
        match &self.kind {
            ApiKind::StatefulWidget { state } if !state.is_empty() => Some(state),
            _ => None,
        }
    }

    /// Whether this type is rendered into a frame at all.
    pub fn is_widget(&self) -> bool {
        matches!(self.kind, ApiKind::Widget | ApiKind::StatefulWidget { .. })
    }

    /// Functions with no receiver — how you get one of these.
    pub fn constructors(&self) -> impl Iterator<Item = &ApiFn> {
        self.functions.iter().filter(|f| f.role == "constructor")
    }

    /// Functions taking `self` — the chainable configuration.
    pub fn builders(&self) -> impl Iterator<Item = &ApiFn> {
        self.functions.iter().filter(|f| f.role == "builder")
    }

    /// Functions taking `&self` or `&mut self`.
    pub fn methods(&self) -> impl Iterator<Item = &ApiFn> {
        self.functions.iter().filter(|f| f.role == "method")
    }

    /// The line to render this into a frame, or `None` if it is not a widget.
    pub fn render_call(&self) -> Option<String> {
        match self.state_type() {
            Some(state) => Some(format!(
                "frame.render_stateful_widget({}, area, &mut state); // state: {state}",
                lower(self.name)
            )),
            None if self.is_widget() => {
                Some(format!("frame.render_widget({}, area);", lower(self.name)))
            }
            None => None,
        }
    }
}

fn lower(name: &str) -> String {
    name.to_lowercase()
}

/// Look a type up by exact name.
pub fn get(name: &str) -> Option<&'static ApiType> {
    API.iter().find(|t| t.name == name)
}

/// Types whose name, module, summary, variants or function names mention the
/// query, case-insensitively.
pub fn search(query: &str) -> Vec<&'static ApiType> {
    let q = query.to_lowercase();
    API.iter()
        .filter(|t| {
            t.name.to_lowercase().contains(&q)
                || t.module.to_lowercase().contains(&q)
                || t.summary.to_lowercase().contains(&q)
                || t.variants.iter().any(|v| v.to_lowercase().contains(&q))
                || t.functions.iter().any(|f| {
                    f.name.to_lowercase().contains(&q) || f.summary.to_lowercase().contains(&q)
                })
        })
        .collect()
}

/// Every widget that needs a companion state value, with that state's name.
pub fn stateful_widgets() -> Vec<(&'static str, &'static str)> {
    API.iter()
        .filter_map(|t| t.state_type().map(|s| (t.name, s)))
        .collect()
}

/// How many public functions the authoring ontology describes.
pub fn function_count() -> usize {
    API.iter().map(|t| t.functions.len()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_catalog_is_populated() {
        assert!(API.len() >= 60, "only {} types", API.len());
        assert!(
            function_count() >= 300,
            "only {} functions",
            function_count()
        );
    }

    #[test]
    fn stateful_widgets_name_their_state() {
        let pairs = stateful_widgets();
        for expected in [
            ("List", "ListState"),
            ("Table", "TableState"),
            ("Editor", "EditorState"),
            ("Scrollbar", "ScrollbarState"),
        ] {
            assert!(
                pairs.contains(&expected),
                "missing {expected:?} in {pairs:?}"
            );
        }
    }

    #[test]
    fn the_layout_system_is_described() {
        let constraint = get("Constraint").expect("Constraint is in the catalog");
        for v in ["Length", "Percentage", "Min", "Max", "Ratio", "Fill"] {
            assert!(constraint.variants.contains(&v), "missing variant {v}");
        }
        assert!(get("Layout").is_some(), "Layout is missing");
    }

    #[test]
    fn signatures_carry_argument_types() {
        let gauge = get("Gauge").expect("Gauge is in the catalog");
        let percent = gauge
            .builders()
            .find(|f| f.name == "percent")
            .expect("Gauge::percent");
        assert!(percent.signature.contains("u16"), "{}", percent.signature);
    }

    #[test]
    fn render_calls_distinguish_stateful_widgets() {
        assert_eq!(
            get("Paragraph").and_then(|t| t.render_call()).as_deref(),
            Some("frame.render_widget(paragraph, area);")
        );
        let list = get("List").and_then(|t| t.render_call()).unwrap();
        assert!(list.contains("render_stateful_widget"), "{list}");
        assert!(list.contains("ListState"), "{list}");
        assert!(get("Constraint").and_then(|t| t.render_call()).is_none());
    }

    #[test]
    fn search_finds_by_function_name() {
        assert!(search("highlight_symbol").iter().any(|t| t.name == "List"));
    }
}
