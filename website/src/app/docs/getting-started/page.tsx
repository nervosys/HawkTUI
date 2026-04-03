export default function GettingStarted() {
  return (
    <>
      <h1>Getting Started with Louie</h1>
      <p>
        Louie is an agentic-first TUI framework in Rust. It combines a modern Elm
        architecture with a structured ontology that lets AI agents discover,
        inspect, and interact with every widget in your application.
      </p>

      <h2>What is Louie?</h2>
      <p>
        Traditional TUI frameworks are built for humans. Louie is built for both:
      </p>
      <ul>
        <li>
          <strong>For humans</strong>: Elm architecture, immediate-mode rendering,
          animation system, rich widget set, and focus/overlay management.
        </li>
        <li>
          <strong>For agents</strong>: Every widget exposes its schema,
          capabilities, actions, and semantic role through a typed ontology. An
          agent can ask <em>&quot;What widgets exist? What actions are available?&quot;</em>{" "}
          and get structured JSON answers.
        </li>
      </ul>

      <h2>Key Features</h2>
      <ul>
        <li><strong>Elm Architecture</strong> &mdash; Predictable Model / Update / View cycle with async Command support.</li>
        <li><strong>Ontology System</strong> &mdash; Every widget exposes schema, capabilities, roles, and actions for agent discovery.</li>
        <li><strong>Agent Protocol</strong> &mdash; JSON Lines over stdin/stdout with 10 typed request/response pairs.</li>
        <li><strong>22 Widgets</strong> &mdash; From Paragraph and Input to BarChart, Calendar, Canvas, and Toast.</li>
        <li><strong>Animation Engine</strong> &mdash; Tweens, springs, 25 easing curves, and timeline sequencing.</li>
        <li><strong>Headless Driver</strong> &mdash; Run without a terminal for automated testing and CI/CD pipelines.</li>
        <li><strong>Focus &amp; Overlay</strong> &mdash; Focus ring navigation and modal/overlay stack.</li>
        <li><strong>Theme System</strong> &mdash; Token-based styling with built-in dark/light themes.</li>
      </ul>

      <h2>Next Steps</h2>
      <ul>
        <li><a href="/docs/installation">Installation</a> &mdash; Add Louie to your project.</li>
        <li><a href="/docs/quick-start">Quick Start</a> &mdash; Build your first Louie app in under 5 minutes.</li>
        <li><a href="/docs/architecture">Architecture</a> &mdash; Understand how Louie is structured.</li>
        <li><a href="/docs/agent-protocol">Agent Protocol</a> &mdash; Connect an AI agent to a Louie app.</li>
      </ul>
    </>
  );
}
