#[cfg(test)]
mod tests {
    use hawktui::core::buffer::Buffer;
    use hawktui::core::rect::Rect;
    use hawktui::core::style::Style;
    use hawktui::core::text::{Line, Text};
    use hawktui::widget::Widget;

    /// Helper: render a widget into a buffer and return row text.
    fn render_to_strings<W: Widget>(widget: W, width: u16, height: u16) -> Vec<String> {
        let area = Rect::new(0, 0, width, height);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);
        (0..height)
            .map(|y| {
                let mut row = String::new();
                for x in 0..width {
                    if let Some(cell) = buf.cell(hawktui::core::rect::Position::new(x, y)) {
                        row.push_str(cell.symbol.as_str());
                    }
                }
                row.trim_end().to_string()
            })
            .collect()
    }

    mod paragraph {
        use super::*;
        use hawktui::widget::paragraph::Paragraph;

        #[test]
        fn renders_single_line() {
            let rows = render_to_strings(Paragraph::new(Text::raw("Hello")), 20, 3);
            assert_eq!(rows[0], "Hello");
            assert_eq!(rows[1], "");
        }

        #[test]
        fn renders_multiline_text() {
            let text = Text {
                lines: vec![
                    Line::from("Line 1"),
                    Line::from("Line 2"),
                    Line::from("Line 3"),
                ],
                alignment: Some(hawktui::layout::Alignment::Left),
                style: Style::default(),
            };
            let rows = render_to_strings(Paragraph::new(text), 20, 5);
            assert_eq!(rows[0], "Line 1");
            assert_eq!(rows[1], "Line 2");
            assert_eq!(rows[2], "Line 3");
        }

        #[test]
        fn truncates_long_line() {
            let rows = render_to_strings(Paragraph::new(Text::raw("ABCDEFGHIJ")), 5, 1);
            assert_eq!(rows[0], "ABCDE");
        }
    }

    mod block_widget {
        use super::*;
        use hawktui::widget::block::{Block, Borders};

        #[test]
        fn renders_borders_with_title() {
            let block = Block::default().title("Test").borders(Borders::ALL);
            let rows = render_to_strings(block, 10, 5);
            // Top border with title
            assert!(rows[0].contains("Test"));
            // Should have border characters on first and last rows
            assert!(rows[0].starts_with('┌') || rows[0].starts_with("┌"));
        }

        #[test]
        fn empty_block_no_borders() {
            let block = Block::default().borders(Borders::NONE);
            let rows = render_to_strings(block, 5, 3);
            // All rows should be empty
            for row in &rows {
                assert_eq!(row, "");
            }
        }
    }

    mod loader {
        use super::*;
        use hawktui::widget::loader::{Loader, SpinnerStyle};

        #[test]
        fn renders_spinner_and_message() {
            let loader = Loader::new("Loading...").tick(0);
            let rows = render_to_strings(loader, 20, 1);
            assert!(rows[0].contains("Loading..."));
        }

        #[test]
        fn different_ticks_show_different_frames() {
            let l0 = Loader::new("").spinner_style(SpinnerStyle::Line).tick(0);
            let l1 = Loader::new("").spinner_style(SpinnerStyle::Line).tick(1);

            let r0 = render_to_strings(l0, 5, 1);
            let r1 = render_to_strings(l1, 5, 1);
            assert_ne!(r0[0], r1[0], "Different ticks should show different frames");
        }
    }

    mod cancellable_loader {
        use super::*;
        use hawktui::widget::cancellable_loader::CancellableLoader;

        #[test]
        fn renders_spinner_and_message() {
            let loader = CancellableLoader::new("Processing...").tick(0);
            let rows = render_to_strings(loader, 40, 1);
            assert!(rows[0].contains("Processing..."));
        }

        #[test]
        fn cancel_action_sets_cancelled() {
            use hawktui::ontology::Discoverable;
            let mut loader = CancellableLoader::new("Working...");
            assert!(!loader.is_cancelled());
            let result = loader.execute_action("cancel", &serde_json::json!({}));
            assert!(result.is_ok());
            assert!(loader.is_cancelled());
        }
    }

    mod editor_widget {
        use super::*;
        use hawktui::widget::editor::Editor;
        use hawktui::widget::editor::EditorState;
        use hawktui::widget::StatefulWidget;

        fn render_editor(editor: &Editor, state: &mut EditorState, w: u16, h: u16) -> Vec<String> {
            let area = Rect::new(0, 0, w, h);
            let mut buf = Buffer::empty(area);
            editor.clone().render(area, &mut buf, state);
            (0..h)
                .map(|y| {
                    let mut row = String::new();
                    for x in 0..w {
                        if let Some(cell) = buf.cell(hawktui::core::rect::Position::new(x, y)) {
                            row.push_str(cell.symbol.as_str());
                        }
                    }
                    row.trim_end().to_string()
                })
                .collect()
        }

        #[test]
        fn renders_text_content() {
            let editor = Editor::new().show_line_numbers(false);
            let mut state = EditorState::with_text("Hello\nWorld");
            let rows = render_editor(&editor, &mut state, 20, 3);
            assert_eq!(rows[0], "Hello");
            assert_eq!(rows[1], "World");
        }

        #[test]
        fn renders_line_numbers() {
            let editor = Editor::new().show_line_numbers(true);
            let mut state = EditorState::with_text("A\nB\nC");
            let rows = render_editor(&editor, &mut state, 20, 3);
            // Line numbers should appear
            assert!(
                rows[0].contains("1"),
                "Expected line number 1, got: {}",
                rows[0]
            );
            assert!(rows[0].contains("A"), "Expected text A, got: {}", rows[0]);
        }
    }

    mod select_list_widget {
        use super::*;
        use hawktui::widget::select_list::{SelectItem, SelectList, SelectListState, SelectMode};
        #[allow(unused_imports)]
        use hawktui::widget::StatefulWidget;

        fn render_select(
            list: &SelectList,
            state: &mut SelectListState,
            w: u16,
            h: u16,
        ) -> Vec<String> {
            let area = Rect::new(0, 0, w, h);
            let mut buf = Buffer::empty(area);
            list.clone().render(area, &mut buf, state);
            (0..h)
                .map(|y| {
                    let mut row = String::new();
                    for x in 0..w {
                        if let Some(cell) = buf.cell(hawktui::core::rect::Position::new(x, y)) {
                            row.push_str(cell.symbol.as_str());
                        }
                    }
                    row.trim_end().to_string()
                })
                .collect()
        }

        #[test]
        fn renders_items() {
            let items = vec![
                SelectItem::new("Alpha", "a"),
                SelectItem::new("Beta", "b"),
                SelectItem::new("Gamma", "c"),
            ];
            let list = SelectList::new(items.clone());
            let mut state = SelectListState::new();
            let rows = render_select(&list, &mut state, 20, 5);
            assert!(
                rows[0].contains("Alpha"),
                "Expected Alpha, got: {}",
                rows[0]
            );
            assert!(rows[1].contains("Beta"), "Expected Beta, got: {}", rows[1]);
        }

        #[test]
        fn selected_item_has_marker() {
            let items = vec![SelectItem::new("Yes", "y"), SelectItem::new("No", "n")];
            let list = SelectList::new(items.clone());
            let mut state = SelectListState::new();
            state.toggle_select(0, SelectMode::Single); // select first item
            let rows = render_select(&list, &mut state, 20, 3);
            // The selected row should differ from unselected
            assert_ne!(rows[0], rows[1], "Selected row should look different");
        }
    }

    mod theme {
        use hawktui::core::style::{Color, Style};
        use hawktui::theme::{Theme, ThemeToken};

        #[test]
        fn dark_theme_has_primary_color() {
            let theme = Theme::dark();
            let primary = theme.get(ThemeToken::Primary);
            assert_eq!(primary.fg, Some(Color::White));
        }

        #[test]
        fn light_theme_has_primary_color() {
            let theme = Theme::light();
            let primary = theme.get(ThemeToken::Primary);
            assert_eq!(primary.fg, Some(Color::Black));
        }

        #[test]
        fn custom_theme_overrides() {
            let theme =
                Theme::new("custom").set(ThemeToken::Primary, Style::default().fg(Color::Magenta));
            assert_eq!(theme.get(ThemeToken::Primary).fg, Some(Color::Magenta));
            // Unset tokens return default
            assert_eq!(theme.get(ThemeToken::Error).fg, None);
        }

        #[test]
        fn set_overwrites_existing() {
            let theme = Theme::dark().set(ThemeToken::Primary, Style::default().fg(Color::Red));
            assert_eq!(theme.get(ThemeToken::Primary).fg, Some(Color::Red));
        }
    }

    mod barchart {
        use super::*;
        use hawktui::widget::barchart::{Bar, BarChart, BarGroup};

        #[test]
        fn renders_single_bar() {
            let chart = BarChart::new(vec![BarGroup::new(vec![Bar::new(100)])]).bar_width(3);
            let rows = render_to_strings(chart, 20, 8);
            // The bar should contain block characters
            let has_bar = rows.iter().any(|r| r.contains('█'));
            assert!(has_bar, "Expected bar to contain block chars: {:?}", rows);
        }

        #[test]
        fn renders_multiple_bars() {
            let chart = BarChart::new(vec![BarGroup::new(vec![
                Bar::new(50).label("A"),
                Bar::new(100).label("B"),
            ])])
            .bar_width(3)
            .bar_gap(1);
            let rows = render_to_strings(chart, 20, 8);
            // Labels on the bottom row
            let label_row = &rows[6]; // height-2 for labels
            assert!(
                label_row.contains('A')
                    || label_row.contains('B')
                    || rows[7].contains('A')
                    || rows[7].contains('B'),
                "Expected bar labels: {:?}",
                rows
            );
        }

        #[test]
        fn discoverable_schema() {
            use hawktui::ontology::Discoverable;
            let schema = BarChart::schema();
            assert_eq!(schema.name, "BarChart");
            assert!(schema.tags.contains(&"chart".to_string()));
        }
    }

    mod chart_widget {
        use super::*;
        use hawktui::widget::chart::{Axis, Chart, Dataset, GraphType, Marker};

        #[test]
        fn renders_scatter_points() {
            let data = Dataset::new(vec![(0.0, 0.0), (1.0, 1.0)])
                .marker(Marker::Block)
                .graph_type(GraphType::Scatter);
            let chart = Chart::new(vec![data])
                .x_axis(Axis::new([0.0, 1.0]))
                .y_axis(Axis::new([0.0, 1.0]));
            let rows = render_to_strings(chart, 20, 10);
            // Should have at least one block character
            let has_point = rows.iter().any(|r| r.contains('█'));
            assert!(has_point, "Expected scatter points: {:?}", rows);
        }

        #[test]
        fn renders_braille_line() {
            let data = Dataset::new(vec![(0.0, 0.0), (0.5, 0.5), (1.0, 1.0)])
                .marker(Marker::Braille)
                .graph_type(GraphType::Line);
            let chart = Chart::new(vec![data])
                .x_axis(Axis::new([0.0, 1.0]))
                .y_axis(Axis::new([0.0, 1.0]));
            let rows = render_to_strings(chart, 30, 15);
            // Should have braille characters (U+2800 block)
            let has_braille = rows
                .iter()
                .any(|r| r.chars().any(|c| c as u32 >= 0x2800 && c as u32 <= 0x28FF));
            assert!(has_braille, "Expected braille chars: {:?}", rows);
        }

        #[test]
        fn discoverable_schema() {
            use hawktui::ontology::Discoverable;
            let schema = Chart::schema();
            assert_eq!(schema.name, "Chart");
            assert!(schema.tags.contains(&"plot".to_string()));
        }
    }

    mod image_widget {
        use super::*;
        use hawktui::widget::image::{Image, ImageProtocol};

        #[test]
        fn fallback_renders_text() {
            let img = Image::new(vec![0u8; 10], "image/png")
                .protocol(ImageProtocol::Fallback)
                .fallback_text("[logo]");
            let rows = render_to_strings(img, 20, 5);
            let has_text = rows.iter().any(|r| r.contains("[logo]"));
            assert!(has_text, "Expected fallback text: {:?}", rows);
        }

        #[test]
        fn discoverable_schema() {
            use hawktui::ontology::Discoverable;
            let schema = Image::schema();
            assert_eq!(schema.name, "Image");
            assert!(schema.tags.contains(&"image".to_string()));
        }
    }

    mod settings_list_widget {
        use super::*;
        use hawktui::widget::settings_list::{Setting, SettingsList, SettingsListState};

        fn render_stateful(
            widget: SettingsList,
            state: &mut SettingsListState,
            w: u16,
            h: u16,
        ) -> Vec<String> {
            use hawktui::widget::StatefulWidget;
            let area = Rect::new(0, 0, w, h);
            let mut buf = Buffer::empty(area);
            StatefulWidget::render(widget, area, &mut buf, state);
            (0..h)
                .map(|y| {
                    let mut row = String::new();
                    for x in 0..w {
                        if let Some(cell) = buf.cell(hawktui::core::rect::Position::new(x, y)) {
                            row.push_str(cell.symbol.as_str());
                        }
                    }
                    row.trim_end().to_string()
                })
                .collect()
        }

        #[test]
        fn renders_settings() {
            let settings = vec![
                Setting::new("Theme", vec!["Dark".into(), "Light".into()]),
                Setting::new("Size", vec!["12".into(), "14".into()]),
            ];
            let widget = SettingsList::new(settings);
            let mut state = SettingsListState::new();
            let rows = render_stateful(widget, &mut state, 40, 5);
            assert!(
                rows[0].contains("Theme"),
                "Expected 'Theme' in first row: {:?}",
                rows
            );
            assert!(
                rows[0].contains("Dark"),
                "Expected 'Dark' value: {:?}",
                rows
            );
        }

        #[test]
        fn cycle_next_changes_value() {
            use hawktui::ontology::Discoverable;
            let settings = vec![Setting::new(
                "Theme",
                vec!["Dark".into(), "Light".into(), "Auto".into()],
            )];
            let mut widget = SettingsList::new(settings);
            let result = widget.execute_action("cycle_next", &serde_json::json!({"index": 0}));
            assert!(result.is_ok());
            assert_eq!(widget.settings()[0].current_value(), "Light");
        }

        #[test]
        fn cycle_prev_wraps() {
            use hawktui::ontology::Discoverable;
            let settings = vec![Setting::new("Theme", vec!["Dark".into(), "Light".into()])];
            let mut widget = SettingsList::new(settings);
            // Current is 0 (Dark), cycle_prev should wrap to 1 (Light)
            let result = widget.execute_action("cycle_prev", &serde_json::json!({"index": 0}));
            assert!(result.is_ok());
            assert_eq!(widget.settings()[0].current_value(), "Light");
        }
    }

    mod fuzzy_matching {
        use hawktui::util::fuzzy::fuzzy_match;

        #[test]
        fn consecutive_match_beats_scattered_same_context() {
            // No word boundaries — pure consecutive vs gap scoring
            let consecutive = fuzzy_match("abc", "xxxabcxxx").unwrap();
            let scattered = fuzzy_match("abc", "xxxaxbxcxxx").unwrap();
            assert!(
                consecutive.score < scattered.score,
                "consecutive {:?} should beat scattered {:?}",
                consecutive.score,
                scattered.score,
            );
        }

        #[test]
        fn exact_prefix_beats_mid_match() {
            // "set" at start should beat "set" in the middle
            let prefix = fuzzy_match("set", "set").unwrap();
            let mid = fuzzy_match("set", "reset").unwrap();
            assert!(prefix.score < mid.score);
        }
    }

    mod undo_stack {
        use hawktui::util::undo::UndoStack;

        #[test]
        fn undo_with_string_state() {
            let mut undo: UndoStack<String> = UndoStack::new(10);
            undo.push("initial".into());
            undo.push("modified".into());
            assert_eq!(undo.pop().unwrap(), "modified");
            assert_eq!(undo.pop().unwrap(), "initial");
        }
    }

    // ── Phase 9 tests ────────────────────────────────────────────────────────

    mod reflow {
        use hawktui::core::reflow::{CharWrapper, LineTruncator, WordWrapper};
        use hawktui::core::style::{Color, Style};
        use hawktui::core::text::{Line, Span};

        fn line_text(line: &Line) -> String {
            line.spans.iter().map(|s| s.content.as_ref()).collect()
        }

        #[test]
        fn word_wrap_breaks_at_space() {
            let lines = vec![Line::raw("hello world foo".to_string())];
            let wrapped = WordWrapper::wrap(&lines, 11, Style::default());
            assert_eq!(wrapped.len(), 2);
            assert_eq!(line_text(&wrapped[0]), "hello world");
            assert_eq!(line_text(&wrapped[1]), "foo");
        }

        #[test]
        fn word_wrap_preserves_span_styles() {
            let styled = Line::from(vec![
                Span::styled("red ".to_string(), Style::default().fg(Color::Red)),
                Span::styled("blue".to_string(), Style::default().fg(Color::Blue)),
            ]);
            let wrapped = WordWrapper::wrap(&[styled], 4, Style::default());
            assert_eq!(wrapped.len(), 2);
            assert_eq!(wrapped[0].spans[0].style.fg, Some(Color::Red));
            assert_eq!(wrapped[1].spans[0].style.fg, Some(Color::Blue));
        }

        #[test]
        fn char_wrap_breaks_mid_word() {
            let lines = vec![Line::raw("abcdefgh".to_string())];
            let wrapped = CharWrapper::wrap(&lines, 3, Style::default());
            assert_eq!(wrapped.len(), 3);
            assert_eq!(line_text(&wrapped[0]), "abc");
            assert_eq!(line_text(&wrapped[1]), "def");
            assert_eq!(line_text(&wrapped[2]), "gh");
        }

        #[test]
        fn truncator_clips_long_line() {
            let lines = vec![Line::raw("hello world".to_string())];
            let truncated = LineTruncator::truncate(&lines, 5, Style::default());
            assert_eq!(truncated.len(), 1);
            assert_eq!(line_text(&truncated[0]), "hello");
        }
    }

    mod paragraph_wrap {
        use super::*;
        use hawktui::widget::paragraph::{Paragraph, Wrap};

        #[test]
        fn word_wrap_renders_multiple_lines() {
            let p = Paragraph::new(Text::raw("hello world foo")).wrap(Wrap::Word);
            let rows = render_to_strings(p, 11, 3);
            assert_eq!(rows[0].trim(), "hello world");
            assert_eq!(rows[1].trim(), "foo");
        }

        #[test]
        fn char_wrap_breaks_long_word() {
            let p = Paragraph::new(Text::raw("abcdefghij")).wrap(Wrap::Char);
            let rows = render_to_strings(p, 4, 4);
            assert_eq!(rows[0].trim(), "abcd");
            assert_eq!(rows[1].trim(), "efgh");
            assert_eq!(rows[2].trim(), "ij");
        }
    }

    mod line_gauge_widget {
        use super::*;
        use hawktui::ontology::Discoverable;
        use hawktui::widget::line_gauge::LineGauge;

        #[test]
        fn renders_filled_and_unfilled() {
            let gauge = LineGauge::new().ratio(0.5);
            let rows = render_to_strings(gauge, 10, 1);
            // First 5 columns should be filled (━), last 5 unfilled (─)
            let line = &rows[0];
            assert!(line.contains('━'));
            assert!(line.contains('─'));
        }

        #[test]
        fn renders_with_label() {
            let gauge = LineGauge::new().ratio(0.5).label("Prog");
            let rows = render_to_strings(gauge, 20, 1);
            assert!(rows[0].contains("Prog"));
        }

        #[test]
        fn discoverable_schema() {
            let schema = LineGauge::schema();
            assert_eq!(schema.name, "LineGauge");
            assert!(!schema.properties.is_empty());
        }
    }

    mod calendar_widget {
        use super::*;
        use hawktui::ontology::Discoverable;
        use hawktui::widget::calendar::Calendar;

        #[test]
        fn renders_day_headers() {
            let cal = Calendar::new(2026, 3).show_header(true);
            let rows = render_to_strings(cal, 21, 7);
            assert!(rows[0].contains("Mo"));
            assert!(rows[0].contains("Su"));
        }

        #[test]
        fn renders_days_of_month() {
            let cal = Calendar::new(2026, 3);
            let rows = render_to_strings(cal, 21, 7);
            // Concatenate all rows and check for day numbers
            let all: String = rows.join(" ");
            assert!(all.contains("1"));
            assert!(all.contains("31"));
        }

        #[test]
        fn discoverable_schema() {
            let schema = Calendar::schema();
            assert_eq!(schema.name, "Calendar");
            assert!(schema.properties.len() >= 2);
        }
    }

    mod list_direction {
        use hawktui::core::buffer::Buffer;
        use hawktui::core::rect::{Position, Rect};
        use hawktui::widget::list::{List, ListDirection, ListState};
        use hawktui::widget::StatefulWidget;

        #[test]
        fn bottom_to_top_renders_last_items_at_bottom() {
            let items = vec!["A", "B", "C"];
            let list = List::new(items).direction(ListDirection::BottomToTop);
            let mut state = ListState::new();
            let area = Rect::new(0, 0, 10, 3);
            let mut buf = Buffer::empty(area);
            StatefulWidget::render(list, area, &mut buf, &mut state);

            // In BottomToTop, the first item in the list (A) is
            // anchored to the bottom row, growing upward.
            let bottom_cell = buf.cell(Position::new(0, 2)).unwrap();
            assert_eq!(bottom_cell.symbol, "A");
            let top_cell = buf.cell(Position::new(0, 0)).unwrap();
            assert_eq!(top_cell.symbol, "C");
        }
    }

    mod block_title_alignment {
        use super::*;
        use hawktui::core::text::Alignment;
        use hawktui::widget::block::Block;

        #[test]
        fn title_centered() {
            let block = Block::bordered()
                .title("Hi")
                .title_alignment(Alignment::Center);
            let rows = render_to_strings(block, 12, 3);
            // "Hi" should be roughly centered in 10 inner columns on the top border
            let top = &rows[0];
            // Find "Hi" position
            let pos = top.find("Hi").expect("title not found");
            // Should be offset from left edge (not at position 1)
            assert!(pos > 1);
        }

        #[test]
        fn title_right_aligned() {
            let block = Block::bordered()
                .title("Hi")
                .title_alignment(Alignment::Right);
            let rows = render_to_strings(block, 12, 3);
            let top = &rows[0];
            let pos = top.find("Hi").expect("title not found");
            // "Hi" should be near the right end
            assert!(pos >= 8);
        }
    }
}
