# Agentic Benchmarks

Hawk TUI measures two different kinds of performance.

**Runtime performance** — how fast the framework paints a frame — is covered in
[BENCHMARKS.md](BENCHMARKS.md).

**Authoring performance** — how quickly and how reliably an AI agent can build a
working TUI with the framework — is covered here. This is the number that
decides what a TUI costs to build in 2026, and no Rust TUI framework publishes
it.

The two are independent. A framework can be fast at runtime and miserable to
write against; the reverse is just as possible. Nothing in this document should
be read as evidence about the other benchmark.

---

## The question

An agent asked to "build me a task manager TUI" does roughly this: recall the
framework's API from its training data, write a few hundred lines, run
`cargo build`, read the errors, and iterate until it compiles and behaves.
Every step of that loop is measurable.

We want to know:

1. **Across frameworks** — does an agent build a working TUI faster, cheaper,
   and more reliably with Hawk TUI than with ratatui or superlighttui?
2. **Because of what** — specifically, how much of any difference is
   attributable to Hawk TUI's **ontology**: the machine-readable description of
   every widget's schema, properties, capabilities, actions, and semantic role?

Question 2 is the one that matters, and it is the one a cross-framework
comparison *cannot* answer. If Hawk TUI wins, that could be the ontology, or
the API shape, or the fact that ratatui's far larger training-data footprint
pulls the model toward a competitor's idioms. Attribution needs a second
factor.

## Design: two factors, one of them within-framework

|               | C0 no context | C1 prose docs | C2 ontology pack | C3 ontology tool |
|---------------|:---:|:---:|:---:|:---:|
| Hawk TUI      | ✓ | ✓ | ✓ | ✓ |
| ratatui       | ✓ | ✓ | — | — |
| superlighttui | ✓ | ✓ | — | — |

**Conditions**

- **C0 — no context.** Task prompt only. The agent works from model priors.
  This measures the training-data prior, which is the honest explanation for a
  large part of any cross-framework gap.
- **C1 — prose docs.** The framework's README plus rustdoc for the relevant
  modules, placed in the working directory. This is what a real user of any of
  the three frameworks gets today.
- **C2 — ontology pack.** C1 plus the generated ontology catalog (JSON) for
  every `Discoverable` widget: properties with types and constraints, semantic
  roles, capabilities, usage hints.
- **C3 — ontology tool.** C1 plus a *queryable* ontology the agent calls on
  demand (`hawktui ontology <search|schema|role>`), rather than a blob pasted
  into context. This separates "the information helped" from "having it in
  context helped", which matters because C2 costs input tokens on every turn
  and C3 does not.

**The ontology effect is the C1 → C2 and C1 → C3 delta within Hawk TUI.** It is
measured with the framework, the model, the tasks, and the verifier all held
constant. The cross-framework row is context for that number, not evidence for
it.

The ratatui and superlighttui cells at C2/C3 are empty because those frameworks
have no ontology to supply. That absence is the thing under study, so we do not
fabricate a substitute for them.

## The task ladder

Six canonical TUI programs of strictly increasing complexity. Each is a shape
that real TUIs actually take, not a synthetic exercise.

|        | Task | Exercises | ~LoC |
|--------|------|-----------|------|
| **T1** | `hello` — static text in a bordered block | minimal API discovery, render entry point | 30 |
| **T2** | `counter` — `+`/`-`/`q` mutate and display a number | event loop, state, key decoding | 60 |
| **T3** | `list` — 20-item scrollable list, arrow keys, selection highlight, status bar | stateful widgets, layout split, scroll state | 120 |
| **T4** | `dashboard` — header/body/footer, body split in two columns, gauge + sparkline + list | nested layout, constraint arithmetic, multiple widget types | 220 |
| **T5** | `browser` — master/detail, Tab switches focus, filter input, detail pane scrolls | focus management, text input, cross-widget state | 350 |
| **T6** | `tasks` — tabs, table with sorting, modal add-item dialog, key-hint footer | overlays, tables, composite state, modal input routing | 550 |

The ladder matters because the interesting failures are not at T1. Every model
can write hello-world against every framework. Ontology value, if it exists,
should grow with the number of distinct widgets and properties the agent has to
recall correctly — so a flat curve across T1–T6 is itself a finding.

### The discovery rungs, T7–T9

T1–T6 all came back at 1.00 with zero API errors, for every framework and every
condition (see `benchmarks/agentic/results/`). A ladder with no failures cannot
measure anything: with no variance in `score` or `api_errors`, no condition can
move them. That is a fact about the ladder, **not** evidence that the ontology
does not help — the two must not be confused.

T1–T6 share a property that makes them easy: each names the widget it wants. An
agent told "a scrollable list with a selection" does not need to discover
anything. So the discovery rungs describe **behaviour only** and never name a
widget, which is precisely the question `search` and `roles` exist to answer:

|        | Task | What must be discovered |
|--------|------|-------------------------|
| **T7** | `settings` — rows of `Name: Value` where the value cycles through a fixed list | a widget for cycling enumerated values |
| **T8** | `meters` — a filled bar, a line-drawn single-row indicator, and an animated spinner, all over one value | three different progress presentations, and that a spinner advances by tick |
| **T9** | `atlas` — a surface addressing points finer than one cell, plus a month grid | sub-cell drawing, and a calendar with day highlighting |

They also test property knowledge rather than recall alone: T8 turns on the
difference between a ratio and a percentage and on what "advance the animation"
means, both of which the schemas carry as typed properties and constraints.

**These rungs are for the within-Hawk-TUI contrast only.** They were written
around behaviours Hawk TUI has widgets for, so a cross-framework comparison on
them would be rigged, and we do not run one. The C1 → C2/C3 contrast holds the
framework constant and is unaffected.

The bias that would invalidate them is different and worth naming: if a task's
answer appeared verbatim in the ontology pack, C2 would win trivially. The
prompts therefore describe outcomes in prose that does not quote any schema
field, and the checks assert only on rendered characters.

### Calibration: the low rungs are at ceiling

A four-run pilot (Hawk TUI, T1 and T3, C1 and C2, one replicate each) scored
**1.00 on every run with zero API errors**. The list task — 20 items, moving
selection, a live status bar, 11 behavioural checkpoints — was solved cleanly
in both conditions.

So T1–T3 discriminate nothing at this model tier: with no variance in `score`
or `api_errors` there is no effect for any condition to have. They are worth
keeping as a floor that catches a framework or a harness that is outright
broken, but replicate budget should go to **T4–T6**, and a null result at T1–T3
must never be reported as "the ontology does not help".

The pilot also showed C2 finishing in fewer turns than C1 on both tasks. At
n=1, with one of the C1 runs polluted by a since-fixed harness bug, that is an
anecdote and is reported here as nothing more.

## The verifier: framework-neutral by construction

The hard part of a cross-framework authoring benchmark is scoring the result
without favouring anyone's API. We score **rendered output**, never source.

Every task spec requires the scaffolded program to accept:

```
<prog> --headless <W>x<H> --script "Down,Down,Enter,q" --dump
```

and print each frame as a plain-text character grid — no ANSI, one frame per
form feed. That contract is trivially satisfiable by all three frameworks (each
supports an offscreen buffer) and says nothing about how the UI is built.

Checks are then assertions over the grid: text present in a region, a selection
marker that moves with `Down`, a gauge bar whose filled width tracks its ratio,
a modal that covers the pane beneath it. The verifier never reads `.rs` files,
so it cannot reward Hawk-shaped code.

**Cost of this choice:** the harness contract is itself a task requirement, and
an agent that builds a perfect TUI but ignores the contract scores zero. That
is a real distortion. We mitigate it by putting the contract in the prompt for
every condition including C0, and by reporting `contract_failed` separately
from behavioural failures so the two are never conflated.

## Metrics

Per run, from the agent transcript and the verifier:

**Outcome**
- `built` — the program compiles
- `score` — fraction of behavioural checkpoints passed
- `contract_failed` — scored zero for harness-contract reasons, not UI reasons

**Efficiency**
- `wall_seconds`, `turns`, `input_tokens`, `output_tokens`, `cost_usd`

**Reliability** — the ontology-sensitive metrics
- `build_attempts` — cargo invocations
- `failed_builds` — how many returned non-zero
- `api_errors` — count of `E0599` / `E0433` / `E0432` / `E0425` / `E0061`
  across failed builds. These are precisely *"you called a method, path, or
  argument list that does not exist"* — the compiler's own name for API
  hallucination, and the single metric an ontology should most directly move.
- `first_build_clean` — did the first compile succeed

The headline reliability claim, if the data supports it, is a reduction in
`api_errors` per task. That is falsifiable, mechanism-linked, and not gameable
by writing a nicer README.

## Statistics

Agents are stochastic; a single run per cell is noise. Each cell runs **N = 5**
replicates with the same prompt and a fresh working directory. We report the
**median** and the interquartile range, and for the C1 → C2/C3 contrasts a
bootstrap confidence interval on the difference of medians (10,000 resamples).

Following the precedent set in [BENCHMARKS.md](BENCHMARKS.md), where two
supposedly identical runs disagreed by up to 2×: **when replicates disagree,
we publish the result least favourable to Hawk TUI.**

## Results

70 runs: 45 across T4–T6 with three frameworks, and 25 across the discovery
rungs T7–T9 with Hawk TUI alone. Raw metrics are in
`benchmarks/agentic/results/`; every number below is reproducible from the
stored frame dumps with `rescore.py` and `analyze.py`.

### The ontology did not measurably help authoring

Across every task and both conditions, the C1 → C2 and C1 → C3 contrasts came
back **"no effect" on every pre-registered metric**. Bootstrap intervals on
`score` and `api_errors` were identically zero; intervals on `turns` and
`cost_usd` spanned zero everywhere except where noted below.

This is the third branch of the decision rule in
[AGENT-DX-PLAN.md](AGENT-DX-PLAN.md) §8: the ontology's value is runtime
introspection, not authoring. The mechanism was visible before the runs began —
the schemas describe 12 % of the public builder API and none of the layout
system — and the data is consistent with it.

### A signal that did not replicate

The T4–T6 grid showed C3 (the queryable ontology) cheaper than C1 at the two
hardest rungs: −$0.084 (95 % CI [−0.260, −0.006]) at T5 and −$0.213
([−0.379, −0.087]) at T6, with `turns` trending −6 at both. Two of 36 intervals
excluded zero against ~1.8 expected by chance — not a finding by the
pre-registered test, but a coherent pattern: same metric, same condition, same
direction, at the harder tasks.

**It did not replicate.** On T7–T9, built specifically to make discovery
matter, C3 cost went the other way: +$0.140, +$0.021 and +$0.160. The single
interval excluding zero across those 30 was `turns` at T8, in the *worse*
direction, by one turn.

Generating a hypothesis on one set of tasks and testing it on a fresh set is the
whole reason the discovery rungs were built before their results were known. The
honest reading is that the T4–T6 pattern was noise that happened to line up, and
the multiple-comparison warning was right.

### The ladder never broke the agent

`score` sat at 1.00 in almost every cell of all nine tasks. `api_errors` — the
metric an ontology should most directly move — fired in **2 runs out of 70**.
Both were C1, which is the only directional hint in the data and is two events.

Even T7–T9, which never name a widget, were solved: the model finds a settings
list, a line gauge and a braille canvas from a behavioural description without
help. The discovery rungs did cost more effort (T7 C1 needed a median 30 turns
against T2's handful) but effort is not failure, and a benchmark that cannot
produce failures cannot show a reliability difference.

**A harder ladder is the prerequisite for any future ontology claim.** Until a
rung exists where the agent measurably fails, C2 and C3 have nothing to improve.

### The complex canonical rungs, T10-T12

T7-T9 made the agent discover a widget but stayed small. T10-T12 are modelled on
the reproductions this repository ships — `examples/lazygit.rs` (1043 lines),
`examples/btop.rs` (749) and `examples/opencode.rs` (665) — and each carries
roughly twice the checks of the earlier rungs:

|        | Task | The coupling that can break |
|--------|------|-----------------------------|
| **T10** | `repo` — four-pane git browser | the diff pane tracks the *Files* selection whichever pane has focus, and each pane keeps its own selection |
| **T11** | `monitor` — meters, sparkline, sortable process table | refresh moves the meters without disturbing row order; sort moves rows without disturbing the meters |
| **T12** | `chat` — two sessions, markdown transcript, composer | each session keeps its own transcript and composer; markdown markers must not reach the screen |

These are framework-neutral shapes — a git browser and a system monitor are not
built around any framework's widget set — so ratatui runs them too, unlike
T7-T9.

**All 27 runs scored 1.000.** Every check, every framework, every condition.

`api_errors` did fire here, twice, where it had fired twice in the previous 72
runs. So the metric has resolution at this complexity and still showed no
difference between conditions. That matters for the decision in
[AGENT-DX-PLAN.md](AGENT-DX-PLAN.md) §8: the ontology's authoring value moves
from *untested* toward *tested and null*, though two events remain too few to
call it settled.

18 intervals were computed across these rungs and **none excluded zero**.

### The gap is not constant

| Task | Hawk TUI vs ratatui, cost | turns |
|---|---|---|
| `t10-repo` | 2.3× | 2.1× |
| `t11-monitor` | 3.0× | 2.5× |
| `t12-chat` | 1.2× | 1.4× |

The T4-T6 figure of 3-4× is not a constant property of the framework. On the
chat task the gap nearly closes. Whatever costs an agent extra effort in Hawk
TUI is concentrated in particular API surfaces rather than spread evenly, and
finding which is a more tractable problem than "be more like ratatui".

### Program size does not predict agent effort

The rungs were sized by the line count of their reference implementations. That
predicted cost badly: `t12-chat` follows the 665-line `opencode.rs` and cost
$0.35, `t10-repo` follows the 1043-line `lazygit.rs` and cost $0.83, and
`t11-monitor` — the *smallest* reference at 749 lines — cost the most at $0.88.

What tracks effort is the number of **independent state relationships the
program must hold at once**. T11's "refresh moves the meters but not the rows,
sort moves the rows but not the meters" is the expensive shape. Anyone
extending this ladder should build tasks around coupled invariants, not around
line counts.

### Why the ontology conditions were null: the agent never opened them

The C2 and C3 results were reported as evidence that the ontology does not help
authoring. That was wrong, and the transcripts say so. Scanning all 99 stored
runs for the file reads and tool calls each condition depends on:

| Condition | Delivery | Runs | Consulted the ontology |
|---|---|---:|---:|
| C2 | files seeded into the working directory | 18 | **5 (28 %)** |
| C3 | a CLI, after reading an instructions file | 25 | **1 (4 %)** |

A null result from a treatment that was never administered measures nothing.
What C2 and C3 actually measured is whether an agent *chooses* to consult
passive reference material, and mostly it does not: an agent that believes it
knows the API has no reason to open a file about it.

The six runs that did consult were self-selecting — three of them `t9-atlas`,
the hardest discovery rung — so their higher cost reflects the tasks that
prompted the lookup, not a cost of looking.

`runner/ontology_usage.py` reports this. **Any future condition must publish its
consultation rate alongside its outcome**, because below roughly half the
outcome numbers cannot be interpreted.

### Two faults, not one

Fixing consumption exposed the second fault, which is about content. The widget
ontology answers *what does this widget hold at runtime* — the right question
for an agent driving a running application, the wrong one for an agent writing
a program:

| | |
|---|---|
| Public functions in the authoring surface | **334** |
| Described by the widget ontology | 40 properties (**12 %**) |
| `Layout`, `Constraint`, `Frame`, `Terminal`, events | **0 of 78** |

An agent cannot write a program from a catalog that omits the layout system.
The `ontology::api` catalog added in response covers 72 types and 350
functions, generated from the signatures — including whether a type renders as
`Widget` or `StatefulWidget` and with which state type.

**Condition C5** supplies that catalog as MCP tools, so it arrives in the tool
surface rather than as a file to open. It therefore changes *two* things at once
against C1 — content and delivery — and cannot separate them on its own. C4,
the runtime schema over the same transport, is the control that isolates
content; it is specified in the runner and not yet run.

### C5: the authoring ontology, measured

`ontology::api` replaced a 40-property runtime schema with 350 functions
covering constructors, builder signatures, enum variants, the layout system and
the `Widget`/`StatefulWidget` split. C5 supplies it as MCP tools. 24 runs,
C1 against C5, on two discovery rungs and two complex ones.

**Consultation rose but did not become universal.**

| Condition | Delivery | Consulted |
|---|---|---:|
| C3 | CLI, after reading an instructions file | 4 % |
| C2 | files seeded into the directory | 28 % |
| **C5** | authoring catalog as MCP tools | **42 %** (5 of 12) |

A tenfold improvement over C3, and still fewer than half of runs. An agent that
believes it knows the API does not reach for a reference, and putting the
reference in its tool list only partly changes that.

**Outcomes were null, with one interval against us.**

| Task | turns Δ | cost Δ | Verdict |
|---|---:|---:|---|
| `t10-repo` | −10 | −$0.33 | no effect |
| `t8-meters` | +5 | −$0.14 | no effect |
| `t9-atlas` | +5 | −$0.06 | no effect |
| `t11-monitor` | **+18** | **+$0.16** | **worse**, both intervals exclude zero |

`score` was 1.000 and `api_errors` 0 in every cell. Of 24 intervals, ~1.2 are
expected to exclude zero by chance; two did, both on one task, both in the worse
direction.

**The one suggestive number is within C5**, comparing runs by whether they
actually used the tools:

| | n | turns | cost |
|---|---:|---:|---:|
| consulted | 5 | 40 | **$0.64** |
| did not | 7 | 37 | **$0.83** |

Runs that consulted were cheaper despite taking more turns. This is
self-selected and n=5 — the agent chooses when to look — so it is a hypothesis
for a larger run, not a result.

**What this does and does not establish.** The artifact is better by every
measure that can be checked without an agent: coverage, correctness,
drift-resistance. The benefit remains unproven, and the reason is the same one
that has now recurred across five grids: `score` never leaves 1.000 and
`api_errors` fires in roughly 3 % of runs. **A reliability aid cannot be priced
on a benchmark with no failures.** The instrument is the limiting factor, not
the ontology.

Two things would change that, neither of them more ontology content: raise
consultation above 42 % — the tool descriptions say what the tools do but not
*when* to call them — and build a task this model tier actually fails.

### Consultation raised to 83 %, outcomes still flat

The C5 tool descriptions said what each tool does but never when to call it,
leaving the agent to decide. Naming the trigger — "call this BEFORE writing the
first line of code that uses a type", "before rendering any list, table, editor
or scrollbar" — roughly doubled consultation.

| Condition | Delivery | Consulted | Tool calls |
|---|---|---:|---:|
| C3 | CLI, after reading an instructions file | 4 % | 2 |
| C5 | authoring catalog as MCP tools | 42 % | 55 |
| **C5 + triggers** | the same tools, with trigger conditions | **83 %** | **107** |

Outcomes did not move:

| Condition | n | turns | cost | `api_errors` | score |
|---|---:|---:|---:|---:|---:|
| C1 — no ontology | 12 | 36 | **$0.78** | 0 | 1.000 |
| C5 | 12 | 38 | $0.79 | 0 | 1.000 |
| C5 + triggers | 12 | 38 | **$0.78** | 2 | 1.000 |

Median cost is identical between no ontology at all and an ontology consulted in
83 % of runs across 107 tool calls. Per task the effect is noise in both
directions: `t9-atlas` and `t10-repo` improved slightly, `t11-monitor` got worse
at every step (19 → 37 → 53 turns).

**This is the result the earlier nulls could not be.** Each previous null had an
excuse — the catalog described the wrong things, or the agent never opened it.
Both are now gone: the ontology covers 350 functions including the layout
system, it is delivered as tools in the agent's own surface, and it is consumed
in five runs out of six. It still changes nothing measurable.

The remaining explanation is the one that has held across every grid in this
document: **`score` is 1.000 and `api_errors` is approximately zero in every
condition.** A tool whose purpose is to prevent mistakes cannot show value
against an agent that is not making any. Note that the only two API errors in
these 36 runs occurred in the *consulted* condition.

**What follows for the framework.** The authoring ontology is a better artifact
by every measure checkable without an agent, and it is worth keeping for the
runtime introspection it was built for and as documentation that cannot drift
from the code. Its value for *authoring* is unproven, and further ontology work
is not what would prove it. The binding constraint is a benchmark with no
failures, and the next step is a task this model tier actually fails.

**A caveat on this specific run.** It differs from the preceding C5 in two ways
— the trigger conditions and the type summaries regenerated shortly before it —
so it cannot attribute the consultation rise to the descriptions alone.

### The agent reads the source, whatever the ontology says

`runner/source_usage.py` and `runner/why_source.py` reconstruct what the agent
does from the stored transcripts. The result reframes every ontology finding in
this document.

| Framework | Runs reading the implementation | Reads per run | First read |
|---|---:|---:|---:|
| **Hawk TUI** | **100 %** | 16–22 | tool call #1 |
| ratatui | 6 % | 0.1 | #8 |

80 % of those reads are core and runtime files, not widgets:
`backend/test.rs`, `event/mod.rs`, `core/buffer.rs`, `terminal.rs`,
`runtime/mod.rs`, `lib.rs`. Three iterations of the ontology — runtime state,
then the widget API, then the program skeleton — left this unchanged:

| Condition | source reads | ontology calls | turns | cost |
|---|---:|---:|---:|---:|
| C1, no ontology | 20 | 0 | 36 | $0.78 |
| C5, widget API | 16 | 6 | 38 | $0.78 |
| C5, + skeleton and traits | 20 | 10 | 48 | $0.92 |

`program_skeleton` was called in **9 of 9** runs. The agent adopted it, then read
the source anyway. `prelude` was called 3 times while `lib.rs` was read 15.

**This is a fact about how current models are trained, not about the ontology.**
They ground themselves in code: 36 source reads came directly after an ontology
answer on the same type. No amount of coverage changes that, and treating the
flat outcome as evidence against the ontology would be the same mistake as
treating the earlier nulls as evidence when the agent had never opened it.

### Sufficiency: the question that survives

If agents are trained to prefer an ontology when one exists, the useful question
is not whether this one changes behaviour but whether it *could* replace the
source. `runner/sufficiency.py` answers that directly: for every file opened
across 24 runs, does the ontology describe that file's public API?

| | Reads covered |
|---|---:|
| Before | 174/193 (90 %) |
| After closing three gaps | **193/193 (100 %)** |

The gaps were `agent/driver.rs`, `agent/session.rs` and — the one that mattered
— `src/testing.rs`, the `Harness` every task needs for headless rendering, with
eleven public functions and none of them described.

The catalog is now 93 types and 480 functions, and **an agent trained to prefer
it would have had no reason to open a single source file in those runs.**

Two defects were found getting there. The generator derived module paths by
stripping a segment, which published `hawktui::backend` for `TestBackend` — an
import that does not resolve, and exactly the kind of wrong answer that sends an
agent to the source. It now emits `examples/api_imports.rs`, every published
path as a `use` statement compiled as an example, so a bad path is a build
failure. And the first run of the sufficiency audit reported thirteen false gaps
because a regex spanning type boundaries captured nothing; it was caught because
the output contradicted something already known to be true.

### What source reading costs, and what the ontology would cost instead

`runner/token_cost.py` measures both from the transcripts. Tokens are estimated
at four characters each, which is close enough for a ratio.

| | calls per run | tokens returned |
|---|---:|---:|
| source reads | 12.0 | **10,664** |
| ontology calls | 10.3 | **1,989** |
| all tool results | — | 25,211 |

**Source reading is 42 % of everything tools put into the context**, and the
ontology answers comparable questions in **5.4× fewer tokens** — a similar
number of calls returning a fifth of the payload, because a signature is a line
and a source file is hundreds.

That is a floor. Tool results are re-sent on every subsequent turn, so ten
thousand tokens read at turn three are paid again at every turn after it; the
compounding is most of what drives the $0.85 mean cost per task.

So for a model trained to prefer an ontology when one exists, on these tasks:

- 42 % of returned-token context is freed, compounding with conversation length
- the same answers arrive in 1,989 tokens rather than 10,664
- no source file is needed at all — sufficiency is 193/193

**This is the size of the prize, not a measured win.** No model trained that way
exists to test against, and an agent that consults the ontology *and* reads the
source — which is what today's does — pays for both. Settling it needs a model
with that preference run against this same grid; the harness, tasks, verifier
and analysis tools are all here and reproducible.

### T13: the first rung that produced a failure

Twelve rungs scored 1.000 because they all tested things the **compiler
catches**. The agent writes a wrong method name, `cargo build` says so, it
iterates. That loop is what `api_errors` measures, and it is why the metric
fired in about 3 % of 190 runs.

`t13-unicode` tests something a compiler cannot catch. A table of `hello`,
`日本語`, `🙂🙂`, `café` and `a日b` must align in **display columns**, while a
separate column reports **character** counts — two different numbers for the
same string. Pad by character count and the program compiles perfectly and
renders misaligned.

Checking that required a new verifier capability. The dump contract stores a
double-width glyph as one character, so character offsets in a frame are not
display columns; `display_gap` measures the distance between two strings on a
row in display columns via `east_asian_width`.

| Framework | Condition | Scores | `api_errors` | turns |
|---|---|---|---:|---:|
| Hawk TUI | C1 | 1.000, 1.000, 1.000 | 0 | 14–35 |
| Hawk TUI | C5 | 1.000, 1.000, 1.000 | 2, 2, 0 | 45–54 |
| ratatui | C1 | 1.000, 1.000, **0.533** | 0 | 16–25 |

The failing run failed exactly where the task aims: `cjk`, `emoji` and `mixed`
column checks, with the ASCII and `café` rows passing. Zero API errors — it
compiled cleanly and rendered wrongly.

**The prediction behind this rung was wrong in both directions.** It was built
expecting Hawk TUI to fail without the ontology and be rescued by it. Hawk TUI
did not fail either way, and C5 cost more: two API errors where C1 had none, and
about 40 % more turns.

What it does establish is the mechanism: **display-correctness tasks produce
failures where API-correctness tasks cannot.** One failure in nine runs is far
too thin to build a claim on, but it is the first non-zero variance in the
`score` column anywhere in this document, and it says where to dig — grapheme
boundaries, combining marks, bidirectional text, terminal-width edge cases.
Those are where implementations diverge and compilers stay silent.

### T14, and what the failing axis actually is

T14 asks for greedy wrapping to 24 display columns and truncation to 10 columns
including an ellipsis — the same unicode-width theme as T13, pushed harder. All
twelve runs scored 1.000, across both frameworks and both conditions.

The prediction behind it was wrong, and the reason is worth more than the rung.
**T14 asks the agent to compute a layout in plain Rust**: call `unicode-width`,
loop over words, accumulate. That is ordinary programming and the model is good
at it. **T13 asks it to make text line up on a terminal grid**, which requires
knowing that the terminal already renders an ideograph as two columns.

The T13 failure shows the difference. It did not miscount padding; it rendered

```
│cjk     日 本 語       3
│emoji   🙂 🙂         2
```

inserting a space after each wide character, apparently to *make* it two columns
wide — so each took three. The text is corrupted, not merely misaligned, and it
compiled with zero API errors. No API documentation would have prevented it: the
agent used the API correctly and misunderstood the medium.

So the discriminating axis is narrower than "display correctness". It is
**beliefs about the rendering surface**, not string arithmetic. That is the only
thing shown to break this agent in ~215 runs, and it is where a further rung
should aim.

### The ontology condition is worse on the hardest rungs

Across T13 and T14, with identical scores everywhere:

| | `api_errors` | turns |
|---|---|---:|
| Hawk TUI C1 (8 runs) | **0** | 15–45 |
| Hawk TUI C5 (8 runs) | 2, 5, 2, 0, 0, 0, 0, 0 | 39–62 |

Seven grids now point the same way. The ontology is complete, sufficient,
correctly delivered and consumed, and on the two rungs where the work is hardest
it costs turns and introduces errors rather than preventing them.

### Three findings that are not about ontologies

1. **Structural complexity does not create failures.** Twelve rungs up to a
   four-pane git browser with cross-pane coupling: 1.000 everywhere.
2. **Misunderstanding the medium does.** One prompt about terminal columns
   produced text corruption that compiled cleanly.
3. **The agent's ceiling is the framework's own correctness** for anything it
   delegates, and its own beliefs for anything it computes.

At n=4 per cell, T14's uniform 1.000 says the rung is too easy, not that agents
are reliable; T13's single failure in nine runs remains one event. Two rungs
built to produce failures yielded one failure between them, which is itself a
result about how hard it is to make this agent fail at TUI work.

### What did separate the frameworks

Effort, by a wide margin, on identical correctness:

| Framework | score (T4–T6) | turns | cost per attempt |
|---|---|---|---|
| ratatui 0.29 | 1.000 | 7–15 | $0.21–0.33 |
| **Hawk TUI** | 1.000 | 30–40 | $0.57–1.04 |
| superlighttui 0.23 | 0.90–1.00 | 49–126 | $1.32–5.74 |

Hawk TUI is roughly **3–4× more expensive to build with than ratatui** and
2–5× cheaper than superlighttui. The ordering tracks how well each framework is
represented in the model's priors, which threat 1 predicted and which no amount
of ontology addressed. Note that superlighttui ships 487 KB of agent-targeted
documentation and still finished last, which is evidence against the volume of
documentation being the lever either.

### Scoring corrections

Two runs were recorded as 0.00 by a verifier bug that stripped trailing blank
rows and misread a correct full-height screen as too short. Re-scored from the
stored dumps they are 0.77 and 0.80. Both belonged to superlighttui, so the
uncorrected data would have read "superlighttui fails T4 and T6" — a conclusion
entirely manufactured by our own parser. `rescore.py` exists because of this and
runs before every analysis.

### What is missing

- `t9-atlas` C3 has 1 replicate rather than 3; the run was interrupted. That
  cell reports as an anecdote and no interval is computed for it.
- N is 3, not the 5 the design specifies.
- One model family only.
- `wall_seconds` for the T4–T6 grid is contaminated (threat 7).

## Threats to validity

Stated up front, because a benchmark that an interested party designed for its
own framework earns scrutiny.

1. **Training-data asymmetry.** ratatui is years older and vastly better
   represented in any model's training data than Hawk TUI, which was renamed
   weeks ago. This biases the cross-framework comparison *against* Hawk TUI —
   and it means a Hawk TUI win at C0 would be surprising enough to warrant
   suspecting the harness before believing the result. The C1 → C2/C3
   within-framework contrast is immune to this, which is the main reason it
   carries the argument.
2. **Self-authored tasks.** We wrote the tasks and the verifier. Task shapes
   were fixed from existing real TUIs (gitui, btop, lazygit, k9s) before any
   condition was run, and the checkpoints reference only behaviour a user could
   observe.
3. **Single model family.** Results are for one agent. The runner is
   model-agnostic — `--agent` selects the driver — and a second family should
   be run before any general claim.
4. **The contract distortion**, above.
5. **Prompt sensitivity.** One phrasing per task, reused verbatim across every
   framework and condition. No per-framework prompt tuning; that would be the
   easiest way to rig this and the hardest to detect.
6. **Framework frozen before the harness was eased.** Runs execute against a
   snapshot of Hawk TUI taken *before* `Buffer::to_text()` was added. That
   method makes the harness contract in this benchmark markedly easier to
   satisfy, and it exists only in Hawk TUI — measuring with it in place would
   let the framework author hand his own framework an advantage on his own
   test. The snapshot does include `examples/ontology_query.rs`, because the
   ontology tooling is the thing under study rather than an aid to the
   contract. A later run should measure the improved tree and report the delta
   separately.

7. **`wall_seconds` was measured on a busy machine.** The grid reported in
   `results/main2/` ran while the framework itself was being edited, compiled
   and tested — hundreds of `cargo build`, `clippy` and `test` invocations
   competing for the same cores. Wall-clock time for those runs is therefore
   inflated by an unknown and unevenly distributed amount, and should not be
   compared across frameworks or conditions.

   `turns`, `score`, `api_errors`, `failed_builds` and `cost_usd` are unaffected:
   they count what the agent did, not how long the machine took to do it. The
   published cross-framework gap rests on `turns` and `cost_usd` for exactly
   this reason. A clean re-run on an idle machine is needed before any
   wall-clock number is quoted.

## What would falsify the ontology hypothesis

Written before the first run, so the result cannot be reinterpreted afterwards:

- If **C2 ≈ C1** on `api_errors` at T4–T6, the ontology does not help authoring,
  and its value is confined to runtime introspection.
- If **C2 helps but C3 does not**, the benefit is context-stuffing rather than
  structured retrieval, and the ontology is an expensive way to ship a
  cheatsheet.
- If the ontology effect is **flat across the ladder**, it is a fixed-cost
  lookup saving rather than a scaling advantage, and should be claimed as such.

Any of these is a publishable result. `benchmarks/agentic/results/` records
every run, including the ones that did not go our way.
