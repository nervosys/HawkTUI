//! # Hawk TUI — An Agentic-First TUI Framework
//!
//! Hawk TUI is a terminal user interface framework built from the ground up for
//! AI agent discoverability. Every widget, layout, and interaction carries
//! structured metadata that agents can introspect at runtime, enabling
//! programmatic discovery, inspection, and control of any TUI application.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │  Agent (AI)              Human (keyboard/mouse)         │
//! │       │                        │                        │
//! │       ▼                        ▼                        │
//! │  ┌─────────┐            ┌───────────┐                   │
//! │  │  agent   │            │   event    │                  │
//! │  │ protocol │            │  system    │                  │
//! │  └────┬─────┘            └─────┬─────┘                  │
//! │       └──────────┬─────────────┘                        │
//! │                  ▼                                      │
//! │            ┌──────────┐                                 │
//! │            │ runtime  │  Elm Architecture                │
//! │            │ (Model → │  update → Command → view)       │
//! │            └────┬─────┘                                 │
//! │                 ▼                                       │
//! │  ┌──────────────────────────────┐                       │
//! │  │ widget layer + ontology      │                       │
//! │  │ (every widget is             │                       │
//! │  │  Discoverable + has schema)  │                       │
//! │  └────────────┬─────────────────┘                       │
//! │               ▼                                         │
//! │  ┌─────────────────────────┐                            │
//! │  │ terminal (double-buffer │                            │
//! │  │  differential render)   │                            │
//! │  └──────────┬──────────────┘                            │
//! │             ▼                                           │
//! │        ┌──────────┐                                     │
//! │        │ backend  │  crossterm / test                   │
//! │        └──────────┘                                     │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Module Guide
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`core`] | Primitives — [`Buffer`](core::buffer::Buffer), [`Cell`](core::cell::Cell), [`Rect`](core::rect::Rect), [`Style`](core::style::Style), [`Text`](core::text::Text) |
//! | [`widget`] | Visual components — all implement [`Widget`](widget::Widget) and [`Discoverable`](ontology::Discoverable) |
//! | [`layout`] | Constraint-based space allocation ([`Layout`](layout::Layout), [`Constraint`](layout::Constraint)) |
//! | [`ontology`] | Agent discoverability — schemas, capabilities, actions, semantic roles |
//! | [`agent`] | JSON Lines RPC protocol for AI agents to drive apps headlessly |
//! | [`runtime`] | Elm architecture event loop — [`Model`](runtime::Model), [`Command`](runtime::Command), [`Program`](runtime::Program) |
//! | [`event`] | Keyboard, mouse, resize, paste, focus, and tick events |
//! | [`terminal`] | Double-buffered differential rendering via [`Frame`](terminal::Frame) |
//! | [`backend`] | Backend trait + crossterm and test implementations |
//! | [`animation`] | Tweens, springs, easing functions, and timelines |
//! | [`focus`] | Focus ring for ordered widget navigation |
//! | [`overlay`] | Modal overlay stack with focus capture |
//!
//! ## Agent Protocol
//!
//! Hawk TUI applications can be controlled entirely by AI agents via the
//! [`agent`] module's JSON Lines protocol over stdin/stdout:
//!
//! ```text
//! Agent → stdin:  {"id":"1","request":{"type":"ping"}}
//! App   → stdout: {"success":true,"id":"1","data":{"status":"pong"}}
//!
//! Agent → stdin:  {"request":{"type":"query_ontology"}}
//! App   → stdout: {"success":true,"data":{...widget catalog...}}
//!
//! Agent → stdin:  {"request":{"type":"execute_action","agent_id":"editor-1","action":"insert_text","params":{"text":"hello"}}}
//! App   → stdout: {"success":true}
//! ```
//!
//! See [`agent::protocol`] for the full message types and the
//! [`agent::rpc`] module for the transport implementation.
//!
//! ## Feature Flags
//!
//! | Flag | Default | Description |
//! |------|---------|-------------|
//! | `crossterm` | **yes** | Crossterm terminal backend (disable for headless/agent-only) |
//! | `bin` | no | Enables `hawktui-server` and `hawktui-demo` binaries (pulls in `tracing`) |
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use hawktui::prelude::*;
//!
//! struct App { count: i32 }
//!
//! #[derive(Debug)]
//! enum Msg { Increment, Decrement, Quit }
//!
//! impl Model for App {
//!     type Msg = Msg;
//!
//!     fn update(&mut self, msg: Msg) -> Command<Msg> {
//!         match msg {
//!             Msg::Increment => self.count += 1,
//!             Msg::Decrement => self.count -= 1,
//!             Msg::Quit => return Command::Quit,
//!         }
//!         Command::None
//!     }
//!
//!     fn view(&self, frame: &mut Frame) {
//!         let area = frame.area();
//!         let text = Text::from(format!("Count: {}", self.count));
//!         frame.render_widget(Paragraph::new(text), area);
//!     }
//!
//!     fn handle_event(&self, event: Event) -> Option<Msg> {
//!         None
//!     }
//! }
//! ```

pub mod agent;
pub mod animation;
pub mod backend;
pub mod core;
pub mod error;
pub mod event;
pub mod focus;
pub mod layout;
pub mod ontology;
pub mod overlay;
pub mod runtime;
pub mod terminal;
pub mod testing;
pub mod theme;
#[doc(hidden)]
pub mod util;
pub mod widget;

/// Prelude: import everything you need for a typical hawktui application.
pub mod prelude {
    pub use crate::animation::{Animation, Easing, Spring, Timeline, Tween};
    #[cfg(feature = "crossterm")]
    pub use crate::backend::crossterm_backend::CrosstermBackend;
    pub use crate::backend::Backend;
    pub use crate::core::buffer::Buffer;
    pub use crate::core::cell::Cell;
    pub use crate::core::rect::Rect;
    pub use crate::core::style::{Color, Modifier, Style, Stylize};
    pub use crate::core::text::{Line, Span, Text};
    pub use crate::error::{Error, Result};
    pub use crate::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent};
    pub use crate::layout::{Alignment, Constraint, Direction, Flex, Layout, Margin};
    pub use crate::ontology::{
        AgentAction, AgentCapability, Discoverable, OntologyRegistry, SemanticRole, WidgetSchema,
    };
    pub use crate::runtime::{Command, Model, Program};
    pub use crate::terminal::Frame;
    pub use crate::theme::{Theme, ThemeToken};
    pub use crate::widget::block::{Block, BorderType, Borders};
    pub use crate::widget::gauge::Gauge;
    pub use crate::widget::list::{List, ListItem, ListState};
    pub use crate::widget::paragraph::Paragraph;
    pub use crate::widget::scrollbar::{Scrollbar, ScrollbarState};
    pub use crate::widget::tabs::Tabs;
    pub use crate::widget::{StatefulWidget, Widget};
}
