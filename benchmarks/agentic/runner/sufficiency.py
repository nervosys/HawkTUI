#!/usr/bin/env python3
"""Could the ontology have answered what the agent went to the source for?

    python sufficiency.py ../results/skeleton/runs.rescored.jsonl

The measured result is that agents read the implementation whether or not an
ontology is available. That is a fact about how today's models are trained --
they ground themselves in code -- and not evidence that the ontology lacks the
information.

The question that survives that confound is sufficiency: for every file the
agent opened, does the ontology describe that file's public API? Where it does,
an agent trained to prefer the ontology would have had no reason to open it.
Where it does not, the gap is real and training would not close it.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SRC = ROOT / "src"
SOURCE_RX = re.compile(r"(?:hawktui-snapshot|HawkTUI)[\\/]+(src[\\/][^\"']*?\.rs)", re.I)

# Trait methods and derives an author never calls directly.
SKIP = {
    "fmt", "clone", "default", "eq", "ne", "hash", "cmp", "partial_cmp", "draw",
    "schema", "capabilities", "actions", "semantic_role", "agent_state",
    "execute_action", "agent_id", "accessibility_label",
}


def catalog() -> dict[str, set[str]]:
    """Function names the ontology describes, keyed by module tail.

    Parsed by splitting on type blocks rather than one regex spanning them: a
    non-greedy match across `ApiType` boundaries silently captured nothing and
    reported every file as an uncovered gap.
    """
    generated = (SRC / "ontology" / "api_generated.rs").read_text(encoding="utf-8")
    by_module: dict[str, set[str]] = {}
    for block in generated.split("ApiType {")[1:]:
        m = re.search(r'module: "([^"]+)"', block)
        if not m:
            continue
        module = m.group(1).rsplit("::", 1)[-1]
        names = set(re.findall(r'ApiFn \{ name: "(\w+)"', block))
        by_module.setdefault(module, set()).update(names)
    return by_module


def pub_fns(path: Path) -> set[str]:
    if not path.is_file():
        return set()
    text = path.read_text(encoding="utf-8")
    return {n for n in re.findall(r"pub fn (\w+)\s*\(", text)} - SKIP



# Transcripts are not committed (see results/.gitignore), so a fresh clone has
# the summary records but none of the per-run detail these tools read. Counting
# what is missing turns a confident empty table into an honest "no data".
def warn_if_missing(missing: list, total: int) -> bool:
    """True when there is nothing to analyse. Prints why."""
    if not missing:
        return False
    if len(missing) == total:
        print(f"no transcripts found for any of the {total} runs.\n"
              "Transcripts are not committed; re-run the grid, or point this at "
              "a results directory that still has its per-run subdirectories.")
        return True
    print(f"warning: {len(missing)} of {total} runs have no transcript; "
          f"the numbers below cover the remaining {total - len(missing)}.\n")
    return False


def main() -> int:
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")
        except AttributeError:
            pass

    paths = [Path(p) for p in sys.argv[1:] if Path(p).is_file()]
    if not paths:
        print("usage: sufficiency.py <runs.rescored.jsonl> [more...]")
        return 2

    reads = Counter()
    absent: list = []
    seen = 0
    for jsonl in paths:
        root = jsonl.parent
        for line in jsonl.read_text(encoding="utf-8").splitlines():
            if not line.strip():
                continue
            record = json.loads(line)
            t = root / record["label"] / "_transcript.jsonl"
            seen += 1
            if not t.is_file():
                absent.append(record["label"])
                continue
            for raw in t.read_text(encoding="utf-8", errors="replace").splitlines():
                raw = raw.strip()
                if not raw.startswith("{"):
                    continue
                try:
                    event = json.loads(raw)
                except json.JSONDecodeError:
                    continue
                if event.get("type") != "assistant":
                    continue
                for block in event.get("message", {}).get("content", []) or []:
                    if block.get("type") != "tool_use":
                        continue
                    hit = SOURCE_RX.search(json.dumps(block.get("input", {})).replace("\\\\", "/"))
                    if hit:
                        reads[hit.group(1).replace("\\", "/")] += 1

    described = catalog()
    if warn_if_missing(absent, seen):
        return 1

    print(f"{'file read':<28}{'reads':>6}{'pub fns':>9}{'described':>11}{'coverage':>10}")
    print("-" * 64)
    covered_reads = uncovered_reads = 0
    gaps: list[tuple[str, int, list[str]]] = []

    for rel, n in reads.most_common():
        path = ROOT / rel
        fns = pub_fns(path)
        # Match on the file stem, and also on its parent for mod.rs files.
        stem = Path(rel).stem
        keys = {stem, Path(rel).parent.name}
        known = set().union(*(described.get(k, set()) for k in keys))
        if not fns:
            # A file with no inherent public functions (traits, re-exports).
            print(f"{rel:<28}{n:>6}{0:>9}{'—':>11}{'n/a':>10}")
            continue
        hit = fns & known
        pct = 100 * len(hit) / len(fns)
        if pct >= 80:
            covered_reads += n
        else:
            uncovered_reads += n
            gaps.append((rel, n, sorted(fns - known)[:8]))
        print(f"{rel:<28}{n:>6}{len(fns):>9}{len(hit):>11}{pct:>9.0f}%")

    total = covered_reads + uncovered_reads
    if total:
        print(f"\n{covered_reads}/{total} reads ({100 * covered_reads / total:.0f}%) were of files "
              f"whose API the ontology already describes.")
        print("An agent trained to prefer the ontology had no need to open those.")
    if gaps:
        print(f"\n{len(gaps)} file(s) the ontology does not adequately describe:")
        for rel, n, missing in gaps:
            print(f"  {rel} ({n} reads) missing: {', '.join(missing)}")
        print("\nThese are real gaps: training would not close them.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
