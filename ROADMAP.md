# Louie Roadmap

## Phase 0 — Foundation (Complete)

- [x] Project structure and Cargo.toml
- [x] Core primitives: Buffer, Cell, Style, Text, Rect
- [x] Layout engine with constraint solver
- [x] Event system: keyboard, mouse, resize, paste, focus
- [x] Hit-testing for clickable regions (HitMap)
- [x] Backend trait + Crossterm backend + TestBackend
- [x] Terminal: double-buffered differential rendering
- [x] Elm architecture runtime (Model/Update/View)
- [x] Animation system: 25 easings, Tween, Spring, Timeline
- [x] Ontology system: Discoverable trait, Schema, Capabilities, Actions, Registry
- [x] Widget set: Block, Paragraph, List, Tabs, Gauge, Input, Table, Sparkline, Scrollbar, Canvas
- [x] All widgets implement Discoverable
- [x] Examples: hello, counter, agent_demo
- [x] Compiles with zero warnings

## Phase 1 — Agent Protocol Layer

The critical missing piece: a structured protocol for AI agents (like OpenCode/Pi,
OpenClaw, or any LLM-based coding agent) to connect to and drive a Louie application
without human interaction.

- [x] `src/agent/protocol.rs` — JSON-based agent protocol messages
  - AgentRequest: query_ontology, execute_action, get_state, get_tree, inject_event, subscribe
  - AgentResponse: success/error with typed payloads
  - AgentEvent: streamed notifications (state_changed, render_update, action_result)
- [x] `src/agent/session.rs` — AgentSession: manages agent connection lifecycle
  - Process incoming requests against the ontology registry and model
  - Emit events when state changes
  - Track subscriptions (which state changes the agent cares about)
- [x] `src/agent/driver.rs` — Headless agent driver
  - Run a Louie app without a terminal (TestBackend)
  - Agent sends protocol messages, receives events
  - Enables automated testing and CI pipelines
- [x] `src/agent/mod.rs` — Module root with re-exports

## Phase 2 — Focus & Overlay

Systems needed for real interactive applications that agents can drive.

- [x] `src/focus.rs` — Focus management
  - Focus ring: ordered list of focusable widget IDs
  - Tab/Shift+Tab navigation
  - Programmatic focus (agent can focus any widget by ID)
  - Focus change events
- [x] `src/overlay.rs` — Modal/overlay system
  - Overlay stack rendered on top of main content
  - Focus capture (overlay traps keyboard input)
  - Dismissal callbacks
  - Agent-addressable overlays (ontology-visible)

## Phase 3 — Rich Widgets

Widgets matching the component richness of pi-tui, needed for real agent UIs.

- [x] `src/widget/markdown.rs` — Markdown renderer
  - Headings, bold, italic, code spans, code blocks, lists, blockquotes
  - Syntax highlighting for code blocks (via inline ANSI)
  - Scrollable with agent Scrollable capability
- [x] `src/widget/editor.rs` — Multi-line text editor
  - Line-based editing with cursor movement
  - Insert, delete, backspace, home/end, word-jump
  - Viewport scrolling when content exceeds area
  - Agent TextInput capability with multiline=true
- [x] `src/widget/loader.rs` — Animated spinner/loader
  - Multiple spinner styles (braille, dots, line, arc)
  - Message text alongside spinner
  - Agent-visible Animated capability
- [x] `src/widget/select_list.rs` — Interactive select list
  - Keyboard navigation (up/down/home/end)
  - Single and multi-select modes
  - Filter/search input
  - Agent Selectable + Searchable capabilities

## Phase 4 — Async & Streaming

- [x] Async command variant: `Command::Task` for spawning background work
  - Returns a future that resolves to a message
  - Enables network I/O, file reads, LLM streaming without blocking the UI
- [x] RPC transport: stdin/stdout JSON Lines protocol
  - Agent connects via stdio, sends JSON requests, receives JSON events
  - Compatible with OpenCode/Pi RPC embedding pattern
  - `src/agent/rpc.rs` — transport implementation

## Phase 5 — Testing & Examples

- [x] Core tests: Buffer, Rect, Style, Layout
- [x] Widget rendering tests using TestBackend
- [x] Agent protocol round-trip tests
- [x] `examples/agent_rpc.rs` — RPC-driven agent demo
  - Starts a Louie app in headless mode
  - Reads JSON commands from stdin, writes events to stdout
  - Demonstrates the full agent protocol

## Phase 6 — Documentation & Polish

- [x] Update README with agent protocol documentation
- [x] Update comparison table (add pi-tui, OpenTUI)
- [x] Crate-level documentation with module overviews
- [x] Agent integration guide (how to connect OpenCode/Pi to a Louie app)
- [x] Performance benchmarks

## Phase 7 — Parity & Polish

New features added beyond the original roadmap:

- [x] `src/widget/cancellable_loader.rs` — CancellableLoader with cancel action (pi-mono parity)
- [x] `src/theme.rs` — Theming system with semantic tokens, dark/light presets
- [x] `tests/widget_render_tests.rs` — 17 widget rendering tests (Buffer-based assertions)
- [x] `docs/agent-integration.md` — Comprehensive agent integration guide
- [x] `benches/core_bench.rs` — Criterion benchmarks (buffer, layout, style, paragraph, protocol serde)

## Phase 8 — Advanced Widgets & Utilities (Complete)

New data visualization widgets, terminal graphics support, and general-purpose utilities
based on parity analysis of pi-mono and ratatui.

### Widgets

- [x] `src/widget/barchart.rs` — BarChart with grouped bars
  - Bar, BarGroup, BarDirection (Vertical/Horizontal)
  - Eighths-resolution bar symbols for sub-cell precision
  - Per-bar styling, value labels, group labels
  - Auto-detected max value
  - Full Discoverable implementation (DataVisualization role)
- [x] `src/widget/chart.rs` — XY chart for line and scatter plots
  - Dataset with Marker types (Braille, Block, HalfBlock, Char)
  - GraphType: Line (Bresenham algorithm) and Scatter
  - Braille sub-cell resolution rendering (2×4 dot grid per cell)
  - Axis with title, bounds, and proportionally-spaced labels
  - Legend with configurable position (TopLeft/TopRight/BottomLeft/BottomRight)
  - Full Discoverable implementation (DataVisualization role)
- [x] `src/widget/image.rs` — Inline terminal image widget
  - ImageProtocol: Kitty graphics, iTerm2 inline images, Fallback text
  - Auto-detection via KITTY_WINDOW_ID / TERM_PROGRAM environment variables
  - Base64 encoding for protocol payloads
  - Configurable max width/height, MIME type, fallback text
  - Full Discoverable implementation (Display role)
- [x] `src/widget/settings_list.rs` — Interactive settings list (pi-mono parity)
  - Setting with cycleable values (cycle_next / cycle_prev)
  - SettingsListState with cursor navigation and scroll offset
  - Optional descriptions for focused setting
  - Both StatefulWidget and Widget implementations
  - Full Discoverable implementation (Focusable + Selectable, cycle actions)

### Utilities

- [x] `src/util/fuzzy.rs` — Fuzzy string matching with scoring
  - Case-insensitive matching with scored results
  - Consecutive bonus, word boundary bonus, gap penalty, late match penalty
  - `fuzzy_match()` for single match, `fuzzy_rank()` for sorted candidate lists
  - 7 unit tests
- [x] `src/util/undo.rs` — Generic undo stack
  - `UndoStack<S>` with configurable max depth
  - Push, pop, peek, clear operations
  - 4 unit tests

### Testing

- [x] 14 new widget render tests (31 total widget render tests)
  - BarChart: single bar, multiple bars, discoverable schema
  - Chart: scatter points, braille line, discoverable schema
  - Image: fallback text, discoverable schema
  - SettingsList: renders settings, cycle next, cycle prev wraps
  - Fuzzy matching: consecutive vs scattered, prefix vs mid-match ranking
  - Undo stack: string state push/pop
- [x] Full test suite: 92 tests passing (18 unit + 39 integration + 31 widget render + 4 doc-tests)

## Phase 9 — Text Infrastructure, Layout Enhancements & New Widgets (Complete)

Text reflow engine, paragraph word-wrap, new widgets (LineGauge, Calendar), list/block
enhancements, and synchronized terminal output.

### Core Infrastructure

- [x] `src/core/reflow.rs` — Text reflow engine
  - `StyledGrapheme` type for per-grapheme styling
  - `line_to_graphemes()` / `graphemes_to_line()` for decomposition/reassembly with span merging
  - `WordWrapper` — breaks at word boundaries, char-break fallback for oversized words
  - `CharWrapper` — breaks at grapheme cluster boundaries
  - `LineTruncator` — clips lines to max width
  - 8 unit tests
- [x] `src/widget/paragraph.rs` — Word-wrap integration
  - Replaced dead-code `wrap_lines()` stub with `reflow_lines()` dispatch
  - `Wrap::Word` → WordWrapper, `Wrap::Char` → CharWrapper, `Wrap::None` → LineTruncator
  - Paragraph rendering now fully reflows text through the reflow engine

### New Widgets

- [x] `src/widget/line_gauge.rs` — Thin single-line progress bar
  - Line-drawing characters: `━` (filled) and `─` (unfilled)
  - `ratio()` / `percent()` setters, configurable symbols
  - Optional label rendered before gauge line
  - Full Discoverable implementation (RangeEditable capability, set_progress action)
- [x] `src/widget/calendar.rs` — Month-view calendar grid
  - Day-of-week headers (Mo Tu We Th Fr Sa Su)
  - Tomohiko Sakamoto's algorithm for first-weekday calculation
  - `days_in_month()` with leap year support
  - Day highlighting with custom styles via `highlight_day(day, style)`
  - Full Discoverable implementation (Display role)

### Widget Enhancements

- [x] `src/widget/list.rs` — ListDirection support
  - `ListDirection` enum: TopToBottom (default) / BottomToTop
  - BottomToTop anchors items to bottom of area, renders upward
  - `direction()` builder method
- [x] `src/widget/block.rs` — Title alignment
  - `title_alignment` and `title_bottom_alignment` fields (Left/Center/Right)
  - Builder methods: `title_alignment()`, `title_bottom_alignment()`
  - Title positioning computed within `max_width = area.width - 2`

### Terminal Backend

- [x] Synchronized output (CSI ?2026h/l)
  - `begin_sync()` / `end_sync()` methods on Backend trait (default no-op)
  - Crossterm implementation writes raw CSI sequences
  - `Terminal::draw()` wraps diff+cursor handling in begin/end sync

### Testing

- [x] 15 new tests (Phase 9 specific)
  - Reflow: word_wrap_breaks_at_space, word_wrap_preserves_span_styles, char_wrap_breaks_mid_word, truncator_clips_long_line
  - Paragraph wrap: word_wrap_renders_multiple_lines, char_wrap_breaks_long_word
  - LineGauge: renders_filled_and_unfilled, renders_with_label, discoverable_schema
  - Calendar: renders_day_headers, renders_days_of_month, discoverable_schema
  - List direction: bottom_to_top_renders_last_items_at_bottom
  - Block title alignment: title_centered, title_right_aligned
- [x] Full test suite: 111 tests passing (26 unit + 39 integration + 46 widget render + 4 doc-tests), 0 warnings
