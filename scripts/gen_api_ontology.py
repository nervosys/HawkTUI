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
CORE_MODULES = {
    "layout::Layout": SRC / "layout" / "mod.rs",
    "terminal::Terminal": SRC / "terminal.rs",
    "core::buffer::Buffer": SRC / "core" / "buffer.rs",
    "core::rect::Rect": SRC / "core" / "rect.rs",
    "core::text::Text": SRC / "core" / "text.rs",
    "core::style::Style": SRC / "core" / "style.rs",
    "runtime::Program": SRC / "runtime" / "mod.rs",
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
        module = "hawktui::" + label.rsplit("::", 1)[0]
        for name, info in parse_file(path).items():
            if info["fns"] or info["variants"]:
                out.append((name, module, info))
    return out


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
    return 0


if __name__ == "__main__":
    sys.exit(main())
