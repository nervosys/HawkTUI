# Feature matrix

What Hawk TUI does, next to the frameworks it is most often compared with. Rows
are capabilities a TUI author actually reaches for, not marketing categories.

**How to read this.** ✓ means the framework ships it in its own crate/package.
"ext" means it exists only through a third-party extension. — means absent.
Rust frameworks were verified against their source; entries for other-language
frameworks come from their own documentation and are marked accordingly.

## Rendering engine

| Capability | Hawk TUI | ratatui 0.29 | SuperLightTUI 0.23 |
| ---------- | :------: | :----------: | :----------------: |
| Double-buffered differential rendering | ✓ | ✓ | ✓ |
| `Copy`, allocation-free cell | ✓ (24 B) | — (heap string) | — (heap string) |
| Direct escape-sequence encoder (no command layer) | ✓ | — | ✓ |
| Per-flag attribute diffing (no full SGR reset) | ✓ | ✓ | ✓ |
| Synchronized output (CSI ?2026) | ✓ | ✓ | ✓ |
| Memoized layout solve | ✓ | ✓ | ✓ |
| Scalar fast path between ASCII and segmentation | ✓ | — | — |
| Space-between / around / evenly distribution | ✓ | partial¹ | ✓ |
| ASCII fast path in text placement | ✓ | — | — |
| Borrowed (non-allocating) text reflow | ✓ | ✓ | ✓ |

¹ ratatui 0.29 has `Flex::SpaceBetween` and `Flex::SpaceAround` but no
space-evenly.

## Text and typography

| Capability | Hawk TUI | ratatui 0.29 | SuperLightTUI 0.23 |
| ---------- | :------: | :----------: | :----------------: |
| Word wrap / char wrap / truncation | ✓ | ✓ | ✓ |
| Grapheme-cluster correctness (combining marks, ZWJ) | ✓ | ✓ | ✓ |
| Wide-character (CJK, emoji) column handling | ✓ | ✓ | ✓ |
| Styled spans within a line | ✓ | ✓ | ✓ |
| Underline color | ✓ | ✓ | ✓ |
| Curly / dotted / dashed underlines | ✓ | — | ✓ |
| Superscript / subscript modifiers | ✓ | — | — |
| Markdown rendering | ✓ | — | ext |
| Syntax highlighting | ✓ (13 languages) | — | ✓ (17 languages) |

## Widgets

| Widget | Hawk TUI | ratatui 0.29 | SuperLightTUI 0.23 |
| ------ | :------: | :----------: | :----------------: |
| Block, Paragraph, List, Tabs, Gauge | ✓ | ✓ | ✓ |
| Table with column constraints and sorting | ✓ | ✓ | ✓ |
| Multi-line editor | ✓ | ext | ✓ |
| Select list with search | ✓ | ext | ✓ |
| Bar chart / line & scatter chart / sparkline | ✓ | ✓ | ✓ |
| Braille canvas with custom shapes | ✓ | ✓ | ✓ |
| Circle and filled-rectangle shapes | ✓ | ✓ | ✓ |
| Geographic map shape | ✓ (any GeoJSON) | ✓ (bundled world) | — |
| Calendar | ✓ | ✓ (feature-gated) | ✓ |
| Line gauge | ✓ | ✓ | ✓ |
| Scrollbar | ✓ | ✓ | ✓ |
| Settings list | ✓ | — | ✓ |
| Modal / overlay stack | ✓ | — | ✓ |
| Animated loader, cancellable loader | ✓ | — | ✓ |

## Terminal integration

| Capability | Hawk TUI | ratatui 0.29 | SuperLightTUI 0.23 |
| ---------- | :------: | :----------: | :----------------: |
| OSC 8 hyperlinks | ✓ | — | ✓ |
| Inline images — Kitty protocol | ✓ | ext | ✓ |
| Inline images — iTerm2 protocol | ✓ | ext | ✓ |
| Inline images — half-block (any 24-bit terminal) | ✓ | ext | ✓ |
| Inline images — Sixel | ✓ | ext | ✓ |
| Mouse capture and events | ✓ | ✓ | ✓ |
| Click-region hit testing built in | ✓ | — | ✓ |
| Bracketed paste | ✓ | ✓ | ✓ |
| Focus management across widgets | ✓ | — | ✓ |
| Theme tokens | ✓ | — | ✓ |

## Agent integration

This is the row set no other TUI framework competes in.

| Capability | Hawk TUI | ratatui 0.29 | SuperLightTUI 0.23 |
| ---------- | :------: | :----------: | :----------------: |
| Typed widget ontology (schema per widget type) | ✓ | — | — |
| Capability advertisement per widget instance | ✓ | — | — |
| Named, parameter-validated actions | ✓ | — | — |
| Semantic roles for role-based discovery | ✓ | — | — |
| Live UI tree snapshot for agents | ✓ | — | — |
| JSON-Lines RPC server (`hawktui-server`) | ✓ | — | — |
| Headless driver (no terminal required) | ✓ | — | — |
| Event injection (keys, mouse, paste, resize) | ✓ | — | — |
| Event subscription streaming | ✓ | — | — |

## Architecture and testing

| Capability | Hawk TUI | ratatui 0.29 | SuperLightTUI 0.23 |
| ---------- | :------: | :----------: | :----------------: |
| Elm architecture runtime included | ✓ | — | — |
| Immediate-mode rendering | ✓ | ✓ | ✓ |
| Animation system (easings, springs, timelines) | ✓ | — | ✓ |
| Test backend with buffer assertions | ✓ | ✓ | ✓ |
| Property-based tests in-tree | ✓ | — | ✓ |
| Head-to-head benchmarks in-tree | ✓ | — | — |

## Where Hawk TUI is behind

Stated plainly, because a matrix that only lists wins is not worth reading:

- **Highlighter coverage.** Both frameworks now highlight; SuperLightTUI
  reaches CSS, HTML, Ruby, and Haskell, which Hawk TUI does not. Hawk TUI
  covers SQL, which SuperLightTUI does not. Neither is a superset.
- **Ecosystem size.** ratatui has years of third-party widgets, templates, and
  tutorials. Several rows above where ratatui reads "ext" are covered well by
  the community; that is a real advantage this table does not capture.
- **Content-driven sizing.** Hawk TUI now has the flexbox vocabulary — gap,
  padding, fill weights, and space-between/around/evenly — but its solver
  still sizes from constraints alone. SuperLightTUI keeps a node tree and can
  measure a child's intrinsic content to size around it; expressing "as tall
  as this text turns out to be" in Hawk TUI means measuring it yourself first.
- **Bundled map data.** ratatui embeds a world coastline dataset, so its
  `Map` shape draws something the moment you construct it. Hawk TUI's
  `CanvasMap` renders any GeoJSON you hand it — including the Natural Earth
  files ratatui's data derives from — but ships no coordinates of its own, so
  a world map costs you a data file.

## Verification

Rust entries were checked against the dependency sources actually compiled for
[`benchmarks/`](../benchmarks) — ratatui 0.29 and superlighttui 0.23 — not from
documentation. Every Hawk TUI ✓ in this table is exercised by a test in
[`tests/`](../tests) or by the benchmark harness.

For performance rather than capability, see [BENCHMARKS.md](BENCHMARKS.md).
