# Benchmarks

Hawk TUI is measured head-to-head against other terminal UI frameworks on
identical workloads. Every number below is reproducible from this repository —
nothing is quoted from a third party's own benchmark.

## What is compared

| Framework | Version | Language | Included as |
| --------- | ------- | -------- | ----------- |
| **Hawk TUI** | this tree | Rust | path dependency |
| [ratatui](https://github.com/ratatui/ratatui) | 0.29 | Rust | crates.io dev-dependency |
| [SuperLightTUI](https://github.com/subinium/SuperLightTUI) | 0.23 | Rust | crates.io dev-dependency (`slt`) |

Two names that come up in this comparison do not belong in it, and it is worth
saying why rather than quietly dropping them:

- **gitui** is an *application* (a Git client built on ratatui), not a
  framework. Timing it measures libgit2 and its own UI, not a rendering engine.
  Its framework — ratatui — is measured directly instead, and the frame-loop
  harness below covers the whole-process question gitui would have answered.
- **ratatat** on crates.io is a parser-combinator library ("expressive parser
  combinators with caching"), unrelated to terminal UI. There is no TUI crate
  by that name to benchmark.

SuperLightTUI participates in the buffer-primitive groups, whose APIs map
one-to-one. Its immediate-mode widget layer has no direct counterpart to a
`Widget::render(area, &mut buffer)` call, so it sits out the widget and layout
groups rather than being compared against something it does not do.

## The workloads

Each group runs the same work through each framework's own public API.

| Group | What it measures |
| ----- | ---------------- |
| `buffer_alloc` | Creating a fresh 200×50 frame buffer |
| `buffer_reset` | Clearing a buffer between frames |
| `buffer_diff` | Comparing two frames at 1 %, 5 %, and 50 % change |
| `set_string` | Writing ASCII text across every row |
| `unicode_set_string` | The same, with CJK, combining marks, and emoji — the path where neither framework can skip segmentation |
| `styled_spans` | A line of 20 differently styled spans, written to every row |
| `layout_solve` | Nested vertical + horizontal constraint solves |
| `render_dashboard` | Five widgets composed into one screen |
| `table_render` | A 4-column, 200-row table with a selected row |
| `list_scroll` | A 1,000-item list scrolled one row per iteration |
| `paragraph_wrap` | Word-wrapping a screenful of prose |
| `buffer_merge` | Compositing an overlay buffer onto a base — what modals cost |
| `terminal_emit` | Encoding a half-changed frame to escape sequences |
| `terminal_emit_style_churn` | The encoder's worst case: every adjacent cell differs in style |

## Results

Machine: Windows 11, x86-64. Criterion, release profile with `lto = true` and
`codegen-units = 1`. Times are Criterion's median point estimates, read from its
saved `estimates.json` rather than scraped from console output; **lower is
better**. The last two columns are Hawk's speedup.

Frameworks within a group are measured back to back, so the ratios hold even
when ambient system load moves the absolute numbers — which it does, by as much
as 2× on a busy desktop. Compare ratios across machines, not microseconds.

| Workload (200×50 screen) | Hawk TUI | ratatui | SuperLightTUI | vs ratatui | vs SLT |
| --- | --- | --- | --- | --- | --- |
| Buffer allocation | 3.2 µs | 21.7 µs | 110.8 µs | **6.8×** | 33.9× |
| Buffer reset | 3.1 µs | 12.6 µs | 97.2 µs | **4.1×** | 31.3× |
| Diff, 1 % changed | 24.3 µs | 129.2 µs | 82.1 µs | **3.6×** | 1.3× |
| Diff, 5 % changed | 50.1 µs | 162.5 µs | 70.5 µs | **3.2×** | 1.4× |
| Diff, 50 % changed | 33.5 µs | 102.1 µs | 41.8 µs | **3.0×** | 1.2× |
| Overlay compositing | 1.6 µs | 31.6 µs | — | **15.3×** | — |
| `set_string`, full screen | 13.5 µs | 201.6 µs | 928.6 µs | **14.2×** | 68.8× |
| Unicode text, full screen | 121.2 µs | 199.0 µs | 1.00 ms | **1.6×** | 8.3× |
| Styled spans, full screen | 12.1 µs | 151.5 µs | — | **11.6×** | — |
| Paragraph word-wrap | 44.6 µs | 144.4 µs | — | **3.1×** | — |
| Nested layout solve | 136 ns | 314 ns | — | **2.1×** | — |
| Dashboard render (5 widgets) | 31.8 µs | 168.0 µs | — | **4.2×** | — |
| Table render, 200 rows | 77.1 µs | 173.7 µs | — | **2.3×** | — |
| List scroll, 1000 items | 62.6 µs | 200.7 µs | — | **2.3×** | — |
| Escape-sequence emit | 92.1 µs | 223.5 µs | — | **2.4×** | — |
| Emit, style churn (worst case) | 223.0 µs | 1.66 ms | — | **1.9×** | — |

Hawk TUI is fastest in every group. Absolute times are from one full run; the
speedup column is the **lower** of two full runs, because this machine's
run-to-run spread is large (see Honesty notes).

### Whole-process frame loop

The micro-benchmarks isolate one operation each; this one runs the entire
redraw: build widgets → lay out → render → diff → encode, 20,000 frames of a
five-widget dashboard at 200×50 with a moving list selection and a moving gauge.
Each framework runs in its own process, so peak memory is attributable.

| Framework | Frames/s   | Peak RSS | Cells repainted | Bytes emitted |
| --------- | ---------- | -------- | --------------- | ------------- |
| Hawk TUI  | **24,528** | 4.90 MB  | 4,868,506       | 12,016,088    |
| ratatui   | 4,979      | 5.36 MB  | 4,868,107       | 12,013,929    |

**4.93× the throughput at 91 % of the memory.** The repainted-cell and byte
counts are the fairness check: both frameworks put the same characters on the
same screen and send the same number of bytes to the terminal — within 0.02 % —
so the throughput difference is engine cost, not one of them doing less work.
(The 399-cell gap out of 4.87 M comes from a word-wrap tie-break at line ends,
where `Wrap::Word` and ratatui's `Wrap { trim: true }` disagree by one space.)

## Where the speed comes from

Each of these is a deliberate design decision, and each one is visible in the
table above.

1. **`Cell` is `Copy` and 24 bytes.** The grapheme cluster lives inline in an
   8-byte [`Symbol`](../src/core/symbol.rs) instead of behind a heap pointer.
   Allocating, resetting, cloning, and comparing cells never touch the allocator
   and never run a destructor. Clusters too long to inline (ZWJ emoji, flags)
   are interned once and referenced by index, so nothing is lost. This is what
   makes buffer allocation, reset, and diff several times faster.
2. **The diff is a flat zip.** Identically positioned buffers compare as two
   slices with no per-cell bounds arithmetic, and `Cell` equality is a bytewise
   comparison of a small `Copy` struct rather than a string comparison.
3. **ASCII fast paths.** `set_string` writes printable ASCII one byte per cell
   with no grapheme segmentation and no width-table lookup — the case that
   covers nearly all TUI text. Reflow does the same when a span is ASCII.
4. **Reflow borrows instead of copying.** A `StyledGrapheme` holds a `&str` into
   the source span, so wrapping a screen of text allocates per output line
   rather than per character.
5. **The escape stream is written directly.** No command objects, no `Display`
   impls, no formatting machinery: integers are formatted by hand into one
   contiguous frame buffer that reaches the writer in a single call. Attribute
   changes emit only the flags that actually changed, so a bold-to-normal
   transition costs one short sequence instead of a reset plus re-asserted
   colors.
6. **Layouts are memoized per thread.** Redraws split the same areas with the
   same constraints forever; a repeat split returns a shared `Rc<[Rect]>`
   without re-solving or allocating. A miss solves with its working set on the
   stack.
7. **Borders are drawn as runs.** Box-drawing glyphs are not ASCII, so a
   `Block` that wrote its border one cell at a time would re-enter grapheme
   segmentation for every cell of the frame. Converting the glyph once and
   stamping it across the row is what took the dashboard from roughly parity
   to several times faster, and roughly doubled the whole frame loop, measured
   before and after in one sitting.
8. **A scalar fast path between ASCII and full segmentation.** A character that
   cannot be extended, followed by another that cannot extend it, is a cluster
   on its own — true for Latin, Greek, Cyrillic, CJK, kana, and Hangul
   syllables. Those skip the segmenter entirely and take their width from the
   range they matched instead of the width tables. Emoji, Indic, Arabic,
   Hebrew, and conjoining jamo still go through full segmentation, and the
   ranges are chosen to prove a character cannot be extended rather than to
   guess.
9. **Features that cost nothing when unused.** Hyperlink targets live in a
   sparse table beside the grid rather than in `Cell`, so a frame with no links
   adds no per-cell memory and no per-cell comparison — the diff checks one
   boolean for the whole buffer. Capability should not be paid for by the
   frames that do not use it.

The text fast paths are chosen per *run*, not per string: a line of mostly
ASCII with one emoji in it takes the byte-per-cell path for the ASCII stretches
and the segmenter only for the emoji. The segmenter is constructed once per
Unicode stretch rather than once per grapheme, and single-scalar clusters use
the per-`char` width table instead of summing over a string.

## Reproducing

```sh
cd benchmarks

# Micro-benchmarks across all three frameworks (Criterion, HTML report in
# target/criterion).
cargo bench

# One workload only
cargo bench --bench shootout -- buffer_diff

# Whole-process frame loop: frames, ms, frames/s, peak RSS in KB, bytes emitted
cargo build --release --bin frameloop
./target/release/frameloop hawk 20000
./target/release/frameloop ratatui 20000
```

The `benchmarks/` crate is a separate, unpublished package: the comparison
dependencies never appear in Hawk TUI's own dependency tree, and `benchmarks/`
is excluded from the published crate.

## Honesty notes

- Numbers come from one machine; absolute values will differ on yours, ratios
  much less so. Re-run before quoting them.
- The frame-loop figures are the best of three interleaved runs per framework,
  which is the least-noisy estimate of throughput. The three runs gave 5.09×,
  4.93×, and 5.15×, so the reported 4.93× is the most conservative of them
  rather than a cherry-picked outlier.
- **This machine is noisy, and the micro-benchmark table says so.** Two full
  Criterion runs, back to back on an otherwise idle machine, disagreed by up to
  2× on absolute times in the same group — the layout solve measured 136 ns in
  one and 266 ns in the other. Hawk TUI was fastest in all sixteen groups in
  both runs, and the published speedups are the smaller of the two, but treat
  the absolute microseconds as one sample rather than a specification. The
  frame loop, which runs whole processes for seconds at a time instead of
  microseconds in a loop, is the steadier measurement of the two.
- Criterion warm-up favors any framework with a cache. ratatui's layout cache
  and Hawk's are both warm in the layout group, which is the fair comparison for
  a redraw loop; a cold split costs Hawk ~440 ns.
- The widget-render group uses each framework's own widgets. They are not
  pixel-identical implementations, so it measures "what it costs to draw this
  screen with this library", not an isolated algorithm. The frame-loop harness
  above pins down the fairness of that comparison with cell and byte counts.
- SuperLightTUI's buffer carries an optional hyperlink per cell and a clip
  stack, which is a feature difference, not only an implementation one; its
  numbers should be read with that in mind.
