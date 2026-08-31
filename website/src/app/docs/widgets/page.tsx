export default function Widgets() {
  const widgets = [
    {
      name: "Block",
      desc: "Container with borders, titles (top/bottom), and title alignment.",
      role: "Container",
      caps: ["Focusable"],
    },
    {
      name: "Paragraph",
      desc: "Styled text with word wrap, char wrap, and line truncation via the reflow engine.",
      role: "Display",
      caps: ["Scrollable"],
    },
    {
      name: "List",
      desc: "Selectable list with highlight styling and top-to-bottom/bottom-to-top direction.",
      role: "Selection",
      caps: ["Focusable", "Scrollable", "Selectable"],
    },
    {
      name: "Tabs",
      desc: "Tab bar for view navigation with selectable tabs.",
      role: "Navigation",
      caps: ["Focusable", "Selectable"],
    },
    {
      name: "Gauge",
      desc: "Progress bar with ratio or percentage display.",
      role: "Progress",
      caps: ["RangeEditable"],
    },
    {
      name: "LineGauge",
      desc: "Thin single-line progress bar with line-drawing characters.",
      role: "Progress",
      caps: ["RangeEditable"],
    },
    {
      name: "Input",
      desc: "Single-line text input with cursor, placeholder, and programmatic text control.",
      role: "TextInput",
      caps: ["Focusable", "TextInput"],
    },
    {
      name: "Editor",
      desc: "Multi-line text editor with line numbers, cursor movement, and viewport scrolling.",
      role: "TextInput",
      caps: ["Focusable", "TextInput", "Scrollable", "Copyable"],
    },
    {
      name: "Table",
      desc: "Data table with columns, row selection, sorting, and scrolling.",
      role: "DataVisualization",
      caps: ["Focusable", "Scrollable", "Selectable", "Sortable"],
    },
    {
      name: "Markdown",
      desc: "Markdown renderer with headings, bold, italic, code spans, code blocks, lists, and blockquotes.",
      role: "Display",
      caps: ["Scrollable"],
    },
    {
      name: "SelectList",
      desc: "Interactive single/multi-select list with keyboard navigation and filter/search.",
      role: "Selection",
      caps: ["Focusable", "Selectable", "Searchable"],
    },
    {
      name: "Loader",
      desc: "Animated spinner with message text. Multiple styles: braille, dots, line, arc.",
      role: "Progress",
      caps: ["Animated"],
    },
    {
      name: "CancellableLoader",
      desc: "Loader with a cancel action for long-running operations.",
      role: "Progress",
      caps: ["Animated"],
    },
    {
      name: "Sparkline",
      desc: "Inline data trend chart using block characters.",
      role: "DataVisualization",
      caps: [],
    },
    {
      name: "Scrollbar",
      desc: "Scrollbar indicator for content that overflows its area.",
      role: "Navigation",
      caps: ["Scrollable"],
    },
    {
      name: "Canvas",
      desc: "Braille-resolution drawing surface for custom graphics.",
      role: "Display",
      caps: [],
    },
    {
      name: "BarChart",
      desc: "Grouped bar chart with vertical/horizontal direction and sub-cell precision.",
      role: "DataVisualization",
      caps: [],
    },
    {
      name: "Chart",
      desc: "XY line/scatter plot with braille dots, axes, legend, and multiple datasets.",
      role: "DataVisualization",
      caps: [],
    },
    {
      name: "Image",
      desc: "Inline terminal image with Kitty, iTerm2, and fallback text protocols.",
      role: "Display",
      caps: [],
    },
    {
      name: "Calendar",
      desc: "Month-view calendar grid with day-of-week headers and day highlighting.",
      role: "Display",
      caps: [],
    },
    {
      name: "SettingsList",
      desc: "Key-value settings list with cycleable values and optional descriptions.",
      role: "Selection",
      caps: ["Focusable", "Selectable"],
    },
    {
      name: "ModalBox",
      desc: "Centered modal overlay with dimmed background and focus capture.",
      role: "Container",
      caps: ["Focusable"],
    },
  ];

  return (
    <>
      <h1>Widget Catalog</h1>
      <p>
        Hawk TUI ships 22 widgets, each implementing both the <code>Widget</code> trait
        (for rendering) and the <code>Discoverable</code> trait (for agent
        discoverability).
      </p>

      <div className="widget-grid">
        {widgets.map((w) => (
          <div key={w.name} className="widget-card">
            <h3>{w.name}</h3>
            <span className="role-badge">{w.role}</span>
            <p>{w.desc}</p>
            {w.caps.length > 0 && (
              <div>
                {w.caps.map((c) => (
                  <span key={c} className="cap-badge">{c}</span>
                ))}
              </div>
            )}
          </div>
        ))}
      </div>

      <h2>Widget and StatefulWidget Traits</h2>
      <pre><code>{`pub trait Widget {
    fn render(self, area: Rect, buf: &mut Buffer);
}

pub trait StatefulWidget {
    type State;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State);
}`}</code></pre>
      <p>
        Stateless widgets (Paragraph, Block, Gauge) implement <code>Widget</code>.
        Stateful widgets (List, Table, SelectList) implement <code>StatefulWidget</code>{" "}
        and track selection, scroll offset, etc. in their <code>State</code> type.
      </p>

      <h2>Rendering</h2>
      <pre><code>{`// Stateless
frame.render_widget(
    Paragraph::new("Hello").block(Block::default().borders(Borders::ALL)),
    area,
);

// Stateful
let mut list_state = ListState::default().with_selected(Some(0));
frame.render_stateful_widget(
    List::new(items).highlight_symbol("> "),
    area,
    &mut list_state,
);`}</code></pre>

      <h2>Discoverable</h2>
      <p>
        Every widget also implements the <code>Discoverable</code> trait. See the{" "}
        <a href="/docs/ontology">Ontology</a> docs for details on how agents
        discover and interact with widgets.
      </p>
    </>
  );
}
