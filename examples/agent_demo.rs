//! Agent demo: showcases the ontology system for agent discoverability.
//!
//! Displays widgets and their ontology metadata, demonstrating how an agent
//! can discover, inspect, and interact with the UI.

use louie::ontology::registry::OntologyRegistry;
use louie::ontology::Discoverable;
use louie::prelude::*;
use louie::runtime::{Command, Model, Program, ProgramOptions};
use louie::widget::gauge::Gauge;
use louie::widget::list::{List, ListState};

struct App {
    list_state: ListState,
    items: Vec<String>,
    selected_schema: String,
}

enum Msg {
    SelectNext,
    SelectPrevious,
    ShowSchema,
    ExportCatalog,
    Quit,
}

impl App {
    fn new() -> Self {
        let items = vec![
            "Block".to_string(),
            "Paragraph".to_string(),
            "List".to_string(),
            "Tabs".to_string(),
            "Gauge".to_string(),
            "Input".to_string(),
            "Table".to_string(),
            "Canvas".to_string(),
            "Sparkline".to_string(),
            "Scrollbar".to_string(),
        ];
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            list_state,
            items,
            selected_schema: "Press Enter to view schema.".to_string(),
        }
    }
}

fn format_schema<W: Discoverable>() -> String {
    let schema = W::schema();
    let mut out = String::new();
    out.push_str(&format!("Widget: {}\n", schema.name));
    out.push_str(&format!("Role: {:?}\n", schema.default_role));
    out.push_str(&format!("Description: {}\n", schema.description));
    out.push_str(&format!("Tags: {}\n", schema.tags.join(", ")));
    if let Some(hint) = &schema.usage_hint {
        out.push_str(&format!("Usage: {}\n", hint));
    }
    out.push_str(&format!("Properties: {}\n", schema.properties.len()));
    for prop in &schema.properties {
        out.push_str(&format!(
            "  - {} ({:?}): {}\n",
            prop.name, prop.property_type, prop.description
        ));
    }
    out
}

fn schema_for_index(index: usize) -> String {
    match index {
        0 => format_schema::<Block>(),
        1 => format_schema::<Paragraph>(),
        2 => format_schema::<List>(),
        3 => format_schema::<louie::widget::tabs::Tabs>(),
        4 => format_schema::<Gauge>(),
        5 => format_schema::<louie::widget::input::Input>(),
        6 => format_schema::<louie::widget::table::Table>(),
        7 => format_schema::<louie::widget::canvas::Canvas>(),
        8 => format_schema::<louie::widget::sparkline::Sparkline>(),
        9 => format_schema::<louie::widget::scrollbar::Scrollbar>(),
        _ => "Unknown widget".to_string(),
    }
}

impl Model for App {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::SelectNext => {
                self.list_state.select_next(self.items.len());
                Command::None
            }
            Msg::SelectPrevious => {
                self.list_state.select_previous();
                Command::None
            }
            Msg::ShowSchema => {
                if let Some(i) = self.list_state.selected {
                    self.selected_schema = schema_for_index(i);
                }
                Command::None
            }
            Msg::ExportCatalog => {
                let mut registry = OntologyRegistry::new();
                registry.register::<Block>();
                registry.register::<Paragraph>();
                registry.register::<List>();
                registry.register::<louie::widget::tabs::Tabs>();
                registry.register::<Gauge>();
                registry.register::<louie::widget::input::Input>();
                registry.register::<louie::widget::table::Table>();
                registry.register::<louie::widget::canvas::Canvas>();
                registry.register::<louie::widget::sparkline::Sparkline>();
                registry.register::<louie::widget::scrollbar::Scrollbar>();

                let catalog = registry.export_catalog();
                self.selected_schema = serde_json::to_string_pretty(&catalog)
                    .unwrap_or_else(|e| format!("Error: {e}"));
                Command::None
            }
            Msg::Quit => Command::Quit,
        }
    }

    fn view(&self, frame: &mut Frame<'_>) {
        let area = frame.area();

        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(area);

        // Widget list
        let list = List::new(self.items.iter().map(|s| s.clone()))
            .block(
                Block::default()
                    .title("Widgets (↑↓ to navigate)")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .highlight_symbol("▸ ")
            .highlight_style(Style::default().bold().fg(Color::Cyan));

        let mut list_state = self.list_state.clone();
        frame.render_stateful_widget(list, main_chunks[0], &mut list_state);

        // Schema display
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(main_chunks[1]);

        let schema_display = Paragraph::new(Text::raw(self.selected_schema.clone()))
            .block(
                Block::default()
                    .title("Ontology Schema (Enter to load)")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(schema_display, right_chunks[0]);

        // Help bar
        let help = Paragraph::new(Text::from(Line::from(vec![
            Span::styled("↑↓", Style::default().bold()),
            Span::raw(" Navigate  "),
            Span::styled("Enter", Style::default().bold()),
            Span::raw(" Show Schema  "),
            Span::styled("e", Style::default().bold()),
            Span::raw(" Export Catalog  "),
            Span::styled("q", Style::default().bold()),
            Span::raw(" Quit"),
        ])))
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
        frame.render_widget(help, right_chunks[1]);
    }

    fn handle_event(&self, event: Event) -> Option<Msg> {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => return Some(Msg::SelectPrevious),
                KeyCode::Down | KeyCode::Char('j') => return Some(Msg::SelectNext),
                KeyCode::Enter => return Some(Msg::ShowSchema),
                KeyCode::Char('e') => return Some(Msg::ExportCatalog),
                KeyCode::Char('q') | KeyCode::Esc => return Some(Msg::Quit),
                _ => {}
            }
        }
        None
    }

    fn register_ontology(&self, registry: &mut OntologyRegistry) {
        registry.register::<Block>();
        registry.register::<Paragraph>();
        registry.register::<List>();
        registry.register::<louie::widget::tabs::Tabs>();
        registry.register::<Gauge>();
        registry.register::<louie::widget::input::Input>();
        registry.register::<louie::widget::table::Table>();
        registry.register::<louie::widget::canvas::Canvas>();
        registry.register::<louie::widget::sparkline::Sparkline>();
        registry.register::<louie::widget::scrollbar::Scrollbar>();
    }
}

fn main() -> std::io::Result<()> {
    let app = App::new();
    let backend = CrosstermBackend::new(std::io::stdout());
    let options = ProgramOptions {
        tick_rate: None,
        ..Default::default()
    };
    Program::new(app, backend)?.with_options(options).run()?;
    Ok(())
}
