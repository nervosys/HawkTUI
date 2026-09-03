#!/usr/bin/env python3
"""Self-test for the complex canonical rungs, T10-T12.

These three are modelled on the reproductions this repository ships —
`examples/lazygit.rs`, `examples/btop.rs`, `examples/opencode.rs` — and each
carries roughly twice the checks of the earlier rungs. The more checks a task
has, the more ways one of them can be silently unfalsifiable, so every one is
exercised here against a correct dump and against defects designed to trip it.

    python selftest_complex.py
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
W, H = 100, 30


def pad(text: str, width: int = W) -> str:
    return text.ljust(width)[:width]


def beside(left: list[str], right: list[str]) -> list[str]:
    return [a + b for a, b in zip(left, right)]


# ------------------------------------------------------------------ t10 repo

FILES = ["M  src/main.rs", "A  src/lib.rs", "M  README.md", "D  old.txt", "?? notes.md"]
PATHS = ["src/main.rs", "src/lib.rs", "README.md", "old.txt", "notes.md"]
BRANCHES = ["* main", "  feature/ui", "  fix/parser"]
COMMITS = ["a1b2c3d Initial commit", "b2c3d4e Add parser",
           "c3d4e5f Fix wrapping", "d4e5f6a Update docs"]
PANES = ["Status", "Files", "Branches", "Commits"]


def repo_frame(focus: int, sel: list[int], diff_for: int | None = None,
               leak: bool = False) -> str:
    """focus: 0..3. sel: per-pane selection for Files/Branches/Commits."""
    bodies = [["On branch main", "2 staged, 3 modified"], FILES, BRANCHES, COMMITS]
    left: list[str] = []
    for i, (title, body) in enumerate(zip(PANES, bodies)):
        focused = i == focus
        marked = []
        for j, line in enumerate(body):
            if i == 0:
                marked.append(line)
            elif (focused or leak) and j == sel[i - 1]:
                marked.append("> " + line)
            else:
                marked.append("  " + line)
        left += box(("*" if focused else "") + title, 40, 7, marked)
    left = left[: H - 1]
    while len(left) < H - 1:
        left.append(" " * 40)

    shown = PATHS[sel[0] if diff_for is None else diff_for]
    right = box("Diff", 60, H - 1,
                [f"--- a/{shown}", f"+++ b/{shown}", "-old line", "+new line"])
    rows = beside(left, right)
    rows.append(pad("Tab pane  q quit"))
    return "\n".join(rows)


def repo_dump(states) -> str:
    return FF.join(repo_frame(*s) for s in states)


# --------------------------------------------------------------- t11 monitor

PROCS = [("1201", "rustc", 42, 18), ("880", "firefox", 17, 35),
         ("2310", "cargo", 63, 9), ("145", "systemd", 3, 2), ("1990", "zsh", 11, 4)]


def monitor_frame(bump: int, order: str) -> str:
    cores = [(f"CPU{i}", (10 * (i + 1) + bump) % 105) for i in range(4)]
    cpu_body = [f"{n} {v}% " + "█" * max(3, v // 5) for n, v in cores]
    mem = (55 + bump) % 105
    mem_body = [f"MEM {mem}% " + "█" * max(3, mem // 5), "6.4 GiB / 16.0 GiB"]
    left = box("CPU", 50, 8, cpu_body)
    right = box("Memory", 50, 8, mem_body)
    rows = beside(left, right)
    rows += box("Network", W, 7, ["▁▃▂▅▄▆▃▇"])

    procs = list(PROCS)
    if order == "desc":
        procs.sort(key=lambda p: -p[2])
    elif order == "asc":
        procs.sort(key=lambda p: p[2])
    table = ["PID    NAME      CPU%  MEM%"]
    table += [f"{p:<6} {n:<9} {c:<5} {m}" for p, n, c, m in procs]
    rows += box("Processes", W, H - 1 - len(rows), table)
    rows.append(pad("s sort  r refresh  q quit"))
    return "\n".join(rows)


def monitor_dump(states) -> str:
    return FF.join(monitor_frame(*s) for s in states)


# ------------------------------------------------------------------ t12 chat

ALPHA = ["you: how do I center text", "bot: use the center alignment"]
BETA = ["you: what is a gauge", "bot: a bar showing a ratio from 0 to 1"]


def chat_frame(session: int, alpha: list[str], beta: list[str], composer: str) -> str:
    bar = "[Alpha] Beta" if session == 0 else "Alpha  [Beta]"
    msgs = alpha if session == 0 else beta
    rows = [pad(bar)]
    rows += box("Transcript", W, H - 5, msgs)
    rows += box("Message", W, 3, [composer])
    rows.append(pad("Enter send  Tab session  Esc quit"))
    return "\n".join(rows)


def chat_dump(states) -> str:
    return FF.join(chat_frame(*s) for s in states)


def main() -> int:
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")
        except AttributeError:
            pass

    t10, t11, t12 = TASKS / "t10-repo", TASKS / "t11-monitor", TASKS / "t12-chat"

    print("t10-repo")
    # Tab -> Files, Down, Down, Tab -> Branches, Down.
    correct = [(0, [0, 0, 0]), (1, [0, 0, 0]), (1, [1, 0, 0]),
               (1, [2, 0, 0]), (2, [2, 0, 0]), (2, [2, 1, 0])]
    expect("correct", score(t10, repo_dump(correct)), want_score=1.0,
           want_contract_failed=False, want_failed_ids=set())
    # The diff pane follows whichever pane has focus instead of Files.
    wrong_diff = correct[:4] + [(2, [2, 0, 0], 0), (2, [2, 1, 0], 0)]
    expect("diff follows focus", score(t10, repo_dump(wrong_diff)),
           want_failed_ids={"file-selection-persists", "diff-unaffected-by-branch-pane"})
    # Every pane draws its selection marker, focused or not.
    leaky = FF.join(repo_frame(f, sel, leak=True) for f, sel in correct)
    expect("markers in unfocused panes", score(t10, leaky),
           want_failed_ids={"no-marker-when-unfocused"})

    print("t11-monitor")
    correct11 = [(0, "none"), (5, "none"), (5, "desc"), (5, "asc")]
    expect("correct", score(t11, monitor_dump(correct11)), want_score=1.0,
           want_contract_failed=False, want_failed_ids=set())
    # `r` bumps nothing.
    expect("refresh does nothing", score(t11, monitor_dump([(0, "none"), (0, "none"), (0, "desc"), (0, "asc")])),
           want_failed_ids={"refresh-raises-cores", "refresh-raises-memory",
                            "sort-leaves-meters-alone"})
    # `s` sorts once and never toggles.
    expect("sort never toggles", score(t11, monitor_dump([(0, "none"), (5, "none"), (5, "desc"), (5, "desc")])),
           want_failed_ids={"sort-toggles-ascending"})

    print("t12-chat")
    sent_a = ALPHA + ["you: hi", "bot: ok"]
    correct12 = [(0, ALPHA, BETA, ""), (0, ALPHA, BETA, "h"), (0, ALPHA, BETA, "hi"),
                 (0, sent_a, BETA, ""), (1, sent_a, BETA, "")]
    expect("correct", score(t12, chat_dump(correct12)), want_score=1.0,
           want_contract_failed=False, want_failed_ids=set())
    # Markdown markers left on screen.
    raw = [(0, ["you: how do I center text", "bot: use the **center** alignment"], BETA, "")]
    expect("markdown not rendered", score(t12, chat_dump(raw + correct12[1:])),
           want_failed_ids={"markdown-bold-stripped", "no-raw-asterisks"})
    # Sessions share one transcript.
    shared = correct12[:4] + [(1, sent_a, sent_a, "")]
    expect("sessions share a transcript", score(t12, chat_dump(shared)),
           want_failed_ids={"beta-has-its-own-messages", "beta-code-span-stripped",
                            "sessions-are-separate"})

    print()
    if selftest.FAILURES:
        print(f"{len(selftest.FAILURES)} complex self-test failure(s):")
        for f in selftest.FAILURES:
            print(f"  - {f}")
        return 1
    print("complex self-test passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
