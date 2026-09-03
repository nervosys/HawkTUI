#!/usr/bin/env python3
"""What does reading the source cost, and what would the ontology cost instead?

    python token_cost.py ../results/skeleton/runs.rescored.jsonl

The measured finding is that agents read the implementation whether or not an
ontology exists, because that is how they are trained. If a model were trained
to prefer the ontology, the saving is the difference between what those source
reads put into the context and what the equivalent ontology answers would.

This measures both from the stored transcripts: the bytes returned by every
source read, and the bytes returned by every ontology call. It does not claim
the saving is achievable today -- no such model exists to test against -- only
what is on the table if one arrives.

Tokens are estimated at 4 characters each, which is close enough for a ratio and
is stated rather than hidden.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

CHARS_PER_TOKEN = 4
SOURCE_RX = re.compile(r"(?:hawktui-snapshot|HawkTUI)[\\/]+src[\\/]", re.I)
MCP_PREFIX = "mcp__hawktui__"
READ_TOOLS = ("Read", "Grep", "Glob", "Bash", "PowerShell")


def result_text(block: dict) -> str:
    content = block.get("content")
    if isinstance(content, list):
        return "\n".join(b.get("text", "") for b in content if isinstance(b, dict))
    return str(content or "")


def measure(transcript: Path) -> dict:
    """Bytes returned by source reads and by ontology calls, per run."""
    pending: dict[str, str] = {}
    out = {"source_chars": 0, "source_calls": 0,
           "onto_chars": 0, "onto_calls": 0, "total_result_chars": 0}
    if not transcript.is_file():
        return out

    for line in transcript.read_text(encoding="utf-8", errors="replace").splitlines():
        line = line.strip()
        if not line.startswith("{"):
            continue
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue

        if event.get("type") == "assistant":
            for block in event.get("message", {}).get("content", []) or []:
                if block.get("type") != "tool_use":
                    continue
                name = str(block.get("name", ""))
                payload = json.dumps(block.get("input", {}))
                if name.startswith(MCP_PREFIX):
                    pending[block.get("id", "")] = "ontology"
                elif name in READ_TOOLS and SOURCE_RX.search(payload.replace("\\\\", "/")):
                    pending[block.get("id", "")] = "source"

        elif event.get("type") == "user":
            for block in event.get("message", {}).get("content", []) or []:
                if block.get("type") != "tool_result":
                    continue
                text = result_text(block)
                out["total_result_chars"] += len(text)
                kind = pending.pop(block.get("tool_use_id", ""), None)
                if kind == "source":
                    out["source_chars"] += len(text)
                    out["source_calls"] += 1
                elif kind == "ontology":
                    out["onto_chars"] += len(text)
                    out["onto_calls"] += 1
    return out


def main() -> int:
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")
        except AttributeError:
            pass

    paths = [Path(p) for p in sys.argv[1:] if Path(p).is_file()]
    if not paths:
        print("usage: token_cost.py <runs.rescored.jsonl> [more...]")
        return 2

    runs = []
    for jsonl in paths:
        root = jsonl.parent
        for line in jsonl.read_text(encoding="utf-8").splitlines():
            if not line.strip():
                continue
            record = json.loads(line)
            m = measure(root / record["label"] / "_transcript.jsonl")
            m["cost"] = float(record.get("cost_usd") or 0)
            m["label"] = record["label"]
            runs.append(m)

    if not runs:
        print("no runs")
        return 1

    n = len(runs)
    src_tok = sum(r["source_chars"] for r in runs) / CHARS_PER_TOKEN / n
    onto_tok = sum(r["onto_chars"] for r in runs) / CHARS_PER_TOKEN / n
    all_tok = sum(r["total_result_chars"] for r in runs) / CHARS_PER_TOKEN / n
    cost = sum(r["cost"] for r in runs) / n
    src_calls = sum(r["source_calls"] for r in runs) / n
    onto_calls = sum(r["onto_calls"] for r in runs) / n

    print(f"{n} runs, mean per run\n")
    print(f"  source reads        {src_calls:>6.1f} calls  {src_tok:>9,.0f} est. tokens returned")
    print(f"  ontology calls      {onto_calls:>6.1f} calls  {onto_tok:>9,.0f} est. tokens returned")
    print(f"  all tool results    {'':>6}        {all_tok:>9,.0f} est. tokens returned")
    if all_tok:
        print(f"\n  source reading is {100 * src_tok / all_tok:.0f}% of everything tools "
              f"returned into context")
    if onto_tok and src_tok:
        print(f"  the ontology answered in {src_tok / onto_tok:.1f}x fewer tokens per run")

    # Tool results are re-sent on every subsequent turn, so their cost compounds
    # with conversation length. This is the floor, not the true figure.
    print(f"\n  mean cost per task: ${cost:.2f}")
    if all_tok:
        share = src_tok / all_tok
        print(f"  if a trained agent skipped source reads entirely, the context they\n"
              f"  occupy — {share:.0%} of returned tokens — would be freed. Tool results are\n"
              f"  re-sent each turn, so the saving compounds with conversation length;\n"
              f"  this is a floor, not an estimate of the full effect.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
