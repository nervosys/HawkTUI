#!/usr/bin/env python3
"""Self-test for the verifier.

Synthesises frame dumps for a few tasks — one correct, several with a single
deliberate defect each — and asserts that the verifier scores them the way a
human would. A benchmark whose instrument is untested measures nothing, and the
failure mode that matters here is a verifier that passes everything.

    python selftest.py
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from verify import score  # noqa: E402

TASKS = Path(__file__).resolve().parents[1] / "tasks"
FF = "\x0c"

W, H = 80, 24


def box(title: str, width: int, height: int, body: list[str]) -> list[str]:
    """A bordered box with `title`, `body` lines inside, padded to size."""
    head = f"┌─ {title} ".ljust(width - 1, "─") + "┐"
    rows = [head]
    for i in range(height - 2):
        inner = body[i] if i < len(body) else ""
        rows.append("│" + inner.ljust(width - 2)[: width - 2] + "│")
    rows.append("└" + "─" * (width - 2) + "┘")
    return rows


# ----------------------------------------------------------------- t2 counter


def counter_frame(n: int) -> str:
    rows = box("Counter", W, H, [f"Count: {n}"])
    return "\n".join(rows)


def counter_dump(values: list[int]) -> str:
    return FF.join(counter_frame(v) for v in values)


# -------------------------------------------------------------------- t3 list


def list_frame(sel: int) -> str:
    items = [
        ("> " if i == sel else "  ") + f"Item {i + 1:02}" for i in range(20)
    ]
    rows = box("Items", W, H - 1, items)
    rows.append(f"{sel + 1} / 20".ljust(W))
    return "\n".join(rows)


def list_dump(sels: list[int]) -> str:
    return FF.join(list_frame(s) for s in sels)


# ------------------------------------------------------------------ assertions

FAILURES: list[str] = []


def expect(label: str, result: dict, *, want_score: float | None = None,
           want_contract_failed: bool | None = None, want_failed_ids: set | None = None):
    failed_ids = {c["id"] for c in result["checks"] if not c["passed"]}
    if want_score is not None and abs(result["score"] - want_score) > 1e-9:
        FAILURES.append(
            f"{label}: score {result['score']:.3f}, expected {want_score:.3f} "
            f"(failed: {sorted(failed_ids) or 'none'})"
        )
    if want_contract_failed is not None and result["contract_failed"] != want_contract_failed:
        FAILURES.append(
            f"{label}: contract_failed={result['contract_failed']}, "
            f"expected {want_contract_failed}"
        )
    if want_failed_ids is not None and failed_ids != want_failed_ids:
        FAILURES.append(f"{label}: failed {sorted(failed_ids)}, expected {sorted(want_failed_ids)}")
    status = "ok " if not FAILURES or FAILURES[-1].split(":")[0] != label else "FAIL"
    print(f"  {status} {label}: score={result['score']:.3f} "
          f"contract_failed={result['contract_failed']} "
          f"failed={sorted(failed_ids) or '[]'}")


def main() -> int:
    t2, t3 = TASKS / "t2-counter", TASKS / "t3-list"

    print("t2-counter")
    # A correct implementation: 0 -> +1 -> +2 -> -1, and `q` quits (4 frames).
    expect("correct", score(t2, counter_dump([0, 1, 2, 1])), want_score=1.0,
           want_contract_failed=False, want_failed_ids=set())
    # `q` ignored: a fifth frame is dumped.
    expect("does not quit", score(t2, counter_dump([0, 1, 2, 1, 1])),
           want_failed_ids={"quits"})
    # `-` treated as another increment.
    expect("bad decrement", score(t2, counter_dump([0, 1, 2, 3])),
           want_failed_ids={"dec"})
    # Renders a fixed screen: keys do nothing.
    expect("static screen", score(t2, counter_dump([0, 0, 0, 0])),
           want_failed_ids={"inc1", "inc2", "dec"})
    # Wrong grid height — a contract violation, not a UI failure.
    short = FF.join("\n".join(box("Counter", W, 10, [f"Count: {n}"])) for n in [0, 1, 2, 1])
    expect("wrong grid size", score(t2, short), want_score=0.0, want_contract_failed=True)
    # Not a frame dump at all.
    expect("empty output", score(t2, ""), want_score=0.0, want_contract_failed=True)

    # A frame whose lower rows are entirely blank is still a full-height frame.
    # An earlier parser stripped every trailing blank row and reported such a
    # screen as too short, turning a correct program into a contract violation.
    sparse = FF.join(
        "\n".join(["Counter".ljust(W), f"Count: {n}".ljust(W)] + [" " * W] * (H - 2))
        for n in [0, 1, 2, 1]
    )
    expect("blank lower rows survive parsing", score(t2, sparse),
           want_score=1.0, want_contract_failed=False, want_failed_ids=set())

    print("t3-list")
    expect("correct", score(t3, list_dump([0, 1, 2, 3])), want_score=1.0,
           want_contract_failed=False, want_failed_ids=set())
    # Selection moves but the status bar never updates.
    stale = FF.join(
        "\n".join(box("Items", W, H - 1,
                      [("> " if i == s else "  ") + f"Item {i + 1:02}" for i in range(20)])
                  + ["1 / 20".ljust(W)])
        for s in [0, 1, 2, 3]
    )
    expect("stale status bar", score(t3, stale), want_failed_ids={"status-moved"})
    # Down moves two rows at a time.
    expect("double step", score(t3, list_dump([0, 2, 4, 6])),
           want_failed_ids={"moved-selection", "marker-travelled", "status-moved"})

    print()
    if FAILURES:
        print(f"{len(FAILURES)} verifier self-test failure(s):")
        for f in FAILURES:
            print(f"  - {f}")
        return 1
    print("verifier self-test passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
