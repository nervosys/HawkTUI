#!/usr/bin/env python3
"""Generate the authoring ontology from the source it describes.

    python scripts/gen_api_ontology.py            # write src/ontology/api_generated.rs
    python scripts/gen_api_ontology.py --check    # fail if the file is stale

The widget ontology answers "what does this widget hold at runtime". That is
the right question for an agent *driving* an application and the wrong one for
an agent *writing* one, which needs constructors, builder methods, argument
types, and whether a widget renders as `Widget` or `StatefulWidget` and with
which state type.

Those facts already exist, in the signatures. Transcribing them by hand would
be 300-odd entries that drift out of date on the first refactor, so they are
parsed out instead and emitted as static data. `tests/api_ontology_tests.rs`
runs this with --check, so a signature change that is not regenerated fails CI.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "src"
OUT = SRC / "ontology" / "api_generated.rs"

# Modules whose public API an author needs. Widgets are discovered from the
# directory; these are the core types a program is built out of.
# Ordered by how often the agent reads each file when it has the source.
# Widgets are discovered from their directory; these are the skeleton a program
# is assembled from, which is what the transcripts show it going to source for.
CORE_MODULES = {
    "backend::test": SRC / "backend" / "test.rs",
    "event": SRC / "event" / "mod.rs",
    "core::buffer": SRC / "core" / "buffer.rs",
    "terminal": SRC / "terminal.rs",
    "core::cell": SRC / "core" / "cell.rs",
    "runtime": SRC / "runtime" / "mod.rs",
    "layout": SRC / "layout" / "mod.rs",
    "core::rect": SRC / "core" / "rect.rs",
    "core::text": SRC / "core" / "text.rs",
    "core::style": SRC / "core" / "style.rs",
    "core::symbol": SRC / "core" / "symbol.rs",
    "backend": SRC / "backend" / "mod.rs",
    "testing": SRC / "testing.rs",
    "agent::driver": SRC / "agent" / "driver.rs",
    "agent::session": SRC / "agent" / "session.rs",
    "focus": SRC / "focus.rs",
    "overlay": SRC / "overlay.rs",
}

# Trait methods and derives that are noise for an author.
SKIP_FNS = {
    "fmt", "clone", "default", "eq", "ne", "hash", "cmp", "partial_cmp",
    "schema", "capabilities", "actions", "semantic_role", "agent_state",
    "execute_action", "agent_id", "accessibility_label", "draw",
}

DOC_RX = re.compile(r"^\s*///\s?(.*)$")
IMPL_RX = re.compile(r"^\s*impl(?:<[^>]*>)?\s+([A-Za-z_][\w:]*)(?:<[^>]*>)?\s*\{")
TRAIT_IMPL_RX = re.compile(
    r"^\s*impl(?:<[^>]*>)?\s+([A-Za-z_][\w:]*)(?:<[^>]*>)?\s+for\s+([A-Za-z_]\w*)")
FN_RX = re.compile(r"^\s*pub fn\s+(\w+)\s*(?:<[^>]*>)?\s*\(")
STRUCT_RX = re.compile(r"^\s*pub struct\s+(\w+)")
ENUM_RX = re.compile(r"^\s*pub enum\s+(\w+)")
TRAIT_RX = re.compile(r"^\s*pub trait\s+(\w+)")
# Inside a trait body, required methods are declared without a body.
TRAIT_FN_RX = re.compile(r"^\s*fn\s+(\w+)\s*(?:<[^>]*>)?\s*\(")


def rust_str(text: str) -> str:
    return '"' + text.replace("\\", "\\\\").replace('"', '\\"') + '"'


def signature(lines: list[str], start: int) -> tuple[str, int]:
    """Collect a signature from `pub fn` to the opening brace or semicolon."""
    depth = 0
    parts = []
    i = start
    while i < len(lines):
        line = lines[i]
        for ch in line:
            if ch in "({[":
                depth += 1
            elif ch in ")}]":
                depth -= 1
            if ch == "{" and depth <= 1:
                parts.append(line[: line.index("{")])
                return " ".join(" ".join(parts).split()), i
        parts.append(line)
        if depth == 0 and line.rstrip().endswith(";"):
            return " ".join(" ".join(parts).split()).rstrip(";"), i
        i += 1
    return " ".join(" ".join(parts).split()), i


def classify(sig: str) -> str:
    """constructor | builder | method, from the receiver."""
    args = sig[sig.index("(") + 1:]
    head = args.split(")")[0].strip()
    if head.startswith("mut self") or head.startswith("self"):
        return "builder"
    if head.startswith("&"):
        return "method"
    return "constructor"


def parse_file(path: Path) -> dict:
    """Types defined in one file, with their public functions and doc lines."""
    lines = path.read_text(encoding="utf-8").splitlines()
    types: dict[str, dict] = {}
    docs: list[str] = []
    current: str | None = None
    stateful: dict[str, str] = {}
    kinds: dict[str, str] = {}
    variants: dict[str, list[str]] = {}
    summaries: dict[str, str] = {}

    i = 0
    while i < len(lines):
        line = lines[i]

        m = DOC_RX.match(line)
        if m:
            docs.append(m.group(1).strip())
            i += 1
            continue

        m = TRAIT_IMPL_RX.match(line)
        if m:
            trait_name, target = m.group(1), m.group(2)
            if trait_name in ("Widget", "StatefulWidget"):
                # Several widgets implement both, with `Widget` delegating to
                # `StatefulWidget` with a default state. The stateful form is
                # the one an author must know about, so it wins regardless of
                # which impl appears later in the file.
                if not (trait_name == "Widget" and kinds.get(target) == "StatefulWidget"):
                    kinds[target] = trait_name
                if trait_name == "StatefulWidget":
                    for j in range(i, min(i + 8, len(lines))):
                        s = re.search(r"type State\s*=\s*([\w:]+)", lines[j])
                        if s:
                            stateful[target] = s.group(1)
                            break
            current = None
            docs = []
            i += 1
            continue

        m = IMPL_RX.match(line)
        if m:
            current = m.group(1)
            types.setdefault(current, {"fns": []})
            docs = []
            i += 1
            continue

        m = TRAIT_RX.match(line)
        if m:
            name = m.group(1)
            types.setdefault(name, {"fns": []})
            kinds[name] = "trait"
            if docs:
                summaries[name] = docs[0]
            # Collect the trait's own method declarations, which is what an
            # implementor has to write.
            depth, j = 0, i
            while j < len(lines):
                depth += lines[j].count("{") - lines[j].count("}")
                fm = TRAIT_FN_RX.match(lines[j])
                if fm and j > i:
                    sig, end = signature(lines, j)
                    required = not sig.rstrip().endswith("}") and "{" not in lines[j]
                    types[name]["fns"].append({
                        "name": fm.group(1), "sig": sig,
                        "kind": "required" if required else "provided",
                        "doc": "",
                    })
                if depth == 0 and j > i:
                    break
                j += 1
            docs = []
            current = None
            i = j + 1
            continue

        for rx, kind in ((STRUCT_RX, "struct"), (ENUM_RX, "enum")):
            m = rx.match(line)
            if m:
                name = m.group(1)
                types.setdefault(name, {"fns": []})
                kinds.setdefault(name, kind)
                if docs:
                    summaries[name] = docs[0]
                if kind == "enum":
                    vs, depth = [], 0
                    for j in range(i, len(lines)):
                        depth += lines[j].count("{") - lines[j].count("}")
                        v = re.match(r"\s*([A-Z]\w*)\s*(\{|\(|,|$)", lines[j])
                        if v and j > i:
                            vs.append(v.group(1))
                        if depth == 0 and j > i:
                            break
                    variants[name] = vs
                docs = []
                break
        else:
            m = FN_RX.match(line)
            if m and current:
                name = m.group(1)
                sig, end = signature(lines, i)
                if name not in SKIP_FNS:
                    types[current]["fns"].append(
                        {"name": name, "sig": sig, "kind": classify(sig),
                         "doc": docs[0] if docs else ""})
                docs = []
                i = end + 1
                continue
            stripped = line.strip()
            # Attributes sit between a doc comment and the item it documents,
            # so they must not clear the pending docs — that dropped the
            # summary from 67 of 72 types.
            if stripped and not stripped.startswith(("//", "#[", "#!")):
                docs = []
        i += 1

    for name, info in types.items():
        info["kind"] = kinds.get(name, "struct")
        info["state"] = stateful.get(name)
        info["variants"] = variants.get(name, [])
        info["summary"] = summaries.get(name, "")
    return types


def collect() -> list[tuple[str, str, dict]]:
    """(type name, module path, info) for everything an author touches."""
    out = []
    for f in sorted((SRC / "widget").glob("*.rs")):
        if f.stem in ("mod", "highlight", "sixel"):
            continue
        module = f"hawktui::widget::{f.stem}"
        for name, info in parse_file(f).items():
            if info["fns"] or info["variants"]:
                out.append((name, module, info))
    for label, path in CORE_MODULES.items():
        if not path.exists():
            continue
        # The key is the module path itself. Deriving it by stripping the last
        # segment once emitted `hawktui::backend` for TestBackend, an import
        # that does not resolve — exactly the kind of wrong answer that sends an
        # agent to the source.
        module = "hawktui::" + label
        for name, info in parse_file(path).items():
            if info["fns"] or info["variants"]:
                out.append((name, module, info))
    return out


def prelude_items() -> list[str]:
    """What `use hawktui::prelude::*` brings into scope.

    lib.rs is among the files the agent reads most, and this is the only reason
    it needs to.
    """
    text = (SRC / "lib.rs").read_text(encoding="utf-8")
    body = text.split("pub mod prelude", 1)[-1]
    items: list[str] = []
    for line in body.splitlines():
        line = line.strip()
        if line.startswith("}"):
            break
        m = re.match(r"pub use crate::([\w:]+)::\{?([^};]*)\}?;", line)
        if not m:
            continue
        for name in m.group(2).split(","):
            name = name.strip()
            if name:
                items.append(name)
    return sorted(set(items))


def emit(entries) -> str:
    lines = [
        "//! The authoring API, generated from the source by",
        "//! `scripts/gen_api_ontology.py`. Do not edit by hand.",
        "//!",
        "//! Regenerate with `python scripts/gen_api_ontology.py`;",
        "//! `tests/api_ontology_tests.rs` fails when this is stale.",
        "",
        "use super::api::{ApiFn, ApiKind, ApiType};",
        "",
        f"/// Every public type an author builds a program out of ({len(entries)} of them).",
        "pub static API: &[ApiType] = &[",
    ]
    for name, module, info in sorted(entries, key=lambda e: e[0]):
        kind = {
            "trait": "ApiKind::Trait",
            "Widget": "ApiKind::Widget",
            "StatefulWidget": f"ApiKind::StatefulWidget {{ state: {rust_str(info['state'] or '')} }}",
            "enum": "ApiKind::Enum",
        }.get(info["kind"], "ApiKind::Struct")
        lines += [
            "    ApiType {",
            f"        name: {rust_str(name)},",
            f"        module: {rust_str(module)},",
            f"        kind: {kind},",
            f"        summary: {rust_str(info['summary'])},",
            f"        variants: &[{', '.join(rust_str(v) for v in info['variants'])}],",
            "        functions: &[",
        ]
        for fn in info["fns"]:
            lines.append(
                f"            ApiFn {{ name: {rust_str(fn['name'])}, "
                f"signature: {rust_str(fn['sig'])}, "
                f"role: {rust_str(fn['kind'])}, "
                f"summary: {rust_str(fn['doc'])} }},")
        lines += ["        ],", "    },"]
    lines += ["];", ""]

    items = prelude_items()
    lines += [
        f"/// Everything `use hawktui::prelude::*` brings into scope ({len(items)} items).",
        "pub static PRELUDE: &[&str] = &[",
    ]
    lines += [f"    {rust_str(i)}," for i in items]
    lines += ["];", ""]
    return "\n".join(lines)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    args = ap.parse_args()

    entries = collect()
    text = emit(entries)
    total_fns = sum(len(i["fns"]) for _, _, i in entries)

    if args.check:
        if not OUT.exists() or OUT.read_text(encoding="utf-8") != text:
            print(f"{OUT.relative_to(ROOT)} is stale; run "
                  f"`python scripts/gen_api_ontology.py`")
            return 1
        print(f"up to date: {len(entries)} types, {total_fns} functions")
        return 0

    OUT.write_text(text, encoding="utf-8", newline="\n")
    print(f"wrote {OUT.relative_to(ROOT)}: {len(entries)} types, {total_fns} functions")

    # Every module path the catalog publishes, as a `use` statement, compiled
    # as an example. A path that does not resolve is a wrong answer served to
    # an agent; this makes it a build failure.
    imports = ROOT / "examples" / "api_imports.rs"
    lines = [
        "//! Generated by `scripts/gen_api_ontology.py`. Do not edit.",
        "//!",
        "//! Imports every type the authoring ontology describes, at the path it",
        "//! publishes. If this stops compiling, the ontology is telling agents to",
        "//! write an import that does not resolve.",
        "",
        "#![allow(unused_imports)]",
        "",
    ]
    seen = set()
    for name, module, _ in sorted(entries, key=lambda e: (e[1], e[0])):
        if (module, name) in seen:
            continue
        seen.add((module, name))
        lines.append(f"use {module}::{name};")
    lines += ["", "fn main() {}", ""]
    imports.write_text("\n".join(lines), encoding="utf-8", newline="\n")
    print(f"wrote {imports.relative_to(ROOT)}: {len(seen)} import paths")
    return 0


if __name__ == "__main__":
    sys.exit(main())
