#!/usr/bin/env python3
"""Turn Criterion's output into the comparison table in docs/BENCHMARKS.md.

    python report.py                      # read target/criterion, print a table
    python report.py --save pass1.json    # also record this pass
    python report.py --compare pass1.json # publish the lower ratio of two passes

Criterion writes one estimates.json per (group, framework). This reads the
median of each and reports Hawk TUI against the others, so the published table
is derived from the measurements rather than transcribed by hand.

Why --compare exists: this benchmark is noisy. Two back-to-back passes on an
idle machine have disagreed by up to 2x on absolute times in the same group.
The documented convention is to publish the *lower* speedup of two passes, and
doing that by hand invites picking the flattering one by accident.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent


def criterion_dir() -> Path:
    """Where Criterion actually wrote, which is not always ./target.

    A global `build.target-dir` in .cargo/config.toml, or CARGO_TARGET_DIR,
    redirects both the build and Criterion's output. Reading ./target when the
    run went elsewhere silently reports a *previous* run's numbers as though
    they were fresh — which is how a stale benchmark gets published.
    """
    import os
    import subprocess

    try:
        proc = subprocess.run(
            ["cargo", "metadata", "--no-deps", "--format-version", "1"],
            cwd=HERE, capture_output=True, text=True, encoding="utf-8", timeout=60,
        )
        if proc.returncode == 0:
            target = Path(json.loads(proc.stdout)["target_directory"])
            if (target / "criterion").is_dir():
                return target / "criterion"
    except Exception:  # noqa: BLE001 - fall through to the explicit candidates
        pass

    for candidate in (
        Path(os.environ["CARGO_TARGET_DIR"]) / "criterion"
        if os.environ.get("CARGO_TARGET_DIR") else None,
        HERE / "target" / "criterion",
    ):
        if candidate and candidate.is_dir():
            return candidate
    return HERE / "target" / "criterion"


CRITERION = criterion_dir()

# Group directory -> the label used in the published table.
LABELS = {
    "buffer_alloc_200x50": "Buffer allocation",
    "buffer_reset_200x50": "Buffer reset",
    "buffer_diff_1pct_200x50": "Diff, 1 % changed",
    "buffer_diff_5pct_200x50": "Diff, 5 % changed",
    "buffer_diff_50pct_200x50": "Diff, 50 % changed",
    "buffer_merge_overlay": "Overlay compositing",
    "set_string_full_screen": "`set_string`, full screen",
    "unicode_set_string_full_screen": "Unicode text, full screen",
    "styled_spans_full_screen": "Styled spans, full screen",
    "paragraph_wrap_200x50": "Paragraph word-wrap",
    "layout_solve_nested": "Nested layout solve",
    "render_dashboard_200x50": "Dashboard render (5 widgets)",
    "table_render_200_rows": "Table render, 200 rows",
    "list_scroll_1000_items": "List scroll, 1000 items",
    "terminal_emit_full_frame": "Escape-sequence emit",
    "terminal_emit_style_churn": "Emit, style churn (worst case)",
}

FRAMEWORKS = ("hawk", "ratatui", "superlighttui")


def median_ns(group: str, framework: str) -> float | None:
    """Criterion's median point estimate, in nanoseconds."""
    path = CRITERION / group / framework / "new" / "estimates.json"
    if not path.is_file():
        return None
    data = json.loads(path.read_text(encoding="utf-8"))
    try:
        return float(data["median"]["point_estimate"])
    except (KeyError, TypeError):
        return None


def human(ns: float) -> str:
    if ns < 1_000:
        return f"{ns:.0f} ns"
    if ns < 1_000_000:
        return f"{ns / 1_000:.1f} µs"
    return f"{ns / 1_000_000:.2f} ms"


def collect() -> dict[str, dict[str, float]]:
    out: dict[str, dict[str, float]] = {}
    for group in LABELS:
        row = {fw: v for fw in FRAMEWORKS if (v := median_ns(group, fw)) is not None}
        if row:
            out[group] = row
    return out


def main() -> int:
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")
        except AttributeError:
            pass

    ap = argparse.ArgumentParser()
    ap.add_argument("--save", type=Path, help="write this pass to a JSON file")
    ap.add_argument("--compare", type=Path, help="publish the lower ratio against this pass")
    args = ap.parse_args()

    current = collect()
    if not current:
        print(f"no Criterion results under {CRITERION}; run `cargo bench --bench shootout`")
        return 1

    missing = [g for g in LABELS if g not in current]
    if missing:
        print(f"{len(current)}/{len(LABELS)} groups measured; still missing: "
              f"{', '.join(missing)}\n")

    other = json.loads(args.compare.read_text(encoding="utf-8")) if args.compare else None

    print("| Workload (200×50 screen) | Hawk TUI | ratatui | SuperLightTUI | vs ratatui | vs SLT |")
    print("| ------------------------ | -------- | ------- | ------------- | ---------- | ------ |")

    losses = []
    for group, label in LABELS.items():
        row = current.get(group)
        if not row or "hawk" not in row:
            continue
        hawk = row["hawk"]

        def ratio(fw: str) -> str:
            if fw not in row:
                return "—"
            r = row[fw] / hawk
            if other and group in other and fw in other[group] and "hawk" in other[group]:
                r = min(r, other[group][fw] / other[group]["hawk"])
            if r < 1.0:
                losses.append(f"{label} vs {fw}: {r:.2f}×")
            return f"**{r:.1f}×**" if fw == "ratatui" else f"{r:.1f}×"

        cells = [
            f"| {label}",
            human(hawk),
            human(row["ratatui"]) if "ratatui" in row else "—",
            human(row["superlighttui"]) if "superlighttui" in row else "—",
            ratio("ratatui"),
            ratio("superlighttui"),
        ]
        print(" | ".join(cells) + " |")

    if other:
        print("\nRatios are the lower of two passes, per the documented convention.")

    # The README states a range across the ratatui column. Derive it here so the
    # sentence cannot drift from the table underneath it.
    ratios = []
    for group, row in current.items():
        if "hawk" not in row or "ratatui" not in row:
            continue
        r = row["ratatui"] / row["hawk"]
        if other and group in other and {"hawk", "ratatui"} <= set(other[group]):
            r = min(r, other[group]["ratatui"] / other[group]["hawk"])
        ratios.append(r)
    if ratios:
        print(f"\nvs ratatui across {len(ratios)} groups: "
              f"{min(ratios):.1f}× to {max(ratios):.1f}× (median {sorted(ratios)[len(ratios)//2]:.1f}×)")

    if losses:
        print(f"\n{len(losses)} workload(s) where Hawk TUI is NOT faster:")
        for item in losses:
            print(f"  {item}")
        print("The README claim of being fastest in all sixteen groups no longer holds.")
    else:
        print("\nHawk TUI is fastest in every measured group.")

    if args.save:
        args.save.write_text(json.dumps(current, indent=2, sort_keys=True) + "\n",
                             encoding="utf-8")
        print(f"\npass recorded: {args.save}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
