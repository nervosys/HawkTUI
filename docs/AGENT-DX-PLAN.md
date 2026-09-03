# Plan: making Hawk TUI efficient and reliable for agents

Hawk TUI's positioning is "agentic-first with a complete ontology for agent
discoverability". This document is an audit of how far the shipped artifacts
actually deliver that, and a prioritised plan to close the gap. It is written
against measurements, not intentions; each measurement is reproducible from the
commands given.

The plan is deliberately contingent. The [agentic
benchmark](AGENTIC-BENCHMARKS.md) exists to tell us which of these items pay
off, and §6 says what to do if it reports that the ontology does not help
authoring at all.

---

## 1. What the audit found

### 1.1 The ontology describes runtime state, not the authoring API

**Fixed** by the authoring catalog in Phase 1.1 below. The measurements in this
section describe the state that motivated it.

21 widgets implement `Discoverable`. Their schemas describe 40 properties in
total. The same widgets expose 180 public builder and mutator methods.

```sh
cargo run --example ontology_query -- export   # the catalog
# see benchmarks/agentic/ for the coverage script
```

| | |
|---|---|
| Public builder/mutator methods across the 21 widgets | 180 |
| Methods described by an ontology property | **22 (12%)** |
| Widgets with **zero** overlap between schema and API | **8 of 21** |

`Table` (0 of 9 methods), `Tabs` (0 of 6), `SettingsList` (0 of 15) and
`Scrollbar` (0 of 8) publish schemas that describe none of the API an author
must call. `Gauge` publishes `ratio` and `label` but not `percent`, `block`,
`style` or `gauge_style`.

This is coherent for the ontology's original purpose — an agent *driving* a
running app wants to know a gauge's ratio, not that a builder method exists.
It is close to useless for an agent *writing* the program, which is the
expensive half of the work.

### 1.2 The ontology says nothing about the scaffolding backbone

No schema covers `Layout`, `Constraint`, `Direction`, `Flex`, `Frame`,
`Terminal`, the event types, or the `Widget` / `StatefulWidget` split and which
widgets need a companion state type. Those are what a real program is made of:
tasks T3–T6 in the benchmark turn almost entirely on them, and a widget
property catalog cannot help with any of it.

### 1.3 A `cargo add hawktui` user receives almost no documentation

`Cargo.toml` has `exclude = ["reference/", ".github/", "docs/", "scripts/",
"website/", "benchmarks/"]`. Everything in `docs/` — the agent protocol, the
integration guide — is excluded from the published crate.

| Crate | Docs shipped inside the package |
|---|---|
| **hawktui** | README only — **24.6 KB** |
| ratatui 0.29 | README + 32 examples — 385 KB |
| superlighttui 0.23 | README + `docs/` — **487 KB**, including `docs/llms.txt` and `docs/AI_GUIDE.md` |

superlighttui's `llms.txt` opens with "Designed for fast iteration and
AI-assisted TUI code generation" and gives coding agents an explicit reading
order across 30 documents. On the axis Hawk TUI claims as its differentiator, a
competitor currently ships twenty times more agent-facing material — and it did
so without an ontology.

### 1.4 The live registry was populated by hand, and was 71% incomplete

`src/bin/hawktui_server.rs:171` registers six widget types: `Paragraph`,
`Gauge`, `List`, `Block`, `Input`, `SelectList`. The other fifteen implement
`Discoverable` and are never registered, so an agent that calls
`query_ontology` against a running server saw under a third of the catalog and
had no way to discover the rest. Nothing failed when a new widget was added
and not registered.

**Fixed** in Phase 0.2 below: the list moved into the library and a test now
fails when it falls behind.

### 1.5 Agents could not easily check their own work

`TestBackend` exposed exactly three methods: `new`, `buffer`, `row_text`.
`Buffer` had no text accessor at all, so asserting on a rendered screen meant
walking cells and concatenating symbols by hand — and getting the
double-width rule right while doing it. There was no whole-screen text dump,
no snapshot assertion, and there is still no headless key-driving harness.

**Partly fixed** in Phase 2.1 below. The snapshot assertion (2.3) and the
headless driver (2.2) are still open.

This matters more than it looks. An agent that can render a frame and read it
back converges by checking; an agent that cannot, guesses and ships. Every
program in the benchmark still has to hand-roll the key-driving half of this
harness before it can be scored at all.

### 1.6 The agent protocol is bespoke

`hawktui-server` speaks a hand-rolled JSON-Lines protocol documented in
`docs/agent-protocol.md`. Agent platforms speak MCP. Every integrator must
write an adapter that MCP would have given them, and the protocol's reach is
limited to whoever reads that document.

---

## 2. Principles

1. **Ship it inside the crate.** Documentation that lives only in the git
   repository does not exist for a user who ran `cargo add`.
2. **Generate, never hand-maintain.** Every hand-maintained catalog drifts;
   §1.4 is that drift already happening. Derive from the type system and fail
   the build when the derivation is incomplete.
3. **Describe the authoring API, not only the runtime state.** The ontology's
   audience is now two different agents with two different questions.
4. **Make self-verification cheap.** The highest-leverage reliability work is
   letting the agent see what it rendered.
5. **Measure with the benchmark.** Every item below names the metric it should
   move. If it does not move it, it was not worth doing.

---

## 3. Phase 0 — ship what already exists (days)

Cheap, certain, and independent of any benchmark result.

**0.1 Put documentation in the package.** ✅ **Done.** `exclude` no longer
drops all of `docs/`; only the internal roadmap, the security audit and the
marketing post are withheld. Added a top-level `AGENTS.md` (how to write a Hawk
TUI program — the program shape, the four things that most often go wrong, and
how to assert on a rendered screen) and an `llms.txt` reading order.

Shipped agent-facing documentation went from **24.6 KB to 86 KB**, against a
stated target of 150 KB. The target is not met and is not worth meeting by
padding: byte count was a poor proxy borrowed from the competitor comparison.
The honest scoreboard is that superlighttui still ships more, and the thing to
measure is whether C1 `score` and `api_errors` move — not the file sizes.

Every code snippet in `AGENTS.md` was compiled before it shipped. A guide that
misstates the API is worse than no guide: it is a hallucination carrying the
framework's authority.

**0.2 Register every discoverable widget.** ✅ **Done.** Replace the hand-written list in
`hawktui_server.rs` with a single `register_builtin_widgets(&mut registry)` in
the library, and add a test asserting that every type implementing
`Discoverable` appears in it. *Target: 21 of 21.*

`ontology::register_builtin_widgets` now registers all 21, and
`tests/ontology_registry_tests.rs` scans `src/widget/` for
`impl Discoverable for` and fails when one is missing — verified by
removing a registration and watching it fail. `hawktui-server` calls it, so
`query_ontology` answers with 21 widget types instead of 6.

**0.3 Ship the ontology query tool.** ✅ **Done.** `hawktui-ontology` is a
shipped binary — `list`, `search`, `schema`, `roles`, `digest`, `export` — so
the ontology is readable from an installed crate rather than only from a
checkout. The formatting moved into `ontology::report`, which returns `String`
and is unit-tested; the binary and `examples/ontology_query.rs` are both thin
wrappers over it, verified to produce byte-identical output.

*Metric: this is what condition C3 tests. The running grid still points C3 at
`cargo run --manifest-path … --example ontology_query`, because changing a
condition mid-grid would make its runs incomparable. Regenerate the context
packs before the next grid.*

## 4. Phase 1 — make the ontology answer authoring questions (weeks)

This is the substantive bet, and the one the benchmark is designed to judge.

**1.1 Add an authoring section to `WidgetSchema`.** ✅ **Done**, though not
where this item expected. Rather than extend `WidgetSchema` — which describes
runtime state and has a different audience — the authoring API became its own
catalog, `ontology::api`, generated from the signatures by
`scripts/gen_api_ontology.py`.

**72 types and 350 functions**, against 40 properties before. It carries what
`WidgetSchema` structurally could not:

- whether a type renders as `Widget` or `StatefulWidget`, **and the state type
  it pairs with** — the mistake `AGENTS.md` lists second, now a typed field
- full signatures with argument types, so `percent(mut self, percent: u16)`
  rather than a property called `ratio`
- the core types a program is built from — `Layout`, `Constraint` with all six
  variants, `Rect`, `Style`, `Text` — which §1.2 noted the ontology described
  *not at all*
- `render_call()`, which emits the exact line including the state variable

Reachable as `hawktui-ontology api|api-search|stateful` and as the MCP tools
`widget_api`, `api_search` and `stateful_widgets`.

**1.2 Make coverage a build failure.** ✅ **Done.**
`tests/api_ontology_tests.rs` enumerates `pub fn` across `src/widget/` and fails
on anything the catalog omits, runs the generator with `--check` so a stale
file fails CI, and asserts the core types are present and that every stateful
widget names a state type the catalog also describes.

It earned its keep immediately: the first run failed with five stateful widgets
instead of six. `SettingsList` implements **both** `Widget` and
`StatefulWidget`, and the generator let the later impl overwrite the earlier,
silently dropping the state pairing. That is the same drift that left the
runtime registry 29% populated in §1.4 — caught in a minute this time.

**1.3 Extend the ontology past widgets.** ✅ **Partly done.** `Layout`,
`Constraint`, `Rect`, `Style`, `Text` and `Buffer` are in the authoring catalog
with their functions and, for enums, their variants. `Frame`, `Terminal` and the
event types are declared in the generator's module list but carry few public
functions of their own; the `Model` trait's three required methods are still
prose in `AGENTS.md` rather than catalog entries.

**1.4 Compile the usage hints.** ✅ **Done.** `usage_hint` is a free-text string today
(`"Gauge::new().percent(42).label(\"Loading...\")"`) that nothing checks. Emit
them into a generated test file and compile it, so a hint that stops compiling
breaks CI. A wrong hint is worse than no hint: it is a hallucination with the
framework's authority behind it.

`tests/usage_hints_compile.rs` writes every code-shaped hint out as real code,
then reads its own source back and asserts each schema hint appears in it, so a
schema edit cannot drift from the code that proves it. Verified by breaking a
hint and watching the test name it.

**Two of the seventeen code hints were wrong** and had been shipping:

| Widget | Was | Problem |
|---|---|---|
| `Canvas` | `.line(CanvasLine { .. })` | `{ .. }` is not Rust |
| `Table` | `TableColumn::new("Name", Fill)` | `Fill` unqualified; it is `TableColumnWidth::Fill` |

Both are fixed. The other fifteen compiled exactly as written. Four widgets
(`BarChart`, `Chart`, `Image`, `SettingsList`) carry prose rather than code; the
test pins that set so a hint cannot silently degrade into a sentence.

**1.5 Consider `#[derive(Discoverable)]`.** A proc macro would make 1.1 and 1.2
structural rather than a discipline. Larger lift; do it only after 1.1 proves
the payload is worth the machinery.

## 5. Phase 2 — let agents verify their own work (weeks)

**2.1 `Buffer::to_text()` and `TestBackend::to_text()`.** ✅ **Done.** A
whole-screen plain text dump, plus `Buffer::row_text`. Wide graphemes
contribute one character and their trailing cell contributes none, so the
text lines up with what the terminal shows. `TestBackend::row_text` now
delegates rather than duplicating the loop, keeping its trimming behaviour.

**2.2 A `hawktui::testing` module.** ✅ **Done.** `Harness::new(model, w, h)`
plus `run_script("Down,Down,q")` returns one frame per key as plain text —
precisely the contract in `benchmarks/agentic/tasks/contract.md`, in three
lines. It folds `Command::Message` and `Command::Batch` back in as the runtime
does and stops on `Command::Quit`, but deliberately does not run
`Command::Task`, because a harness that sometimes spawns a thread is not
deterministic. `parse_key` covers the script vocabulary and rejects unknown
names rather than skipping them.

12 new tests, including a stateful-list model that reproduces the T3 assertions
(marker travels one row per key, status bar tracks it, selection clamps at both
ends).
*Metric: `first_build_clean` and `score`; not yet measured — the running grid
is pinned to a snapshot that predates this.*

**2.3 Snapshot assertions.** ✅ **Done.** `assert_frame!(actual, expected)` and
`testing::frame_diff` report the first differing row and column, plus a height
mismatch when the counts differ, instead of printing two screens. Trailing
padding is ignored on both sides so an expected screen needs no manual padding.

## 6. Phase 3 — meet agents where they are (weeks)

**3.1 An MCP server.** ✅ **Partly done.** `hawktui-mcp` speaks JSON-RPC 2.0
over stdio and exposes the ontology as five tools: `list_widgets`,
`get_widget_schema`, `search_widgets`, `widget_roles`, `ontology_digest`.
`agent::mcp::McpServer` is transport-free — hand it a line, get a response line
— so it is unit-tested without spawning a process (12 tests), and the binary was
driven end to end over a pipe through a real initialize handshake.

Design points worth keeping: a notification receives no reply, as JSON-RPC
requires; a tool that ran and could not answer returns `isError` **content** so
the model can read it, while a malformed request returns a JSON-RPC error; and
the handshake accepts three protocol revisions, falling back to ours when a
client names one we do not know.

**Still to do:** the runtime half. `HeadlessDriver::process_request` already maps
`AgentRequest` to `AgentResponse`, so `execute_action`, `get_state`,
`inject_event` and `get_tree` are an adapter away — but they need a running
model to drive, which means deciding how a client names the application to
launch. Ontology tools work standalone, which is why they came first.

**3.2 `llms.txt` on the docs site and in the package.** ✅ **Done** for the
package (see 0.1). Still to do: serve it from the docs site.

## 7. Phase 4 — keep it from regressing

**4.1 Benchmark gate.** ✅ **Done**, with one deliberate deviation: it runs
**weekly, not nightly**, because every run costs real money and a nightly cadence
buys little against a model-release timescale.

`.github/workflows/agentic-benchmark.yml` runs the grid, re-scores from the
stored dumps, analyses, and gates on `check_regression.py`. It is off unless the
repository variable `RUN_AGENTIC_BENCHMARK` is `true`, never runs on push or
pull request, and is additionally pinned to this repository so a fork with the
secret cannot fire it. Its first step runs both verifier self-tests: if the
instrument cannot tell a correct program from a broken one, nothing after it
means anything.

`check_regression.py` gates on `score` and `api_errors` only, with tolerances
(0.05 and 0.5) sized to the noise this benchmark is known to carry. Cost and
wall time are reported but never fail the build — they move with pricing,
machine load and model latency, none of which is a property of the framework,
and gating on them would produce false alarms that train people to ignore the
gate. Verified against six synthetic scenarios: baseline creation, identical
rerun, a score drop, a change within tolerance, an API-error rise, and a
baseline cell the run does not cover.

The default task set is `t2-counter,t3-list,t7-settings` — two cheap rungs that
catch a broken framework, plus one discovery rung that can actually vary.

---

## 8. If the benchmark says the ontology does not help authoring

The falsification criteria are pre-registered in
[AGENTIC-BENCHMARKS.md](AGENTIC-BENCHMARKS.md). The honest reading of §1.1 is
that **C2 and C3 may well show little effect today**, because a catalog that
covers 12% of the authoring API and none of the layout system cannot answer the
questions T3–T6 actually pose.

That would not falsify the ontology idea; it would falsify the *current
ontology's* fitness for authoring, which is what Phase 1 exists to fix. The
decision rule:

- **C2/C3 beat C1 today** → Phase 1 is confirmed; prioritise it over Phase 0.
- **C2/C3 ≈ C1 today, and Phase 1 moves them** → the mechanism is real and the
  original schema was simply aimed at the other audience. Say exactly that.
- **C2/C3 ≈ C1 even after Phase 1** → the ontology's value is runtime
  introspection, full stop. Redirect the authoring effort into Phase 0 and
  Phase 2, and change the crate description, which currently sells the ontology
  as the headline feature.

The third outcome is a real possibility and the plan should survive it. Phase 0
and Phase 2 are worth doing under every branch, which is why they are cheap and
first.

### The verdict, from 70 runs

**C2/C3 ≈ C1 on every pre-registered metric**, across nine tasks including three
built so the agent has to discover a widget rather than be told which one to
use. See [AGENTIC-BENCHMARKS.md](AGENTIC-BENCHMARKS.md) § Results.

That is the **second** branch, not the third, and the distinction matters. The
rule's third branch requires "C2/C3 ≈ C1 *even after Phase 1*", and Phase 1 has
not been built. What the data shows is that **the ontology as it stands does not
help authoring** — which §1.1 predicted from the 12 % coverage figure.

It does not follow that a schema covering 90 % would help, and it does not
follow that it would not. The experiment cannot distinguish those, for a reason
worth stating plainly: **`score` never left 1.00 and `api_errors` fired in 2 runs
out of 70.** With no failures to prevent, no condition could have shown a
benefit. The instrument lacked the resolution to test Phase 1's premise, so the
premise is untested rather than refuted.

**Decision.** Do not fund 1.1, 1.2, 1.3 or 1.5 yet — not because they are
disproven, but because nothing currently measures whether they work, and a
multi-week schema project justified by an untestable hypothesis is how
frameworks acquire features nobody needed. The prerequisite is a harder ladder:
a rung where a competent agent measurably fails.

**Update, after the complex rungs.** That harder ladder was built: T10-T12 are
modelled on the 650-1000 line reproductions this repository ships, with the
cross-pane coupling real TUIs have. All 27 runs still scored 1.000, and the
ontology contrast was null across 18 further intervals.

`api_errors` did finally fire — twice in 27 runs, against twice in the previous
72 — so the reliability metric has some resolution at this complexity. It showed
no difference between conditions. The finding therefore moves from *untested*
toward *tested and null*: across 99 runs and 12 tasks, supplying the ontology to
an authoring agent has never measurably helped. Two error events is still too
thin to call it settled, but the burden has shifted, and Phase 1 should not be
funded on the hope that a richer schema would change it.

**What the data does support**, and what should be funded instead:

- Effort, not correctness, is where frameworks separate. Hawk TUI costs 3–4× more
  agent effort than ratatui for identical results, and the ordering tracks
  training-data representation.
- Volume of documentation is not the lever either: superlighttui ships 487 KB of
  agent-targeted docs, twenty times Hawk TUI's original 24.6 KB, and finished
  last on every measure.
- The remaining honest levers are API fluency and idiom — making the obvious
  thing the correct thing — plus letting agents check their own work, which is
  what Phase 2 shipped.

**On the crate description.** It currently reads "An agentic-first TUI framework
with complete ontology for agent discoverability". Two words should change
regardless of Phase 1. "Complete" is false: the registry was 29 % populated
until this work, and the schemas still describe 12 % of the authoring API.
"Discoverability" is fair for the runtime audience and misleading for the
authoring one. Something like "with a machine-readable widget ontology for
agent-driven UIs" claims what is actually true.

## 9. Sequencing

| Order | Item | Cost | Confidence |
|---|---|---|---|
| ~~1~~ | ~~0.2 register all widgets~~ | ✅ done | |
| ~~2~~ | ~~2.1 buffer text dump~~ | ✅ done | |
| ~~3~~ | ~~0.1 ship docs in the package~~ | ✅ done | |
| ~~4~~ | ~~0.3 ontology subcommand~~ | ✅ done | |
| ~~5~~ | ~~2.2 `hawktui::testing` harness~~ | ✅ done | |
| 6 | 1.1 + 1.2 authoring schema + coverage test | weeks | the bet |
| 7 | 1.3 core-type schemas | weeks | high |
| ~~8~~ | ~~1.4 compiled usage hints~~ | ✅ done | |
| ~~9~~ | ~~3.1 MCP server (ontology tools)~~ | ✅ done | runtime tools remain |
| ~~10~~ | ~~4.1 benchmark gate~~ | ✅ done | weekly, not nightly |

Items 1–4 are worth shipping before the benchmark returns a verdict, because
they are cheap and correct under every outcome. Item 6 is the one to hold until
the data is in.
