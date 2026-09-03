#!/usr/bin/env python3
"""What is the agent actually looking for in the framework's source?

    python why_source.py ../results/triggers/runs.rescored.jsonl

The ontology is free, structured, and consulted in 83% of runs, and the agent
still reads the implementation 16-22 times per run. That is only explicable by
looking at what it reads and when, so this reconstructs the tool sequence:
which files, in what order, and whether a read directly follows an ontology
query on the same type.
"""

from __future__ import annotations

import json
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

SOURCE_RX = re.compile(r"(?:hawktui-snapshot|HawkTUI)[\\/]+(.*?\.rs)", re.I)
MCP_PREFIX = "mcp__hawktui__"


def events(transcript: Path):
    """(kind, detail) per tool call, in order."""
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
        for block in event.get("message", {}).get("content", []) or []:
            if block.get("type") != "tool_use":
                continue
            name = str(block.get("name", ""))
            payload = json.dumps(block.get("input", {}))
            if name.startswith(MCP_PREFIX):
                arg = block.get("input", {})
                target = arg.get("name") or arg.get("query") or ""
                yield ("ontology", f"{name[len(MCP_PREFIX):]}({target})")
                continue
            hit = SOURCE_RX.search(payload.replace("\\\\", "/"))
            if hit:
                yield ("source", hit.group(1).replace("\\", "/"))


def main() -> int:
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")
        except AttributeError:
            pass

    paths = [Path(p) for p in sys.argv[1:] if Path(p).is_file()]
    if not paths:
        print("usage: why_source.py <runs.rescored.jsonl> [more...]")
        return 2

    files = Counter()
    kinds = Counter()
    after_ontology = Counter()
    sequences: list[list[tuple[str, str]]] = []

    for jsonl in paths:
        root = jsonl.parent
        for line in jsonl.read_text(encoding="utf-8").splitlines():
            if not line.strip():
                continue
            record = json.loads(line)
            t = root / record["label"] / "_transcript.jsonl"
            if not t.is_file():
                continue
            seq = list(events(t))
            sequences.append(seq)
            for i, (kind, detail) in enumerate(seq):
                if kind != "source":
                    continue
                files[detail] += 1
                # Categorise what part of the framework it went to.
                if "/examples/" in detail or detail.startswith("examples/"):
                    kinds["examples"] += 1
                elif "/widget/" in detail:
                    kinds["widget source"] += 1
                elif "/tests/" in detail or detail.startswith("tests/"):
                    kinds["tests"] += 1
                else:
                    kinds["other core source"] += 1
                # Did it read source immediately after asking the ontology?
                if i and seq[i - 1][0] == "ontology":
                    after_ontology[seq[i - 1][1].split("(")[0]] += 1

    print(f"{sum(files.values())} source reads across {len(sequences)} runs\n")
    print("what it reads:")
    for kind, n in kinds.most_common():
        print(f"  {kind:<20} {n:>5}  ({100 * n / sum(kinds.values()):.0f}%)")

    print("\nmost-read files:")
    for f, n in files.most_common(12):
        print(f"  {n:>4}  {f}")

    if after_ontology:
        total_follow = sum(after_ontology.values())
        print(f"\n{total_follow} source reads came directly after an ontology call:")
        for tool, n in after_ontology.most_common():
            print(f"  {n:>4}  after {tool}")
        print("  -> the ontology answer did not settle the question.")

    # Where in the run does source reading start and cluster?
    firsts = [next((i for i, (k, _) in enumerate(s) if k == "source"), None) for s in sequences]
    firsts = [f for f in firsts if f is not None]
    if firsts:
        firsts.sort()
        print(f"\nfirst source read at tool call #{firsts[len(firsts) // 2]} (median)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
