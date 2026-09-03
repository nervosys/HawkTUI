#!/usr/bin/env python3
"""Did the agent actually consult the ontology?

    python ontology_usage.py ../results/*/runs.rescored.jsonl

A null result from the C2 and C3 conditions has two very different
explanations, and the benchmark alone cannot tell them apart:

  1. The agent read the ontology and it did not help.
  2. The agent never opened it.

Only the first is evidence about the ontology's content. This scans the stored
transcripts for the tool calls and file reads that each condition depends on,
so the difference is measurable rather than assumed.
"""

from __future__ import annotations

import json
import sys
from collections import defaultdict
from pathlib import Path

# C3 seeds ONTOLOGY-TOOL.md, which tells the agent to run the query tool.
TOOL_MARKERS = ("ontology_query", "hawktui-ontology")
# C2 seeds these two files and says they are in the working directory.
PACK_FILES = ("ONTOLOGY.json", "ONTOLOGY.md")
# C3's instructions themselves are a file the agent may or may not open.
TOOL_DOC = "ONTOLOGY-TOOL.md"
# C4 serves the ontology over MCP, so its use shows up as a tool *name* rather
# than as a path inside a Bash command or a Read.
MCP_PREFIX = "mcp__hawktui__"
# The authoring half of the MCP surface, added after the widget-schema
# ontology was found to answer the wrong question.
AUTHORING_TOOLS = ("widget_api", "api_search", "stateful_widgets")


def scan(transcript: Path) -> dict:
    """Count the ways this run touched the ontology."""
    hits = {"tool_calls": 0, "pack_reads": 0, "tool_doc_reads": 0,
            "mcp_calls": 0, "api_calls": 0, "turns": 0}
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
            tool = str(block.get("name", ""))
            if tool.startswith(MCP_PREFIX):
                hits["mcp_calls"] += 1
                if any(tool.endswith(t) for t in AUTHORING_TOOLS):
                    hits["api_calls"] += 1
            payload = json.dumps(block.get("input", {}))
            if any(m in payload for m in TOOL_MARKERS):
                hits["tool_calls"] += 1
            if any(f in payload for f in PACK_FILES):
                hits["pack_reads"] += 1
            if TOOL_DOC in payload:
                hits["tool_doc_reads"] += 1
    return hits


def main() -> int:
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")
        except AttributeError:
            pass

    paths = [Path(p) for arg in sys.argv[1:] for p in Path().glob(arg)] or \
            [Path(p) for p in sys.argv[1:]]
    paths = [p for p in paths if p.is_file()]
    if not paths:
        print("usage: ontology_usage.py <runs.rescored.jsonl> [more...]")
        return 2

    by_cond: dict[str, list[dict]] = defaultdict(list)
    for jsonl in paths:
        root = jsonl.parent
        for line in jsonl.read_text(encoding="utf-8").splitlines():
            if not line.strip():
                continue
            record = json.loads(line)
            if record.get("framework") != "hawktui" or record.get("invalid"):
                continue
            hits = scan(root / record["label"] / "_transcript.jsonl")
            hits["label"] = record["label"]
            by_cond[record["condition"]].append(hits)

    print(f"{'cond':<6}{'runs':>5}{'consulted':>11}{'rate':>7}{'mcp':>7}"
          f"{'api':>7}{'cli':>7}{'pack reads':>12}")
    print("-" * 55)
    for cond in sorted(by_cond):
        runs = by_cond[cond]
        consulted = sum(1 for r in runs
                        if r["tool_calls"] or r["pack_reads"] or r["mcp_calls"])
        rate = 100 * consulted / len(runs)
        print(f"{cond:<6}{len(runs):>5}{consulted:>7}/{len(runs):<3}{rate:>6.0f}%"
              f"{sum(r['mcp_calls'] for r in runs):>7}"
              f"{sum(r['api_calls'] for r in runs):>7}"
              f"{sum(r['tool_calls'] for r in runs):>7}"
              f"{sum(r['pack_reads'] for r in runs):>12}")

    print()
    for cond in ("c2", "c3", "c4", "c5"):
        runs = by_cond.get(cond, [])
        if not runs:
            continue
        silent = [r["label"] for r in runs
                  if not (r["tool_calls"] or r["pack_reads"] or r["mcp_calls"])]
        if silent:
            print(f"{cond}: {len(silent)}/{len(runs)} runs never opened the ontology")
            for label in silent[:6]:
                print(f"  {label}")
            if len(silent) > 6:
                print(f"  ... and {len(silent) - 6} more")
            print(f"  -> the {cond} null result says nothing about the ontology's "
                  f"content for these runs.")
        else:
            print(f"{cond}: every run consulted the ontology.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
