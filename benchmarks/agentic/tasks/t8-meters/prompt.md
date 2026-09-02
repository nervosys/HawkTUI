Build a terminal UI program in Rust using the **{{FRAMEWORK}}** crate.

The program is a single binary named `app`. Its Cargo.toml must depend on
{{DEP}}.

## What it must display

A bordered box, filling everything except the bottom row, titled `Transfer`.

Inside it, three indicators of the same underlying completion value, which
starts at 25 percent:

1. A **filled bar** several rows tall in the sense that it spans the full width
   of the box, whose filled portion is proportional to the completion value, and
   whose label is the value as a percentage followed by a percent sign — `25%`.
2. A **single-row indicator** on its own line, drawn with line-drawing
   characters rather than solid blocks, also proportional to the completion
   value, and also labelled with the same percentage.
3. An **animated spinner**: one character on its own line that advances to the
   next frame of its animation each time the animation is stepped, cycling
   through at least four distinct characters before repeating.

The bottom row, exactly one row tall, reads `u up  d down  t tick  q quit`.

## Keys

- `u` increases the completion value by 25, stopping at 100
- `d` decreases it by 25, stopping at 0
- `t` advances the spinner by one animation frame; it changes nothing else
- `q` quits

{{CONTRACT}}

Work in the current directory. When you are done the command

```
cargo run --release -- --headless 80x24 --script "u,t,t,d,q" --dump
```

must print the frames described above.
