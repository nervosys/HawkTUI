# Harness contract

Every benchmark program must implement this contract so that one verifier can
score every framework identically. The contract is injected verbatim into the
prompt for every task, every framework, and every condition.

## Command line

```
<prog> --headless <W>x<H> --script "<KEY>,<KEY>,..." --dump
```

- `--headless WxH` — render into an offscreen buffer of exactly `W` columns by
  `H` rows. Never touch the real terminal, never enter raw mode, never enter the
  alternate screen. The program must run correctly with stdin closed and stdout
  redirected to a file.
- `--script` — a comma-separated list of key names, applied in order.
- `--dump` — write frames to stdout as described below.

## Frame dump format

1. Render the initial frame and print it.
2. For each key in the script: apply the key, render, and print the resulting
   frame.
3. If a key causes the program to quit, exit **without** printing a frame for
   it. (The frame count therefore proves that the quit key works.)

Each frame is exactly `H` lines of exactly `W` **display columns**, padded
with spaces, each line terminated by `\n`. Frames are separated by a single
form feed (`\x0c`) on its own line. No ANSI escape sequences, no colour, no
cursor positioning — plain text only.

Display columns are not characters. A double-width glyph is one character
occupying two columns, written once with nothing after it. A combining mark
is one character occupying none. A row of `W` columns may therefore be more
or fewer than `W` characters long, and the width that matters is always the
one a terminal would render.

Exit code 0 on success.

## Key names

| Name | Meaning |
|---|---|
| `Up` `Down` `Left` `Right` | arrow keys |
| `Enter` `Esc` `Tab` `Backspace` `Space` | named keys |
| any single character | that character, e.g. `a`, `+`, `-`, `q` |

## Why this exists

The verifier scores rendered characters, never source code. It cannot see which
framework produced a frame, so it cannot favour one. The cost is that ignoring
this contract scores zero regardless of UI quality; such runs are reported
separately as `contract_failed` rather than as behavioural failures.
