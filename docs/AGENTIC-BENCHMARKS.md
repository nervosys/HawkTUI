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
