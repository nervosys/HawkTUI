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
import os
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



def packaged_files() -> list[str]:
    """Non-source files `cargo package` would ship, as repo-relative paths.

    Asking cargo keeps C1 honest by construction: change `exclude` or add a
    doc, and the pack follows without anyone remembering to edit this file.
    """
    proc = subprocess.run(
        ["cargo", "package", "--list", "--allow-dirty"],
        cwd=REPO, capture_output=True, text=True, encoding="utf-8", errors="replace",
    )
    if proc.returncode != 0:
        sys.exit("cargo package --list failed: " + proc.stderr)
    skip_prefix = ("src/", "tests/", "benches/", "examples/", ".cargo_vcs_info",
                   "Cargo.lock", "Cargo.toml.orig", ".gitignore", ".well-known/")
    # Two exclusions beyond "not documentation".
    #
    # The benchmark's own write-ups are in the published crate, so a real user
    # does get them, but handing an agent the document that names the tasks,
    # the failure modes and how the checks score would measure reading
    # comprehension of this benchmark rather than authoring ability.
    #
    # Machine config is not documentation: Cargo.toml, deny.toml and the
    # licence teach an author nothing and the agent writes its own manifest.
    contaminating = {
        "docs/HANDOFF.md",
        "docs/AGENTIC-BENCHMARKS.md",
        "docs/BENCHMARKS.md",
    }
    not_docs = {"Cargo.toml", "deny.toml", "LICENSE", "CHANGELOG.md",
                "CONTRIBUTING.md", "SECURITY.md", "ROADMAP.md"}
    out = []
    for line in proc.stdout.splitlines():
        rel = line.strip().replace("\\", "/")
        if not rel or rel.startswith(skip_prefix):
            continue
        if rel in contaminating or rel in not_docs:
            continue
        if (REPO / rel).is_file():
            out.append(rel)
    return out

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

    # Rebuild the packs, but keep the directory's own .gitignore. It is what
    # stops ratatui's and superlighttui's licensed docs from being vendored into
    # this repository, and rmtree'ing it meant every regeneration silently
    # re-armed that trap for whoever committed next.
    keep = (CONTEXT / ".gitignore").read_bytes() if (CONTEXT / ".gitignore").is_file() else None
    if CONTEXT.exists():
        shutil.rmtree(CONTEXT)
    if keep is not None:
        CONTEXT.mkdir(parents=True, exist_ok=True)
        (CONTEXT / ".gitignore").write_bytes(keep)

    manifest = str((REPO / "Cargo.toml")).replace("\\", "/")
    inventory: dict[str, dict[str, int]] = {}

    print("hawktui")
    # C1 for DeweyGUI — the same rule as every other framework: what a user
    # gets from the package, which is its README. Its ontology and MCP docs are
    # deliberately left out, since those are what a C2+ condition would add and
    # handing them over at C1 would make the conditions differ by less than
    # they claim to.
    # Not a hardcoded path: that one named a single developer's home directory,
    # so the DeweyGUI pack silently came out empty for everyone else and the
    # username rode along into a public repository.
    dewey = (Path(os.environ["DEWEYGUI_DIR"]).resolve()
             if os.environ.get("DEWEYGUI_DIR")
             else (REPO.parent / "DeweyGUI"))
    if (dewey / "README.md").is_file():
        copy(dewey / "README.md", CONTEXT / "deweygui" / "c1" / "README.md")

    # C1 — exactly what `cargo add hawktui` delivers, taken from cargo rather
    # than guessed. The guess was wrong: this said "`docs/` is in the package
    # `exclude` list, so none of it reaches a user" and shipped the README
    # alone, while the published crate carries llms.txt, AGENTS.md and six
    # docs/*.md — 21 non-source files. A user got twenty of them and the
    # benchmark's agent got none, so C1 understated Hawk TUI against
    # competitors whose packs are built from what *they* ship, and left the
    # agent with the source as the only place to look.
    for rel in packaged_files():
        copy(REPO / rel, CONTEXT / "hawktui" / "c1" / rel)
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

    # Directories only: the pack root also holds .gitignore and inventory.json.
    for fw_dir in sorted(d for d in CONTEXT.iterdir() if d.is_dir()):
        inventory[fw_dir.name] = {
            cond.name: sum(f.stat().st_size for f in cond.rglob("*") if f.is_file())
            for cond in sorted(c for c in fw_dir.iterdir() if c.is_dir())
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
