Build a terminal UI program in Rust using the **{{FRAMEWORK}}** crate.

The program is a single binary named `app`. Its Cargo.toml must depend on
{{DEP}}.

## What it must display

A single bordered box that fills the whole screen, titled `Counter`. Inside it,
on its own line, the text `Count: N` where `N` is the current count, starting
at `0`.

## Keys

- `+` increments the count by one
- `-` decrements the count by one
- `q` quits

{{CONTRACT}}

Work in the current directory. When you are done the command

```
cargo run --release -- --headless 80x24 --script "+,+,-,q" --dump
```

must print the frames described above.
