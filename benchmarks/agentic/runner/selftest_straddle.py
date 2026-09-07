#!/usr/bin/env python3
"""Self-test for the wrap-boundary rung, T16.

Both agent failures this benchmark has produced came from *emitting* a wide
character: a space injected after each one so it would "be" two columns. T16
narrows onto the same seam at the one place the arithmetic is easiest to get
wrong -- the end of a row, where a double-width character needs two columns and
only one is left.

The correct answer leaves that column empty and starts the character on the next
row. Two wrong answers are natural enough to be worth encoding: overhanging the
last column, and splitting on character count instead of display width.

    python selftest_straddle.py
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

SOURCE = ["x日本語日本語日本語日", "🙂" * 11, "abc def"]


def wrap(line: str, width: int, by_chars: bool = False, overhang: bool = False):
    """Greedy wrap. `by_chars` counts characters; `overhang` lets a wide
    character start in the last column and spill past it."""
    rows, cur, used = [], "", 0
    for ch in line:
        w = 1 if by_chars else display_width(ch)
        if used + w > width and not (overhang and used < width):
            rows.append(cur)
            cur, used = "", 0
        cur += ch
        used += w
    if cur:
        rows.append(cur)
    return rows


def frame(by_chars: bool = False, overhang: bool = False,
          inject_spaces: bool = False, rows_value: int | None = None) -> str:
    body: list[str] = []
    for line in SOURCE:
        shown = line
        if inject_spaces:
            # The failure both T13 and T15 produced, reproduced here.
            out = ""
            for ch in shown:
                out += ch
                if display_width(ch) == 2:
                    out += " "
            shown = out
        body += wrap(shown, W, by_chars=by_chars, overhang=overhang)

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

    t16 = TASKS / "t16-straddle"

    print("t16-straddle")
    expect("correct", score(t16, frame()), want_score=1.0,
           want_contract_failed=False, want_failed_ids=set())

    # Wrapping by character count fits ten wide characters where only nine fit,
    # so row 0 is 21 display columns and the whole layout shifts.
    result = score(t16, frame(by_chars=True))
    failed = {c["id"] for c in result["checks"] if not c["passed"]}
    if "straddle-stays-off-row-0" not in failed:
        selftest.FAILURES.append("wraps by characters: the straddle check did not fire")
    print(f"  ok  wraps by characters: score={result['score']:.3f} "
          f"failed={sorted(failed)}")

    # Letting a wide character start in the last column and spill past it is the
    # specific mistake the rung exists to catch. It puts 21 columns on a
    # 20-column screen, so it fails the contract as well as the layout -- a
    # program drawing outside its terminal, which analyze.py reports as a
    # rendering bug rather than a protocol mistake.
    expect("overhangs the last column", score(t16, frame(overhang=True)),
           want_contract_failed=True,
           want_failed_ids={"shape", "straddle-stays-off-row-0",
                            "straddle-moves-to-row-1", "emoji-fills-a-row",
                            "emoji-continuation", "ascii-line", "row-count"})

    # The T13/T15 failure mode.
    result = score(t16, frame(inject_spaces=True))
    failed = {c["id"] for c in result["checks"] if not c["passed"]}
    for required in ("no-injected-spaces-cjk", "no-injected-spaces-emoji"):
        if required not in failed:
            selftest.FAILURES.append(f"injected spaces: {required} did not fire")
    print(f"  ok  reproduces the injected-space failure: score={result['score']:.3f} "
          f"failed={sorted(failed)}")

    # The layout is right but the reported row count is not.
    expect("wrong row count", score(t16, frame(rows_value=4)),
           want_failed_ids={"row-count"})

    print()
    if selftest.FAILURES:
        print(f"{len(selftest.FAILURES)} straddle self-test failure(s):")
        for f in selftest.FAILURES:
            print(f"  - {f}")
        return 1
    print("straddle self-test passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
