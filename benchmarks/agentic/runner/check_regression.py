#!/usr/bin/env python3
"""Compare a run against a recorded baseline and fail on a regression.

    python check_regression.py --runs ../results/<stamp>/runs.rescored.jsonl \
                               --baseline ../baseline.json
    python check_regression.py --runs ... --baseline ... --update

Agent behaviour drifts with every model release. Without a gate, the claims in
the README go stale silently and nobody notices until someone reruns the
benchmark and gets different numbers.

What counts as a regression
---------------------------
Only the outcome metrics, and only when the change is larger than the noise
this benchmark is known to carry:

* `score` falling by more than `--score-tolerance` (default 0.05)
* `api_errors` rising by more than `--api-error-tolerance` (default 0.5)
* a cell that used to build and now does not

Cost and wall time are reported but never fail the build. They move with
pricing, machine load and model latency, none of which is a property of the
framework, and gating on them would produce false alarms that train people to
ignore the gate.
"""

from __future__ import annotations

import argparse
import json
import statistics as st
import sys
from collections import defaultdict
from pathlib import Path

# Metrics carried in a baseline. Only the first two can fail a run.
GATED = ("score", "api_errors")
REPORTED = ("built_rate", "turns", "wall_seconds", "cost_usd")


def load_runs(path: Path) -> dict[str, dict]:
    """Median metrics per cell, excluding invalid runs."""
    cells: dict[str, list[dict]] = defaultdict(list)
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        record = json.loads(line)
        if record.get("invalid") or record.get("dry_run"):
            continue
        key = f"{record['task']}/{record['framework']}/{record['condition']}"
        cells[key].append(record)

    summary = {}
    for key, records in cells.items():
        # A contract failure never counts as a behavioural score of zero.
        scored = [r for r in records if not r.get("contract_failed")]
        summary[key] = {
            "n": len(records),
            "score": st.median([float(r.get("score") or 0) for r in scored]) if scored else 0.0,
            "api_errors": st.median([float(r.get("api_errors") or 0) for r in records]),
            "built_rate": sum(1 for r in records if r.get("built")) / len(records),
            "turns": st.median([float(r.get("turns") or 0) for r in records]),
            "wall_seconds": st.median([float(r.get("wall_seconds") or 0) for r in records]),
            "cost_usd": st.median([float(r.get("cost_usd") or 0) for r in records]),
        }
    return summary


def compare(current: dict, baseline: dict, score_tol: float, api_tol: float):
    """Return (regressions, improvements, unmeasured)."""
    regressions, improvements, unmeasured = [], [], []

    for key, base in sorted(baseline.items()):
        now = current.get(key)
        if now is None:
            unmeasured.append(key)
            continue

        drop = base["score"] - now["score"]
        if drop > score_tol:
            regressions.append(
                f"{key}: score {base['score']:.3f} -> {now['score']:.3f} (-{drop:.3f})"
            )
        elif drop < -score_tol:
            improvements.append(
                f"{key}: score {base['score']:.3f} -> {now['score']:.3f} (+{-drop:.3f})"
            )

        rise = now["api_errors"] - base["api_errors"]
        if rise > api_tol:
            regressions.append(
                f"{key}: api_errors {base['api_errors']:.1f} -> {now['api_errors']:.1f} (+{rise:.1f})"
            )

        if base["built_rate"] > 0 and now["built_rate"] == 0:
            regressions.append(f"{key}: nothing builds any more (was {base['built_rate']:.0%})")

    return regressions, improvements, unmeasured


def main() -> int:
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")
        except AttributeError:
            pass

    ap = argparse.ArgumentParser()
    ap.add_argument("--runs", required=True, type=Path)
    ap.add_argument("--baseline", required=True, type=Path)
    ap.add_argument("--update", action="store_true", help="write the baseline from this run")
    ap.add_argument("--score-tolerance", type=float, default=0.05)
    ap.add_argument("--api-error-tolerance", type=float, default=0.5)
    args = ap.parse_args()

    current = load_runs(args.runs)
    if not current:
        print("no valid runs to compare")
        return 1

    if args.update or not args.baseline.exists():
        args.baseline.write_text(
            json.dumps(current, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )
        action = "updated" if args.update else "created (none existed)"
        print(f"baseline {action}: {args.baseline} ({len(current)} cells)")
        for key, cell in sorted(current.items()):
            print(f"  {key}: score {cell['score']:.3f}, api_errors "
                  f"{cell['api_errors']:.1f}, n={cell['n']}")
        return 0

    baseline = json.loads(args.baseline.read_text(encoding="utf-8"))
    regressions, improvements, unmeasured = compare(
        current, baseline, args.score_tolerance, args.api_error_tolerance
    )

    print(f"{len(current)} cells measured against {len(baseline)} in the baseline\n")
    for key in sorted(set(current) & set(baseline)):
        now, base = current[key], baseline[key]
        deltas = " ".join(
            f"{m}={now[m]:.2f}({now[m] - base[m]:+.2f})" for m in REPORTED if m in base
        )
        print(f"  {key}: score={now['score']:.3f}"
              f"({now['score'] - base['score']:+.3f}) {deltas}")

    if unmeasured:
        print(f"\n{len(unmeasured)} baseline cell(s) not covered by this run:")
        for key in unmeasured:
            print(f"  {key}")

    if improvements:
        print("\nimprovements:")
        for item in improvements:
            print(f"  {item}")

    if regressions:
        print(f"\n{len(regressions)} REGRESSION(S):")
        for item in regressions:
            print(f"  {item}")
        return 1

    print("\nno regression")
    return 0


if __name__ == "__main__":
    sys.exit(main())
