//! Tests for the headless test harness.

use hawktui::event::KeyCode;
use hawktui::prelude::*;
use hawktui::runtime::{Command, Model};
use hawktui::testing::{parse_key, Harness};
use hawktui::widget::list::{List, ListItem, ListState};

// ---------------------------------------------------------------- a list app

/// A selectable list, the shape most TUIs are built around: the selection lives
/// in the model, and the widget is rebuilt every frame from it.
struct ListApp {
    items: Vec<String>,
    selected: usize,
    quit_count: usize,
}

enum Msg {
    Down,
    Up,
    Quit,
}

impl ListApp {
    fn new(n: usize) -> Self {
        Self {
            items: (1..=n).map(|i| format!("Item {i:02}")).collect(),
            selected: 0,
            quit_count: 0,
        }
    }
}

impl Model for ListApp {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Down => {
                self.selected = (self.selected + 1).min(self.items.len() - 1);
                Command::None
            }
            Msg::Up => {
                self.selected = self.selected.saturating_sub(1);
                Command::None
            }
            Msg::Quit => {
                self.quit_count += 1;
                Command::Quit
            }
        }
    }

    fn view(&self, frame: &mut Frame<'_>) {
        let [list_area, status] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(frame.area());

        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, text)| {
                let marker = if i == self.selected { "> " } else { "  " };
                ListItem::new(format!("{marker}{text}"))
            })
            .collect();
        let mut state = ListState::default();
        frame.render_stateful_widget(List::new(items), list_area, &mut state);

        let bar = Paragraph::new(Text::from(format!(
            "{} / {}",
            self.selected + 1,
            self.items.len()
        )));
        frame.render_widget(bar, status);
    }

    fn handle_event(&self, event: Event) -> Option<Msg> {
        let Event::Key(key) = event else { return None };
        match key.code {
            KeyCode::Down => Some(Msg::Down),
            KeyCode::Up => Some(Msg::Up),
            KeyCode::Char('q') => Some(Msg::Quit),
            _ => None,
        }
    }
}

fn row_containing(frame: &str, needle: &str) -> Option<usize> {
    frame.lines().position(|line| line.contains(needle))
}

#[test]
fn a_script_yields_one_frame_per_key_plus_the_initial_render() {
    let mut harness = Harness::new(ListApp::new(20), 40, 10).unwrap();
    let frames = harness.run_script("Down,Down,Down").unwrap();
    assert_eq!(frames.len(), 4, "initial frame plus one per key");
}

#[test]
fn quitting_ends_the_run_without_a_final_frame() {
    let mut harness = Harness::new(ListApp::new(20), 40, 10).unwrap();
    let frames = harness.run_script("Down,q,Down,Down").unwrap();

    assert_eq!(frames.len(), 2, "initial frame, then one for Down");
    assert!(!harness.is_running());
    assert_eq!(
        harness.model().selected,
        1,
        "keys after the quit must not reach the model"
    );
}

#[test]
fn selection_and_status_bar_move_together() {
    let mut harness = Harness::new(ListApp::new(20), 40, 10).unwrap();
    let frames = harness.run_script("Down,Down,Down").unwrap();

    assert!(frames[0].contains("> Item 01"));
    assert!(frames[3].contains("> Item 04"));
    assert!(!frames[3].contains("> Item 01"));

    let first = row_containing(&frames[0], "> Item").unwrap();
    let last = row_containing(&frames[3], "> Item").unwrap();
    assert_eq!(last - first, 3, "the marker travels one row per key");

    assert!(frames[0].lines().last().unwrap().contains("1 / 20"));
    assert!(frames[3].lines().last().unwrap().contains("4 / 20"));
}

#[test]
fn selection_stops_at_both_ends() {
    let mut harness = Harness::new(ListApp::new(3), 40, 6).unwrap();
    harness.run_script("Up,Up").unwrap();
    assert_eq!(harness.model().selected, 0);

    harness.run_script("Down,Down,Down,Down").unwrap();
    assert_eq!(harness.model().selected, 2);
}

#[test]
fn frames_are_padded_to_the_screen_size() {
    let mut harness = Harness::new(ListApp::new(3), 24, 5).unwrap();
    let frame = harness.render().unwrap();
    let lines: Vec<&str> = frame.lines().collect();
    assert_eq!(lines.len(), 5);
    assert!(lines.iter().all(|l| l.chars().count() == 24), "{lines:?}");
}

#[test]
fn an_unknown_key_name_is_an_error_not_a_silent_skip() {
    let mut harness = Harness::new(ListApp::new(3), 20, 4).unwrap();
    let err = harness.run_script("Down,Wiggle").unwrap_err();
    assert!(err.to_string().contains("Wiggle"), "{err}");
}

// ------------------------------------------------------- command processing

struct CommandApp {
    log: Vec<&'static str>,
}

enum CmdMsg {
    Chain,
    Landed,
    BatchThenQuit,
}

impl Model for CommandApp {
    type Msg = CmdMsg;

    fn update(&mut self, msg: CmdMsg) -> Command<CmdMsg> {
        match msg {
            CmdMsg::Chain => {
                self.log.push("chain");
                Command::Message(CmdMsg::Landed)
            }
            CmdMsg::Landed => {
                self.log.push("landed");
                Command::None
            }
            CmdMsg::BatchThenQuit => {
                self.log.push("batch");
                Command::Batch(vec![Command::Message(CmdMsg::Landed), Command::Quit])
            }
        }
    }

    fn view(&self, frame: &mut Frame<'_>) {
        let text = Paragraph::new(Text::from(self.log.join(",")));
        frame.render_widget(text, frame.area());
    }

    fn handle_event(&self, event: Event) -> Option<CmdMsg> {
        let Event::Key(key) = event else { return None };
        match key.code {
            KeyCode::Char('c') => Some(CmdMsg::Chain),
            KeyCode::Char('b') => Some(CmdMsg::BatchThenQuit),
            _ => None,
        }
    }
}

#[test]
fn a_returned_message_is_folded_back_in() {
    let mut harness = Harness::new(CommandApp { log: vec![] }, 20, 1).unwrap();
    harness.char('c');
    assert_eq!(harness.model().log, vec!["chain", "landed"]);
}

#[test]
fn a_batch_runs_every_command_including_quit() {
    let mut harness = Harness::new(CommandApp { log: vec![] }, 20, 1).unwrap();
    harness.char('b');
    assert_eq!(harness.model().log, vec!["batch", "landed"]);
    assert!(!harness.is_running());
}

// -------------------------------------------------------------- key parsing

#[test]
fn key_names_cover_the_script_vocabulary() {
    assert_eq!(parse_key("Up"), Some(KeyCode::Up));
    assert_eq!(parse_key("Enter"), Some(KeyCode::Enter));
    assert_eq!(parse_key("Backspace"), Some(KeyCode::Backspace));
    assert_eq!(parse_key("Space"), Some(KeyCode::Char(' ')));
    assert_eq!(parse_key("F5"), Some(KeyCode::F(5)));
    assert_eq!(parse_key("+"), Some(KeyCode::Char('+')));
    assert_eq!(parse_key("é"), Some(KeyCode::Char('é')));
}

#[test]
fn key_names_reject_what_they_cannot_represent() {
    assert_eq!(parse_key("F13"), None, "only F1 through F12 exist");
    assert_eq!(parse_key("Downn"), None);
    assert_eq!(parse_key(""), None);
    assert_eq!(parse_key("down"), None, "names are case sensitive");
}

// ------------------------------------------------------------ frame_diff

#[test]
fn identical_frames_have_no_diff() {
    assert!(hawktui::testing::frame_diff("ab\ncd", "ab\ncd").is_none());
}

#[test]
fn trailing_padding_is_ignored_on_both_sides() {
    assert!(hawktui::testing::frame_diff("ab   \ncd  ", "ab\ncd").is_none());
}

#[test]
fn a_diff_names_the_first_differing_row_and_column() {
    let report = hawktui::testing::frame_diff("abc\ndef", "abc\ndXf").unwrap();
    assert!(report.contains("row 1"), "{report}");
    assert!(report.contains("column 1"), "{report}");
    assert!(report.contains("1 row(s) differ"), "{report}");
}

#[test]
fn a_height_mismatch_is_called_out() {
    let report = hawktui::testing::frame_diff("a\nb\nc", "a\nb").unwrap();
    assert!(report.contains("rendered 3 rows, expected 2"), "{report}");
}

#[test]
fn the_macro_passes_a_matching_frame() {
    let mut harness = Harness::new(ListApp::new(2), 12, 3).unwrap();
    let frame = harness.render().unwrap();
    hawktui::assert_frame!(frame, &frame.clone());
}

#[test]
#[should_panic(expected = "rendered frame does not match")]
fn the_macro_panics_on_a_mismatch() {
    hawktui::assert_frame!("abc", "xyz");
}
