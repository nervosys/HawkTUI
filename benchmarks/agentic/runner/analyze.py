#!/usr/bin/env python3
"""Turn runs.jsonl into the report tables.

    python analyze.py ../results/<stamp>/runs.jsonl

Reports median and interquartile range per cell, and for the within-Hawk-TUI
ontology contrasts a bootstrap confidence interval on the difference of
medians. Runs marked `invalid` (the agent never ran) are excluded and counted
separately; a cell that lost runs that way is flagged, because a silently
thinned cell is how a benchmark starts lying.
"""

from __future__ import annotations

import argparse
import json
import random
import statistics as st
import sys
from collections import defaultdict
from pathlib import Path

METRICS = [
    ("score", "score", 3, "higher"),
    ("api_errors", "API errors", 1, "lower"),
    ("failed_builds", "failed builds", 1, "lower"),
    ("turns", "turns", 0, "lower"),
    ("wall_seconds", "wall s", 0, "lower"),
    ("cost_usd", "cost $", 3, "lower"),
]


def quartiles(values: list[float]) -> tuple[float, float, float]:
    s = sorted(values)
    if len(s) == 1:
        return s[0], s[0], s[0]
    q = st.quantiles(s, n=4, method="inclusive")
    return q[0], st.median(s), q[2]


def bootstrap_ci(a: list[float], b: list[float], n: int = 10000, seed: int = 0):
    """CI on median(b) - median(a). Returns None when a cell is too thin."""
    if len(a) < 3 or len(b) < 3:
        return None
    rng = random.Random(seed)
    diffs = []
    for _ in range(n):
        ra = [rng.choice(a) for _ in a]
        rb = [rng.choice(b) for _ in b]
        diffs.append(st.median(rb) - st.median(ra))
    diffs.sort()
    return diffs[int(0.025 * n)], st.median(b) - st.median(a), diffs[int(0.975 * n)]


def main() -> int:
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")
        except AttributeError:
            pass

    ap = argparse.ArgumentParser()
    ap.add_argument("jsonl", type=Path)
    args = ap.parse_args()

    records, invalid = [], 0
    for line in args.jsonl.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        r = json.loads(line)
        if r.get("invalid") or r.get("dry_run"):
            invalid += 1
            continue
        records.append(r)

    if not records:
        print("no valid runs")
        return 1

    modes = {r.get("bare_mode") for r in records}
    if len(modes) > 1:
        print("WARNING: runs mix --bare and non-bare agent modes; do not "
              "compare them directly.\n")
    print(f"{len(records)} valid runs" + (f", {invalid} invalid (excluded)" if invalid else ""))
    print(f"agent mode: {'bare' if modes == {True} else 'non-bare (confounds recorded)'}\n")

    cells: dict[tuple, list[dict]] = defaultdict(list)
    for r in records:
        cells[(r["task"], r["framework"], r["condition"])].append(r)

    # ------------------------------------------------------------ per-cell
    print("per cell — median (IQR)\n")
    head = f"{'task':<14}{'framework':<15}{'cond':<6}{'n':>3}  " + "".join(
        f"{label:>16}" for _, label, _, _ in METRICS
    )
    print(head)
    print("-" * len(head))
    for key in sorted(cells):
        task, fw, cond = key
        rows = cells[key]
        line = f"{task:<14}{fw:<15}{cond:<6}{len(rows):>3}  "
        for metric, _, places, _ in METRICS:
            # A contract failure means the program never produced a scorable
            # screen. Folding it in as a zero would conflate "ignored the dump
            # format" with "built the wrong UI", which the design forbids; those
            # runs are counted separately below instead.
            pool = [r for r in rows if not r.get("contract_failed")] if metric == "score" else rows
            vals = [float(r.get(metric) or 0) for r in pool if r.get(metric) is not None]
            if not vals:
                line += f"{'—':>16}"
                continue
            lo, med, hi = quartiles(vals)
            line += f"{med:>10.{places}f} ({hi - lo:.{places}f})".rjust(16)
        print(line)

    # --------------------------------------------------- ontology contrasts
    intervals = 0
    flagged: list[str] = []
    print("\nontology effect within Hawk TUI — median difference vs C1")
    print("(negative is better for every metric except score)\n")
    for target in ("c2", "c3"):
        pairs = sorted({t for (t, f, c) in cells if f == "hawktui" and c in ("c1", target)})
        for task in pairs:
            base = cells.get((task, "hawktui", "c1"))
            comp = cells.get((task, "hawktui", target))
            if not base or not comp:
                continue
            print(f"  {task}  c1 (n={len(base)}) -> {target} (n={len(comp)})")
            for metric, label, places, direction in METRICS:
                # Same rule as the per-cell table: a contract failure never
                # counts as a behavioural score of zero.
                keep = base, comp
                if metric == "score":
                    keep = tuple(
                        [r for r in rows if not r.get("contract_failed")] for rows in keep
                    )
                a = [float(r.get(metric) or 0) for r in keep[0]]
                b = [float(r.get(metric) or 0) for r in keep[1]]
                if not a or not b:
                    continue
                ci = bootstrap_ci(a, b)
                if ci is None:
                    delta = st.median(b) - st.median(a)
                    print(f"    {label:<14} {delta:+.{places}f}   "
                          f"(n too small for a CI; treat as anecdote)")
                    continue
                lo, delta, hi = ci
                intervals += 1
                crosses = lo <= 0 <= hi
                verdict = "no effect" if crosses else (
                    "better" if ((delta < 0) == (direction == "lower")) else "WORSE"
                )
                if not crosses:
                    flagged.append(f"{task} c1->{target} {label} ({verdict})")
                print(f"    {label:<14} {delta:+.{places}f}  "
                      f"95% CI [{lo:+.{places}f}, {hi:+.{places}f}]  {verdict}")
            print()

    # ------------------------------------------------- multiple comparisons
    # Every extra interval is another chance to see an effect that is not
    # there. Reporting the count alongside the hits keeps a lucky interval from
    # being read as a result.
    if intervals:
        expected_false = 0.05 * intervals
        print(f"{intervals} confidence intervals computed; at 95% roughly "
              f"{expected_false:.1f} are expected to exclude zero by chance alone.")
        if flagged:
            print("intervals excluding zero:")
            for item in flagged:
                print(f"  {item}")
            if len(flagged) <= expected_false + 1:
                print("  -> no more than chance predicts. Suggestive at best; "
                      "confirm with more replicates before reporting it.")
        else:
            print("no interval excludes zero.")
        print()

    # ------------------------------------------------------------- integrity
    thin = [k for k, v in cells.items() if len(v) < 3]
    if thin:
        print("cells with fewer than 3 valid runs — not enough for an interval:")
        for task, fw, cond in sorted(thin):
            print(f"  {task} {fw} {cond} (n={len(cells[(task, fw, cond)])})")
    contract = [r for r in records if r.get("contract_failed")]
    if contract:
        print(f"\n{len(contract)} run(s) failed the harness contract rather than the UI:")
        for r in contract:
            print(f"  {r['label']}: {r.get('run_error', 'malformed frame dump')[:100]}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
