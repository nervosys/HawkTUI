#!/usr/bin/env python3
"""Self-test for the wrapping rung, T14.

T13 established that display-correctness tasks are the only ones this agent
fails: they compile whatever you get wrong, so there is no build error to guide
it. T14 pushes the same axis — greedy wrapping to a display-column budget, and
truncation to a display width including a one-column ellipsis.

The defects below are what a character-based implementation actually produces,
and each must be caught.

    python selftest_wrap.py
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import selftest  # noqa: E402
from selftest import expect  # noqa: E402
from verify import display_width, score  # noqa: E402

TASKS = Path(__file__).resolve().parents[1] / "tasks"
W, H = 40, 14

TEXT = "the quick 日本語 fox jumps 🙂 over café lazy dogs"
LONG = "日本語テキストが長い"


def wrap(text: str, budget: int, by_chars: bool = False) -> list[str]:
    """Greedy wrap. `by_chars` reproduces the classic bug: measuring the line
    in characters rather than display columns, which overruns the budget."""
    measure = len if by_chars else display_width
    lines, cur = [], []
    for word in text.split():
        trial = " ".join(cur + [word])
        if not cur or measure(trial) <= budget:
            cur.append(word)
        else:
            lines.append(" ".join(cur))
            cur = [word]
    lines.append(" ".join(cur))
    return lines


def truncate(text: str, budget: int, by_chars: bool = False) -> str:
    """Truncate to `budget` columns including a one-column ellipsis."""
    measure = len if by_chars else display_width
    out = ""
    for ch in text:
        if measure(out + ch) + 1 > budget - 1:
            break
        out += ch
    return out + "…"


def frame(by_chars: bool = False, lines_value: int | None = None) -> str:
    body = wrap(TEXT, 24, by_chars)
    shown = list(body) + [""] + [truncate(LONG, 10, by_chars)]

    rows = ["┌─ Text " + "─" * (W - 9) + "┐"]
    for i in range(H - 3):
        inner = shown[i] if i < len(shown) else ""
        pad = " " * max(0, (W - 2) - display_width(inner))
        rows.append("│" + inner + pad + "│")
    rows.append("└" + "─" * (W - 2) + "┘")
    rows = rows[: H - 1]
    n = len(body) if lines_value is None else lines_value
    rows.append(f"lines: {n}".ljust(W))
    return "\n".join(rows)


def main() -> int:
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")
        except AttributeError:
            pass

    t14 = TASKS / "t14-wrap"

    print("t14-wrap")
    expect("correct", score(t14, frame()), want_score=1.0,
           want_contract_failed=False, want_failed_ids=set())

    # Measuring the line in characters lets wide glyphs overrun the budget, and
    # truncation keeps too many characters.
    expect("measures lines in characters", score(t14, frame(by_chars=True)),
           want_failed_ids={
               "line-1-not-overfull", "line-2-not-overfull", "line-2-exact",
               "dogs-on-its-own-line", "line-count", "truncated-present",
               "truncated-width", "truncation-not-by-chars",
           })

    # The wrapped-line count is reported wrongly while the layout is right.
    expect("wrong line count", score(t14, frame(lines_value=2)),
           want_failed_ids={"line-count"})

    print()
    if selftest.FAILURES:
        print(f"{len(selftest.FAILURES)} wrap self-test failure(s):")
        for f in selftest.FAILURES:
            print(f"  - {f}")
        return 1
    print("wrap self-test passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
