Build a terminal UI program in Rust using the **{{FRAMEWORK}}** crate.

The program is a single binary named `app`. Its Cargo.toml must depend on
{{DEP}}.

## Layout

The screen is split vertically into two regions:

- **List** — everything except the bottom row.
- **Status bar** — the bottom row, exactly one row tall.

## List

A bordered box titled `Items` containing 20 items. Item `i` (1-based) has the
text `Item NN`, zero-padded to two digits: `Item 01`, `Item 02`, … `Item 20`.

Exactly one item is selected at a time. The selected item's line begins with
the two characters `> ` (greater-than, space). Unselected items begin with two
spaces. The first item is selected when the program starts.

## Status bar

The bottom row shows `S / 20` where `S` is the 1-based index of the selected
item — so `1 / 20` at startup.

## Keys

- `Down` moves the selection to the next item; it stops at the last item
- `Up` moves the selection to the previous item; it stops at the first item
- `q` quits

{{CONTRACT}}

Work in the current directory. When you are done the command

```
cargo run --release -- --headless 80x24 --script "Down,Down,Down,q" --dump
```

must print the frames described above.
