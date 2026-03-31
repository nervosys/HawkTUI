//! Minimal hello-world example for louie.
//!
//! Displays a styled greeting and quits on 'q' or Esc.

use louie::prelude::*;
use louie::runtime::{Command, Model, Program, ProgramOptions};

struct App;

enum Msg {
    Quit,
}

impl Model for App {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Quit => Command::Quit,
        }
    }

    fn view(&self, frame: &mut Frame<'_>) {
        let area = frame.area();

        let block = Block::default()
            .title("Hello Louie")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let greeting = Paragraph::new(Text::from(vec![
            Line::from(Span::styled(
                "Welcome to Louie!",
                Style::default().bold().fg(Color::Cyan),
            )),
            Line::from(""),
            Line::from("An agentic-first TUI framework for Rust."),
            Line::from(""),
            Line::from(Span::styled("Press 'q' to quit.", Style::default().dim())),
        ]))
        .block(block)
        .alignment(Alignment::Center);

        frame.render_widget(greeting, area);
    }

    fn handle_event(&self, event: Event) -> Option<Msg> {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Some(Msg::Quit),
                _ => {}
            }
        }
        None
    }
}

fn main() -> std::io::Result<()> {
    let backend = CrosstermBackend::new(std::io::stdout());
    let options = ProgramOptions {
        tick_rate: None, // No animation needed
        ..Default::default()
    };
    Program::new(App, backend)?.with_options(options).run()?;
    Ok(())
}
