#!/usr/bin/env python3
"""Build the context packs for each framework and condition.

    python make_context.py

Fairness rule
-------------
C1 is "what a real user of the published crate gets today" — the documentation
actually shipped inside the crate tarball, verbatim, not a curated subset. That
rule currently *disadvantages* Hawk TUI, which excludes `docs/` from its
package and therefore ships only a README, while superlighttui ships 520 KB of
agent-targeted documentation including an `llms.txt`. We use the rule anyway,
because a benchmark that quietly trims a competitor's strongest material is
worthless.

C2 and C3 add the Hawk TUI ontology on top of C1. Both are generated
mechanically from the registered widget schemas by `examples/ontology_query.rs`
— no hand-written prose — so that the contrast measures the ontology rather
than a better-written cheatsheet.
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent
REPO = ROOT.parent.parent
CONTEXT = ROOT / "context"

REGISTRY_GLOB = "registry/src/*/"
CRATE_DIRS = {
    "ratatui": "ratatui-0.29.0",
    "superlighttui": "superlighttui-0.23.0",
}


def find_crate(name: str) -> Path | None:
    cargo_home = Path.home() / ".cargo"
    for candidate in cargo_home.glob(f"{REGISTRY_GLOB}{CRATE_DIRS[name]}"):
        if (candidate / "README.md").is_file():
            return candidate
    return None


def ontology(*args: str) -> str:
    proc = subprocess.run(
        ["cargo", "run", "--quiet", "--example", "ontology_query", "--", *args],
        cwd=REPO,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.returncode != 0:
        sys.exit(f"ontology_query {' '.join(args)} failed:\n{proc.stderr}")
    return proc.stdout


C3_INSTRUCTIONS = """# Widget ontology (queryable)

This project ships a machine-readable ontology describing every Hawk TUI
widget: its properties with types and constraints, its semantic role, the
actions it supports, and a usage hint. Query it instead of guessing at the API.

```sh
cargo run --quiet --manifest-path {manifest} --example ontology_query -- list
cargo run --quiet --manifest-path {manifest} --example ontology_query -- search scroll
cargo run --quiet --manifest-path {manifest} --example ontology_query -- schema Gauge
cargo run --quiet --manifest-path {manifest} --example ontology_query -- roles
```

- `list` — every widget type with its semantic role and description
- `search QUERY` — widget types matching a name, description or tag
- `schema NAME` — full schema for one widget type, including constraints
- `roles` — widget types grouped by semantic role, for finding the right widget
  when you know what it should *do* but not what it is called
"""


def write(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")
    print(f"  {path.relative_to(ROOT)}  ({len(content.encode('utf-8')):,} bytes)")


def copy(src: Path, dst: Path) -> None:
    dst.parent.mkdir(parents=True, exist_ok=True)
    if src.is_dir():
        shutil.copytree(src, dst, dirs_exist_ok=True)
        size = sum(f.stat().st_size for f in dst.rglob("*") if f.is_file())
        print(f"  {dst.relative_to(ROOT)}/  ({size:,} bytes)")
    else:
        shutil.copy2(src, dst)
        print(f"  {dst.relative_to(ROOT)}  ({dst.stat().st_size:,} bytes)")


def main() -> int:
    global REPO
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", type=Path, default=None,
                    help="Hawk TUI tree to generate the ontology from "
                         "(default: this checkout). Point it at a frozen "
                         "snapshot so the packs match the tree under test.")
    args = ap.parse_args()
    if args.repo:
        REPO = args.repo.resolve()

    if CONTEXT.exists():
        shutil.rmtree(CONTEXT)

    manifest = str((REPO / "Cargo.toml")).replace("\\", "/")
    inventory: dict[str, dict[str, int]] = {}

    print("hawktui")
    # C1 — exactly what `cargo add hawktui` delivers: the README. `docs/` is in
    # the package `exclude` list, so none of it reaches a user.
    copy(REPO / "README.md", CONTEXT / "hawktui" / "c1" / "README.md")
    # C2 — the ontology as a static pack.
    write(CONTEXT / "hawktui" / "c2" / "ONTOLOGY.json", ontology("export"))
    write(CONTEXT / "hawktui" / "c2" / "ONTOLOGY.md", ontology("digest"))
    # C3 — the ontology as a tool the agent queries on demand.
    write(
        CONTEXT / "hawktui" / "c3" / "ONTOLOGY-TOOL.md",
        C3_INSTRUCTIONS.format(manifest=manifest),
    )

    for name in CRATE_DIRS:
        print(name)
        crate = find_crate(name)
        if crate is None:
            print(f"  ! {name} source not in the cargo registry; run "
                  f"`cargo fetch` in benchmarks/ first")
            continue
        copy(crate / "README.md", CONTEXT / name / "c1" / "README.md")
        # Ship whatever documentation the crate itself ships.
        for extra in ("docs", "examples"):
            if (crate / extra).is_dir():
                copy(crate / extra, CONTEXT / name / "c1" / extra)

    for fw_dir in sorted(CONTEXT.iterdir()):
        inventory[fw_dir.name] = {
            cond.name: sum(f.stat().st_size for f in cond.rglob("*") if f.is_file())
            for cond in sorted(fw_dir.iterdir())
        }
    write(CONTEXT / "inventory.json", json.dumps(inventory, indent=2) + "\n")

    print("\ncontext pack sizes (bytes, cumulative per condition):")
    for fw, conds in inventory.items():
        c1 = conds.get("c1", 0)
        for cond, size in conds.items():
            total = size if cond == "c1" else c1 + size
            print(f"  {fw:<16} {cond}  {total:>9,}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
