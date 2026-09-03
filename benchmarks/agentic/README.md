# Agentic benchmark

Measures how quickly and how reliably an AI coding agent can scaffold working
TUI programs with different Rust TUI frameworks, and isolates how much of any
difference comes from Hawk TUI's ontology.

Methodology, conditions, metrics, threats to validity and the pre-registered
falsification criteria are in [`docs/AGENTIC-BENCHMARKS.md`](../../docs/AGENTIC-BENCHMARKS.md).
Read that before reading any numbers out of `results/`.

## Layout

```
tasks/          T1–T6 task ladder: prompt.md + checks.json per task
tasks/contract.md   the framework-neutral harness contract
runner/verify.py    scores a program from its rendered frames
runner/selftest.py  proves the verifier discriminates
runner/selftest_discovery.py  proves the T7-T9 checks discriminate
runner/rescore.py   re-scores stored dumps with the current verifier
runner/analyze.py   medians, IQR, bootstrap CIs, multiple-comparison count

Analysis of agent behaviour, all reading the stored transcripts:

runner/ontology_usage.py  did the agent consult the ontology at all
runner/source_usage.py    did it read the framework's source instead
runner/why_source.py      which files, in what order, and after what
runner/sufficiency.py     could the ontology have answered those reads
runner/token_cost.py      what source reading costs in context tokens
runner/make_context.py  builds the C0–C3 context packs
runner/run.py       drives the agent, collects metrics, writes runs.jsonl
context/        generated — do not edit by hand
results/        one directory per invocation; runs.jsonl plus every workdir
```

## Running it

```sh
# 1. Prove the instrument works before trusting it.
python runner/selftest.py            # T2/T3 checks
python runner/selftest_discovery.py  # T7-T9 checks
python runner/selftest_complex.py    # T10-T12 checks
python runner/selftest_unicode.py    # T13 display-width checks

# 2. Build the context packs (regenerate whenever the ontology changes).
python runner/make_context.py

# 3. Check prompts and seeding without spending agent time.
python runner/run.py --tasks t1-hello --conditions c0,c1,c2,c3 --dry-run

# 4. Run for real.
python runner/run.py --tasks t2-counter,t3-list \
                     --frameworks hawktui,ratatui \
                     --conditions c1,c2 --replicates 5
```

```sh
# 5. Re-score from the stored dumps, then analyse.
python runner/rescore.py results/<stamp>/runs.jsonl
python runner/analyze.py results/<stamp>/runs.rescored.jsonl
```

Each run gets a fresh directory containing the exact prompt it was given, the
agent's full transcript, the frame dump that was scored, and the per-check
result — so any number in a report can be traced back to the frames that
produced it.

**Always re-score before analysing.** `run.py` scores with whatever verifier was
loaded when it started, so a verifier fix mid-grid does not reach the runs
already recorded. `rescore.py` replays the stored dumps through the current
verifier and writes `runs.rescored.jsonl`, leaving the original file untouched
for audit. This is not hypothetical: an early `parse_frames` stripped every
trailing blank row, which turned a competitor's correct full-height screen into
a contract violation scored 0.00 when its real score was 0.77.

## Requirements

- `python` 3.10+
- a Rust toolchain, and network access the first time (ratatui and
  superlighttui are fetched from crates.io)
- the `claude` CLI on `PATH`, authenticated

Set `ANTHROPIC_API_KEY` if you can. The runner then passes `--bare`, which
disables CLAUDE.md discovery, hooks, memory and plugins — all of which are
uncontrolled context that varies between machines. Without an API key the
runner falls back to a normal session and records `bare_mode: false` on every
run, so results from the two modes are never silently mixed.

## Costs

A full grid — 6 tasks × 3 frameworks × 2 conditions (plus 2 Hawk-only
conditions) × 5 replicates — is 150 agent runs. Start with one task and one
replicate to calibrate before committing to a grid.

## Adding a task

Create `tasks/<id>/prompt.md` and `tasks/<id>/checks.json`. The prompt is
templated with `{{FRAMEWORK}}`, `{{DEP}}` and `{{CONTRACT}}`; everything else
must be identical across frameworks, or the comparison is measuring your
phrasing. Add checks that assert on *rendered characters only* — the verifier
never reads source, and it must stay that way for the results to mean anything.
