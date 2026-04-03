export default function Ontology() {
  return (
    <>
      <h1>Ontology System</h1>
      <p>
        The ontology system is what makes Louie <em>agentic-first</em>. Every widget
        exposes structured metadata that AI agents can query at runtime &mdash; no
        hardcoded assumptions, no screen-scraping, no trial-and-error.
      </p>

      <h2>The Discoverable Trait</h2>
      <p>
        Every widget implements <code>Discoverable</code>:
      </p>
      <pre><code>{`pub trait Discoverable {
    fn schema() -> WidgetSchema;                      // Type name, properties, constraints
    fn capabilities(&self) -> Vec<AgentCapability>;   // What it can do
    fn actions(&self) -> Vec<AgentAction>;            // Named operations
    fn semantic_role(&self) -> SemanticRole;           // Purpose category
    fn agent_state(&self) -> serde_json::Value;       // Current state as JSON
    fn execute_action(
        &mut self, action: &str, params: &serde_json::Value,
    ) -> Result<serde_json::Value, String>;

    // Optional:
    fn agent_id(&self) -> Option<&str> { None }
    fn accessibility_label(&self) -> Option<String> { None }
}`}</code></pre>

      <h2>Widget Schema</h2>
      <p>
        The schema describes a widget&apos;s type, properties, and constraints:
      </p>
      <pre><code>{`{
  "name": "Input",
  "description": "A text input field with cursor management.",
  "default_role": "TextInput",
  "properties": [
    {
      "name": "placeholder",
      "description": "Placeholder text shown when empty.",
      "property_type": "String",
      "required": false
    }
  ],
  "tags": ["input", "text", "form", "edit"]
}`}</code></pre>

      <h2>Semantic Roles</h2>
      <p>
        Roles categorize widgets by purpose, letting agents find widgets by
        <em> what they do</em> rather than what they&apos;re called:
      </p>
      <table>
        <thead><tr><th>Role</th><th>Description</th><th>Example Widgets</th></tr></thead>
        <tbody>
          <tr><td><code>Container</code></td><td>Groups other widgets</td><td>Block</td></tr>
          <tr><td><code>Display</code></td><td>Shows read-only content</td><td>Paragraph, Image, Calendar</td></tr>
          <tr><td><code>TextInput</code></td><td>Accepts text input</td><td>Input, Editor</td></tr>
          <tr><td><code>Selection</code></td><td>Choose from options</td><td>List, Tabs, SelectList</td></tr>
          <tr><td><code>Progress</code></td><td>Shows progress</td><td>Gauge, LineGauge, Loader</td></tr>
          <tr><td><code>Navigation</code></td><td>Navigates between views</td><td>Tabs</td></tr>
          <tr><td><code>DataVisualization</code></td><td>Visualizes data</td><td>BarChart, Chart, Sparkline</td></tr>
        </tbody>
      </table>

      <h2>Capabilities</h2>
      <p>
        18 capability types describe what interactions a widget supports:
      </p>
      <ul>
        <li><code>Focusable</code> &mdash; Can receive keyboard focus</li>
        <li><code>Clickable</code> &mdash; Responds to mouse clicks</li>
        <li><code>Scrollable</code> &mdash; Has scrollable content</li>
        <li><code>TextInput</code> &mdash; Accepts text entry</li>
        <li><code>Selectable</code> &mdash; Has selectable items</li>
        <li><code>RangeEditable</code> &mdash; Editable numeric range (e.g. gauges)</li>
        <li><code>Sortable</code> &mdash; Columns/items can be sorted</li>
        <li><code>Searchable</code> &mdash; Supports filter/search</li>
        <li><code>Copyable</code> &mdash; Content can be copied</li>
        <li><code>Animated</code> &mdash; Has animation state</li>
        <li><code>HasKeyBindings</code> &mdash; Responds to keyboard shortcuts</li>
      </ul>

      <h2>Actions</h2>
      <p>
        Named operations with typed parameters that agents can invoke:
      </p>
      <pre><code>{`AgentAction {
    name: "set_text",
    description: "Set the input text content.",
    parameters: [
        ActionParam {
            name: "text",
            param_type: ActionParamType::String,
            required: true,
            description: "The text to set.",
        }
    ],
}`}</code></pre>
      <p>
        Actions are validated against the parameter schema before execution,
        preventing injection attacks (INJ-2).
      </p>

      <h2>Ontology Registry</h2>
      <p>
        The registry provides a searchable catalog of all widget types and a live
        UI tree of instantiated widgets:
      </p>
      <pre><code>{`let mut registry = OntologyRegistry::new();
registry.register::<Block>();
registry.register::<Paragraph>();
registry.register::<Input>();

// Search by semantic role
let inputs = registry.find_by_role(SemanticRole::TextInput);

// Export full JSON catalog
let catalog = registry.export_catalog();`}</code></pre>

      <h2>UI Tree</h2>
      <p>
        The live UI tree (<code>UiTree</code>) exposes every widget instance with
        its agent ID, type, role, state, label, bounds, and capabilities. Agents
        query this via the <code>get_tree</code> protocol request.
      </p>
    </>
  );
}
