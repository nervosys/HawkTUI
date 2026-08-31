export default function Layout() {
  return (
    <>
      <h1>Layout</h1>
      <p>
        Hawk TUI&apos;s layout engine divides rectangular areas into sub-areas using
        constraints. It supports vertical and horizontal splitting, margins,
        spacing, and flexible distribution.
      </p>

      <h2>Basic Usage</h2>
      <pre><code>{`use hawktui::layout::{Layout, Constraint, Direction};

let chunks = Layout::new(
    Direction::Vertical,
    &[
        Constraint::Length(3),       // Fixed 3 rows
        Constraint::Fill(1),         // Remaining space
        Constraint::Length(1),       // Fixed 1 row
    ],
)
.split(frame.area());

frame.render_widget(header, chunks[0]);
frame.render_widget(content, chunks[1]);
frame.render_widget(status_bar, chunks[2]);`}</code></pre>

      <h2>Constraints</h2>
      <table>
        <thead><tr><th>Constraint</th><th>Description</th></tr></thead>
        <tbody>
          <tr><td><code>Length(n)</code></td><td>Exactly <code>n</code> cells</td></tr>
          <tr><td><code>Percentage(p)</code></td><td><code>p%</code> of available space</td></tr>
          <tr><td><code>Min(n)</code></td><td>At least <code>n</code> cells</td></tr>
          <tr><td><code>Max(n)</code></td><td>At most <code>n</code> cells</td></tr>
          <tr><td><code>Fill(weight)</code></td><td>Fill remaining space (proportional by weight)</td></tr>
          <tr><td><code>Ratio(num, den)</code></td><td>Fraction <code>num/den</code> of available space</td></tr>
        </tbody>
      </table>

      <h2>Direction</h2>
      <pre><code>{`// Vertical: split rows (top to bottom)
Layout::vertical(&[Constraint::Length(3), Constraint::Fill(1)])

// Horizontal: split columns (left to right)
Layout::horizontal(&[Constraint::Percentage(30), Constraint::Fill(1)])`}</code></pre>

      <h2>Flex Distribution</h2>
      <p>
        Control how remaining space is distributed when constraints don&apos;t
        consume the full area:
      </p>
      <pre><code>{`use hawktui::layout::Flex;

Layout::vertical(&[Constraint::Length(5), Constraint::Length(5)])
    .flex(Flex::Center)  // Center the two blocks vertically
    .split(area);`}</code></pre>
      <table>
        <thead><tr><th>Flex</th><th>Behavior</th></tr></thead>
        <tbody>
          <tr><td><code>Start</code></td><td>Pack items at the start (default)</td></tr>
          <tr><td><code>Center</code></td><td>Center items in the area</td></tr>
          <tr><td><code>End</code></td><td>Pack items at the end</td></tr>
          <tr><td><code>SpaceBetween</code></td><td>Equal space between items</td></tr>
          <tr><td><code>SpaceAround</code></td><td>Equal space around items</td></tr>
        </tbody>
      </table>

      <h2>Margins &amp; Spacing</h2>
      <pre><code>{`Layout::vertical(&[Constraint::Fill(1), Constraint::Fill(1)])
    .margin(Margin { horizontal: 2, vertical: 1 })
    .spacing(1)
    .split(area);`}</code></pre>

      <h2>Nested Layouts</h2>
      <p>Compose layouts by splitting sub-areas:</p>
      <pre><code>{`let outer = Layout::vertical(&[
    Constraint::Length(3),
    Constraint::Fill(1),
]).split(frame.area());

let inner = Layout::horizontal(&[
    Constraint::Percentage(30),
    Constraint::Fill(1),
]).split(outer[1]);

frame.render_widget(sidebar, inner[0]);
frame.render_widget(main_content, inner[1]);`}</code></pre>
    </>
  );
}
