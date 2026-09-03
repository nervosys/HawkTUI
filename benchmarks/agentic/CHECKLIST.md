# Agentic benchmark build — progress

- [x] **1. Design doc** — `docs/AGENTIC-BENCHMARKS.md`: 2-factor design, task ladder,
      metrics, statistics, threats to validity, pre-registered falsification criteria
- [x] **2. Task suite** — T1–T6 (`tasks/`), each a prompt plus machine-checkable
      behavioural checks, over the shared harness contract in `tasks/contract.md`
- [x] **3. Verifier** — `runner/verify.py`, scores rendered character grids only;
      `runner/selftest.py` proves it discriminates (9 synthetic cases, all correct)
- [x] **4. Runner** — `runner/run.py`, drives the agent per cell, parses the
      transcript for build/API-error metrics, scores, writes `runs.jsonl`
- [x] **5. Context packs** — `runner/make_context.py` builds C0–C3; the ontology
      packs come from `examples/ontology_query.rs`, generated not hand-written
- [x] **6. Pilot** — instrument validated end to end (agent → build → frame dump →
      verify → metrics → analysis). Found and fixed two harness defects; results
      are anecdote-grade at n=1 and are labelled as such. `runner/analyze.py`
      reports medians, IQR and bootstrap CIs, and refuses to draw conclusions
      from thin cells.
- [x] **7. Improvement plan** — `docs/AGENT-DX-PLAN.md`: audit with measurements,
      four phases, per-item target metrics, and a decision rule for the case
      where the benchmark says the ontology does not help authoring

- [x] **9. Complex canonical rungs** — T10–T12 (`repo`, `monitor`, `chat`),
      modelled on the lazygit/btop/opencode reproductions this repo ships, with
      cross-pane coupling the earlier rungs could not test. All 27 runs scored
      1.000; `api_errors` fired for the first time and still showed no
      condition effect.
- [x] **8. Discovery rungs** — T7–T9 (`settings`, `meters`, `atlas`) describe
      behaviour without naming a widget, because T1–T6 all finished at 1.00 with
      zero API errors and a ladder with no failures measures nothing

## Findings — 70 runs, 9 tasks

Full write-up in `docs/AGENTIC-BENCHMARKS.md` § Results.

- **The ontology contrast is null on every pre-registered metric**, on all nine
  tasks including the three built so the agent must discover a widget rather
  than be told which one.
- **A signal that did not replicate.** T4–T6 showed C3 cheaper at the two
  hardest rungs, two intervals of 36 excluding zero against ~1.8 expected. On
  the fresh T7–T9 tasks it reversed. Hypothesis generated on one task set,
  tested on another, not confirmed — which is the reason the discovery rungs
  were built before their results were known.
- **The ladder never broke the agent.** `score` stayed at 1.00 almost
  everywhere; `api_errors` fired in 2 runs out of 70, both C1. With no failures
  to prevent, no condition could show a benefit, so Phase 1's premise is
  untested rather than refuted.
- **Effort separates frameworks, correctness does not.** ratatui 7–15 turns,
  Hawk TUI 30–40, superlighttui 49–126, for the same scores. The ordering tracks
  training-data representation.
- **Documentation volume is not the lever.** superlighttui ships 487 KB of
  agent-targeted docs against Hawk TUI's original 24.6 KB and finished last on
  every measure.
- **Two verifier bugs of our own**, both scoring a competitor 0.00 when the real
  scores were 0.77 and 0.80. Found, fixed, regression-tested; `rescore.py` now
  replays stored dumps through the current verifier before any analysis.

## Not done

- **A ladder that can fail.** The prerequisite for any future ontology claim: a
  rung where a competent agent measurably fails. Nine tasks did not produce one.
- N=5 as the design specifies; these grids ran at N=3.
- A second model family, needed before any general claim about agents.
- A clean re-run of `wall_seconds` on an idle machine (threat 7).
