# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
- **Error Types**: Unified `louie::Error` enum (`Io`, `Json`, `Protocol`, `Action`, `WidgetNotFound`, `Layout`) with `From` impls and `source()` chaining
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

[0.1.0]: https://github.com/nervosys/Louie/releases/tag/v0.1.0
