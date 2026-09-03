//! The minimal complete Hawk TUI program.
//!
//! This file is served verbatim by `hawktui-ontology skeleton` and by the
//! `program_skeleton` MCP tool, via `include_str!`. It is a compiled example,
//! so the skeleton an agent is handed cannot drift from one that builds.
//!
//! It shows the four things the transcripts say agents go to the source for:
//! the `Model` trait's three required methods, how a `Program` is started, how
//! a stateful widget is rendered with its state, and how to drive the whole
//! thing headlessly in a test.

use hawktui::prelude::*;
use hawktui::runtime::{Command, Model, Program, ProgramOptions};
use hawktui::widget::list::{List, ListItem, ListState};

/// Application state. Plain data — no framework types required.
struct App {
    items: Vec<String>,
    selected: usize,
}

/// One variant per thing that can happen.
enum Msg {
    Down,
    Up,
    Quit,
}

impl Model for App {
    type Msg = Msg;

    /// 1. Fold a message into the state. Return `Command::Quit` to stop.
    fn update(&mut self, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Down => {
                self.selected = (self.selected + 1).min(self.items.len().saturating_sub(1));
                Command::None
            }
            Msg::Up => {
                self.selected = self.selected.saturating_sub(1);
                Command::None
            }
            Msg::Quit => Command::Quit,
        }
    }

    /// 2. Draw. Takes `&self`, so state that a widget mutates lives here and is
    ///    passed in, not stored in the widget.
    fn view(&self, frame: &mut Frame<'_>) {
        let [body, status] =
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

        // List is a StatefulWidget: it needs a companion ListState, and it is
        // drawn with render_stateful_widget rather than render_widget.
        let mut state = ListState::default();
        let list = List::new(items).block(Block::default().title("Items").borders(Borders::ALL));
        frame.render_stateful_widget(list, body, &mut state);

        let bar = Paragraph::new(Text::from(format!(
            "{} / {}",
            self.selected + 1,
            self.items.len()
        )));
        frame.render_widget(bar, status);
    }

    /// 3. Turn a terminal event into a message, or `None` to ignore it.
    fn handle_event(&self, event: Event) -> Option<Msg> {
        let Event::Key(key) = event else { return None };
        match key.code {
            KeyCode::Down => Some(Msg::Down),
            KeyCode::Up => Some(Msg::Up),
            KeyCode::Char('q') | KeyCode::Esc => Some(Msg::Quit),
            _ => None,
        }
    }
}

fn app() -> App {
    App {
        items: vec!["alpha".into(), "beta".into(), "gamma".into()],
        selected: 0,
    }
}

fn main() -> std::io::Result<()> {
    // ProgramOptions::default() takes over the terminal: raw mode, alternate
    // screen, mouse capture, and a 16 ms tick. Turn them off for anything
    // headless or with redirected output.
    let options = ProgramOptions {
        tick_rate: None,
        ..Default::default()
    };
    let backend = CrosstermBackend::new(std::io::stdout());
    Program::new(app(), backend)?.with_options(options).run()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hawktui::testing::Harness;

    /// Drive the program without a terminal: apply keys, read back each frame
    /// as plain text.
    #[test]
    fn selection_moves_and_q_quits() {
        let mut harness = Harness::new(app(), 30, 6).unwrap();
        let frames = harness.run_script("Down,Down,q").unwrap();

        // One frame for the initial render, one per key — and none for `q`,
        // because quitting ends the run.
        assert_eq!(frames.len(), 3);
        assert!(frames[0].contains("> alpha"));
        assert!(frames[2].contains("> gamma"));
        assert!(!harness.is_running());
    }
}
