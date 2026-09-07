#!/usr/bin/env python3
"""The pre-registered test of the working-constraint paragraph.

    python paragraph_test.py <runs.rescored.jsonl> [more.jsonl ...]

Written and committed before the runs it analyses existed, so the analysis
could not be chosen after seeing them. The design is in
docs/AGENTIC-BENCHMARKS.md under "Pre-registration: the twenty runs".

The question: does a paragraph telling the agent not to read the framework's
implementation change how often it draws a wide glyph wrong? Untreated ratatui
failed 2 of 5 on `t16-straddle`; with the paragraph, 0 of 5. Two against zero in
fives is not a result (p = 0.22), so both arms were grown to fifteen.

Fixed in advance and not negotiable after the fact:

* A run is a failure if `score < 1.000` or `contract_failed`. Contract failures
  count here, unlike in the score medians, because a program that drew 27
  columns onto a 20-column screen failed at the thing this rung measures.
* One-sided Fisher's exact on failure counts, untreated against prohibition.
* Compliance gates the treated arm. A prohibition run with `source_reads > 0`
  did not receive the treatment; the arm is reported as partially administered
  rather than averaged as though it had.
* One prohibition run producing the failure falsifies the hypothesis whatever
  the rates say.
"""

from __future__ import annotations

import json
import sys
from math import comb
from pathlib import Path


def fisher_one_sided(a: int, na: int, b: int, nb: int) -> float:
    """P(as many or more failures in arm A) under the null of no difference."""
    total = a + b
    n = na + nb
    return sum(
        comb(na, k) * comb(nb, total - k) / comb(n, total)
        for k in range(a, min(na, total) + 1)
    )


def failed(record: dict) -> bool:
    return bool(record.get("contract_failed")) or float(record.get("score") or 0) < 1.0


def main() -> int:
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")
        except AttributeError:
            pass

    if len(sys.argv) < 2:
        print(__doc__)
        return 2

    arms: dict[str, list[dict]] = {"none": [], "forbid": [], "permit": []}
    for path in sys.argv[1:]:
        for line in Path(path).read_text(encoding="utf-8").splitlines():
            if not line.strip():
                continue
            r = json.loads(line)
            if r.get("invalid") or r.get("dry_run"):
                continue
            if r.get("task") != "t16-straddle" or r.get("framework") != "ratatui":
                continue
            arms.setdefault(r.get("source_note") or "none", []).append(r)

    # ------------------------------------------------------------ compliance
    print("compliance — the gate, before any outcome\n")
    administered = True
    for note in ("none", "forbid", "permit"):
        runs = arms.get(note) or []
        if not runs:
            continue
        # An absent field is "not recorded", never "read nothing". Reading a
        # missing measurement as a zero is how this benchmark twice reported a
        # treatment as administered when it was not.
        unknown = [r for r in runs if r.get("source_reads") is None]
        reads = [int(r["source_reads"]) for r in runs if r.get("source_reads") is not None]
        read_any = sum(1 for x in reads if x)
        line = f"  {note:<8} n={len(runs):<3} runs reading source: {read_any}/{len(reads)}"
        if unknown:
            line += f"   ({len(unknown)} NOT RECORDED)"
        print(line + f"   per-run {sorted(reads)}")
        if note == "forbid" and (read_any or unknown):
            administered = False
    if not administered:
        print("\n  WARNING: the prohibition arm either read the source or cannot be")
        print("  shown not to have. It is only partially administered, and the test")
        print("  below is not the pre-registered comparison.")
    print()

    # --------------------------------------------------------------- outcome
    print("outcome\n")
    table = {}
    for note in ("none", "forbid", "permit"):
        runs = arms.get(note) or []
        if not runs:
            continue
        bad = [r for r in runs if failed(r)]
        table[note] = (len(bad), len(runs))
        print(f"  {note:<8} {len(bad)} failures in {len(runs)} runs")
        for r in bad:
            print(f"      {r['label']}  score={float(r.get('score') or 0):.3f}"
                  f"  contract={bool(r.get('contract_failed'))}")
    print()

    if "none" in table and "forbid" in table:
        a, na = table["none"]
        b, nb = table["forbid"]
        p = fisher_one_sided(a, na, b, nb)
        print(f"one-sided Fisher's exact, none ({a}/{na}) vs forbid ({b}/{nb}): "
              f"p = {p:.4f}")
        if b:
            print("\n  The prohibition arm produced a failure. The pre-registered")
            print("  falsifier fires: the paragraph does not prevent it, whatever")
            print("  the rates are.")
        elif p <= 0.05:
            print("\n  Below the pre-registered 0.05. The paragraph is associated with")
            print("  the absence of this failure. It remains an association between a")
            print("  prompt and an outcome, on one rung, with one model.")
        else:
            print("\n  Above 0.05. No evidence the paragraph changes the failure rate;")
            print("  if the untreated arm came in low, the earlier 2-of-5 overstated")
            print("  the rate and the effect is smaller than it appeared.")
    # ------------------------------------------------------- exploratory only
    # Added before the data existed, and labelled for what it is. Conditioning
    # on source reads conditions on a choice the agent made after seeing its
    # prompt, so this cannot carry a causal claim -- but the untreated failures
    # so far have all come from runs that read nothing, which makes zero-read
    # runs the risk set and makes the untreated failure rate a function of how
    # often the agent happens to skip the source. That is not a stable property
    # and it is worth seeing rather than averaging away.
    #
    # Amended after the run that falsified it, with the original left standing
    # above: `t16-straddle__ratatui__c1__r7` of the untreated ten failed having
    # read the source four times, producing a dump byte-identical to the
    # zero-read failures. Reading the source does not protect a run, so
    # zero-read runs are not the risk set and this split separates nothing.
    # It is printed anyway, because a reader who was told the hypothesis
    # should be able to see the table that killed it.
    print("\nexploratory, not the pre-registered test — outcome by whether the "
          "run read source\n")
    for note in ("none", "forbid", "permit"):
        runs = [r for r in (arms.get(note) or []) if r.get("source_reads") is not None]
        if not runs:
            continue
        for label, subset in (
            ("read nothing", [r for r in runs if not r["source_reads"]]),
            ("read source", [r for r in runs if r["source_reads"]]),
        ):
            if subset:
                bad = sum(1 for r in subset if failed(r))
                print(f"  {note:<8} {label:<13} {bad} failures in {len(subset)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
