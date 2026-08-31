# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- **Renamed to Hawk TUI**: the project, crate, and import name are now `hawktui`
  (`use hawktui::prelude::*`), the binaries are `hawktui-server` and
  `hawktui-demo`, and the repository moved to
  [nervosys/HawkTUI](https://github.com/nervosys/HawkTUI). 1.0.0 was published
  to crates.io as `louietui` with the import name `louie`; entries in the
  released sections below use the current names.
- **`Cell` is now `Copy` and 24 bytes**: the grapheme cluster is stored inline
  in the new [`core::symbol::Symbol`] type (8 bytes) instead of a
  `CompactString`. Clusters longer than 7 UTF-8 bytes are interned process-wide
  and referenced by index, so no content is lost. `cell.symbol` is a `Symbol`
  rather than a string — call `cell.symbol.as_str()` (or `cell.symbol()`) where
  a `&str` is needed. **Breaking.**
- **`Layout::split` returns `Rc<[Rect]>`** and memoizes results per thread;
  `Layout::solve` is the uncached escape hatch. Indexing and iteration are
  unchanged, so most call sites need no edit. **Breaking for code that consumed
  the returned `Vec`.**
- **`StyledGrapheme` borrows its text** (`StyledGrapheme<'a> { grapheme: &'a str }`)
  instead of owning a `String` per character. **Breaking.**
- **Layout builders accept any iterator** of constraints
  (`impl IntoIterator<Item = Constraint>`), not just `Vec`-convertible types.
- `Text`, `Line`, and `Span` accept any `&str`, not only `&'static str`. The
  data was already copied; the lifetime bound served no purpose.

### Added

- **OSC 8 hyperlinks**: `Buffer::set_string_linked` and `Buffer::set_hyperlink`
  attach a URL to a run of cells, which terminals that support OSC 8 render as
  clickable. Targets live in a sparse table beside the grid rather than inside
  `Cell`, so a frame without links costs nothing — not a byte per cell, not a
  comparison in the diff. `Backend::draw_linked` carries them to the terminal,
  with a default implementation for backends that ignore links.
- **Half-block images**: `Image::from_rgba` / `Pixels` render decoded pixels
  with `▀` glyphs — two vertical pixels per cell — so images work in any
  24-bit-color terminal with no graphics protocol at all. Alpha below 50 %
  leaves the underlying cell untouched.
- **Syntax highlighting**: `widget::highlight` is a dependency-free lexer
  covering Rust, Python, JavaScript, TypeScript, Go, C, C++, Java, JSON, TOML,
  YAML, shell, and SQL. `Highlighter` carries block-comment and multi-line
  string state across lines, so a viewport can resume anywhere. `Markdown`
  highlights fenced blocks that name a language (on by default), and `Editor`
  takes `.syntax(&RUST)` or `.syntax_named("py")`.
- **`CanvasMap`**: geographic paths on a canvas, in longitude/latitude degrees,
  with `MapData::from_geojson` reading `LineString`, `MultiLineString`,
  `Polygon`, and `MultiPolygon` geometries out of any GeoJSON document.
  `Canvas::geographic()` sets whole-world bounds. Segments that jump the
  antimeridian are treated as seams, not drawn across the map.
- **Flex distribution**: `Flex::SpaceBetween` and `Flex::SpaceAround` now
  actually distribute the slack between segments rather than only shifting
  them, and `Flex::SpaceEvenly` joins them.
- **Layout shorthands**: `gap`, `padding`, `horizontal_margin`, and
  `vertical_margin`.
- **Sixel images**: `ImageProtocol::Sixel` and `Pixels::to_sixel` encode
  decoded pixels as a DEC Sixel sequence — self-contained, no image or color
  crates, quantizing to the 6×6×6 cube plus a grayscale ramp, with run-length
  compression and per-band color overlay. `Image::detect_protocol` now also
  recognizes Sixel-capable terminals.
- **Canvas shapes**: `CanvasCircle`, `CanvasFilledRect`, and `Canvas::shape`
  for any user type implementing `Shape`.
- **`Layout::areas::<N>()`**: splits into a fixed number of regions that
  destructure at the call site (`let [header, body, footer] = …`), filling any
  surplus with empty rects rather than panicking.
- **`Buffer::set_grapheme` / `set_symbol`**: write one already-decided glyph
  without scanning or segmenting it.
- **`Sparkline::direction`**: `SparklineDirection::RightToLeft` puts the newest
  sample on the left for feeds that prepend.
- **`List::scroll_padding`**: keeps N rows visible above and below the
  selection, so the cursor never sits flush against the viewport edge.
- **`docs/FEATURES.md`**: a capability matrix against ratatui and
  SuperLightTUI, verified against their sources, including the rows where Hawk
  TUI is behind.
- **`backend::ansi`**: escape sequences are encoded directly into a reusable
  frame buffer with hand-written integer formatting, and attribute changes emit
  only the flags that actually changed. One `write_all` per frame.
- **`benchmarks/`**: an unpublished crate benchmarking Hawk TUI head-to-head
  against ratatui and SuperLightTUI on identical workloads, plus a
  process-level frame-loop harness reporting throughput and peak RSS. See
  [docs/BENCHMARKS.md](docs/BENCHMARKS.md).

### Performance

Measured against ratatui 0.29 on a 200×50 screen (see
[docs/BENCHMARKS.md](docs/BENCHMARKS.md) for methodology):

- Full redraw loop: **4.93× the frames per second** at 91 % of the peak
  memory, repainting the same cells and emitting the same number of bytes
  (within 0.02 %).
- Fastest of the three frameworks in all sixteen measured workloads, in each of
  two full runs. Taking the lower ratio of the two: overlay compositing
  **15.3×**, `set_string` **14.2×**, styled spans **11.6×**, buffer allocation
  **6.8×**, dashboard **4.2×**, reset **4.1×**, diff **3.0–3.6×**, word-wrap
  **3.1×**, escape emit **2.4×**, table render **2.3×**, list scroll **2.3×**,
  layout solve **2.1×**, style-churn emit **1.9×**, Unicode text **1.6×**.
- Text placement now takes its fast path per ASCII *run* rather than per
  string, so mixed scripts pay the Unicode cost only for their Unicode parts,
  and a scalar fast path handles Latin, Greek, Cyrillic, CJK, kana, and Hangul
  without the segmenter at all — taking their width from the matched range
  instead of the width tables. Unicode text placement is **1.7×** ratatui,
  up from parity.
- `Block` draws its borders as runs instead of one `set_string` per cell. Box
  glyphs are not ASCII, so the old path re-entered grapheme segmentation for
  every border cell of every frame; this alone took the dashboard from roughly
  parity with ratatui to several times faster, and roughly doubled the whole
  frame loop.
- The buffer diff compares 32-cell blocks first and only walks the cells inside
  blocks that differ, so untouched regions are skipped wholesale.
- `List` no longer restyles a row after writing an item whose spans carry no
  style of their own — the row was already painted.
- Wider workload coverage in the shootout: non-ASCII text, styled spans,
  table render, list scrolling, buffer compositing, and a style-churn emit
  worst case.

## [1.0.0] - 2025-07-17

### Added

- **Property-Based Tests**: 10 proptest-driven tests for agent protocol round-tripping and fuzz deserialization
- **Structured Logging**: Replaced `eprintln` in `hawktui-server` with `tracing` + `tracing-subscriber` (LOG-1 upgrade)
- **CONTRIBUTING.md**: Contributor guide with development workflow, commit conventions, and widget addition guide
- **SECURITY.md**: Security policy with responsible disclosure process and hardening summary
- **cargo-deny**: License auditing and advisory checks via `deny.toml` + CI job
- **CI Coverage**: `cargo-tarpaulin` coverage reporting with Codecov upload
- **CI Benchmarks**: `criterion` benchmark regression detection via `github-action-benchmark`
- **README Install Section**: `cargo add hawktui` instructions and MSRV note

### Changed

- **API Stability**: `util` module marked `#[doc(hidden)]` — internal utilities are no longer part of the public API
- **Buffer Safety**: `Buffer::IndexMut` no longer panics on out-of-bounds writes; uses a scratch cell for defense-in-depth (MEM-1)
- **Dependencies**: Added `tracing` 0.1, `tracing-subscriber` 0.3; added `proptest` 1 (dev)

### Security

- Comprehensive doc comments added to all agent protocol types, event types, animation API, overlay system, runtime, terminal, focus, and widget traits

## [0.1.0] - 2025-07-16

### Added

- **Elm Architecture Runtime**: `Model`, `Command`, `Program` with async task support, tick rates, and event loop
- **Agent Protocol**: JSON Lines over stdin/stdout — 10 request types (`ping`, `query_ontology`, `get_schema`, `get_tree`, `get_state`, `execute_action`, `inject_event`, `subscribe`, `unsubscribe`, `quit`)
- **Protocol Versioning**: `PROTOCOL_VERSION` constant returned in ping responses for compatibility checking
- **Ontology System**: `Discoverable` trait, `WidgetSchema`, `AgentCapability` (18 variants), `SemanticRole` (15 variants), `AgentAction` with typed parameter validation
- **Ontology Registry**: Type catalog + live UI tree with search, role-based discovery, and action parameter validation (INJ-2)
- **Headless Driver**: `HeadlessDriver` for agent-only operation, automated testing, and CI/CD integration
- **RPC Transport**: `RpcTransport` with stdin/stdout JSON Lines, rate limiting (1000 req/s), and 1 MB line size cap
- **21 Widgets**: Paragraph, List, Block, Tabs, Gauge, LineGauge, Scrollbar, Table, Input, Editor, SelectList, SettingsList, Loader, CancellableLoader, Sparkline, BarChart, Calendar, Chart, Image, Toast, Canvas
- **Focus System**: `FocusManager` with focus ring (Tab/Shift+Tab) and programmatic focus control
- **Overlay System**: `OverlayStack` with focus capture and `ModalBox` centered modal widget
- **Animation**: Tweens, springs, easing functions (11 curves), and timeline sequencing
- **Layout Engine**: Constraint-based layout with `Length`, `Percentage`, `Min`, `Max`, `Fill`, `Ratio` constraints
- **Core Primitives**: `Buffer` (double-buffered differential rendering), `Cell`, `Rect`, `Style`, `Text`/`Span`/`Line`, `Color` (16 + RGB + indexed)
- **Theme System**: `Theme` with token-based styling, built-in dark/light themes
- **Error Types**: Unified `hawktui::Error` enum (`Io`, `Json`, `Protocol`, `Action`, `WidgetNotFound`, `Layout`) with `From` impls and `source()` chaining
- **Backend Abstraction**: `Backend` trait with crossterm (optional) and test backends

### Security

- Input sanitization on all agent protocol fields (INP-1, INP-2, INP-3)
- Subscription limit (100 max) to prevent resource exhaustion (INP-4)
- Terminal dimension clamping (1–1024) on injected resize events (INP-2)
- Rate limiting on RPC transport (1000 requests/second) (INP-4)
- Action parameter schema validation before dispatch (INJ-2)
- Structured logging with redacted sensitive fields (LOG-1)
- Auth handshake support in agent session (AUTH-1)
- Binary path validation for external commands (BIN-1)

[1.0.0]: https://github.com/nervosys/HawkTUI/releases/tag/v1.0.0
[0.1.0]: https://github.com/nervosys/HawkTUI/releases/tag/v0.1.0
