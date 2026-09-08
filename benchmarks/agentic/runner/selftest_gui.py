#!/usr/bin/env python3
"""Self-test for the GUI task set, g1-counter and g2-todo.

These are the first tasks in this harness whose frames are widget trees rather
than character grids. The verifier cannot tell a correct GUI program from a
broken one unless these discriminate, and a task set for a framework nobody has
run yet is exactly where a silent mistake would sit unnoticed.

Frames here are written in DeweyGUI's snapshot shape — `<type> #<id> [x,y wxh]
<key>=<value>`, sorted properties, whole-pixel bounds — because that is what the
contract those tasks carry actually specifies.

    python selftest_gui.py
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import selftest  # noqa: E402
from selftest import expect  # noqa: E402
from verify import score  # noqa: E402

TASKS = Path(__file__).resolve().parents[1] / "tasks"
FF = "\x0c"


# ------------------------------------------------------------------ g1-counter


def counter(n: int, *, drop: str = "", static: bool = False) -> str:
    rows = [
        "Column #root [0,0 240x120]",
        "  Label #title [8,8 224x24] text=Counter",
        f"  Label #count [8,40 224x24] text=Count: {0 if static else n}",
        "  Button #inc [8,72 48x24] label=+ enabled=true",
    ]
    return "\n".join(r for r in rows if f"#{drop} " not in r)


def counter_dump(**kw) -> str:
    return (FF + "\n").join(counter(i, **kw) for i in range(3))


# --------------------------------------------------------------------- g2-todo

ITEMS = ["Write tests", "Fix wrapping", "Ship it"]


def todo(done: set[int], items: list[str] | None = None, *,
         reindex: bool = True) -> str:
    items = ITEMS if items is None else items
    rows = [
        "Column #root [0,0 320x240]",
        "  Label #title [8,8 304x24] text=Todo",
        f"  Label #count [8,40 304x24] text={len(done)} done of {len(items)}",
    ]
    for i, text in enumerate(items):
        # `reindex=False` keeps an item's original index after a removal, which
        # is the mistake the survivor-is-reindexed check exists to catch.
        idx = i if reindex else ITEMS.index(text)
        y = 72 + i * 32
        rows.append(f"  Row #row-{idx} [8,{y} 304x32]")
        rows.append(f"    Checkbox #check-{idx} [8,{y} 24x24] "
                    f"checked={'true' if i in done else 'false'}")
        rows.append(f"    Label #item-{idx} [40,{y} 264x24] text={text}")
    rows.append("  Button #clear [8,200 96x24] label=Clear done enabled=true")
    return "\n".join(rows)


def todo_dump(*, clear_works: bool = True, reindex: bool = True) -> str:
    after = (todo(set(), ["Fix wrapping"], reindex=reindex) if clear_works
             else todo({0, 2}))
    return (FF + "\n").join([
        todo(set()),
        todo({0}),
        todo({0, 2}),
        after,
    ])


def main() -> int:
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")
        except AttributeError:
            pass

    g1, g2 = TASKS / "g1-counter", TASKS / "g2-todo"

    print("g1-counter")
    expect("correct", score(g1, counter_dump()), want_score=1.0,
           want_contract_failed=False, want_failed_ids=set())
    expect("clicks do nothing", score(g1, counter_dump(static=True)),
           want_failed_ids={"first-click", "second-click",
                            "click-changes-the-tree"})
    # A label reading "+" is not a button anyone can click by id.
    expect("no addressable button", score(g1, counter_dump(drop="inc")),
           want_failed_ids={"inc-addressable"})
    # Three frames prove the quit worked; two mean `q` did not.
    two = (FF + "\n").join(counter(i) for i in range(2))
    expect("did not quit", score(g1, two), want_contract_failed=True)

    print("\ng2-todo")
    expect("correct", score(g2, todo_dump()), want_score=1.0,
           want_contract_failed=False, want_failed_ids=set())
    # `survivor-is-the-middle-one` still passes here, because a list that lost
    # nothing still contains the middle item. It is a check about what survived,
    # not about how many did, and the count checks are what catch this.
    expect("clear does nothing", score(g2, todo_dump(clear_works=False)),
           want_failed_ids={"clear-removes-done", "one-item-left",
                            "done-items-are-gone"})
    # The survivor keeps its old index, so `item-0` no longer addresses the
    # first item on the list — the bug the prompt spends a paragraph on.
    expect("survivor keeps its old index",
           score(g2, todo_dump(reindex=False)),
           want_failed_ids={"survivor-is-reindexed"})

    print()
    if selftest.FAILURES:
        print(f"{len(selftest.FAILURES)} GUI self-test failure(s):")
        for f in selftest.FAILURES:
            print(f"  - {f}")
        return 1
    print("GUI self-test passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
