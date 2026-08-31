#!/usr/bin/env python3
"""
hawktui-demo.py — Self-running demo of the Hawk TUI Agent Protocol.

Records a complete agent interaction session for demo/marketing purposes.
Run this script, then use the output for video recording or documentation.

Usage:
    # Build first
    cargo build --release --bin hawktui-server

    # Run the demo
    python3 scripts/hawktui-demo.py

    # Or record with asciinema
    asciinema rec demo.cast -c "python3 scripts/hawktui-demo.py"
"""

import json
import subprocess
import sys
import time

# ANSI color helpers
BOLD = "\033[1m"
DIM = "\033[2m"
GREEN = "\033[32m"
CYAN = "\033[36m"
YELLOW = "\033[33m"
MAGENTA = "\033[35m"
RESET = "\033[0m"
BLUE = "\033[34m"


def slow_print(text, delay=0.03):
    """Print text character by character for dramatic effect."""
    for ch in text:
        sys.stdout.write(ch)
        sys.stdout.flush()
        time.sleep(delay)
    print()


def section(title):
    """Print a section header."""
    print()
    print(f"{BOLD}{CYAN}{'─' * 60}{RESET}")
    slow_print(f"{BOLD}{CYAN}  {title}{RESET}", 0.02)
    print(f"{BOLD}{CYAN}{'─' * 60}{RESET}")
    print()
    time.sleep(0.5)


def show_request(req):
    """Pretty-print an outgoing request."""
    print(f"  {DIM}agent →{RESET}  {YELLOW}{json.dumps(req)}{RESET}")
    time.sleep(0.3)


def show_response(resp, compact=False):
    """Pretty-print an incoming response."""
    if compact:
        print(f"  {DIM}server →{RESET} {GREEN}{json.dumps(resp)}{RESET}")
    else:
        formatted = json.dumps(resp, indent=2)
        for line in formatted.split("\n"):
            print(f"  {DIM}server →{RESET} {GREEN}{line}{RESET}")
    time.sleep(0.2)


def main():
    # Find the binary
    import os
    binary = os.path.join("target", "release", "hawktui-server")
    if sys.platform == "win32":
        binary += ".exe"
    if not os.path.exists(binary):
        binary = os.path.join("target", "debug", "hawktui-server")
        if sys.platform == "win32":
            binary += ".exe"
    if not os.path.exists(binary):
        print(f"{BOLD}Error:{RESET} hawktui-server not found. Run: cargo build --bin hawktui-server")
        sys.exit(1)

    slow_print(f"{BOLD}{MAGENTA}Hawk TUI Agent Protocol — Live Demo{RESET}", 0.04)
    print(f"{DIM}Showing how an AI agent discovers and controls a TUI application{RESET}")
    print(f"{DIM}through structured JSON messages — no screen-scraping needed.{RESET}")
    time.sleep(1.5)

    # Start the server
    section("1. Spawn the server")
    slow_print(f"  $ hawktui-server", 0.05)
    time.sleep(0.5)

    proc = subprocess.Popen(
        [binary],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )

    def send(request, req_id=None):
        msg = {**request}
        if req_id:
            msg["id"] = req_id
        proc.stdin.write(json.dumps(msg) + "\n")
        proc.stdin.flush()
        line = proc.stdout.readline()
        return json.loads(line)

    print(f"  {GREEN}✓ Server running (PID {proc.pid}){RESET}")
    time.sleep(1)

    # Step 1: Ping
    section("2. Test connectivity")
    req = {"type": "ping"}
    show_request(req)
    resp = send(req)
    show_response(resp, compact=True)
    print(f"\n  {GREEN}✓ Connection alive{RESET}")
    time.sleep(1)

    # Step 2: Discover ontology
    section("3. Discover the UI — query_ontology")
    slow_print(f"  {DIM}The agent asks: \"What widgets exist in this app?\"{RESET}", 0.02)
    print()
    req = {"type": "query_ontology"}
    show_request(req)
    resp = send(req, "discover-1")
    print()

    if resp.get("success") and resp.get("data"):
        for schema in resp["data"]:
            name = schema.get("name", "?")
            role = schema.get("default_role", "?")
            tags = ", ".join(schema.get("tags", []))
            print(f"  {BOLD}{name:15}{RESET} {DIM}role={role:12} tags=[{tags}]{RESET}")
            time.sleep(0.15)

    print(f"\n  {GREEN}✓ Found {len(resp.get('data', []))} widget types with full schemas{RESET}")
    time.sleep(1.5)

    # Step 3: Get a specific schema
    section("4. Inspect a widget schema")
    slow_print(f"  {DIM}\"Tell me everything about the Gauge widget.\"{RESET}", 0.02)
    print()
    req = {"type": "get_schema", "widget_type": "Gauge"}
    show_request(req)
    resp = send(req, "schema-1")
    show_response(resp)
    time.sleep(1.5)

    # Step 4: Get UI tree
    section("5. Read the UI tree")
    slow_print(f"  {DIM}\"What's the current state of the entire UI?\"{RESET}", 0.02)
    print()
    req = {"type": "get_tree"}
    show_request(req)
    resp = send(req, "tree-1")
    show_response(resp, compact=True)
    time.sleep(1.5)

    # Step 5: Inject events (drive the counter)
    section("6. Control the app — inject keyboard events")
    slow_print(f"  {DIM}The agent presses Up 3 times to increment the counter.{RESET}", 0.02)
    print()

    for i in range(3):
        req = {"type": "inject_event", "event": {"kind": "key", "code": "Up"}}
        show_request(req)
        resp = send(req, f"key-{i}")
        show_response(resp, compact=True)
        time.sleep(0.4)

    time.sleep(0.5)

    # Step 6: Observe the result
    section("7. Observe the result — get_state")
    slow_print(f"  {DIM}\"What does the UI look like now?\"{RESET}", 0.02)
    print()
    req = {"type": "get_tree"}
    show_request(req)
    resp = send(req, "verify-1")
    show_response(resp, compact=True)
    time.sleep(1.5)

    # Step 7: Clean shutdown
    section("8. Clean shutdown")
    req = {"type": "quit"}
    show_request(req)
    resp = send(req)
    show_response(resp, compact=True)
    proc.wait(timeout=5)
    print(f"\n  {GREEN}✓ Server exited cleanly{RESET}")
    time.sleep(1)

    # Summary
    print()
    print(f"{BOLD}{CYAN}{'─' * 60}{RESET}")
    print()
    slow_print(f"{BOLD}  What just happened:{RESET}", 0.03)
    print()
    print(f"  1. Agent spawned a Hawk TUI app as a headless process")
    print(f"  2. Agent discovered all widgets via {YELLOW}query_ontology{RESET}")
    print(f"  3. Agent inspected widget schemas — types, constraints, actions")
    print(f"  4. Agent read the UI tree — positions, state, capabilities")
    print(f"  5. Agent controlled the app via {YELLOW}inject_event{RESET}")
    print(f"  6. Agent verified the result via {YELLOW}get_tree{RESET}")
    print(f"  7. Agent shut down cleanly via {YELLOW}quit{RESET}")
    print()
    slow_print(f"{BOLD}  No screen-scraping. No hardcoded selectors. No brittleness.{RESET}", 0.03)
    slow_print(f"{BOLD}  Just a self-describing UI ontology.{RESET}", 0.03)
    print()
    print(f"  {DIM}github.com/nervosys/HawkTUI{RESET}")
    print()


if __name__ == "__main__":
    main()
