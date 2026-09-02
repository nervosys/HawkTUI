Build a terminal UI program in Rust using the **{{FRAMEWORK}}** crate.

The program is a single binary named `app`. Its Cargo.toml must depend on
{{DEP}}.

## What it must display

A bordered box, filling everything except the bottom row, titled `Settings`.

Inside it, one row per setting, written as `Name: Value`. Exactly one row is
selected; its line begins with the two characters `> ` and the others begin with
two spaces. The first row is selected at startup.

The settings, in this order, each cycling through a fixed list of values:

| Name | Values, in cycle order | Starts at |
|---|---|---|
| `Theme` | `Dark`, `Light`, `Solarized` | `Dark` |
| `Font Size` | `12`, `14`, `16` | `14` |
| `Vim Mode` | `Off`, `On` | `Off` |

The bottom row, exactly one row tall, reads `Enter cycle  q quit`.

## Keys

- `Down` / `Up` move the selection between settings, stopping at either end
- `Enter` and `Right` change the selected setting to the next value in its
  cycle, wrapping from the last value back to the first
- `Left` changes it to the previous value, wrapping from the first back to the
  last
- `q` quits

{{CONTRACT}}

Work in the current directory. When you are done the command

```
cargo run --release -- --headless 80x24 --script "Enter,Down,Right,Right,q" --dump
```

must print the frames described above.
