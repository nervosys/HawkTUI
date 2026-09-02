#!/usr/bin/env python3
"""Re-score completed runs from their saved frame dumps.

    python rescore.py ../results/main2/runs.jsonl

Writes `runs.rescored.jsonl` beside the input and leaves the original intact, so
a scoring change can always be audited against what was originally recorded.

This exists because the verifier is code and code has bugs. The first version of
`parse_frames` stripped every trailing blank row, so a program that correctly
emitted a full-height screen with an empty lower half was recorded as a contract
violation. Re-scoring the stored dumps repairs those records without re-running
a single agent.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from verify import score as score_dump  # noqa: E402

TASKS = Path(__file__).resolve().parents[1] / "tasks"


def main() -> int:
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")
        except AttributeError:
            pass

    if len(sys.argv) != 2:
        print("usage: rescore.py <runs.jsonl>")
        return 2

    src = Path(sys.argv[1])
    run_root = src.parent
    dst = src.with_name("runs.rescored.jsonl")

    changed = 0
    records = []
    for line in src.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        record = json.loads(line)
        # An earlier runner let cargo walk up the directory tree when the
        # working directory had no Cargo.toml, so it built and reported success
        # for an unrelated package. The run directory still holds the evidence:
        # no manifest means nothing of the agent's was ever built.
        run_dir = run_root / record["label"]
        if record.get("built") and not (run_dir / "Cargo.toml").is_file():
            record["built"] = False
            record["build_error"] = (
                "no Cargo.toml in the working directory; the crate was not "
                "created where the task asked for it"
            )
            print(f"  {record['label']}: built True -> False (no manifest)")

        dump = run_root / record["label"] / "_dump.txt"
        if not dump.is_file():
            records.append(record)
            continue

        before = (record.get("score"), record.get("contract_failed"))
        result = score_dump(TASKS / record["task"], dump.read_text(encoding="utf-8"))
        record.update({k: v for k, v in result.items() if k != "task"})
        after = (record.get("score"), record.get("contract_failed"))

        if before != after:
            changed += 1
            print(f"  {record['label']}: score {before[0]} -> {after[0]}, "
                  f"contract_failed {before[1]} -> {after[1]}")
        records.append(record)

    with dst.open("w", encoding="utf-8") as fh:
        for record in records:
            fh.write(json.dumps(record) + "\n")

    print(f"\n{len(records)} runs re-scored, {changed} changed -> {dst}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
