#!/usr/bin/env python3
"""Self-test for the discovery rungs, T7-T9.

Same discipline as `selftest.py`: synthesise one correct dump per task and
several with a single deliberate defect each, and assert the verifier scores
them the way a human would. Checks written but never exercised are as
untrustworthy as code that was never run — and these three tasks carry the
ontology contrast, so a check that cannot fail would quietly manufacture a
result.

    python selftest_discovery.py
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import selftest  # noqa: E402
from selftest import box, expect  # noqa: E402
from verify import score  # noqa: E402

TASKS = Path(__file__).resolve().parents[1] / "tasks"
FF = "\x0c"


def pad(text: str, width: int) -> str:
    return text.ljust(width)[:width]


def beside(left: list[str], right: list[str]) -> list[str]:
    """Join two equal-height blocks of rows side by side."""
    return [a + b for a, b in zip(left, right)]


# ------------------------------------------------------------- t7 settings

W7, H7 = 80, 24
THEMES = ["Dark", "Light", "Solarized"]
SIZES = ["12", "14", "16"]
VIM = ["Off", "On"]


def settings_frame(sel: int, theme: int, size: int, vim: int) -> str:
    values = [("Theme", THEMES[theme]), ("Font Size", SIZES[size]), ("Vim Mode", VIM[vim])]
    body = [
        ("> " if i == sel else "  ") + f"{name}: {value}"
        for i, (name, value) in enumerate(values)
    ]
    rows = box("Settings", W7, H7 - 1, body)
    rows.append(pad("Enter cycle  q quit", W7))
    return "\n".join(rows)


def settings_dump(states: list[tuple[int, int, int, int]]) -> str:
    return FF.join(settings_frame(*state) for state in states)


# --------------------------------------------------------------- t8 meters

W8, H8 = 80, 24
SPINNER = ["|", "/", "-", "\\"]


def meters_frame(percent: int, spin: int) -> str:
    filled = max(3, percent // 5)
    body = [
        "█" * filled + "░" * (60 - filled) + f" {percent}%",
        "─" * filled + " " * (60 - filled) + f" {percent}%",
        SPINNER[spin % len(SPINNER)],
    ]
    rows = box("Transfer", W8, H8 - 1, body)
    rows.append(pad("u up  d down  t tick  q quit", W8))
    return "\n".join(rows)


def meters_dump(states: list[tuple[int, int]]) -> str:
    return FF.join(meters_frame(*state) for state in states)


# ---------------------------------------------------------------- t9 atlas

W9, H9 = 100, 30
MONTHS = {
    (2026, 2): "February 2026",
    (2026, 3): "March 2026",
    (2026, 4): "April 2026",
}


def atlas_frame(year: int, month: int, highlight: bool = True) -> str:
    plot_body = [
        "  ⠀⠤⠒⠉⠉⠒⠤⠀",
        " ⠔⠁      ⠈⠢",
        "⠸⠀        ⠀⠇",
        " ⠢⡀      ⢀⠔",
        "  ⠀⠤⠒⠉⠉⠒⠤⠀",
        "     ⠈⠉⠁",
    ]
    day15 = "[15]" if highlight else "  15"
    cal_body = [
        MONTHS[(year, month)],
        "Mo Tu We Th Fr Sa Su",
        " 1  2  3  4  5  6  7",
        " 8  9 10 11 12 13 14",
        f"{day15} 16 17 18 19 20 21",
        "22 23 24 25 26 27 28",
    ]
    left = box("Plot", W9 // 2, H9 - 1, plot_body)
    right = box("Calendar", W9 // 2, H9 - 1, cal_body)
    rows = beside(left, right)
    rows.append(pad("n next  p prev  q quit", W9))
    return "\n".join(rows)


def atlas_dump(months: list[tuple[int, int]], highlight: bool = True) -> str:
    return FF.join(atlas_frame(y, m, highlight) for y, m in months)


def main() -> int:
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")
        except AttributeError:
            pass

    t7, t8, t9 = TASKS / "t7-settings", TASKS / "t8-meters", TASKS / "t9-atlas"

    print("t7-settings")
    # Enter cycles Theme, Down selects Font Size, Right twice: 14 -> 16 -> 12.
    correct7 = [(0, 0, 1, 0), (0, 1, 1, 0), (1, 1, 1, 0), (1, 1, 2, 0), (1, 1, 0, 0)]
    expect("correct", score(t7, settings_dump(correct7)), want_score=1.0,
           want_contract_failed=False, want_failed_ids=set())
    # Cycling stops at the last value instead of wrapping.
    no_wrap = correct7[:-1] + [(1, 1, 2, 0)]
    expect("cycle does not wrap", score(t7, settings_dump(no_wrap)),
           want_failed_ids={"cycle-wraps"})
    # Enter changes every setting, not just the selected one.
    bleed = [(0, 0, 1, 0), (0, 1, 2, 1), (1, 1, 2, 1), (1, 1, 0, 1), (1, 1, 1, 1)]
    expect("cycle changes every setting", score(t7, settings_dump(bleed)),
           want_failed_ids={"cycle-leaves-others-alone", "right-cycles", "cycle-wraps"})
    # `q` ignored.
    expect("does not quit", score(t7, settings_dump(correct7 + [(1, 1, 0, 0)])),
           want_failed_ids={"quits"})

    print("t8-meters")
    correct8 = [(25, 0), (50, 0), (50, 1), (50, 2), (25, 2)]
    expect("correct", score(t8, meters_dump(correct8)), want_score=1.0,
           want_contract_failed=False, want_failed_ids=set())
    # The spinner never advances, so a tick changes nothing.
    static_spinner = [(25, 0), (50, 0), (50, 0), (50, 0), (25, 0)]
    expect("spinner never advances", score(t8, meters_dump(static_spinner)),
           want_failed_ids={"tick-changes-something", "tick-advances-again"})
    # `t` also bumps the value, which it must not.
    ticking_changes_value = [(25, 0), (50, 0), (75, 1), (100, 2), (75, 2)]
    expect("tick moves the value", score(t8, meters_dump(ticking_changes_value)),
           want_failed_ids={"tick-leaves-value-alone", "down-lowers-value"})

    print("t9-atlas")
    correct9 = [(2026, 3), (2026, 4), (2026, 3), (2026, 2)]
    expect("correct", score(t9, atlas_dump(correct9)), want_score=1.0,
           want_contract_failed=False, want_failed_ids=set())
    # The highlight is dropped once the month changes.
    mixed = FF.join(
        atlas_frame(y, m, highlight=(i == 0)) for i, (y, m) in enumerate(correct9)
    )
    expect("highlight lost on month change", score(t9, mixed),
           want_failed_ids={"highlight-follows-month"})
    # `p` is ignored, so the month never goes back.
    stuck = [(2026, 3), (2026, 4), (2026, 4), (2026, 4)]
    expect("prev does nothing", score(t9, atlas_dump(stuck)),
           want_failed_ids={"prev-returns", "prev-again"})

    print()
    if selftest.FAILURES:
        print(f"{len(selftest.FAILURES)} discovery self-test failure(s):")
        for f in selftest.FAILURES:
            print(f"  - {f}")
        return 1
    print("discovery self-test passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
