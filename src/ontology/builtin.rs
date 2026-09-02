//! Registration of every built-in discoverable widget.
//!
//! This exists so that the catalog an agent sees is complete by default.
//! Before it, each embedder assembled its own list by hand and
//! `hawktui-server` registered six of the twenty-one discoverable widgets, so
//! an agent asking a running application what it could do was told about less
//! than a third of the framework.
//!
//! `tests/ontology_registry_tests.rs` scans the widget sources for
//! `impl Discoverable for` and fails when a type is missing here, so the list
//! cannot silently fall behind again.

use super::registry::OntologyRegistry;
use crate::widget::barchart::BarChart;
use crate::widget::block::Block;
use crate::widget::calendar::Calendar;
use crate::widget::cancellable_loader::CancellableLoader;
use crate::widget::canvas::Canvas;
use crate::widget::chart::Chart;
use crate::widget::editor::Editor;
use crate::widget::gauge::Gauge;
use crate::widget::image::Image;
use crate::widget::input::Input;
use crate::widget::line_gauge::LineGauge;
use crate::widget::list::List;
use crate::widget::loader::Loader;
use crate::widget::markdown::Markdown;
use crate::widget::paragraph::Paragraph;
use crate::widget::scrollbar::Scrollbar;
use crate::widget::select_list::SelectList;
use crate::widget::settings_list::SettingsList;
use crate::widget::sparkline::Sparkline;
use crate::widget::table::Table;
use crate::widget::tabs::Tabs;

/// Register the schema of every built-in widget that implements
/// [`Discoverable`](super::Discoverable).
///
/// ```
/// use hawktui::ontology::{register_builtin_widgets, registry::OntologyRegistry};
///
/// let mut registry = OntologyRegistry::new();
/// register_builtin_widgets(&mut registry);
///
/// let gauge = registry.get_schema("Gauge").expect("Gauge is built in");
/// assert_eq!(gauge.name, "Gauge");
/// ```
pub fn register_builtin_widgets(registry: &mut OntologyRegistry) {
    registry.register::<BarChart>();
    registry.register::<Block>();
    registry.register::<Calendar>();
    registry.register::<CancellableLoader>();
    registry.register::<Canvas>();
    registry.register::<Chart>();
    registry.register::<Editor>();
    registry.register::<Gauge>();
    registry.register::<Image>();
    registry.register::<Input>();
    registry.register::<LineGauge>();
    registry.register::<List>();
    registry.register::<Loader>();
    registry.register::<Markdown>();
    registry.register::<Paragraph>();
    registry.register::<Scrollbar>();
    registry.register::<SelectList>();
    registry.register::<SettingsList>();
    registry.register::<Sparkline>();
    registry.register::<Table>();
    registry.register::<Tabs>();
}

/// A registry preloaded with every built-in widget schema.
pub fn builtin_registry() -> OntologyRegistry {
    let mut registry = OntologyRegistry::new();
    register_builtin_widgets(&mut registry);
    registry
}
