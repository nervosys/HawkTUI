# Hawk TUI: the TUI framework for agentic AI

[![CI](https://github.com/nervosys/HawkTUI/actions/workflows/ci.yml/badge.svg)](https://github.com/nervosys/HawkTUI/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/hawktui.svg)](https://crates.io/crates/hawktui)
[![docs.rs](https://docs.rs/hawktui/badge.svg)](https://docs.rs/hawktui)
[![MSRV](https://img.shields.io/badge/MSRV-1.80-blue.svg)](https://releases.rs/docs/1.80.0/)
[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

**A rendering engine measurably faster than ratatui — and the only TUI framework with a complete ontology for agent discoverability.**

Hawk TUI combines the best of modern TUI frameworks (ratatui, bubbletea, ink, etc) with a structured metadata layer that lets AI agents discover, inspect, and interact with every widget in your application — no hardcoded assumptions, no trial-and-error.

Speed is not a side effect here: on an identical full redraw loop Hawk TUI sustains **4.9× the frames per second of ratatui** at 91 % of the memory, and it is the fastest of the three frameworks in all sixteen measured workloads — by 1.6× to 15×. See [Performance](#performance) for the numbers and how to reproduce them.

## Why Hawk TUI?

Traditional TUI frameworks are built for humans. Hawk TUI is built for both:

- **For humans**: Elm architecture, immediate-mode rendering, animation system, rich widget set
- **For agents**: Every widget exposes its schema, capabilities, actions, and semantic role through a typed ontology
- **For both**: A rendering engine with no allocator traffic on the hot path — a `Copy`, 24-byte cell, a flat buffer diff, and a hand-written escape-sequence encoder

An agent connecting to a Hawk TUI app can ask: *"What widgets exist? What can I click? What text fields accept input? What actions are available?"* — and get structured JSON answers.

## Installation

```sh
cargo add hawktui
```

Or add it manually:

```toml
[dependencies]
hawktui = "1"
```

For headless / agent-only builds, drop the terminal backend:

```toml
[dependencies]
hawktui = { version = "1", default-features = false }
```

**Minimum supported Rust version:** 1.80

### Migrating from `louietui`

1.0.0 was published to crates.io as
[`louietui`](https://crates.io/crates/louietui), with the import name `louie`
and the binaries `louie-server` and `louie-demo`. The move is a rename plus the
API changes listed under [Unreleased](CHANGELOG.md):

```diff
-louietui = "1"
+hawktui = "1"
```

```diff
-use louie::prelude::*;
+use hawktui::prelude::*;
```

`cell.symbol` is now a `Symbol` rather than a string — call
`cell.symbol.as_str()` or `cell.symbol()`; `Layout::split` returns
`Rc<[Rect]>` instead of `Vec<Rect>` (indexing and iteration are unchanged, so
most call sites need no edit, and `Layout::solve` is the uncached escape
hatch); and `StyledGrapheme` borrows its text instead of owning a `String` per
character. The CHANGELOG marks every breaking change.

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
│  ├ LineGauge      │  ├ Memoized solver           │
│  ├ Input          │  ├ Direction (V/H)           │
│  ├ Table          │  └ Flex / gap / padding      │
│  ├ Editor         ├──────────────────────────────┤
│  ├ Markdown       │  Text Engine                 │
│  ├ SelectList     │  ├ Word wrap                 │
│  ├ Loader         │  ├ Char wrap / truncation    │
│  ├ Sparkline      │  └ Syntax highlighting       │
│  ├ Scrollbar      ├──────────────────────────────┤
│  ├ Canvas         │  Utilities                   │
│  ├ BarChart       │  ├ Fuzzy matching            │
│  ├ Chart          │  └ Undo stack                │
│  ├ Image          ├──────────────────────────────┤
│  ├ Calendar       │  Terminal                    │
│  └ SettingsList   │  └ Synchronized output       │
├──────────────────────────────────────────────────┤
│  Core: Buffer, Cell(Symbol), Style, Text, Reflow │
├──────────────────────────────────────────────────┤
│  Backend: ANSI encoder │ Crossterm │ TestBackend │
└──────────────────────────────────────────────────┘
```

### Elm Architecture

Hawk TUI uses **The Elm Architecture** (TEA), inspired by bubbletea:

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

Like ratatui, Hawk TUI maintains two buffers and only writes the cells that changed between frames to the terminal, minimizing I/O overhead.

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
  "description": "A single-line text input field with cursor navigation and editing.",
  "default_role": "Input",
  "properties": [
    {
      "name": "placeholder",
      "description": "Hint text shown when the input is empty.",
      "property_type": "String",
      "required": false,
      "default_value": null,
      "constraints": []
    }
  ],
  "actions": ["set_value", "get_value", "clear", "insert_text"],
  "usage_hint": "Input::new().placeholder(\"Type here...\")",
  "tags": ["input", "text", "form", "editable"]
}
```

### Capabilities

18 capability types (plus `Custom`) including `Focusable`, `Clickable`, `Scrollable`, `TextInput`, `Selectable`, `RangeEditable`, `Sortable`, `Searchable`, `HasKeyBindings`, and more.

### Ontology Registry

```rust
// Every widget that implements Discoverable, in one call
let registry = hawktui::ontology::builtin_registry();

// Search by semantic role
let inputs = registry.find_by_role(SemanticRole::Input);

// Full JSON catalog
let catalog = registry.export_catalog();
```

Or read it from the command line, without writing any code:

```sh
hawktui-ontology list             # every widget with its role
hawktui-ontology schema Gauge     # one widget in full
hawktui-ontology search scroll
```

**Scope, stated plainly:** the ontology describes a widget's *runtime state* and
semantic role — what it holds and what an agent can do to it while the program
runs. It is not a catalog of builder methods. Use it to choose a widget; read
the rustdoc for the methods that construct one. [`AGENTS.md`](AGENTS.md) covers
the authoring side.

## Widget Set

| Widget                | Description                              | Agent Capabilities                                            |
| --------------------- | ---------------------------------------- | ------------------------------------------------------------- |
| **Block**             | Container with borders and title         | — (Container role)                                            |
| **Paragraph**         | Styled text with wrapping and scrolling  | Scrollable, Copyable                                          |
| **List**              | Selectable list with highlight           | Focusable, Scrollable, Selectable, HasKeyBindings             |
| **Tabs**              | Tab bar navigation                       | Focusable, Selectable, HasKeyBindings                         |
| **Gauge**             | Progress bar (ratio/percentage)          | RangeEditable                                                 |
| **Input**             | Single-line text input with cursor       | Focusable, TextInput, Copyable, HasKeyBindings                |
| **Editor**            | Multi-line editor, line numbers, syntax  | Focusable, TextInput, Scrollable, Copyable, HasKeyBindings    |
| **Table**             | Data table with columns and sorting      | Focusable, Scrollable, Selectable, Sortable, HasKeyBindings   |
| **Markdown**          | Markdown with highlighted fenced code    | Scrollable                                                    |
| **SelectList**        | Interactive single/multi-select list     | Focusable, Scrollable, Selectable, Searchable, HasKeyBindings |
| **Loader**            | Animated spinner with message            | Animated                                                      |
| **Sparkline**         | Inline data trend chart                  | —                                                             |
| **Scrollbar**         | Scrollbar indicator                      | Scrollable                                                    |
| **Canvas**            | Braille drawing surface, incl. GeoJSON   | —                                                             |
| **ModalBox**          | Centered modal overlay with dimmed bg    | — (overlay module; not `Discoverable`)                        |
| **BarChart**          | Grouped bar chart (vertical/horizontal)  | —                                                             |
| **Chart**             | XY line/scatter plot with braille dots   | —                                                             |
| **Image**             | Inline image (Kitty/iTerm2/Sixel/blocks) | —                                                             |
| **SettingsList**      | Key-value settings with cycling values   | Focusable, Selectable                                         |
| **CancellableLoader** | Loader with cancel action                | Animated                                                      |
| **LineGauge**         | Thin single-line progress bar            | RangeEditable                                                 |
| **Calendar**          | Month-view calendar grid with highlights | —                                                             |

All widget types except `ModalBox` implement `Discoverable`; capabilities above are the ones each widget advertises to agents.

## Terminal Capabilities

Things the renderer can do that most TUI frameworks leave to extensions:

```rust
// OSC 8 hyperlinks — clickable text in terminals that support it, plain text
// everywhere else. Frames without links cost nothing: no per-cell storage, no
// extra work in the diff.
buf.set_string_linked(2, 1, "open the docs", Style::default(), "https://example.com");

// Images with no graphics protocol: two vertical pixels per cell, drawn with
// half-blocks, in any 24-bit color terminal. Bring your own decoder.
let img = Image::from_rgba(width, height, rgba)?;      // half-block by default
let img = Image::new(png_bytes, "image/png")           // or a real protocol,
    .protocol(Image::detect_protocol())                // Kitty, iTerm2, Sixel,
    .pixels(decoded);                                  // with pixels to fall
                                                       // back on

// Canvas shapes, including geographic paths and your own
Canvas::new()
    .circle(CanvasCircle { x: 50.0, y: 50.0, radius: 20.0, color: Color::Cyan })
    .filled_rect(CanvasFilledRect { x: 0.0, y: 0.0, width: 10.0, height: 5.0, color: Color::Blue })
    .shape(MyShape::new());

Canvas::new()                                          // whole-world bounds,
    .geographic()                                      // any GeoJSON coastline
    .map(CanvasMap::new(MapData::from_geojson(&coastlines)?).color(Color::Cyan));

// Syntax highlighting with no parser, no grammar files, and no dependencies:
// thirteen languages, and state that survives a line break so a viewport can
// resume anywhere.
Editor::new().syntax_named("rs");                      // or .syntax(&RUST)
Markdown::new(readme);                                 // ```rust blocks, lit up
```

A capability-by-capability comparison against ratatui and SuperLightTUI —
including the rows where Hawk TUI is behind — is in
[docs/FEATURES.md](docs/FEATURES.md).

## Quick Start

```rust
use hawktui::prelude::*;
use hawktui::runtime::{Command, Model, Program};

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
        let greeting = Paragraph::new("Hello, Hawk TUI!")
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
    Program::new(App, backend)?.run()?;
    Ok(())
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

## Agent Protocol (hawktui-server)

Hawk TUI ships a standalone headless server that AI agents can spawn and control via JSON Lines on stdin/stdout:

```sh
# Build (the binaries live behind the `bin` feature)
cargo build --release --features bin --bin hawktui-server

# Test connectivity
echo '{"type":"ping"}' | ./target/release/hawktui-server

# Discover all widget types
echo '{"type":"query_ontology"}' | ./target/release/hawktui-server

# Run the interactive demo
python3 scripts/hawktui-demo.py
```

See [docs/agent-protocol.md](docs/agent-protocol.md) for the full protocol specification, and [docs/agent-integration.md](docs/agent-integration.md) for integration guides (Python, TypeScript, Rust).

### MCP

Agents already speak the Model Context Protocol, so `hawktui-mcp` serves the
ontology as MCP tools over JSON-RPC on stdio — no adapter to write:

```json
{ "mcpServers": { "hawktui": { "command": "hawktui-mcp" } } }
```

Tools: `list_widgets`, `get_widget_schema`, `search_widgets`, `widget_roles`,
`ontology_digest`.

### Testing without a terminal

```rust
use hawktui::testing::Harness;

let mut harness = Harness::new(App::new(), 80, 24)?;
let frames = harness.run_script("Down,Down,q")?;
assert!(frames[2].contains("> Item 03"));
```

`Harness` applies keys and returns each frame as plain text; `assert_frame!`
reports the first differing row and column. See [`AGENTS.md`](AGENTS.md).

## Feature Flags

| Feature     | Default | Description                                                           |
| ----------- | ------- | --------------------------------------------------------------------- |
| `crossterm` | ✓       | Crossterm terminal backend (disable for headless / agent-only)        |
| `bin`       |         | Enables `hawktui-server` and `hawktui-demo` binaries (pulls in `tracing`) |

`hawktui-ontology` and `hawktui-mcp` need no feature flag; they depend only on
what the library already requires.

## Animation System

25 easing functions, spring physics, and timeline sequencing:

```rust
use hawktui::animation::{Tween, Easing, Spring, Timeline};
use std::time::Duration;

let tween = Tween::new(0.0, 1.0, Duration::from_millis(300))
    .easing(Easing::EaseInOutCubic);
let spring = Spring::new(0.0, 1.0)
    .stiffness(170.0)   // default 170
    .damping(26.0);     // default 26
```

## Performance

Measured against other frameworks on identical workloads, on one machine, with
the harness in [`benchmarks/`](benchmarks). Lower is better; the last column is
Hawk TUI's speedup.

| Workload (200×50 screen)      | Hawk TUI | ratatui  | SuperLightTUI | Speedup vs ratatui |
| ----------------------------- | -------- | -------- | ------------- | ------------------ |
| Overlay compositing           | 1.6 µs   | 31.6 µs  | —             | **15.3×**          |
| `set_string`, full screen     | 13.5 µs  | 201.6 µs | 928.6 µs      | **14.2×**          |
| Styled spans, full screen     | 12.1 µs  | 151.5 µs | —             | **11.6×**          |
| Buffer allocation             | 3.2 µs   | 21.7 µs  | 110.8 µs      | **6.8×**           |
| Dashboard render (5 widgets)  | 31.8 µs  | 168.0 µs | —             | **4.2×**           |
| Buffer reset                  | 3.1 µs   | 12.6 µs  | 97.2 µs       | **4.1×**           |
| Diff, 5 % of cells changed    | 50.1 µs  | 162.5 µs | 70.5 µs       | **3.2×**           |
| Paragraph word-wrap           | 44.6 µs  | 144.4 µs | —             | **3.1×**           |
| Escape-sequence emit          | 92.1 µs  | 223.5 µs | —             | **2.4×**           |
| Table render, 200 rows        | 77.1 µs  | 173.7 µs | —             | **2.3×**           |
| List scroll, 1000 items       | 62.6 µs  | 200.7 µs | —             | **2.3×**           |
| Nested layout solve           | 136 ns   | 314 ns   | —             | **2.1×**           |
| Unicode text, full screen     | 121.2 µs | 199.0 µs | —             | **1.6×**           |

Sixteen workloads are measured in total; Hawk TUI is fastest in every one, in
each of two full runs. Times are from one run; the speedup column is the lower
of the two, because run-to-run spread on this machine is wide enough to matter.
[docs/BENCHMARKS.md](docs/BENCHMARKS.md) says how wide.

End to end — build widgets, lay out, render, diff, encode, 20,000 frames of a
five-widget dashboard, each framework in its own process:

| Framework | Frames/s   | Peak RSS | Cells repainted | Bytes emitted |
| --------- | ---------- | -------- | --------------- | ------------- |
| Hawk TUI  | **24,528** | 4.90 MB  | 4,868,506       | 12,016,088    |
| ratatui   | 4,979      | 5.36 MB  | 4,868,107       | 12,013,929    |

Both frameworks repaint the same cells and send the same number of bytes to the
terminal — within 0.02 % — so the gap is engine cost rather than one of them
doing less work.

Where it comes from: cells are `Copy` and 24 bytes with the grapheme stored
inline, the diff is a flat slice zip, text takes a byte-per-cell path for each
ASCII run so mixed scripts pay the Unicode cost only where they must, reflow
borrows from the source text instead of copying it per character, escape
sequences are written straight into one frame-sized byte buffer with only the
attributes that changed, and layout results are memoized per thread. Optional
capabilities stay off the hot path: a frame with no hyperlinks pays nothing for
the feature.

Full methodology, per-workload detail, and reproduction steps:
[docs/BENCHMARKS.md](docs/BENCHMARKS.md).

## Comparison

| Feature                | Hawk TUI | ratatui        | bubbletea | ink      | pi-tui (pi-mono) | OpenTUI                 |
| ---------------------- | ----- | -------------- | --------- | -------- | ---------------- | ----------------------- |
| Frame throughput¹      | 4.9×  | 1.0× (baseline)| —         | —        | —                 | —                       |
| Language               | Rust  | Rust           | Go        | JS/React | TypeScript       | TypeScript + Zig        |
| Architecture           | Elm   | Immediate-mode | Elm       | React    | Immediate-mode   | Component (React/Solid) |
| Agent ontology         | ✓     | —              | —         | —        | —                | —                       |
| Agent protocol (RPC)   | ✓     | —              | —         | —        | ✓ (internal)     | —                       |
| Widget schema export   | ✓     | —              | —         | —        | —                | —                       |
| Headless driver        | ✓     | —              | —         | —        | —                | —                       |
| Focus management       | ✓     | —              | ✓         | ✓        | ✓                | ✓                       |
| Overlay / modal system | ✓     | —              | —         | —        | ✓                | —                       |
| Clickable regions      | ✓     | —              | —         | ✓        | —                | ✓                       |
| Animation system       | ✓     | —              | —         | —        | —                | —                       |
| Markdown widget        | ✓     | —              | —         | ✓        | ✓                | —                       |
| Code editor widget     | ✓     | —              | —         | —        | ✓                | —                       |
| Bar/line/scatter chart | ✓     | ✓ (Chart)      | —         | —        | —                | —                       |
| Terminal image support | ✓     | ext            | —         | —        | ✓                | ✓                       |
| Half-block images      | ✓     | ext            | —         | —        | ✓                | —                       |
| OSC 8 hyperlinks       | ✓     | —              | —         | —        | ✓                | —                       |
| Settings list widget   | ✓     | —              | —         | —        | ✓                | —                       |
| Fuzzy matching         | ✓     | —              | —         | —        | ✓                | —                       |
| Theme system           | ✓     | —              | —         | ✓        | ✓                | —                       |
| Text reflow/word-wrap  | ✓     | ✓              | —         | ✓        | —                | —                       |
| Calendar widget        | ✓     | ✓ (ext)        | —         | —        | —                | —                       |
| Line gauge             | ✓     | ✓              | —         | —        | —                | —                       |
| Block title alignment  | ✓     | ✓              | —         | —        | —                | —                       |
| List direction (B↔T)   | ✓     | ✓              | —         | —        | —                | —                       |
| Synchronized output    | ✓     | ✓              | —         | —        | —                | —                       |

¹ Full redraw loop, measured; see [Performance](#performance). Frameworks in
other languages are not benchmarked here — a cross-runtime number would not be
defensible — so their cells are left blank rather than guessed.

pi-tui is [`@mariozechner/pi-tui`](https://github.com/badlogic/pi-mono) (vendored under `reference/`); [OpenTUI](https://github.com/sst/opentui) is the TypeScript/Zig library that powers OpenCode.

### Agent authoring cost

Feature presence is one thing; what it costs an AI agent to build with the
framework is another, and for a project that calls itself agentic-first it is
the more honest number. Measured over 45 runs on three canonical TUI programs
([methodology](docs/AGENTIC-BENCHMARKS.md)):

| Framework | Programs built correctly | Agent turns | Cost per program |
|---|---|---|---|
| ratatui 0.29 | 3 of 3 | 7–15 | **$0.21–0.33** |
| **Hawk TUI** | 3 of 3 | 30–40 | $0.57–1.04 |
| superlighttui 0.23 | 2.9 of 3 | 49–126 | $1.32–5.74 |

**Hawk TUI is roughly 3–4× more expensive to build with than ratatui**, for
identical results. The likeliest explanation is that ratatui is years older and
far better represented in a model's training data, which no framework feature
can fix quickly — but it is the current state of things and worth knowing before
you choose.

The same study found that supplying the widget ontology to the authoring agent
made **no measurable difference** on any pre-registered metric. Its value is
runtime introspection — an agent *driving* a running program — not code
generation. The results, including the ones that went against us, are in
[docs/AGENTIC-BENCHMARKS.md](docs/AGENTIC-BENCHMARKS.md).

## License

Hawk TUI is dual-licensed:

- **Open source**: [GNU Affero General Public License v3.0 (AGPLv3)](https://www.gnu.org/licenses/agpl-3.0.html) — free for open-source projects that comply with AGPLv3 terms, including the requirement to release source code of derivative works and network-accessible services.
- **Commercial**: A proprietary commercial license is available for organizations that cannot or prefer not to comply with AGPLv3. Contact [NERVOSYS](https://nervosys.ai/) for commercial licensing inquiries.
