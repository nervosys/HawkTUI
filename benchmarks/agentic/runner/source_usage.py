#!/usr/bin/env python3
"""Did the agent read the framework's source instead of its ontology?

    python source_usage.py ../results/*/runs.rescored.jsonl

Every condition grants the agent `--add-dir <framework source>`, because
withholding it from the competitors would have handed Hawk TUI an unearned
advantage. But it also means the agent can read the implementation, and an
ontology is redundant against the code it was derived from.

If the agent routinely greps the source, that is a far better explanation for
the flat ontology results than anything about the ontology itself — and it is
checkable from the stored transcripts.
"""

from __future__ import annotations

import json
import sys
from collections import defaultdict
from pathlib import Path

# Paths that only appear when the agent is reading the framework's own code.
SOURCE_MARKERS = ("src/widget", "src\\widget", "src/layout", "src\\layout",
                  "src/core", "src\\core", "hawktui-snapshot", "ratatui-0.29",
                  "superlighttui-0.23")
READ_TOOLS = ("Read", "Grep", "Glob", "Bash", "PowerShell")


def scan(transcript: Path) -> dict:
    hits = {"source_reads": 0, "first_source_turn": None, "turns": 0}
    if not transcript.is_file():
        return hits
    for line in transcript.read_text(encoding="utf-8", errors="replace").splitlines():
        line = line.strip()
        if not line.startswith("{"):
            continue
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        if event.get("type") != "assistant":
            continue
        hits["turns"] += 1
        for block in event.get("message", {}).get("content", []) or []:
            if block.get("type") != "tool_use":
                continue
            if str(block.get("name", "")) not in READ_TOOLS:
                continue
            payload = json.dumps(block.get("input", {}))
            if any(m in payload for m in SOURCE_MARKERS):
                hits["source_reads"] += 1
                if hits["first_source_turn"] is None:
                    hits["first_source_turn"] = hits["turns"]
    return hits


def main() -> int:
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")
        except AttributeError:
            pass

    paths = [Path(p) for p in sys.argv[1:] if Path(p).is_file()]
    if not paths:
        print("usage: source_usage.py <runs.rescored.jsonl> [more...]")
        return 2

    by_cond: dict[str, list[dict]] = defaultdict(list)
    for jsonl in paths:
        root = jsonl.parent
        for line in jsonl.read_text(encoding="utf-8").splitlines():
            if not line.strip():
                continue
            record = json.loads(line)
            if record.get("invalid"):
                continue
            hits = scan(root / record["label"] / "_transcript.jsonl")
            hits["framework"] = record["framework"]
            by_cond[f"{record['framework']}/{record['condition']}"].append(hits)

    print(f"{'framework/cond':<22}{'runs':>5}{'read source':>13}{'rate':>7}"
          f"{'reads':>8}{'median 1st turn':>17}")
    print("-" * 72)
    for key in sorted(by_cond):
        runs = by_cond[key]
        readers = [r for r in runs if r["source_reads"]]
        firsts = sorted(r["first_source_turn"] for r in readers)
        median_first = firsts[len(firsts) // 2] if firsts else "—"
        print(f"{key:<22}{len(runs):>5}{len(readers):>8}/{len(runs):<4}"
              f"{100 * len(readers) / len(runs):>6.0f}%"
              f"{sum(r['source_reads'] for r in runs):>8}{median_first:>17}")

    total = [r for runs in by_cond.values() for r in runs]
    readers = [r for r in total if r["source_reads"]]
    print(f"\n{len(readers)}/{len(total)} runs ({100 * len(readers) / len(total):.0f}%) "
          f"read the framework's source directly.")
    if len(readers) / len(total) > 0.5:
        print("The agent has the implementation. An ontology derived from that same\n"
              "source cannot tell it anything the source does not, so the flat\n"
              "ontology results may be measuring redundancy rather than value.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
