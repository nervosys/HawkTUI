#!/usr/bin/env python3
"""Self-test for the widget-tree frame format.

The harness was built around a character grid, because a TUI has one and two
TUI frameworks can therefore be scored by the same verifier. A GUI toolkit has
no cells: DeweyGUI emits one line per widget over the identical `--headless
WxH --script --dump` contract, where the line count follows the interface
rather than the screen and `--headless 240x120` means logical pixels.

So the frame *body* is what varies, not the command line. `frame_format:
"tree"` keeps the frame splitting, the check vocabulary and the scoring, and
drops the two assumptions that do not travel: fixed row count and fixed width.

    python selftest_tree.py
"""

from __future__ import annotations

import json
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import selftest  # noqa: E402
from selftest import expect  # noqa: E402
from verify import GRID, TREE, parse_frames, score  # noqa: E402

FF = "\x0c"

# The shape DeweyGUI's UiTree::snapshot writes: two-space indent per depth,
# #agent_id, bounds rounded to whole pixels, properties sorted.
TREE_DUMP = "\n".join([
    "Column #root [0,0 240x120]",
    "  Text #title [8,8 224x24] content=Counter",
    "  Row #controls [8,40 224x32]",
    "    Button #dec [8,40 48x24] label=- enabled=true",
    "    Text #value [64,40 32x24] content=0",
    "    Button #inc [104,40 48x24] label=+ enabled=true",
])

TREE_AFTER_CLICK = TREE_DUMP.replace("content=0", "content=1")

CHECKS = {
    "id": "tree-demo",
    "name": "tree demo",
    "rung": 1,
    "frame_format": "tree",
    "grid": {"w": 240, "h": 120},
    "script": ["Down", "q"],
    "checks": [
        {"id": "quits", "kind": "frame_count", "equals": 2},
        {"id": "has-title", "kind": "contains", "frame": 0, "pattern": "content=Counter"},
        {"id": "has-buttons", "kind": "count_matching_lines",
         "pattern": "^\\s*Button ", "min": 2, "max": 2},
        {"id": "starts-at-zero", "kind": "contains", "frame": 0, "pattern": "content=0"},
        {"id": "counts-up", "kind": "contains", "frame": 1, "pattern": "content=1"},
        {"id": "changed", "kind": "frames_differ", "a": 0, "b": 1},
        {"id": "no-ansi", "kind": "absent", "frame": 0, "pattern": "\\x1b\\["},
    ],
}


def task_dir(checks: dict) -> Path:
    d = Path(tempfile.mkdtemp(prefix="tree-task-"))
    (d / "checks.json").write_text(json.dumps(checks), encoding="utf-8")
    return d


def main() -> int:
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")
        except AttributeError:
            pass

    print("tree frame format")

    # Rows are left exactly as written: padding them would invent a width the
    # renderer never had, and every width-based check is disabled anyway.
    frames = parse_frames(TREE_DUMP, 240, 120, TREE)
    if frames[0].rows[0] != "Column #root [0,0 240x120]":
        selftest.FAILURES.append("tree rows were padded or altered")
    if len(frames[0].rows) != 6:
        selftest.FAILURES.append(f"expected 6 tree rows, got {len(frames[0].rows)}")
    print(f"  ok  parses {len(frames[0].rows)} unpadded lines")

    # The same dump under the grid format is 6 rows where 120 are declared, so
    # the formats are genuinely different rather than one being a relaxation.
    grid_frames = parse_frames(TREE_DUMP, 240, 120, GRID)
    ok, _ = grid_frames[0].shape_ok()
    if ok:
        selftest.FAILURES.append("a widget tree passed grid shape checking")
    print("  ok  the same dump fails grid shape checking")

    t = task_dir(CHECKS)
    expect("correct", score(t, TREE_DUMP + FF + "\n" + TREE_AFTER_CLICK),
           want_score=1.0, want_contract_failed=False, want_failed_ids=set())

    # A tree that never changed: the interaction did nothing.
    expect("value did not change", score(t, TREE_DUMP + FF + "\n" + TREE_DUMP),
           want_failed_ids={"counts-up", "changed"})

    # A widget missing from the tree.
    missing = "\n".join(l for l in TREE_DUMP.splitlines() if "#inc" not in l)
    expect("a button is missing",
           score(t, missing + FF + "\n" + TREE_AFTER_CLICK),
           want_failed_ids={"has-buttons"})

    # Grid-only checks must announce themselves as unusable rather than
    # inventing a column number for something with no columns.
    grid_only = dict(CHECKS)
    grid_only["checks"] = CHECKS["checks"] + [
        {"id": "border", "kind": "border_column", "frame": 0, "pattern": "[|]"}
    ]
    result = score(task_dir(grid_only), TREE_DUMP + FF + "\n" + TREE_AFTER_CLICK)
    border = next(c for c in result["checks"] if c["id"] == "border")
    if border["passed"] or "character grid" not in border["detail"]:
        selftest.FAILURES.append(
            f"grid-only check gave an unhelpful result: {border}"
        )
    print(f"  ok  grid-only check refuses: {border['detail'][:60]}…")

    print()
    if selftest.FAILURES:
        print(f"{len(selftest.FAILURES)} tree self-test failure(s):")
        for f in selftest.FAILURES:
            print(f"  - {f}")
        return 1
    print("tree self-test passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
