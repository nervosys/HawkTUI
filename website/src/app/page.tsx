import Link from "next/link";

const features = [
  { title: "Elm Architecture", desc: "Predictable Model/Update/View cycle with async Command support." },
  { title: "Ontology System", desc: "Every widget exposes schema, capabilities, roles, and actions for agent discovery." },
  { title: "Agent Protocol", desc: "JSON Lines over stdin/stdout with 10 typed request/response pairs." },
  { title: "22 Widgets", desc: "From Paragraph and Input to BarChart, Calendar, Canvas, and Toast." },
  { title: "Animation Engine", desc: "Tweens, springs, 25 easing curves, and timeline sequencing." },
  { title: "Headless Driver", desc: "Run without a terminal for automated testing and CI/CD pipelines." },
];

export default function Home() {
  return (
    <>
      <section className="hero">
        <div className="hero-tag">// AGENTIC TUI FRAMEWORK</div>
        <h1>Louie: The TUI framework for agentic AI</h1>
        <p>
          Louie gives AI agents a complete, discoverable interface to terminal
          applications &mdash; with an Elm architecture, full widget ontology, and
          a JSON Lines protocol.
        </p>
        <div className="install-block">cargo add louie</div>
        <div className="hero-buttons">
          <Link href="/docs/getting-started" className="btn btn-primary">
            Get Started
          </Link>
          <Link href="/docs/getting-started" className="btn btn-outline-docs">
            Documentation
          </Link>
          <Link
            href="https://github.com/nervosys/louie"
            className="btn btn-secondary"
          >
            GitHub
          </Link>
        </div>
        <div className="status-bar">
          <div className="status-item"><span className="status-dot"></span> v1.0.0 STABLE</div>
          <div className="status-item"><span className="status-dot"></span> 217 TESTS PASSING</div>
          <div className="status-item"><span className="status-dot"></span> 22 WIDGETS ACTIVE</div>
        </div>
      </section>

      <div className="feature-grid">
        {features.map((f) => (
          <div key={f.title} className="feature-card">
            <h3>{f.title}</h3>
            <p>{f.desc}</p>
          </div>
        ))}
      </div>

      <section className="comparison-section">
        <h2>How Louie Compares</h2>
        <table>
          <thead>
            <tr>
              <th>Feature</th>
              <th>Louie</th>
              <th>OpenTUI</th>
              <th>pi-tui</th>
              <th>ratatui</th>
              <th>bubbletea</th>
              <th>ink</th>
            </tr>
          </thead>
          <tbody>
            {[
              ["Language", "Rust", "TS/Zig", "TS", "Rust", "Go", "JS/TS"],
              ["Architecture", "Elm", "Component", "Component", "Immediate", "Elm", "React"],
              ["Agent protocol", "\u2713", "\u2717", "\u2717", "\u2717", "\u2717", "\u2717"],
              ["Widget ontology", "\u2713", "\u2717", "\u2717", "\u2717", "\u2717", "\u2717"],
              ["Discoverability", "\u2713", "\u2717", "\u2717", "\u2717", "\u2717", "\u2717"],
              ["Headless driver", "\u2713", "\u2717", "\u2717", "\u2717", "\u2717", "\u2717"],
              ["Animation engine", "\u2713", "\u2717", "\u2717", "\u2717", "\u2717", "\u2717"],
              ["Focus system", "\u2713", "\u2713", "\u2713", "Partial", "\u2713", "\u2713"],
              ["Overlay/modal", "\u2713", "\u2717", "\u2713", "\u2717", "\u2717", "\u2717"],
              ["Async support", "\u2713", "\u2713", "\u2713", "Manual", "\u2713", "\u2713"],
              ["Theme system", "\u2713", "Partial", "\u2713", "Partial", "\u2713", "CSS"],
              ["Widget count", "22", "Primitives", "~12", "15+", "~10", "~12"],
              ["Static export", "\u2713", "\u2713", "\u2717", "\u2717", "\u2713", "\u2713"],
              ["Binary size", "~2 MB", "~4 MB", "~15 MB", "~1 MB", "~5 MB", "~30 MB"],
              ["License", "AGPL-3.0", "MIT", "MIT", "MIT", "MIT", "MIT"],
            ].map((row, i) => (
              <tr key={i}>
                <td style={{ fontWeight: 600, color: "#fff" }}>{row[0]}</td>
                {row.slice(1).map((cell, j) => (
                  <td key={j}>
                    {cell === "\u2713" ? (
                      <span className="check">{cell}</span>
                    ) : cell === "\u2717" ? (
                      <span className="cross">{cell}</span>
                    ) : (
                      cell
                    )}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </section>

      <section className="cta-section">
        <h2>Start building agentic TUIs</h2>
        <p>Get up and running in under five minutes.</p>
        <div className="cta-buttons">
          <Link href="/docs/getting-started" className="btn btn-primary">
            Read the Docs
          </Link>
          <Link
            href="https://github.com/nervosys/louie"
            className="btn btn-secondary"
          >
            View on GitHub
          </Link>
        </div>
      </section>
    </>
  );
}
