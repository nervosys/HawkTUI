#!/usr/bin/env python3
"""Framework-neutral verifier for the agentic TUI benchmark.

Scores a scaffolded program purely from the character grids it renders. It
never opens a `.rs` file, so it cannot tell which framework produced a frame
and cannot favour one.

    verify.py --task tasks/t3-list --program-dir runs/xyz
    verify.py --task tasks/t3-list --dump-file captured.txt   # score a capture

Prints a JSON result to stdout and exits 0 if every check passed, 1 otherwise.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import unicodedata
from pathlib import Path

FORM_FEED = "\x0c"


# ---------------------------------------------------------------- frame model


class Frame:
    """One rendered screen: exactly `h` rows padded to `w` columns."""

    def __init__(self, rows: list[str], w: int, h: int):
        self.declared_w = w
        self.declared_h = h
        self.rows = [r.ljust(w) for r in rows]

    @property
    def text(self) -> str:
        return "\n".join(self.rows)

    def row(self, i: int) -> str:
        return self.rows[i] if -len(self.rows) <= i < len(self.rows) else ""

    def region(self, x: int, y: int, w: int, h: int) -> list[str]:
        return [r[x : x + w] for r in self.rows[y : y + h]]

    def shape_ok(self) -> tuple[bool, str]:
        if len(self.rows) != self.declared_h:
            return False, f"expected {self.declared_h} rows, got {len(self.rows)}"
        over = [i for i, r in enumerate(self.rows) if len(r.rstrip()) > self.declared_w]
        if over:
            return False, f"rows wider than {self.declared_w}: {over[:5]}"
        return True, ""


def display_width(text: str) -> int:
    """Terminal columns occupied by `text`.

    The dump stores a double-width glyph as one character, so a character
    offset into a frame row is not a display column. Any check about alignment
    has to convert between them.
    """
    width = 0
    for ch in text:
        if unicodedata.combining(ch):
            continue
        width += 2 if unicodedata.east_asian_width(ch) in ("W", "F") else 1
    return width


def parse_frames(stdout: str, w: int, h: int) -> list[Frame]:
    """Split a dump into frames. Trailing-space trimming is tolerated.

    Only the newline that brackets a form feed is removed — never a blank row
    that is part of the frame. Stripping every blank row, as an earlier version
    did, silently truncated any screen whose lower rows were empty and reported
    it as a contract violation: a program that correctly emitted 30 rows with 16
    blank ones was scored as having emitted 14.
    """
    frames = []
    for chunk in stdout.split(FORM_FEED):
        # Drop one bracketing newline on each side, and nothing else.
        if chunk.startswith("\n"):
            chunk = chunk[1:]
        if chunk.endswith("\n"):
            chunk = chunk[:-1]
        if not chunk:
            continue
        rows = [r.rstrip("\r") for r in chunk.split("\n")]
        if rows:
            frames.append(Frame(rows, w, h))
    return frames


# -------------------------------------------------------------- check kinds


def _row_of(frame: Frame, pattern: str) -> int | None:
    rx = re.compile(pattern)
    for i, row in enumerate(frame.rows):
        if rx.search(row):
            return i
    return None


def _any_line(lines: list[str], pattern: str) -> bool:
    rx = re.compile(pattern)
    return any(rx.search(line) for line in lines)


def evaluate(check: dict, frames: list[Frame]) -> tuple[bool, str]:
    kind = check["kind"]

    if kind == "frame_count":
        want = check["equals"]
        return len(frames) == want, f"expected {want} frames, got {len(frames)}"

    def frame_at(idx: int) -> Frame | None:
        return frames[idx] if 0 <= idx < len(frames) else None

    if kind in ("frames_differ", "frames_equal", "row_of_delta"):
        a = frame_at(check.get("a", check.get("from", 0)))
        b = frame_at(check.get("b", check.get("to", 0)))
        if a is None or b is None:
            return False, "frame out of range"
        if kind == "frames_differ":
            return a.text != b.text, "frames are identical"
        if kind == "frames_equal":
            return a.text == b.text, "frames differ"
        ra, rb = _row_of(a, check["pattern"]), _row_of(b, check["pattern"])
        if ra is None or rb is None:
            return False, f"pattern {check['pattern']!r} not found in both frames"
        got = rb - ra
        return got == check["delta"], f"expected delta {check['delta']}, got {got}"

    frame = frame_at(check.get("frame", 0))
    if frame is None:
        return False, f"frame {check.get('frame', 0)} not rendered"

    if kind == "grid_shape":
        return frame.shape_ok()

    if kind == "contains":
        return _any_line(frame.rows, check["pattern"]), f"{check['pattern']!r} not found"

    if kind == "absent":
        return (
            not _any_line(frame.rows, check["pattern"]),
            f"{check['pattern']!r} should be absent",
        )

    if kind == "region_contains":
        region = frame.region(check["x"], check["y"], check["w"], check["h"])
        return (
            _any_line(region, check["pattern"]),
            f"{check['pattern']!r} not in region "
            f"({check['x']},{check['y']} {check['w']}x{check['h']})",
        )

    if kind == "row_matches":
        row = frame.row(check["row"])
        return bool(re.search(check["pattern"], row)), (
            f"row {check['row']} = {row.rstrip()!r} does not match {check['pattern']!r}"
        )

    if kind == "count_matching_lines":
        rx = re.compile(check["pattern"])
        n = sum(1 for r in frame.rows if rx.search(r))
        lo, hi = check.get("min", 0), check.get("max", 10**9)
        return lo <= n <= hi, f"{n} matching lines, wanted {lo}..{hi}"

    if kind == "display_gap":
        # Distance in display columns between the start of two strings on the
        # same row, for checking that columns line up when their contents are
        # different widths.
        rx_from, rx_to = re.compile(check["from"]), re.compile(check["to"])
        for row in frame.rows:
            a, b = rx_from.search(row), rx_to.search(row)
            if not (a and b):
                continue
            gap = display_width(row[a.start() : b.start()])
            want = check["equals"]
            return gap == want, (
                f"row {row.strip()!r}: {check['from']!r} to {check['to']!r} spans "
                f"{gap} display columns, expected {want}"
            )
        return False, f"no row contains both {check['from']!r} and {check['to']!r}"

    if kind == "row_order":
        a, b = _row_of(frame, check["before"]), _row_of(frame, check["after"])
        if a is None or b is None:
            return False, f"{check['before']!r} or {check['after']!r} not found"
        return a < b, f"{check['before']!r} at row {a} is not above {check['after']!r} at row {b}"

    return False, f"unknown check kind {kind!r}"


# -------------------------------------------------------------------- driver


def run_program(program_dir: Path, grid: dict, script: list[str], timeout: int):
    manifest = program_dir / "Cargo.toml"
    if not manifest.is_file():
        return None, "no Cargo.toml in the program directory"
    # Pin cargo to this crate; without it a missing manifest sends cargo up the
    # directory tree to build an unrelated package.
    cmd = [
        "cargo",
        "run",
        "--release",
        "--quiet",
        "--manifest-path",
        str(manifest),
        "--",
        "--headless",
        f"{grid['w']}x{grid['h']}",
        "--script",
        ",".join(script),
        "--dump",
    ]
    try:
        proc = subprocess.run(
            cmd,
            cwd=program_dir,
            capture_output=True,
            text=True,
            timeout=timeout,
            encoding="utf-8",
            errors="replace",
            stdin=subprocess.DEVNULL,
        )
    except subprocess.TimeoutExpired:
        return None, f"timed out after {timeout}s"
    if proc.returncode != 0:
        tail = (proc.stderr or "").strip().splitlines()[-20:]
        return None, "non-zero exit:\n" + "\n".join(tail)
    return proc.stdout, ""


def score(task_dir: Path, stdout: str) -> dict:
    spec = json.loads((task_dir / "checks.json").read_text(encoding="utf-8"))
    grid = spec["grid"]
    frames = parse_frames(stdout, grid["w"], grid["h"])

    results, earned, total = [], 0.0, 0.0
    contract_failed = not frames
    for check in spec["checks"]:
        passed, detail = evaluate(check, frames)
        weight = float(check.get("weight", 1))
        is_contract = bool(check.get("contract"))
        if is_contract:
            contract_failed = contract_failed or not passed
        else:
            total += weight
            earned += weight if passed else 0.0
        results.append(
            {
                "id": check["id"],
                "kind": check["kind"],
                "contract": is_contract,
                "passed": passed,
                "detail": "" if passed else detail,
            }
        )

    return {
        "task": spec["id"],
        "rung": spec["rung"],
        "frames_rendered": len(frames),
        "contract_failed": contract_failed,
        "score": 0.0 if contract_failed else (earned / total if total else 0.0),
        "checks_passed": sum(1 for r in results if r["passed"] and not r["contract"]),
        "checks_total": sum(1 for r in results if not r["contract"]),
        "checks": results,
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--task", required=True, type=Path)
    ap.add_argument("--program-dir", type=Path)
    ap.add_argument("--dump-file", type=Path)
    ap.add_argument("--timeout", type=int, default=120)
    args = ap.parse_args()

    spec = json.loads((args.task / "checks.json").read_text(encoding="utf-8"))

    if args.dump_file:
        stdout, err = args.dump_file.read_text(encoding="utf-8"), ""
    elif args.program_dir:
        stdout, err = run_program(args.program_dir, spec["grid"], spec["script"], args.timeout)
    else:
        ap.error("one of --program-dir or --dump-file is required")

    if stdout is None:
        result = {
            "task": spec["id"],
            "rung": spec["rung"],
            "frames_rendered": 0,
            "contract_failed": True,
            "score": 0.0,
            "checks_passed": 0,
            "checks_total": len([c for c in spec["checks"] if not c.get("contract")]),
            "run_error": err,
            "checks": [],
        }
    else:
        result = score(args.task, stdout)

    json.dump(result, sys.stdout, indent=2)
    sys.stdout.write("\n")
    return 0 if result["score"] == 1.0 and not result["contract_failed"] else 1


if __name__ == "__main__":
    sys.exit(main())
