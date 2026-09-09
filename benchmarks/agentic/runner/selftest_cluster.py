#!/usr/bin/env python3
"""Self-test for the grapheme-cluster rung, T17.

T16 narrowed onto emitting a wide character. T17 narrows one step further, onto
the unit being measured: a family emoji is five codepoints, one cluster and two
columns, and a thumbs-up with a skin tone is two codepoints, one cluster and two
columns. Every naive unit -- codepoint, char, byte -- gives the wrong answer, and
they give different wrong answers, so the rung distinguishes them.

The verifier could not measure this until the ZWJ and skin-tone fix: it scored
the family at eight columns and the toned thumb at four. This self-test would
have failed against that verifier, which is the point of writing it first.

    python selftest_cluster.py
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import selftest  # noqa: E402
from selftest import expect  # noqa: E402
from verify import display_width, score  # noqa: E402

TASKS = Path(__file__).resolve().parents[1] / "tasks"
W, H = 20, 8

FAM = "\U0001F468‍\U0001F469‍\U0001F467"
TONE = "\U0001F44D\U0001F3FD"
SOURCE = [[FAM] * 11, [TONE] * 11, list("abc def")]


def wrap(clusters: list[str], width: int, by_codepoints: bool = False,
         split_clusters: bool = False) -> list[str]:
    """Greedy wrap over clusters.

    `by_codepoints` charges two columns per codepoint instead of per cluster,
    the error the old verifier itself made. `split_clusters` wraps over
    codepoints, so a family can break across a row boundary.
    """
    units = []
    for c in clusters:
        units.extend(list(c)) if split_clusters else units.append(c)
    rows, cur, used = [], "", 0
    for u in units:
        w = sum(display_width(ch) for ch in u) if by_codepoints else display_width(u)
        if used + w > width:
            rows.append(cur)
            cur, used = "", 0
        cur += u
        used += w
    if cur:
        rows.append(cur)
    return rows


def frame(by_codepoints: bool = False, split_clusters: bool = False,
          inject_spaces: bool = False, rows_value: int | None = None) -> str:
    body: list[str] = []
    for line in SOURCE:
        shown = line
        if inject_spaces:
            shown = []
            for c in line:
                shown.append(c)
                if display_width(c) == 2:
                    shown.append(" ")
        body += wrap(shown, W, by_codepoints=by_codepoints,
                     split_clusters=split_clusters)
    rows = list(body[: H - 1])
    while len(rows) < H - 1:
        rows.append("")
    n = len(body) if rows_value is None else rows_value
    rows.append(f"rows: {n}")
    return "\n".join(rows)


def main() -> int:
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")
        except AttributeError:
            pass

    t17 = TASKS / "t17-cluster"
    print("t17-cluster")

    expect("correct", score(t17, frame()), want_score=1.0,
           want_contract_failed=False, want_failed_ids=set())

    # Charging per codepoint is what the verifier itself did before the fix:
    # the family becomes eight columns, so barely two fit on a row.
    result = score(t17, frame(by_codepoints=True))
    failed = {c["id"] for c in result["checks"] if not c["passed"]}
    if "family-fills-row-0" not in failed:
        selftest.FAILURES.append("counts codepoints: the row-0 check did not fire")
    print(f"  ok  counts codepoints as columns: score={result['score']:.3f} "
          f"failed={sorted(failed)}")

    # Wrapping over codepoints splits a family across the row boundary, which
    # is the failure this rung exists to catch.
    result = score(t17, frame(split_clusters=True))
    failed = {c["id"] for c in result["checks"] if not c["passed"]}
    if "family-never-split" not in failed:
        selftest.FAILURES.append("splits clusters: family-never-split did not fire")
    print(f"  ok  splits clusters at the boundary: score={result['score']:.3f} "
          f"failed={sorted(failed)}")

    # The T13/T15 failure mode, carried forward.
    result = score(t17, frame(inject_spaces=True))
    failed = {c["id"] for c in result["checks"] if not c["passed"]}
    if "no-injected-spaces" not in failed:
        selftest.FAILURES.append("injected spaces: no-injected-spaces did not fire")
    print(f"  ok  injects spaces after each glyph: score={result['score']:.3f} "
          f"failed={sorted(failed)}")

    # Splitting cannot drop a skin tone -- the modifier is zero-width, so it
    # rides along -- so the bare-thumb check gets its own frame.
    bare = frame().replace(TONE, "👍")
    result = score(t17, bare)
    failed = {c["id"] for c in result["checks"] if not c["passed"]}
    if "tone-modifier-kept" not in failed:
        selftest.FAILURES.append("bare thumbs-up: tone-modifier-kept did not fire")
    print(f"  ok  drops the skin tone: score={result['score']:.3f} "
          f"failed={sorted(failed)}")

    # A correct layout that miscounts its own rows fails only the status line.
    expect("miscounts rows", score(t17, frame(rows_value=4)),
           want_failed_ids={"row-count"})

    print()
    if selftest.FAILURES:
        print(f"{len(selftest.FAILURES)} cluster self-test failure(s):")
        for f in selftest.FAILURES:
            print(f"  - {f}")
        return 1
    print("cluster self-test passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
