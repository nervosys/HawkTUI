# Louie: the TUI framework for agentic AI

[![CI](https://github.com/nervosys/Louie/actions/workflows/ci.yml/badge.svg)](https://github.com/nervosys/Louie/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/louie.svg)](https://crates.io/crates/louie)
[![docs.rs](https://docs.rs/louie/badge.svg)](https://docs.rs/louie)
[![MSRV](https://img.shields.io/badge/MSRV-1.80-blue.svg)](https://releases.rs/docs/1.80.0/)
[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

**An agentic-first TUI framework in Rust with a complete ontology for agent discoverability.**

Louie combines the best of modern TUI frameworks (ratatui, bubbletea, ink, etc) with a structured metadata layer that lets AI agents discover, inspect, and interact with every widget in your application — no hardcoded assumptions, no trial-and-error.

## Why Louie?

Traditional TUI frameworks are built for humans. Louie is built for both:

- **For humans**: Elm architecture, immediate-mode rendering, animation system, rich widget set
- **For agents**: Every widget exposes its schema, capabilities, actions, and semantic role through a typed ontology

An agent connecting to a Louie app can ask: *"What widgets exist? What can I click? What text fields accept input? What actions are available?"* — and get structured JSON answers.

## Installation

Add Louie to your `Cargo.toml`:

```sh
cargo add louie
```

Or add it manually:

```toml
[dependencies]
louie = "1"
```

**Minimum supported Rust version:** 1.80

## Architecture

```
┌──────────────────────────────────────────────────┐
│                   Runtime (Elm)                  │
│         Model → Update → View → Render           │
├──────────────────────────────────────────────────┤
│  Agent Protocol  │  Ontology     │  Animation    │
│  ├ RPC Transport │  ├ Schema     │  ├ Easing     │
│  ├ HeadlessDriver│  ├ Capability │  ├ Tween      │
│  ├ AgentSession  │  ├ Action     │  ├ Spring     │
│  └ Protocol      │  └ Registry   │  └ Timeline   │
├──────────────────────────────────────────────────┤
│  Widgets          │  Focus & Overlay             │
│  ├ Block          │  ├ FocusManager              │
│  ├ Paragraph      │  ├ OverlayStack              │
│  ├ List           │  └ ModalBox                  │
│  ├ Tabs           ├──────────────────────────────┤
│  ├ Gauge          │  Layout                      │
│  ├ LineGauge      │  ├ Constraint solver         │
│  ├ Input          │  ├ Direction (V/H)           │
│  ├ Table          │  └ Flex distribution         │
│  ├ Editor         ├──────────────────────────────┤
│  ├ Markdown       │  Text Engine                 │
│  ├ SelectList     │  ├ Word wrap                 │
│  ├ Loader         │  ├ Char wrap                 │
│  ├ Sparkline      │  └ Line truncation           │
│  ├ Scrollbar      ├──────────────────────────────┤
│  ├ Canvas         │  Utilities                   │
│  ├ BarChart       │  ├ Fuzzy matching            │
│  ├ Chart          │  └ Undo stack                │
│  ├ Image          ├──────────────────────────────┤
│  ├ Calendar       │  Terminal                    │
│  └ SettingsList   │  └ Synchronized output       │
├──────────────────────────────────────────────────┤
│  Core: Buffer, Cell, Style, Text, Reflow, Rect   │
├──────────────────────────────────────────────────┤
│  Backend: Crossterm │ TestBackend                │
└──────────────────────────────────────────────────┘
```

### Elm Architecture

Louie uses **The Elm Architecture** (TEA), inspired by bubbletea:

```rust
pub trait Model: Sized {
    type Msg: Send + 'static;

    fn update(&mut self, msg: Self::Msg) -> Command<Self::Msg>;
    fn view(&self, frame: &mut Frame<'_>);
    fn handle_event(&self, event: Event) -> Option<Self::Msg>;
}
```

Your application state is a plain struct. Events produce messages, messages update state, state renders to a frame. No shared mutability, no callbacks — pure data flow.

### Double-Buffered Differential Rendering

Like ratatui, Louie maintains two buffers and only writes the cells that changed between frames to the terminal, minimizing I/O overhead.

## Ontology System

Every widget implements the `Discoverable` trait:

```rust
pub trait Discoverable {
    fn schema() -> WidgetSchema;         // Type name, properties, constraints
    fn capabilities(&self) -> Vec<AgentCapability>;  // What it can do
    fn actions(&self) -> Vec<AgentAction>;           // Named operations
    fn semantic_role(&self) -> SemanticRole;         // Purpose category
    fn agent_state(&self) -> serde_json::Value;      // Current state as JSON
    fn execute_action(&mut self, action: &str, params: &serde_json::Value) -> Result<serde_json::Value, String>;
}
```

### Widget Schema

```json
{
  "name": "Input",
  "description": "A text input field with cursor management.",
  "default_role": "TextInput",
  "properties": [
    {
      "name": "placeholder",
      "description": "Placeholder text shown when empty.",
      "property_type": "String",
      "required": false
    }
  ],
  "tags": ["input", "text", "form", "edit"]
}
```

### Capabilities

18 capability types including `Focusable`, `Clickable`, `Scrollable`, `TextInput`, `Selectable`, `RangeEditable`, `Sortable`, `Searchable`, `HasKeyBindings`, and more.

### Ontology Registry

```rust
let mut registry = OntologyRegistry::new();
registry.register::<Block>();
registry.register::<Paragraph>();
registry.register::<Input>();

// Search by semantic role
let inputs = registry.find_by_role(SemanticRole::TextInput);

// Full JSON catalog
let catalog = registry.export_catalog();
```

## Widget Set

| Widget                | Description                              | Agent Capabilities                          |
| --------------------- | ---------------------------------------- | ------------------------------------------- |
| **Block**             | Container with borders and title         | Focusable                                   |
| **Paragraph**         | Styled text with wrapping and scrolling  | Scrollable                                  |
| **List**              | Selectable list with highlight           | Focusable, Scrollable, Selectable           |
| **Tabs**              | Tab bar navigation                       | Focusable, Selectable                       |
| **Gauge**             | Progress bar (ratio/percentage)          | RangeEditable                               |
| **Input**             | Single-line text input with cursor       | Focusable, TextInput                        |
| **Editor**            | Multi-line text editor with line numbers | Focusable, TextInput, Scrollable, Copyable  |
| **Table**             | Data table with columns and sorting      | Focusable, Scrollable, Selectable, Sortable |
| **Markdown**          | Markdown renderer (headings, code, bold) | Scrollable                                  |
| **SelectList**        | Interactive single/multi-select list     | Focusable, Selectable, Searchable           |
| **Loader**            | Animated spinner with message            | Animated                                    |
| **Sparkline**         | Inline data trend chart                  | —                                           |
| **Scrollbar**         | Scrollbar indicator                      | Scrollable                                  |
| **Canvas**            | Braille-resolution drawing surface       | —                                           |
| **ModalBox**          | Centered modal overlay with dimmed bg    | Focusable (captures focus)                  |
| **BarChart**          | Grouped bar chart (vertical/horizontal)  | —                                           |
| **Chart**             | XY line/scatter plot with braille dots   | —                                           |
| **Image**             | Inline image (Kitty/iTerm2/fallback)     | —                                           |
| **SettingsList**      | Key-value settings with cycling values   | Focusable, Selectable                       |
| **CancellableLoader** | Loader with cancel action                | Animated                                    |
| **LineGauge**         | Thin single-line progress bar            | RangeEditable                               |
| **Calendar**          | Month-view calendar grid with highlights | —                                           |

## Quick Start

```rust
use louie::prelude::*;
use louie::runtime::{Command, Model, Program};

struct App;

#[derive(Debug)]
enum Msg { Quit }

impl Model for App {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Quit => Command::Quit,
        }
    }

    fn view(&self, frame: &mut Frame<'_>) {
        let greeting = Paragraph::new("Hello, Louie!")
            .block(Block::default().title("Demo").borders(Borders::ALL));
        frame.render_widget(greeting, frame.area());
    }

    fn handle_event(&self, event: Event) -> Option<Msg> {
        if let Event::Key(key) = event {
            if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                return Some(Msg::Quit);
            }
        }
        None
    }
}

fn main() -> std::io::Result<()> {
    let backend = CrosstermBackend::new(std::io::stdout());
    Program::new(App, backend)?.run()
}
```

## Examples

```sh
cargo run --example hello         # Minimal greeting
cargo run --example counter       # Increment/decrement with animated gauge
cargo run --example agent_demo    # Browse widget ontology schemas
cargo run --example agent_rpc     # Headless RPC server (JSON Lines on stdin/stdout)
cargo run --example opencode      # OpenCode-style AI chat assistant
cargo run --example lazygit       # Lazygit-style Git client
cargo run --example btop          # btop-style system resource monitor
```

## Agent Protocol (louie-server)

Louie ships a standalone headless server that AI agents can spawn and control via JSON Lines on stdin/stdout:

```sh
# Build
cargo build --release --bin louie-server

# Test connectivity
echo '{"type":"ping"}' | ./target/release/louie-server

# Discover all widget types
echo '{"type":"query_ontology"}' | ./target/release/louie-server

# Run the interactive demo
python3 scripts/louie-demo.py
```

See [docs/agent-protocol.md](docs/agent-protocol.md) for the full protocol specification, and [docs/agent-integration.md](docs/agent-integration.md) for integration guides (Python, TypeScript, Rust).

## Feature Flags

| Feature     | Default | Description                                                           |
| ----------- | ------- | --------------------------------------------------------------------- |
| `crossterm` | ✓       | Crossterm terminal backend (disable for headless / agent-only)        |
| `bin`       |         | Enables `louie-server` and `louie-demo` binaries (pulls in `tracing`) |

## Animation System

25 easing functions, spring physics, and timeline sequencing:

```rust
use louie::animation::{Tween, Easing, Spring, Timeline};
use std::time::Duration;

let tween = Tween::new(0.0, 1.0, Duration::from_millis(300), Easing::EaseInOutCubic);
let spring = Spring::new(0.0, 1.0, 170.0, 26.0);  // stiffness, damping
```

## Comparison

| Feature                | Louie | ratatui        | bubbletea | ink      | pi-tui (OpenCode) | OpenTUI  |
| ---------------------- | ----- | -------------- | --------- | -------- | ----------------- | -------- |
| Language               | Rust  | Rust           | Go        | JS/React | TypeScript        | Python   |
| Architecture           | Elm   | Immediate-mode | Elm       | React    | Immediate-mode    | Elm-like |
| Agent ontology         | ✓     | —              | —         | —        | —                 | —        |
| Agent protocol (RPC)   | ✓     | —              | —         | —        | ✓ (internal)      | —        |
| Widget schema export   | ✓     | —              | —         | —        | —                 | —        |
| Headless driver        | ✓     | —              | —         | —        | —                 | —        |
| Focus management       | ✓     | —              | ✓         | ✓        | ✓                 | ✓        |
| Overlay / modal system | ✓     | —              | —         | —        | ✓                 | —        |
| Clickable regions      | ✓     | —              | —         | ✓        | —                 | —        |
| Animation system       | ✓     | —              | —         | —        | —                 | —        |
| Markdown widget        | ✓     | —              | —         | ✓        | ✓                 | —        |
| Code editor widget     | ✓     | —              | —         | —        | ✓                 | —        |
| Bar/line/scatter chart | ✓     | ✓ (Chart)      | —         | —        | —                 | —        |
| Terminal image support | ✓     | —              | —         | —        | ✓                 | —        |
| Settings list widget   | ✓     | —              | —         | —        | ✓                 | —        |
| Fuzzy matching         | ✓     | —              | —         | —        | ✓                 | —        |
| Theme system           | ✓     | —              | —         | ✓        | ✓                 | —        |
| Text reflow/word-wrap  | ✓     | ✓              | —         | ✓        | —                 | —        |
| Calendar widget        | ✓     | ✓ (ext)        | —         | —        | —                 | —        |
| Line gauge             | ✓     | ✓              | —         | —        | —                 | —        |
| Block title alignment  | ✓     | ✓              | —         | —        | —                 | —        |
| List direction (B↔T)   | ✓     | ✓              | —         | —        | —                 | —        |
| Synchronized output    | ✓     | ✓              | —         | —        | —                 | —        |

## License

Louie is dual-licensed:

- **Open source**: [GNU Affero General Public License v3.0 (AGPLv3)](https://www.gnu.org/licenses/agpl-3.0.html) — free for open-source projects that comply with AGPLv3 terms, including the requirement to release source code of derivative works and network-accessible services.
- **Commercial**: A proprietary commercial license is available for organizations that cannot or prefer not to comply with AGPLv3. Contact [NERVOSYS](https://nervosys.ai/) for commercial licensing inquiries.
