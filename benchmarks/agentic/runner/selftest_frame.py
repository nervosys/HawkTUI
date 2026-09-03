#!/usr/bin/env python3
"""Self-test for the border-alignment rung, T15.

T13 produced the only real failure in this benchmark, and it was not an
arithmetic slip: the run injected a space after each wide character, apparently
to *make* it two columns wide, so each took three. That is a wrong belief about
the rendering surface, and it is the single thing shown to break this agent.

T15 aims at it directly. Four of its five lines are exactly 10 display columns
wide while having 10, 5, 11 and 9 characters, so padding by character count
makes the right border visibly ragged.

    python selftest_frame.py
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import selftest  # noqa: E402
from selftest import expect  # noqa: E402
from verify import display_width, score  # noqa: E402

TASKS = Path(__file__).resolve().parents[1] / "tasks"
W, H = 30, 10

LINES = ["plain text", "日本語です", "café table", "🙂 ok", "mixed 語 x"]
INNER = 10


def frame(pad_by_chars: bool = False, inject_spaces: bool = False,
          inner_value: int | None = None) -> str:
    """Render the box, optionally with one of the two real failure modes."""
    body = []
    for line in LINES:
        shown = line
        if inject_spaces:
            # The T13 failure, reproduced: a space after every wide character.
            out = ""
            for ch in shown:
                out += ch
                if display_width(ch) == 2:
                    out += " "
            shown = out
        pad = INNER - (len(shown) if pad_by_chars else display_width(shown))
        body.append(shown + " " * max(0, pad))

    rows = ["┌─ Frame " + "─" * (INNER - 8) + "┐"]
    for i in range(H - 3):
        inner = body[i] if i < len(body) else " " * INNER
        rows.append("│" + inner + "│")
    rows.append("└" + "─" * INNER + "┘")
    rows = rows[: H - 1]
    n = INNER if inner_value is None else inner_value
    rows.append(f"inner: {n}".ljust(W))
    return "\n".join(rows)


def main() -> int:
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")
        except AttributeError:
            pass

    t15 = TASKS / "t15-frame"

    print("t15-frame")
    expect("correct", score(t15, frame()), want_score=1.0,
           want_contract_failed=False, want_failed_ids=set())

    # Padding by character count: lines of equal display width but different
    # character counts end up different lengths, so the border goes ragged.
    expect("pads by characters", score(t15, frame(pad_by_chars=True)),
           want_failed_ids={"border-is-aligned"})

    # The exact T13 failure: a space after each wide character.
    result = score(t15, frame(inject_spaces=True))
    failed = {c["id"] for c in result["checks"] if not c["passed"]}
    for required in ("no-injected-spaces-cjk", "border-is-aligned"):
        if required not in failed:
            selftest.FAILURES.append(f"injected spaces: {required} did not fire")
    print(f"  ok  reproduces the T13 failure: score={result['score']:.3f} "
          f"failed={sorted(failed)}")

    # The reported inner width is wrong while the layout is right.
    expect("wrong inner width", score(t15, frame(inner_value=11)),
           want_failed_ids={"inner-width"})

    print()
    if selftest.FAILURES:
        print(f"{len(selftest.FAILURES)} frame self-test failure(s):")
        for f in selftest.FAILURES:
            print(f"  - {f}")
        return 1
    print("frame self-test passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
