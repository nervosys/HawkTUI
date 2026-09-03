#!/usr/bin/env python3
"""Runner for the agentic TUI benchmark.

Drives an AI coding agent through the task ladder once per
(task x framework x condition x replicate) cell, in a fresh working directory
each time, then scores the result with the framework-neutral verifier and
appends one JSON object per run to a JSONL file.

    python run.py --tasks t1-hello,t2-counter --frameworks hawktui,ratatui \
                  --conditions c1,c2 --replicates 3

Design notes
------------
* The agent runs with ``--bare``, which disables CLAUDE.md auto-discovery,
  hooks, memory and plugins. Those would otherwise be uncontrolled context that
  differs between runs, and CLAUDE.md discovery in particular would leak this
  repository's own documentation into the "no context" condition.
* Every condition, including C0, receives the identical task prompt. Conditions
  differ *only* in which files are seeded into the working directory.
* Nothing here reads the agent's source output. Scoring is entirely behavioural.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from verify import score as score_dump  # noqa: E402

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent                    # benchmarks/agentic
REPO = ROOT.parent.parent             # the Hawk TUI checkout

TASKS_DIR = ROOT / "tasks"
CONTEXT_DIR = ROOT / "context"
RESULTS_DIR = ROOT / "results"

# Compiler errors that mean "you named something that does not exist" — the
# machine-checkable signature of API hallucination.
API_ERROR_CODES = ("E0599", "E0433", "E0432", "E0425", "E0061", "E0560", "E0609")
API_ERROR_RX = re.compile(r"\b(" + "|".join(API_ERROR_CODES) + r")\b")
CARGO_RX = re.compile(r"\bcargo\s+(build|check|run|test|clippy)\b")
# cargo reports a failed build on stdout with an exit code the tool layer does
# not always surface, so match its actual diagnostic shape rather than a bare
# "error:" substring, which fires on any output that merely mentions the word.
BUILD_FAILED_RX = re.compile(
    r"^error(\[E\d{4}\])?:|could not compile|error: aborting", re.MULTILINE
)

FRAMEWORKS = {
    "hawktui": {
        "crate": "hawktui",
        # Derived by dep_line() from the depended-on tree's own Cargo.toml: the
        # package name differs between the frozen snapshot and the current
        # checkout, and a hardcoded copy here went stale exactly that way.
        "dep": None,
        "ontology": True,
        "registry_dir": None,          # read from the checkout instead
    },
    "ratatui": {
        "crate": "ratatui",
        "dep": '`ratatui = "0.29"`',
        "ontology": False,
        "registry_dir": "ratatui-0.29.0",
    },
    "superlighttui": {
        "crate": "superlighttui",
        "dep": '`superlighttui = "0.23"`',
        "ontology": False,
        "registry_dir": "superlighttui-0.23.0",
    },
}


# The Hawk TUI tree the scaffolded programs depend on. Overridable with
# --hawktui-path so a run can be pinned to a frozen snapshot while the working
# tree is edited; otherwise a mid-run edit silently changes what is measured.
HAWKTUI_PATH = REPO


def dep_line(framework: str) -> str:
    """The `[dependencies]` line the task prompt tells the agent to use.

    For Hawk TUI this is read from the tree being depended on rather than
    hardcoded. The package was renamed to `hawktui-rs` while the frozen
    benchmark snapshot still declares `hawktui`, and a stale template made every
    scaffolded crate fail to resolve — a harness fault that the transcript would
    have recorded against the agent.
    """
    entry = FRAMEWORKS[framework]
    if entry["registry_dir"] is not None:
        return entry["dep"]

    manifest = HAWKTUI_PATH / "Cargo.toml"
    package = "hawktui"
    if manifest.is_file():
        in_package = False
        for line in manifest.read_text(encoding="utf-8").splitlines():
            stripped = line.strip()
            if stripped.startswith("["):
                in_package = stripped == "[package]"
                continue
            if in_package and stripped.startswith("name"):
                package = stripped.split("=", 1)[1].strip().strip('"')
                break

    path = str(HAWKTUI_PATH).replace("\\", "/")
    if package == "hawktui":
        return f'`hawktui = {{ path = "{path}" }}`'
    # The import name stays `hawktui` whatever the package is called.
    return f'`hawktui = {{ package = "{package}", path = "{path}" }}`'


def source_dir(framework: str) -> Path | None:
    """The framework's own source tree, granted to the agent as a readable dir.

    Every framework gets one. Hawk TUI needs it for the path dependency, so
    withholding it from the others would hand Hawk TUI a source-reading
    advantage that has nothing to do with the ontology.
    """
    entry = FRAMEWORKS[framework]
    if entry["registry_dir"] is None:
        return HAWKTUI_PATH
    for candidate in (Path.home() / ".cargo").glob(f"registry/src/*/{entry['registry_dir']}"):
        if (candidate / "Cargo.toml").is_file():
            return candidate
    return None

CONDITIONS = ("c0", "c1", "c2", "c3", "c4", "c5")

# C4 and C5 both serve the ontology as MCP tools; they differ in what the
# ontology contains. C4 is the runtime widget schema, C5 adds the authoring
# API — constructors, builder signatures, and the Widget/StatefulWidget
# split. Both come from the working tree, since the frozen snapshot predates
# `hawktui-mcp`; that is disclosed in the results.
# C4 serves the ontology as MCP tools. Built from the working tree, since
# the frozen snapshot predates `hawktui-mcp`; its catalog differs from the
# snapshot's by two corrected usage hints, which is disclosed in the results.
MCP_BINARY = Path.home() / ".cargo-target" / "release" / "hawktui-mcp.exe"


# ------------------------------------------------------------------- prompting


def build_prompt(task: str, framework: str, seeded: list[str]) -> str:
    fw = FRAMEWORKS[framework]
    prompt = (TASKS_DIR / task / "prompt.md").read_text(encoding="utf-8")
    contract = (TASKS_DIR / "contract.md").read_text(encoding="utf-8")

    # Keep only the normative part of the contract. The surrounding rationale
    # says the program is being benchmarked and scored, which is framing a real
    # user would never supply and which could itself change how the agent works.
    body = contract.split("## Command line", 1)[1].split("## Why this exists")[0]
    body = ("\n## Command line" + body).rstrip().replace("\n## ", "\n### ").lstrip("\n")

    text = (
        prompt.replace("{{FRAMEWORK}}", fw["crate"])
        .replace("{{DEP}}", dep_line(framework))
        .replace("{{CONTRACT}}", "## Harness contract\n\n" + body)
    )

    # Conditions differ in what is on disk; say so, or the agent may never open
    # the files. C0 seeds nothing and gets no such section.
    if seeded:
        listing = "\n".join(f"- `{name}`" for name in sorted(seeded))
        text += (
            "\n## Reference material\n\n"
            "These files are in the working directory:\n\n" + listing + "\n"
        )
    return text


def seed_context(workdir: Path, framework: str, condition: str) -> list[str]:
    """Copy the condition's context files into the working directory."""
    seeded: list[str] = []
    if condition == "c0":
        return seeded
    for level in ("c1", "c2", "c3"):
        src = CONTEXT_DIR / framework / level
        if not src.is_dir():
            continue
        # c2 and c3 are supersets of c1.
        if level != "c1" and level != condition:
            continue
        for item in src.iterdir():
            dst = workdir / item.name
            if item.is_dir():
                shutil.copytree(item, dst, dirs_exist_ok=True)
            else:
                shutil.copy2(item, dst)
            seeded.append(item.name)
    return seeded


# ---------------------------------------------------------------- agent driver


def run_agent(prompt: str, workdir: Path, model: str, timeout: int, src: Path | None,
              mcp_config: Path | None = None) -> dict:
    """Invoke the agent headlessly and return its parsed transcript."""
    # `--bare` is the clean-room mode: no CLAUDE.md discovery, hooks, memory or
    # plugins. It authenticates only via ANTHROPIC_API_KEY, never OAuth, so it
    # is unavailable on an OAuth-only machine. When it is unavailable we fall
    # back and record the fact, because the fallback admits confounds (an
    # enabled LSP plugin, user settings) that a reader must be able to see.
    bare = bool(os.environ.get("ANTHROPIC_API_KEY"))
    cmd = [
        "claude",
        *(["--bare"] if bare else []),
        "-p",
        prompt,
        "--output-format",
        "stream-json",
        "--verbose",
        "--model",
        model,
        "--permission-mode",
        "bypassPermissions",
        "--allow-dangerously-skip-permissions",
    ]
    if src is not None:
        cmd += ["--add-dir", str(src)]
    if mcp_config is not None:
        # --strict-mcp-config so no server from the developer's own settings can
        # leak in and make the condition mean something different per machine.
        cmd += ["--mcp-config", str(mcp_config), "--strict-mcp-config"]
    env = dict(os.environ, CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC="1")
    started = time.monotonic()
    try:
        proc = subprocess.run(
            cmd,
            cwd=workdir,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=timeout,
            stdin=subprocess.DEVNULL,
            env=env,
        )
        out, err, rc = proc.stdout, proc.stderr, proc.returncode
        timed_out = False
    except subprocess.TimeoutExpired as e:
        out = e.stdout.decode("utf-8", "replace") if isinstance(e.stdout, bytes) else (e.stdout or "")
        err = e.stderr.decode("utf-8", "replace") if isinstance(e.stderr, bytes) else (e.stderr or "")
        rc, timed_out = -1, True
    wall = time.monotonic() - started

    (workdir / "_transcript.jsonl").write_text(out, encoding="utf-8")
    if err.strip():
        (workdir / "_agent_stderr.txt").write_text(err, encoding="utf-8")

    metrics = parse_transcript(out)
    metrics.update(
        wall_seconds=round(wall, 2),
        agent_exit_code=rc,
        timed_out=timed_out,
        bare_mode=bare,
    )
    return metrics


def parse_transcript(stream: str) -> dict:
    """Extract efficiency and reliability metrics from a stream-json transcript."""
    turns = 0
    cost = input_tokens = output_tokens = 0
    build_attempts = failed_builds = api_errors = 0
    first_build_clean = None
    pending: dict[str, str] = {}   # tool_use_id -> cargo command
    error_codes: dict[str, int] = {}
    agent_error = None
    plugins: list[str] = []

    for line in stream.splitlines():
        line = line.strip()
        if not line.startswith("{"):
            continue
        try:
            ev = json.loads(line)
        except json.JSONDecodeError:
            continue

        kind = ev.get("type")

        if kind == "system" and ev.get("subtype") == "init":
            plugins = [p.get("name", "") for p in ev.get("plugins") or []]

        if kind == "assistant":
            turns += 1
            for block in ev.get("message", {}).get("content", []) or []:
                if block.get("type") != "tool_use":
                    continue
                cmd = str(block.get("input", {}).get("command", ""))
                if CARGO_RX.search(cmd):
                    pending[block.get("id", "")] = cmd

        elif kind == "user":
            for block in ev.get("message", {}).get("content", []) or []:
                if block.get("type") != "tool_result":
                    continue
                tid = block.get("tool_use_id", "")
                if tid not in pending:
                    continue
                del pending[tid]
                build_attempts += 1
                content = block.get("content")
                if isinstance(content, list):
                    text = "\n".join(
                        b.get("text", "") for b in content if isinstance(b, dict)
                    )
                else:
                    text = str(content or "")
                failed = bool(block.get("is_error")) or bool(BUILD_FAILED_RX.search(text))
                if failed:
                    failed_builds += 1
                    for code in API_ERROR_RX.findall(text):
                        error_codes[code] = error_codes.get(code, 0) + 1
                        api_errors += 1
                if first_build_clean is None:
                    first_build_clean = not failed

        elif kind == "result":
            cost = ev.get("total_cost_usd") or 0
            usage = ev.get("usage") or {}
            input_tokens = (
                (usage.get("input_tokens") or 0)
                + (usage.get("cache_read_input_tokens") or 0)
                + (usage.get("cache_creation_input_tokens") or 0)
            )
            output_tokens = usage.get("output_tokens") or 0
            turns = ev.get("num_turns") or turns
            if ev.get("is_error"):
                agent_error = str(ev.get("result") or ev.get("terminal_reason") or "error")

    return {
        "agent_error": agent_error,
        "agent_plugins": plugins,
        "turns": turns,
        "cost_usd": round(float(cost), 4),
        "input_tokens": input_tokens,
        "output_tokens": output_tokens,
        "build_attempts": build_attempts,
        "failed_builds": failed_builds,
        "api_errors": api_errors,
        "api_error_codes": error_codes,
        "first_build_clean": first_build_clean,
    }


# -------------------------------------------------------------------- scoring


def capture_dump(workdir: Path, grid: dict, script: list[str], timeout: int):
    # --manifest-path pins cargo to this run's crate. Without it, a workdir with
    # no Cargo.toml makes cargo walk up the tree and build whatever ancestor
    # manifest it finds — which once produced `built: true` for a completely
    # unrelated package in this repository.
    cmd = [
        "cargo", "run", "--release", "--quiet",
        "--manifest-path", str((workdir / "Cargo.toml").resolve()), "--",
        "--headless", f"{grid['w']}x{grid['h']}",
        "--script", ",".join(script),
        "--dump",
    ]
    try:
        proc = subprocess.run(
            cmd, cwd=workdir, capture_output=True, text=True, timeout=timeout,
            encoding="utf-8", errors="replace", stdin=subprocess.DEVNULL,
        )
    except subprocess.TimeoutExpired:
        return None, f"program timed out after {timeout}s"
    except FileNotFoundError:
        return None, "cargo not found"
    if proc.returncode != 0:
        tail = "\n".join((proc.stderr or "").strip().splitlines()[-20:])
        return None, f"program exited {proc.returncode}:\n{tail}"
    return proc.stdout, ""


def builds(workdir: Path, timeout: int) -> tuple[bool, str]:
    manifest = (workdir / "Cargo.toml").resolve()
    if not manifest.is_file():
        # The task says to work in the current directory. An agent that put its
        # crate somewhere else has not met the contract, and saying so beats
        # letting cargo find an ancestor manifest and reporting a success that
        # belongs to another package.
        return False, (
            "no Cargo.toml in the working directory; the crate was not created "
            "where the task asked for it"
        )
    try:
        proc = subprocess.run(
            ["cargo", "build", "--release", "--quiet", "--manifest-path", str(manifest)],
            cwd=workdir, capture_output=True, text=True, timeout=timeout,
            encoding="utf-8", errors="replace", stdin=subprocess.DEVNULL,
        )
    except subprocess.TimeoutExpired:
        return False, "build timed out"
    except FileNotFoundError:
        return False, "cargo not found"
    return proc.returncode == 0, "\n".join(
        (proc.stderr or "").strip().splitlines()[-30:]
    )


def prewarm(frameworks: list[str], target_dir: Path, timeout: int) -> None:
    """Compile each framework once before the grid starts.

    Two reasons. Disk: without a shared target directory every run rebuilds the
    whole dependency tree into its own, which cost ~50 MB and several minutes
    each. Validity: with a shared directory but no pre-warm, the first run of a
    framework pays a cold compile and later replicates do not, so `wall_seconds`
    would measure build-cache state instead of agent effort.
    """
    import tempfile

    for framework in frameworks:
        # The same line the prompt gives the agent, so a dependency that warms
        # here is one that resolves there. Building it a second way is how the
        # two drifted apart in the first place.
        dep = dep_line(framework).strip("`")
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "src").mkdir()
            (root / "src" / "main.rs").write_text("fn main() {}\n", encoding="utf-8")
            (root / "Cargo.toml").write_text(
                "[workspace]\n\n"
                "[package]\n"
                'name = "prewarm"\n'
                'version = "0.0.0"\n'
                'edition = "2021"\n\n'
                f"[dependencies]\n{dep}\n",
                encoding="utf-8",
            )
            print(f"  warming {framework} … ", end="", flush=True)
            proc = subprocess.run(
                ["cargo", "build", "--release", "--quiet"],
                cwd=root, capture_output=True, text=True, timeout=timeout,
                encoding="utf-8", errors="replace", stdin=subprocess.DEVNULL,
            )
            if proc.returncode == 0:
                print("ok")
            else:
                tail = "\n".join((proc.stderr or "").strip().splitlines()[-5:])
                print(f"FAILED\n{tail}")


# ----------------------------------------------------------------------- main


def run_cell(task: str, framework: str, condition: str, rep: int, args, out_dir: Path) -> dict:
    spec = json.loads((TASKS_DIR / task / "checks.json").read_text(encoding="utf-8"))
    label = f"{task}__{framework}__{condition}__r{rep}"
    workdir = out_dir / label
    workdir.mkdir(parents=True, exist_ok=True)

    record = {
        "label": label,
        "task": task,
        "rung": spec["rung"],
        "framework": framework,
        "condition": condition,
        "replicate": rep,
        "model": args.model,
        "started": datetime.now(timezone.utc).isoformat(timespec="seconds"),
    }

    record["context_files"] = seed_context(workdir, framework, condition)
    prompt = build_prompt(task, framework, record["context_files"])
    (workdir / "_prompt.md").write_text(prompt, encoding="utf-8")

    print(f"  ▸ {label} … ", end="", flush=True)

    if args.dry_run:
        print("skipped (dry run)")
        record["dry_run"] = True
        return record

    src = None if args.no_source else source_dir(framework)
    record["source_dir"] = str(src) if src else None
    record["source_withheld"] = bool(args.no_source)

    # C4 differs from C1 only in that the ontology arrives as MCP tools, which
    # the agent sees in its tool list without opening a file. C2 and C3 both
    # required the agent to choose to look; it almost never did.
    mcp_config = None
    if condition in ("c4", "c5"):
        if not MCP_BINARY.is_file():
            record["invalid"] = True
            record["agent_error"] = f"hawktui-mcp not built at {MCP_BINARY}"
            print(f"INVALID: {record['agent_error']}")
            return record
        mcp_config = workdir / ".mcp.json"
        mcp_config.write_text(json.dumps({
            "mcpServers": {"hawktui": {"command": str(MCP_BINARY), "args": []}}
        }, indent=2), encoding="utf-8")
        record["mcp"] = True

    record.update(run_agent(prompt, workdir, args.model, args.agent_timeout, src,
                            mcp_config))

    # An agent that never ran (auth failure, quota, transport error) is a broken
    # instrument, not a framework failure. Mark it invalid so it is excluded
    # from the analysis rather than recorded as a score of zero.
    if record.get("agent_error"):
        record["invalid"] = True
        print(f"INVALID: {record['agent_error']}")
        return record

    ok, build_log = builds(workdir, args.build_timeout)
    record["built"] = ok
    if not ok:
        record["build_error"] = build_log

    if ok:
        dump, err = capture_dump(workdir, spec["grid"], spec["script"], args.program_timeout)
        if dump is None:
            record.update(
                score=0.0, contract_failed=True, frames_rendered=0,
                checks_passed=0, run_error=err,
                checks_total=len([c for c in spec["checks"] if not c.get("contract")]),
            )
        else:
            (workdir / "_dump.txt").write_text(dump, encoding="utf-8")
            result = score_dump(TASKS_DIR / task, dump)
            record.update({k: v for k, v in result.items() if k != "task"})
    else:
        record.update(
            score=0.0, contract_failed=False, frames_rendered=0, checks_passed=0,
            checks_total=len([c for c in spec["checks"] if not c.get("contract")]),
        )

    print(
        f"built={record['built']} score={record['score']:.2f} "
        f"api_errors={record.get('api_errors', 0)} "
        f"{record.get('wall_seconds', 0):.0f}s ${record.get('cost_usd', 0):.2f}"
    )
    return record


def main() -> int:
    # Frames and progress markers are Unicode; the Windows console defaults to
    # cp1252 and would abort the run on the first box-drawing character.
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")
        except AttributeError:
            pass

    ap = argparse.ArgumentParser()
    ap.add_argument("--tasks", default="t1-hello,t2-counter,t3-list,t4-dashboard,t5-browser,t6-tasks")
    ap.add_argument("--frameworks", default="hawktui,ratatui,superlighttui")
    ap.add_argument("--conditions", default="c1,c2")
    ap.add_argument("--replicates", type=int, default=5)
    ap.add_argument("--model", default="sonnet")
    ap.add_argument("--agent-timeout", type=int, default=1800)
    ap.add_argument("--build-timeout", type=int, default=600)
    ap.add_argument("--program-timeout", type=int, default=120)
    ap.add_argument("--out", type=Path, default=None,
                    help="where run directories go. Defaults inside results/, "
                         "which is convenient but not isolated: a scaffolded "
                         "crate once edited benchmarks/Cargo.toml to add itself "
                         "as a workspace member. Use --isolate for a temp root.")
    ap.add_argument("--isolate", action="store_true",
                    help="put the whole run outside the repository, so an "
                         "agent cannot reach the host tree. runs.jsonl goes "
                         "there too, since rescore.py resolves run directories "
                         "relative to it; copy it into results/ to publish.")
    ap.add_argument("--target-dir", type=Path, default=None,
                    help="shared CARGO_TARGET_DIR for every run "
                         "(default: <out>/_target)")
    ap.add_argument("--no-prewarm", action="store_true")
    ap.add_argument("--no-source", action="store_true",
                    help="withhold --add-dir, so the agent cannot read the "
                         "framework's implementation. 100%% of Hawk TUI runs "
                         "read it otherwise, which makes an ontology derived "
                         "from that source redundant.")
    ap.add_argument("--hawktui-path", type=Path, default=None,
                    help="Hawk TUI tree to depend on (default: this checkout)")
    ap.add_argument("--dry-run", action="store_true", help="seed and prompt, but do not call the agent")
    args = ap.parse_args()

    if args.hawktui_path:
        global HAWKTUI_PATH
        HAWKTUI_PATH = args.hawktui_path.resolve()

    tasks = [t for t in args.tasks.split(",") if t]
    frameworks = [f for f in args.frameworks.split(",") if f]
    conditions = [c for c in args.conditions.split(",") if c]

    for f in frameworks:
        if f not in FRAMEWORKS:
            ap.error(f"unknown framework {f!r}; choose from {', '.join(FRAMEWORKS)}")
    for c in conditions:
        if c not in CONDITIONS:
            ap.error(f"unknown condition {c!r}; choose from {', '.join(CONDITIONS)}")

    stamp = datetime.now().strftime("%Y%m%d-%H%M%S")
    # Absolute: every cargo invocation below runs with cwd set to the run
    # directory and is handed --manifest-path, so a relative path would resolve
    # against the wrong base and report a manifest the agent did create as
    # missing.
    out_dir = (args.out or (RESULTS_DIR / stamp)).resolve()
    if args.isolate:
        import tempfile
        out_dir = Path(tempfile.gettempdir()) / "hawktui-agentic" / stamp
    out_dir.mkdir(parents=True, exist_ok=True)
    jsonl = out_dir / "runs.jsonl"

    cells = [
        (t, f, c, r)
        for t in tasks
        for f in frameworks
        for c in conditions
        for r in range(1, args.replicates + 1)
        # Only Hawk TUI has an ontology; C2/C3 do not exist for the others.
        if not (c in ("c2", "c3", "c4", "c5") and not FRAMEWORKS[f]["ontology"])
    ]

    # One target directory for the whole grid: 45 private ones cost ~2 GB and
    # a cold dependency build per run.
    target_dir = (args.target_dir or (out_dir / "_target")).resolve()
    target_dir.mkdir(parents=True, exist_ok=True)
    os.environ["CARGO_TARGET_DIR"] = str(target_dir)

    if not args.dry_run and not args.no_prewarm:
        print("pre-warming build caches so every run starts equally warm")
        prewarm(frameworks, target_dir, args.build_timeout)

    print(f"{len(cells)} runs → {out_dir}")
    for task, framework, condition, rep in cells:
        record = run_cell(task, framework, condition, rep, args, out_dir)
        with jsonl.open("a", encoding="utf-8") as fh:
            fh.write(json.dumps(record) + "\n")

    print(f"\nwrote {jsonl}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
