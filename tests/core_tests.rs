#[cfg(test)]
mod tests {
    mod rect {
        use louie::core::rect::{Margin, Position, Rect};

        #[test]
        fn new_and_accessors() {
            let r = Rect::new(2, 3, 10, 5);
            assert_eq!(r.left(), 2);
            assert_eq!(r.right(), 12);
            assert_eq!(r.top(), 3);
            assert_eq!(r.bottom(), 8);
            assert_eq!(r.area(), 50);
            assert!(!r.is_empty());
        }

        #[test]
        fn zero_rect_is_empty() {
            assert!(Rect::ZERO.is_empty());
            assert!(Rect::new(0, 0, 5, 0).is_empty());
            assert!(Rect::new(0, 0, 0, 5).is_empty());
        }

        #[test]
        fn intersection() {
            let a = Rect::new(0, 0, 10, 10);
            let b = Rect::new(5, 5, 10, 10);
            let i = a.intersection(b);
            assert_eq!(i, Rect::new(5, 5, 5, 5));
        }

        #[test]
        fn intersection_no_overlap() {
            let a = Rect::new(0, 0, 5, 5);
            let b = Rect::new(10, 10, 5, 5);
            let i = a.intersection(b);
            assert!(i.is_empty());
        }

        #[test]
        fn union_rects() {
            let a = Rect::new(2, 3, 4, 5);
            let b = Rect::new(10, 1, 3, 8);
            let u = a.union(b);
            assert_eq!(u, Rect::new(2, 1, 11, 8));
        }

        #[test]
        fn contains_position() {
            let r = Rect::new(5, 5, 10, 10);
            assert!(r.contains(Position::new(5, 5)));
            assert!(r.contains(Position::new(14, 14)));
            assert!(!r.contains(Position::new(15, 15)));
            assert!(!r.contains(Position::new(4, 5)));
        }

        #[test]
        fn inner_margin() {
            let r = Rect::new(0, 0, 20, 10);
            let m = Margin::symmetric(1, 2);
            let inner = r.inner(m);
            assert_eq!(inner, Rect::new(2, 1, 16, 8));
        }
    }

    mod buffer {
        use louie::core::buffer::Buffer;
        use louie::core::rect::{Position, Rect};
        use louie::core::style::Style;

        #[test]
        fn empty_buffer_dimensions() {
            let buf = Buffer::empty(Rect::new(0, 0, 10, 5));
            assert_eq!(buf.area, Rect::new(0, 0, 10, 5));
            assert_eq!(buf.content.len(), 50);
        }

        #[test]
        fn set_and_get_string() {
            let mut buf = Buffer::empty(Rect::new(0, 0, 20, 3));
            buf.set_string(0, 0, "Hello", Style::default());
            let cell = buf.cell(Position::new(0, 0)).unwrap();
            assert_eq!(cell.symbol, "H");
            let cell = buf.cell(Position::new(4, 0)).unwrap();
            assert_eq!(cell.symbol, "o");
        }

        #[test]
        fn with_lines() {
            let buf = Buffer::with_lines(["abc", "defgh"]);
            assert_eq!(buf.area.width, 5);
            assert_eq!(buf.area.height, 2);
            let cell = buf.cell(Position::new(0, 1)).unwrap();
            assert_eq!(cell.symbol, "d");
        }

        #[test]
        fn out_of_bounds_returns_none() {
            let buf = Buffer::empty(Rect::new(0, 0, 5, 5));
            assert!(buf.cell(Position::new(10, 10)).is_none());
        }

        #[test]
        fn reset_clears_content() {
            let mut buf = Buffer::empty(Rect::new(0, 0, 10, 1));
            buf.set_string(0, 0, "test", Style::default());
            buf.reset();
            let cell = buf.cell(Position::new(0, 0)).unwrap();
            assert_eq!(cell.symbol, " ");
        }
    }

    mod style {
        use louie::core::style::{Color, Style};

        #[test]
        fn default_style_has_no_colors() {
            let s = Style::default();
            assert_eq!(s.fg, None);
            assert_eq!(s.bg, None);
        }

        #[test]
        fn builder_methods() {
            let s = Style::default()
                .fg(Color::Red)
                .bg(Color::Blue)
                .bold()
                .italic();
            assert_eq!(s.fg, Some(Color::Red));
            assert_eq!(s.bg, Some(Color::Blue));
            assert!(s.add_modifier.contains(louie::core::style::Modifier::BOLD));
            assert!(s
                .add_modifier
                .contains(louie::core::style::Modifier::ITALIC));
        }

        #[test]
        fn patch_overrides_set_fields() {
            let base = Style::default().fg(Color::Red).bold();
            let over = Style::default().fg(Color::Green).italic();
            let merged = base.patch(over);
            assert_eq!(merged.fg, Some(Color::Green));
            assert!(merged
                .add_modifier
                .contains(louie::core::style::Modifier::BOLD));
            assert!(merged
                .add_modifier
                .contains(louie::core::style::Modifier::ITALIC));
        }
    }

    mod layout {
        use louie::core::rect::Rect;
        use louie::layout::{Constraint, Direction, Layout};

        #[test]
        fn vertical_split_fixed() {
            let area = Rect::new(0, 0, 40, 20);
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(5), Constraint::Length(15)])
                .split(area);
            assert_eq!(chunks.len(), 2);
            assert_eq!(chunks[0].height, 5);
            assert_eq!(chunks[1].height, 15);
        }

        #[test]
        fn horizontal_split_percentage() {
            let area = Rect::new(0, 0, 100, 10);
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                .split(area);
            assert_eq!(chunks.len(), 2);
            assert_eq!(chunks[0].width, 30);
            assert_eq!(chunks[1].width, 70);
        }

        #[test]
        fn single_fill_takes_all_space() {
            let area = Rect::new(0, 0, 40, 20);
            let chunks = Layout::default()
                .constraints([Constraint::Fill(1)])
                .split(area);
            assert_eq!(chunks.len(), 1);
            assert_eq!(chunks[0], area);
        }
    }

    mod widget_rendering {
        use louie::core::buffer::Buffer;
        use louie::core::rect::Rect;
        use louie::widget::Widget;

        #[test]
        fn str_renders_at_position() {
            let area = Rect::new(0, 0, 20, 1);
            let mut buf = Buffer::empty(area);
            "Hello".render(area, &mut buf);
            let text: String = (0..5)
                .map(|x| {
                    buf.cell(louie::core::rect::Position::new(x, 0))
                        .unwrap()
                        .symbol
                        .clone()
                })
                .collect();
            assert_eq!(text, "Hello");
        }
    }

    mod editor_state {
        use louie::widget::editor::EditorState;

        #[test]
        fn new_has_one_empty_line() {
            let state = EditorState::new();
            assert_eq!(state.lines.len(), 1);
            assert_eq!(state.lines[0], "");
        }

        #[test]
        fn with_text_preserves_lines() {
            let state = EditorState::with_text("line 1\nline 2\nline 3");
            assert_eq!(state.lines.len(), 3);
            assert_eq!(state.lines[1], "line 2");
        }

        #[test]
        fn insert_char_and_text() {
            let mut state = EditorState::new();
            state.insert_char('H');
            state.insert_char('i');
            assert_eq!(state.text(), "Hi");
            assert_eq!(state.cursor_col, 2);
        }

        #[test]
        fn insert_newline_splits_line() {
            let mut state = EditorState::with_text("Hello World");
            state.cursor_col = 5;
            state.insert_newline();
            assert_eq!(state.lines.len(), 2);
            assert_eq!(state.lines[0], "Hello");
            assert_eq!(state.lines[1], " World");
            assert_eq!(state.cursor_row, 1);
            assert_eq!(state.cursor_col, 0);
        }

        #[test]
        fn delete_before_merges_lines() {
            let mut state = EditorState::with_text("ab\ncd");
            state.cursor_row = 1;
            state.cursor_col = 0;
            state.delete_before();
            assert_eq!(state.lines.len(), 1);
            assert_eq!(state.text(), "abcd");
            assert_eq!(state.cursor_col, 2);
        }

        #[test]
        fn cursor_movement() {
            let mut state = EditorState::with_text("abc\ndef\nghi");
            // Start at 0,0
            state.move_right();
            assert_eq!(state.cursor_col, 1);
            state.move_down();
            assert_eq!(state.cursor_row, 1);
            assert_eq!(state.cursor_col, 1);
            state.move_end();
            assert_eq!(state.cursor_col, 3);
            state.move_home();
            assert_eq!(state.cursor_col, 0);
            state.move_up();
            assert_eq!(state.cursor_row, 0);
        }
    }

    mod select_list_state {
        use louie::widget::select_list::{SelectListState, SelectMode};

        #[test]
        fn move_wraps_around() {
            let mut state = SelectListState::new();
            state.move_down(5); // 0 -> 1
            assert_eq!(state.cursor, 1);
            state.move_up(5); // 1 -> 0
            assert_eq!(state.cursor, 0);
            state.move_up(5); // 0 -> 4 (wrap)
            assert_eq!(state.cursor, 4);
        }

        #[test]
        fn single_select_replaces() {
            let mut state = SelectListState::new();
            state.toggle_select(0, SelectMode::Single);
            state.toggle_select(2, SelectMode::Single);
            assert_eq!(state.selected_indices(), &[2]);
        }

        #[test]
        fn multi_select_toggles() {
            let mut state = SelectListState::new();
            state.toggle_select(0, SelectMode::Multi);
            state.toggle_select(2, SelectMode::Multi);
            assert_eq!(state.selected_indices(), &[0, 2]);
            state.toggle_select(0, SelectMode::Multi);
            assert_eq!(state.selected_indices(), &[2]);
        }
    }

    mod focus {
        use louie::focus::FocusManager;

        #[test]
        fn focus_ring_navigation() {
            let mut fm = FocusManager::new();
            fm.register("a");
            fm.register("b");
            fm.register("c");
            assert_eq!(fm.focused_id(), None);

            fm.focus_next();
            assert_eq!(fm.focused_id(), Some("a"));

            fm.focus_next();
            assert_eq!(fm.focused_id(), Some("b"));

            fm.focus_previous();
            assert_eq!(fm.focused_id(), Some("a"));

            // Wrap around
            fm.focus_previous();
            assert_eq!(fm.focused_id(), Some("c"));
        }

        #[test]
        fn focus_by_id() {
            let mut fm = FocusManager::new();
            fm.register("x");
            fm.register("y");
            fm.focus_id("y");
            assert_eq!(fm.focused_id(), Some("y"));
            assert!(fm.is_focused("y"));
            assert!(!fm.is_focused("x"));
        }

        #[test]
        fn blur_clears_focus() {
            let mut fm = FocusManager::new();
            fm.register("item");
            fm.focus_next();
            assert!(fm.focused_id().is_some());
            fm.blur();
            assert_eq!(fm.focused_id(), None);
        }
    }

    mod agent_protocol {
        use louie::agent::protocol::{AgentRequest, AgentResponse, RequestEnvelope};

        #[test]
        fn request_roundtrip_json() {
            let req = AgentRequest::Ping;
            let json = serde_json::to_string(&req).unwrap();
            let parsed: AgentRequest = serde_json::from_str(&json).unwrap();
            assert!(matches!(parsed, AgentRequest::Ping));
        }

        #[test]
        fn envelope_with_id() {
            let envelope = RequestEnvelope {
                id: Some("req-42".into()),
                request: AgentRequest::GetState {
                    agent_id: "widget-1".into(),
                },
            };
            let json = serde_json::to_string(&envelope).unwrap();
            assert!(json.contains("req-42"));
            let parsed: RequestEnvelope = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed.id, Some("req-42".into()));
        }

        #[test]
        fn response_ok() {
            let resp = AgentResponse::ok(serde_json::json!(null));
            assert!(resp.success);
            assert!(resp.error.is_none());
        }

        #[test]
        fn response_err() {
            let resp = AgentResponse::err("something failed");
            assert!(!resp.success);
            assert_eq!(resp.error, Some("something failed".into()));
        }

        #[test]
        fn response_with_id() {
            let resp = AgentResponse::ok(serde_json::json!(null)).with_id("req-1");
            assert_eq!(resp.id, Some("req-1".into()));
        }
    }

    mod overlay {
        use louie::core::rect::Rect;
        use louie::overlay::{Overlay, OverlayStack};

        #[test]
        fn push_and_top() {
            let mut stack = OverlayStack::new();
            assert!(stack.top().is_none());

            stack.push(Overlay {
                id: "modal1".into(),
                area: Rect::new(5, 5, 30, 10),
                captures_focus: true,
            });
            assert_eq!(stack.top().unwrap().id, "modal1");
        }

        #[test]
        fn remove_by_id() {
            let mut stack = OverlayStack::new();
            stack.push(Overlay {
                id: "a".into(),
                area: Rect::new(0, 0, 10, 10),
                captures_focus: false,
            });
            stack.push(Overlay {
                id: "b".into(),
                area: Rect::new(0, 0, 10, 10),
                captures_focus: true,
            });
            stack.remove("a");
            assert_eq!(stack.top().unwrap().id, "b");
        }

        #[test]
        fn focus_capture() {
            let mut stack = OverlayStack::new();
            assert!(!stack.has_focus_capture());

            stack.push(Overlay {
                id: "modal".into(),
                area: Rect::ZERO,
                captures_focus: true,
            });
            assert!(stack.has_focus_capture());
            assert_eq!(stack.focus_capture_id(), Some("modal"));
        }
    }
}
