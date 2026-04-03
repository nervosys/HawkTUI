export default function Architecture() {
  return (
    <>
      <h1>Architecture</h1>
      <p>
        Louie is organized as a layered architecture with clear separation of
        concerns. Every layer can be used by both human users and AI agents.
      </p>

      <div className="arch-diagram">{`┌──────────────────────────────────────────────────┐
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
│  ├ List / Table   │  └ ModalBox                  │
│  ├ Input / Editor ├──────────────────────────────┤
│  ├ Gauge / Chart  │  Layout                      │
│  ├ Markdown       │  ├ Constraint solver         │
│  ├ Canvas / Image │  ├ Direction (V/H)           │
│  └ 14 more...     │  └ Flex distribution         │
├──────────────────────────────────────────────────┤
│  Core: Buffer, Cell, Style, Text, Reflow, Rect   │
├──────────────────────────────────────────────────┤
│  Backend: Crossterm │ TestBackend                │
└──────────────────────────────────────────────────┘`}</div>

      <h2>Layer Overview</h2>

      <h3>Backend</h3>
      <p>
        Abstracts the terminal. <code>CrosstermBackend</code> provides real
        terminal I/O; <code>TestBackend</code> captures output in-memory for tests.
        Both implement the <code>Backend</code> trait. Synchronized output
        (CSI&nbsp;?2026h/l) prevents partial-frame tearing.
      </p>

      <h3>Core</h3>
      <p>
        Foundational types: <code>Buffer</code> (double-buffered grid of cells),
        <code>Cell</code> (character + style), <code>Rect</code> (area coordinates),
        <code>Style</code>/<code>Color</code>/<code>Modifier</code>, <code>Text</code>/
        <code>Span</code>/<code>Line</code>, and the <code>reflow</code> engine for
        word/char wrapping and line truncation.
      </p>

      <h3>Layout</h3>
      <p>
        Constraint-based layout with <code>Length</code>, <code>Percentage</code>,{" "}
        <code>Min</code>, <code>Max</code>, <code>Fill</code>, and <code>Ratio</code>{" "}
        constraints. Supports vertical/horizontal direction, margins, spacing, and
        flex distribution (Start, Center, End, SpaceBetween, SpaceAround).
      </p>

      <h3>Widgets</h3>
      <p>
        22 widgets, each implementing the <code>Widget</code> trait (render to buffer)
        and the <code>Discoverable</code> trait (expose metadata for agents). See
        the <a href="/docs/widgets">Widget Catalog</a>.
      </p>

      <h3>Focus &amp; Overlay</h3>
      <p>
        <code>FocusManager</code> maintains a focus ring with Tab/Shift+Tab navigation
        and programmatic focus by ID. <code>OverlayStack</code> manages modal layers
        with focus capture.
      </p>

      <h3>Agent Protocol &amp; Ontology</h3>
      <p>
        The ontology system assigns every widget a schema, capabilities, actions,
        and semantic role. The agent protocol layer exposes these over a JSON Lines
        RPC transport. See <a href="/docs/ontology">Ontology</a> and{" "}
        <a href="/docs/agent-protocol">Agent Protocol</a>.
      </p>

      <h3>Animation</h3>
      <p>
        Tweens, springs, 25 easing curves, and timeline sequencing. Used by widgets
        like Loader and available for any custom animation.
      </p>

      <h3>Runtime</h3>
      <p>
        The <code>Program</code> type manages the Elm architecture event loop:
        init &rarr; render &rarr; poll events &rarr; dispatch &rarr; render &rarr; repeat.
        Supports async <code>Command::Task</code> for background work, tick rates,
        and ontology export.
      </p>

      <h2>Data Flow</h2>
      <pre><code>{`Terminal Event (key, mouse, resize)
    ↓
handle_event() → Option<Msg>
    ↓
update(msg) → Command<Msg>
    ↓
view(&self, frame)    ← renders widgets into Buffer
    ↓
Terminal::draw()      ← diffs old/new buffer, writes changed cells`}</code></pre>
      <p>
        No shared mutability, no callbacks &mdash; pure data flow from events to
        state to rendering.
      </p>
    </>
  );
}
