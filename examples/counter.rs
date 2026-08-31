//! Counter example demonstrating the Elm architecture in hawktui.
//!
//! Increment/decrement with arrow keys, animated gauge, quit with 'q'.

use std::time::Duration;

use hawktui::prelude::*;
use hawktui::runtime::{Command, Model, Program};
use hawktui::widget::gauge::Gauge;

struct App {
    count: i64,
    gauge_ratio: f64,
}

enum Msg {
    Increment,
    Decrement,
    Reset,
    Tick,
    Quit,
}

impl Model for App {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Increment => {
                self.count += 1;
                self.gauge_ratio = (self.count as f64 / 100.0).clamp(0.0, 1.0);
                Command::None
            }
            Msg::Decrement => {
                self.count -= 1;
                self.gauge_ratio = (self.count as f64 / 100.0).clamp(0.0, 1.0);
                Command::None
            }
            Msg::Reset => {
                self.count = 0;
                self.gauge_ratio = 0.0;
                Command::None
            }
            Msg::Tick => {
                let target = (self.count as f64 / 100.0).clamp(0.0, 1.0);
                self.gauge_ratio += (target - self.gauge_ratio) * 0.1;
                Command::None
            }
            Msg::Quit => Command::Quit,
        }
    }

    fn view(&self, frame: &mut Frame<'_>) {
        let area = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area);

        // Title
        let title = Paragraph::new(format!("Counter: {}", self.count))
            .block(
                Block::default()
                    .title("Hawk TUI Counter")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .alignment(Alignment::Center)
            .style(Style::default().bold().fg(Color::Yellow));
        frame.render_widget(title, chunks[0]);

        // Gauge
        let gauge = Gauge::new()
            .ratio(self.gauge_ratio)
            .block(Block::default().title("Progress").borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Green));
        frame.render_widget(gauge, chunks[1]);

        // Instructions
        let help = Paragraph::new(Text::from(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("↑/k", Style::default().bold()),
                Span::raw(" increment  "),
                Span::styled("↓/j", Style::default().bold()),
                Span::raw(" decrement  "),
                Span::styled("r", Style::default().bold()),
                Span::raw(" reset  "),
                Span::styled("q", Style::default().bold()),
                Span::raw(" quit"),
            ]),
        ]))
        .block(Block::default().borders(Borders::ALL).title("Controls"))
        .alignment(Alignment::Center);
        frame.render_widget(help, chunks[2]);
    }

    fn handle_event(&self, event: Event) -> Option<Msg> {
        match event {
            Event::Key(KeyEvent { code, .. }) => match code {
                KeyCode::Up | KeyCode::Char('k') => Some(Msg::Increment),
                KeyCode::Down | KeyCode::Char('j') => Some(Msg::Decrement),
                KeyCode::Char('r') => Some(Msg::Reset),
                KeyCode::Char('q') | KeyCode::Esc => Some(Msg::Quit),
                _ => None,
            },
            Event::Tick => Some(Msg::Tick),
            _ => None,
        }
    }

    fn init(&self) -> Command<Msg> {
        Command::SetTickRate(Duration::from_millis(16))
    }
}

fn main() -> std::io::Result<()> {
    let app = App {
        count: 0,
        gauge_ratio: 0.0,
    };
    let backend = CrosstermBackend::new(std::io::stdout());
    Program::new(app, backend)?.run()?;
    Ok(())
}
