//! OpenCode-style AI coding assistant TUI — built with Hawk TUI.
//!
//! Demonstrates a chat interface with:
//! - Model selector tabs
//! - Scrollable Markdown-rendered conversation
//! - Text input with history
//! - Token/cost tracking in the status bar
//! - Agent ontology integration

use hawktui::core::text::Alignment;
use hawktui::ontology::registry::OntologyRegistry;
use hawktui::prelude::*;
use hawktui::runtime::{Command, Model, Program, ProgramOptions};
use hawktui::widget::input::{Input, InputState};
use hawktui::widget::markdown::Markdown;
use hawktui::widget::scrollbar::{Scrollbar, ScrollbarOrientation, ScrollbarState};
use hawktui::widget::tabs::Tabs;

// ── Colour palette ──────────────────────────────────────────────────────────

const BORDER: Color = Color::DarkGray;
const ACCENT: Color = Color::Cyan;
const DIM: Color = Color::DarkGray;
const USER_COLOR: Color = Color::Green;
const ASSISTANT_COLOR: Color = Color::Cyan;
const SYSTEM_COLOR: Color = Color::Yellow;

// ── Domain types ────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
enum Role {
    User,
    Assistant,
    System,
}

#[derive(Clone, Debug)]
struct ChatMessage {
    role: Role,
    content: String,
}

#[derive(Clone, Debug)]
struct ModelInfo {
    name: &'static str,
    provider: &'static str,
}

const MODELS: &[ModelInfo] = &[
    ModelInfo {
        name: "claude-opus-4",
        provider: "Anthropic",
    },
    ModelInfo {
        name: "claude-sonnet-4",
        provider: "Anthropic",
    },
    ModelInfo {
        name: "gpt-4.1",
        provider: "OpenAI",
    },
    ModelInfo {
        name: "o3",
        provider: "OpenAI",
    },
    ModelInfo {
        name: "gemini-2.5-pro",
        provider: "Google",
    },
];

// ── App state ───────────────────────────────────────────────────────────────

struct App {
    // Chat
    messages: Vec<ChatMessage>,
    input: InputState,
    chat_scroll_offset: u16,

    // Model selector
    model_index: usize,

    // Stats
    total_tokens: u64,
    total_cost_cents: u64,
    message_count: u64,

    // UI state
    show_help: bool,
    input_focused: bool,
}

// ── Messages ────────────────────────────────────────────────────────────────

enum Msg {
    // Input
    InputChar(char),
    InputBackspace,
    InputDelete,
    InputLeft,
    InputRight,
    InputHome,
    InputEnd,
    Submit,

    // Chat scroll
    ScrollUp,
    ScrollDown,
    ScrollTop,
    ScrollBottom,

    // Model
    NextModel,
    PrevModel,

    // UI
    ToggleHelp,
    Quit,
}

impl App {
    fn new() -> Self {
        let welcome = ChatMessage {
            role: Role::System,
            content: "Welcome to **Hawk TUI Chat** — an OpenCode-style AI assistant TUI.\n\
                      Type a message and press **Enter** to send.\n\
                      Use **Tab** to cycle models, **?** for help."
                .into(),
        };

        Self {
            messages: vec![welcome],
            input: InputState::new(),
            chat_scroll_offset: 0,
            model_index: 0,
            total_tokens: 0,
            total_cost_cents: 0,
            message_count: 0,
            show_help: false,
            input_focused: true,
        }
    }

    fn active_model(&self) -> &ModelInfo {
        &MODELS[self.model_index]
    }

    /// Simulates an assistant response (in a real app this would call an API).
    fn simulate_reply(&self, user_msg: &str) -> String {
        let model = self.active_model();
        if user_msg.to_lowercase().contains("help") {
            format!(
                "## Available Commands\n\n\
                 - Type naturally to ask coding questions\n\
                 - `Tab` / `Shift+Tab` — cycle models\n\
                 - `Ctrl+U` / `Ctrl+D` — scroll chat\n\
                 - `?` — toggle help overlay\n\
                 - `Esc` / `Ctrl+C` — quit\n\n\
                 > Currently using **{}** via {}\n\n\
                 I can help with code review, refactoring, debugging, and more.",
                model.name, model.provider
            )
        } else if user_msg.to_lowercase().contains("hello")
            || user_msg.to_lowercase().contains("hi")
        {
            format!(
                "Hello! I'm running on **{}** ({}).\n\n\
                 How can I help you today? I can:\n\
                 - Review and explain code\n\
                 - Debug issues\n\
                 - Suggest refactors\n\
                 - Write tests\n\
                 - Answer technical questions",
                model.name, model.provider
            )
        } else if user_msg.to_lowercase().contains("rust") {
            "Great question about Rust! Here's a quick example:\n\n\
             ```rust\n\
             fn main() {\n\
                 let greeting = \"Hello from Hawk TUI!\";\n\
                 println!(\"{greeting}\");\n\
             }\n\
             ```\n\n\
             Rust's ownership system ensures memory safety at compile time \
             without a garbage collector. The borrow checker enforces that:\n\n\
             1. Each value has exactly one owner\n\
             2. References cannot outlive their referent\n\
             3. You can have either *one* mutable ref or *many* immutable refs"
                .into()
        } else {
            format!(
                "I received your message ({} chars). In a real deployment, this \
                 would be sent to **{}** for processing.\n\n\
                 This is a demo of the **Hawk TUI** TUI framework — \
                 an agentic-first terminal UI library built in Rust.",
                user_msg.len(),
                model.name
            )
        }
    }

    fn total_chat_lines(&self) -> usize {
        // Rough estimate: each message is ~3+ lines (role label + content + blank)
        self.messages
            .iter()
            .map(|m| m.content.lines().count() + 3)
            .sum()
    }
}

// ── Model impl ──────────────────────────────────────────────────────────────

impl Model for App {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) -> Command<Msg> {
        match msg {
            // ── Input editing ───────────────
            Msg::InputChar(ch) => {
                self.input.insert_char(ch);
                Command::None
            }
            Msg::InputBackspace => {
                self.input.delete_char_before();
                Command::None
            }
            Msg::InputDelete => {
                self.input.delete_char_after();
                Command::None
            }
            Msg::InputLeft => {
                self.input.move_left();
                Command::None
            }
            Msg::InputRight => {
                self.input.move_right();
                Command::None
            }
            Msg::InputHome => {
                self.input.move_start();
                Command::None
            }
            Msg::InputEnd => {
                self.input.move_end();
                Command::None
            }

            // ── Submit ──────────────────────
            Msg::Submit => {
                let text = self.input.value.trim().to_string();
                if text.is_empty() {
                    return Command::None;
                }
                self.input.clear();

                // Add user message
                self.messages.push(ChatMessage {
                    role: Role::User,
                    content: text.clone(),
                });
                self.message_count += 1;

                // Simulate reply
                let reply = self.simulate_reply(&text);
                let reply_tokens = (reply.len() as u64) / 4; // rough token estimate
                self.total_tokens += reply_tokens + (text.len() as u64 / 4);
                self.total_cost_cents += reply_tokens / 10;

                self.messages.push(ChatMessage {
                    role: Role::Assistant,
                    content: reply,
                });

                // Auto-scroll to bottom
                self.chat_scroll_offset = 0; // 0 means bottom in our rendering

                Command::None
            }

            // ── Chat scroll ─────────────────
            Msg::ScrollUp => {
                self.chat_scroll_offset = self.chat_scroll_offset.saturating_add(3);
                Command::None
            }
            Msg::ScrollDown => {
                self.chat_scroll_offset = self.chat_scroll_offset.saturating_sub(3);
                Command::None
            }
            Msg::ScrollTop => {
                let max = self.total_chat_lines() as u16;
                self.chat_scroll_offset = max;
                Command::None
            }
            Msg::ScrollBottom => {
                self.chat_scroll_offset = 0;
                Command::None
            }

            // ── Model selector ──────────────
            Msg::NextModel => {
                self.model_index = (self.model_index + 1) % MODELS.len();
                Command::None
            }
            Msg::PrevModel => {
                self.model_index = if self.model_index == 0 {
                    MODELS.len() - 1
                } else {
                    self.model_index - 1
                };
                Command::None
            }

            // ── UI ──────────────────────────
            Msg::ToggleHelp => {
                self.show_help = !self.show_help;
                Command::None
            }
            Msg::Quit => Command::Quit,
        }
    }

    fn view(&self, frame: &mut Frame<'_>) {
        let area = frame.area();

        // ┌─────────────────────────────────────────────┐
        // │ Model Tabs                                   │  3 rows
        // ├─────────────────────────────────────────────┤
        // │                                             │
        // │  Chat History (scrollable, markdown)        │  Fill
        // │                                             │
        // ├─────────────────────────────────────────────┤
        // │ > input                                     │  3 rows
        // ├─────────────────────────────────────────────┤
        // │ status bar                                  │  1 row
        // └─────────────────────────────────────────────┘

        let main = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // tabs
                Constraint::Min(5),    // chat
                Constraint::Length(3), // input
                Constraint::Length(1), // status
            ])
            .split(area);

        self.render_tabs(frame, main[0]);
        self.render_chat(frame, main[1]);
        self.render_input(frame, main[2]);
        self.render_status(frame, main[3]);

        if self.show_help {
            self.render_help_overlay(frame, area);
        }
    }

    fn handle_event(&self, event: Event) -> Option<Msg> {
        if let Event::Key(key) = event {
            // Global shortcuts (always active)
            if key.is_ctrl(KeyCode::Char('c')) {
                return Some(Msg::Quit);
            }

            // Help overlay captures all input
            if self.show_help {
                return Some(Msg::ToggleHelp); // any key dismisses
            }

            match key.code {
                KeyCode::Esc => return Some(Msg::Quit),

                // Model cycling
                KeyCode::Tab => return Some(Msg::NextModel),
                KeyCode::BackTab => return Some(Msg::PrevModel),

                // Chat scrolling
                KeyCode::Up if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Some(Msg::ScrollUp)
                }
                KeyCode::Down if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Some(Msg::ScrollDown)
                }
                KeyCode::PageUp => return Some(Msg::ScrollUp),
                KeyCode::PageDown => return Some(Msg::ScrollDown),
                KeyCode::Home if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Some(Msg::ScrollTop)
                }
                KeyCode::End if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Some(Msg::ScrollBottom)
                }

                // Input editing
                KeyCode::Enter => return Some(Msg::Submit),
                KeyCode::Backspace => return Some(Msg::InputBackspace),
                KeyCode::Delete => return Some(Msg::InputDelete),
                KeyCode::Left => return Some(Msg::InputLeft),
                KeyCode::Right => return Some(Msg::InputRight),
                KeyCode::Home => return Some(Msg::InputHome),
                KeyCode::End => return Some(Msg::InputEnd),

                KeyCode::Char('?')
                    if !key.modifiers.contains(KeyModifiers::SHIFT)
                        && self.input.value.is_empty() =>
                {
                    return Some(Msg::ToggleHelp)
                }
                KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Some(Msg::ScrollUp)
                }
                KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Some(Msg::ScrollDown)
                }
                KeyCode::Char(c) => return Some(Msg::InputChar(c)),
                _ => {}
            }
        }
        None
    }

    fn register_ontology(&self, registry: &mut OntologyRegistry) {
        registry.register::<Block>();
        registry.register::<Paragraph>();
        registry.register::<hawktui::widget::input::Input>();
        registry.register::<hawktui::widget::tabs::Tabs>();
        registry.register::<hawktui::widget::markdown::Markdown>();
        registry.register::<hawktui::widget::scrollbar::Scrollbar>();
    }
}

// ── View helpers ────────────────────────────────────────────────────────────

impl App {
    fn render_tabs(&self, frame: &mut Frame<'_>, area: Rect) {
        let titles: Vec<String> = MODELS.iter().map(|m| format!(" {} ", m.name)).collect();

        let tabs = Tabs::new(titles)
            .block(
                Block::default()
                    .title(" Hawk TUI Chat ")
                    .title_alignment(Alignment::Left)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(BORDER)),
            )
            .select(self.model_index)
            .highlight_style(Style::default().fg(Color::Black).bg(ACCENT).bold())
            .style(Style::default().fg(DIM))
            .divider("│");

        frame.render_widget(tabs, area);
    }

    fn render_chat(&self, frame: &mut Frame<'_>, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(BORDER));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.is_empty() || self.messages.is_empty() {
            return;
        }

        // Build chat lines from messages
        let mut all_lines: Vec<Line> = Vec::new();

        for msg in &self.messages {
            // Role label
            let (label, label_style) = match msg.role {
                Role::User => ("  You".to_string(), Style::default().fg(USER_COLOR).bold()),
                Role::Assistant => (
                    format!("  {}", self.active_model().name),
                    Style::default().fg(ASSISTANT_COLOR).bold(),
                ),
                Role::System => (
                    "  System".to_string(),
                    Style::default().fg(SYSTEM_COLOR).bold(),
                ),
            };
            all_lines.push(Line::from(vec![Span::styled(label, label_style)]));

            // Message content — styled per role
            let content_style = match msg.role {
                Role::User => Style::default().fg(Color::White),
                Role::Assistant => Style::default().fg(Color::Gray),
                Role::System => Style::default().fg(SYSTEM_COLOR),
            };
            for text_line in msg.content.lines() {
                all_lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(text_line.to_string(), content_style),
                ]));
            }

            // Blank separator
            all_lines.push(Line::raw(""));
        }

        // Apply scroll offset from bottom
        let visible_height = inner.height as usize;
        let total = all_lines.len();
        let scroll_from_bottom = self.chat_scroll_offset as usize;

        let end = total.saturating_sub(scroll_from_bottom);
        let start = end.saturating_sub(visible_height);

        let visible: Vec<Line> = all_lines[start..end].to_vec();

        // Render visible lines
        for (i, line) in visible.iter().enumerate() {
            let y = inner.y + i as u16;
            if y >= inner.bottom() {
                break;
            }
            let mut col = 0u16;
            for span in &line.spans {
                if col >= inner.width {
                    break;
                }
                let written = frame.buffer_mut().set_string_truncated(
                    inner.x + col,
                    y,
                    &span.content,
                    inner.width - col,
                    span.style,
                );
                col += written;
            }
        }

        // Scrollbar
        if total > visible_height {
            let mut scroll_state = ScrollbarState::new(total, visible_height).position(start);
            let scrollbar =
                Scrollbar::new(ScrollbarOrientation::Vertical).style(Style::default().fg(DIM));
            let scroll_area = Rect::new(
                area.right().saturating_sub(1),
                area.y + 1,
                1,
                area.height.saturating_sub(2),
            );
            frame.render_stateful_widget(scrollbar, scroll_area, &mut scroll_state);
        }
    }

    fn render_input(&self, frame: &mut Frame<'_>, area: Rect) {
        let model = self.active_model();
        let title = format!(" {} ({}) ", model.name, model.provider);

        let input_widget = Input::new()
            .block(
                Block::default()
                    .title(title)
                    .title_alignment(Alignment::Right)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(if self.input_focused {
                        Style::default().fg(ACCENT)
                    } else {
                        Style::default().fg(BORDER)
                    }),
            )
            .style(Style::default().fg(Color::White))
            .placeholder("Type a message... (? for help)")
            .placeholder_style(Style::default().fg(DIM));

        let mut input_state = self.input.clone();
        frame.render_stateful_widget(input_widget, area, &mut input_state);
    }

    fn render_status(&self, frame: &mut Frame<'_>, area: Rect) {
        let cost_display = if self.total_cost_cents > 100 {
            format!("${:.2}", self.total_cost_cents as f64 / 100.0)
        } else {
            format!("{}¢", self.total_cost_cents)
        };

        let left = vec![
            Span::styled(" Tab", Style::default().fg(ACCENT).bold()),
            Span::styled(" Model  ", Style::default().fg(DIM)),
            Span::styled("Enter", Style::default().fg(ACCENT).bold()),
            Span::styled(" Send  ", Style::default().fg(DIM)),
            Span::styled("Ctrl+U/D", Style::default().fg(ACCENT).bold()),
            Span::styled(" Scroll  ", Style::default().fg(DIM)),
            Span::styled("?", Style::default().fg(ACCENT).bold()),
            Span::styled(" Help  ", Style::default().fg(DIM)),
            Span::styled("Esc", Style::default().fg(ACCENT).bold()),
            Span::styled(" Quit", Style::default().fg(DIM)),
        ];

        let right = format!(
            "msgs: {}  tokens: {}  cost: {} ",
            self.message_count, self.total_tokens, cost_display
        );

        // Left-aligned shortcuts
        let status_line = Line::from(left);
        let left_para = Paragraph::new(status_line);
        frame.render_widget(left_para, area);

        // Right-aligned stats
        let right_width = right.len() as u16;
        if area.width > right_width {
            let right_area = Rect::new(area.x + area.width - right_width, area.y, right_width, 1);
            let right_para = Paragraph::new(Span::styled(right, Style::default().fg(DIM)));
            frame.render_widget(right_para, right_area);
        }
    }

    fn render_help_overlay(&self, frame: &mut Frame<'_>, area: Rect) {
        // Dimmed background + centered help box
        let overlay_style = Style::default().fg(DIM).bg(Color::Black);
        frame.buffer_mut().set_style(area, overlay_style);

        let width = 50u16.min(area.width.saturating_sub(4));
        let height = 16u16.min(area.height.saturating_sub(4));
        let x = area.x + (area.width.saturating_sub(width)) / 2;
        let y = area.y + (area.height.saturating_sub(height)) / 2;
        let help_area = Rect::new(x, y, width, height);

        let help_text = "\
# Keyboard Shortcuts

**Tab** / **Shift+Tab** — Cycle AI model
**Enter** — Send message
**Ctrl+U** — Scroll up
**Ctrl+D** — Scroll down
**PageUp** / **PageDown** — Scroll
**Ctrl+Home** — Scroll to top
**Ctrl+End** — Scroll to bottom
**Ctrl+C** / **Esc** — Quit
**?** — Toggle this help

*Press any key to dismiss*";

        let md = Markdown::new(help_text)
            .block(
                Block::default()
                    .title(" Help ")
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::default().fg(ACCENT))
                    .style(Style::default().bg(Color::Black)),
            )
            .style(Style::default().fg(Color::White).bg(Color::Black));

        frame.render_widget(md, help_area);
    }
}

// ── Main ────────────────────────────────────────────────────────────────────

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
