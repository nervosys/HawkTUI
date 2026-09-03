#!/usr/bin/env python3
"""Self-test for the unicode-width rung, T13.

This rung exists because every earlier task scored 1.000: the agent never fails,
so nothing about reliability can be measured. Its checks assert *display column*
alignment, which is where an implementation is most likely to be wrong while
still compiling — a compiler cannot catch a table misaligned by treating a CJK
ideograph as one column.

The failure modes below are the plausible ones, and each must be caught:

  - padding by character count instead of display width (the classic bug)
  - counting display width where the task asks for characters
  - getting the total wrong by summing characters rather than columns

    python selftest_unicode.py
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import selftest  # noqa: E402
from selftest import expect  # noqa: E402
from verify import display_width, score  # noqa: E402

TASKS = Path(__file__).resolve().parents[1] / "tasks"
W, H = 60, 12

ROWS = [
    ("ascii", "hello", 5),
    ("cjk", "日本語", 3),
    ("emoji", "🙂🙂", 2),
    ("accent", "café", 4),
    ("mixed", "a日b", 3),
]


def pad_to(text: str, columns: int) -> str:
    """Pad `text` with spaces until it occupies `columns` display columns."""
    return text + " " * max(0, columns - display_width(text))


def frame(correct_padding: bool = True, correct_counts: bool = True,
          total: int | None = None) -> str:
    """Render the table, optionally with a specific defect."""
    body = []
    for label, sample, count in ROWS:
        if correct_padding:
            left = pad_to(label, 8) + pad_to(sample, 12)
        else:
            # The classic bug: pad by character count, so any row containing a
            # wide glyph ends up short by one column per wide character.
            left = label.ljust(8) + sample.ljust(12)
        shown = count if correct_counts else display_width(sample)
        body.append(left + str(shown))

    rows = ["┌─ Widths " + "─" * (W - 11) + "┐"]
    for i in range(H - 2):
        inner = body[i] if i < len(body) else ""
        rows.append("│" + pad_to(inner, W - 2) + "│")
    rows.append("└" + "─" * (W - 2) + "┘")
    rows = rows[: H - 1]
    if total is None:
        total = sum(display_width(s) for _, s, _ in ROWS)
    rows.append(f"total width: {total}".ljust(W))
    return "\n".join(rows)


def main() -> int:
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")
        except AttributeError:
            pass

    t13 = TASKS / "t13-unicode"

    print("t13-unicode")
    expect("correct", score(t13, frame()), want_score=1.0,
           want_contract_failed=False, want_failed_ids=set())

    # Padding by characters. The labels are ASCII, so the sample column still
    # lands correctly; the error only shows up once a wide sample has been
    # padded, which is why it is the count column that moves.
    expect("pads by characters, not columns", score(t13, frame(correct_padding=False)),
           want_failed_ids={"cjk-count-column", "emoji-count-column",
                            "mixed-count-column"})

    # Reports display width where the task asks for a character count.
    expect("counts columns, not characters", score(t13, frame(correct_counts=False)),
           want_failed_ids={"cjk-count-column", "emoji-count-column",
                            "mixed-count-column"})

    # Sums characters rather than display columns: 5+3+2+4+3 = 17, not 23.
    expect("totals characters, not columns", score(t13, frame(total=17)),
           want_failed_ids={"total-display-width"})

    print()
    if selftest.FAILURES:
        print(f"{len(selftest.FAILURES)} unicode self-test failure(s):")
        for f in selftest.FAILURES:
            print(f"  - {f}")
        return 1
    print("unicode self-test passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
