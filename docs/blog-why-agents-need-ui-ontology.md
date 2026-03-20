# Why AI Agents Need a UI Ontology, Not Screen-Scraping

*By [NERVOSYS](https://github.com/nervosys) — March 2026*

---

Every AI coding agent on the market has the same dirty secret: when it needs to interact with a terminal UI, it's blind.

Claude Code, Codex, OpenCode, Cursor — they all hit the same wall. The moment an agent needs to read a TUI's output or control its widgets, it falls back to the crudest possible technique: parsing raw terminal text. It reads ANSI escape sequences character by character, guesses where widgets begin and end, and hopes the layout doesn't change.

This is screen-scraping. It's the same fragile approach web automated tests abandoned a decade ago in favor of structured selectors. And yet, in 2026, it's still how the most sophisticated AI systems in the world interact with terminal applications.

We built [Louie](https://github.com/nervosys/louie) to fix this.

## The problem in concrete terms

Picture an AI agent trying to use a terminal application. Here's what it actually sees:

```
\033[1;1H\033[38;5;4m┌─ Tasks ─────────────────────┐\033[0m
\033[2;1H\033[38;5;4m│\033[0m \033[7m> Review PR #42      \033[0m \033[38;5;4m│\033[0m
\033[3;1H\033[38;5;4m│\033[0m   Deploy staging       \033[38;5;4m│\033[0m
\033[4;1H\033[38;5;4m│\033[0m   Write unit tests     \033[38;5;4m│\033[0m
```

That's a List widget with three items, one selected (highlighted with `\033[7m`). But to the agent, it's an opaque stream of bytes. To extract meaning, the agent must:

1. Parse ANSI CSI sequences to strip formatting
2. Detect box-drawing characters to find widget boundaries
3. Recognize highlighting patterns to determine selection state
4. Track cursor position across frames to detect changes
5. Maintain a mental model of what each screen region means

And this breaks the moment someone changes the border style, reorders the layout, adds a title, or resizes the terminal.

Compare that to what the same agent sees through Louie's protocol:

```json
{
  "type": "query_ontology"
}
```

Response:

```json
{
  "success": true,
  "data": [{
    "name": "List",
    "default_role": "Selection",
    "properties": [
      {"name": "items", "property_type": {"Array": "String"}},
      {"name": "highlight_symbol", "property_type": "String"}
    ],
    "tags": ["list", "selection", "scrollable", "menu"]
  }]
}
```

The widget tells the agent what it is, what it contains, and what you can do with it. No parsing. No guessing. No breakage when the layout changes.

## What an ontology gives you

An ontology is a structured, typed description of entities and their relationships. In the context of a UI framework, every widget publishes:

**Schema** — its type name, properties with types and constraints, tags for search.

```json
{
  "name": "Gauge",
  "properties": [
    {"name": "ratio", "property_type": "Float", "constraints": [{"Min": 0.0}, {"Max": 1.0}]}
  ]
}
```

**Semantic role** — is it an input? A display? A container? Navigation? An agent seeing `"default_role": "Input"` knows it should type into this widget, not just read from it.

**Capabilities** — `Focusable`, `Scrollable`, `Selectable`, `TextInput`, `Searchable`. These are first-class concepts, not things the agent has to infer from behavior.

**Actions** — named operations with typed parameters. Instead of sending `Down Down Down Enter` and hoping, the agent sends:

```json
{
  "type": "execute_action",
  "agent_id": "task-list",
  "action": "select",
  "params": {"index": 2}
}
```

This is the difference between driving a car with a steering wheel and driving a car by reaching through the windshield and pushing the road.

## The architecture

Louie is a Rust TUI framework (think ratatui meets bubbletea) with a structural addition: every widget implements a `Discoverable` trait.

```rust
pub trait Discoverable {
    fn schema() -> WidgetSchema;
    fn capabilities(&self) -> Vec<AgentCapability>;
    fn actions(&self) -> Vec<AgentAction>;
    fn semantic_role(&self) -> SemanticRole;
    fn agent_state(&self) -> serde_json::Value;
    fn execute_action(&mut self, action: &str, params: Value) -> Result<(), String>;
}
```

This metadata is collected into an `OntologyRegistry` that the agent can query at runtime. The registry supports search by name, tag, or semantic role.

The agent protocol runs over stdin/stdout JSON Lines — the same transport pattern used by LSP, MCP, and existing coding agents. The agent spawns `louie-server` as a child process and communicates through structured messages:

```
Agent                              louie-server
  │                                      │
  │──── {"type":"ping"} ────────────────►│
  │◄─── {"success":true,"data":"pong"} ──│
  │                                      │
  │──── {"type":"query_ontology"} ──────►│
  │◄─── [Widget schemas...] ─────────────│
  │                                      │
  │──── {"type":"get_tree"} ────────────►│
  │◄─── [UI tree with state...] ─────────│
  │                                      │
  │──── {"type":"inject_event",...} ────►│
  │◄─── {"status":"injected"} ───────────│
  │                                      │
  │──── {"type":"quit"} ────────────────►│
  │◄─── {"status":"quitting"} ───────────│
```

No terminal needed. No rendering. The agent gets structured access to the entire UI state.

## Why this matters now

Three trends are converging:

**1. Coding agents are becoming operators, not just generators.** Claude Code, Codex, and Devin don't just write code — they run terminals, interact with tools, debug failures. They need to *operate* UIs, not just *generate* them.

**2. Terminal UIs are the natural habitat for developer tools.** Every coding agent runs in a terminal. The tools they interact with (git, kubectl, docker, build systems) are terminal-native. Rich TUIs like lazygit, k9s, and btop are replacing GUIs for developer workflows. The agent needs to drive these interfaces.

**3. The accessibility model doesn't translate.** Web accessibility (ARIA) lets screen readers interact with web UIs structurally. But terminals have no equivalent. There's no a11y tree for a TUI. Louie's ontology fills this gap.

Without an ontology, every agent integration with every TUI is a custom, fragile, one-off screen-scraping adapter. With an ontology, it's a protocol call.

## For the skeptics

**"Why not just use the API directly? Skip the TUI."**

Because the TUI *is* the interface. Many tools don't have programmatic APIs — the TUI is the only way to interact with them. And for tools that do have APIs, the TUI adds context, feedback loops, and state visualization that raw API calls don't provide. The ontology lets agents get the best of both: structured programmatic access *and* the rich state model of the UI.

**"Can't you just use computer vision / multimodal models?"**

You could screenshot a terminal and feed it to GPT-4V. This works for simple cases but fails at scale: it's slow (image encoding is expensive), resolution-limited (terminal text is small), and still fundamentally imprecise (the model is *guessing* at widget boundaries). The ontology is exact, instant, and typed.

**"Won't every TUI need to adopt this?"**

Yes — that's the catch, and it's also the opportunity. New TUI applications built with Louie get agent compatibility for free. For existing applications, the ontology pattern can be retrofitted as a wrapper layer (similar to how ARIA was retrofitted onto web components). The agent protocol is intentionally compatible with the JSON Lines transports already used by LSP and MCP — it's not a foreign concept.

## Try it

```shell
# Clone and build
git clone https://github.com/nervosys/louie
cd louie
cargo build --release --bin louie-server

# Test the protocol
echo '{"type":"ping"}' | ./target/release/louie-server
# → {"success":true,"data":{"status":"pong"}}

# Discover all widgets
echo '{"type":"query_ontology"}' | ./target/release/louie-server
# → Full schemas with types, constraints, capabilities, actions

# Run the interactive demo
python3 scripts/louie-demo.py
```

The protocol spec is at [docs/agent-protocol.md](docs/agent-protocol.md). Integration examples for Python, TypeScript, and Rust are included.

## The bottom line

AI agents interacting with terminal UIs through screen-scraping is like web browsers rendering pages by regex-parsing HTML. It works until it doesn't, and it always doesn't.

A self-describing UI ontology turns a terminal application from an opaque bitmap into a structured API. The agent doesn't need to *see* the UI — it needs to *understand* it.

That's what Louie provides. [Open source under AGPL](https://github.com/nervosys/louie). Commercial licenses available for embedding in proprietary systems.

---

*Louie is built by [NERVOSYS](https://nervosys.ai). We are interested in partnering with organizations looking for faster and safer UI interactions for agentic AI systems. [Get in touch](https://nervosys.ai).*
