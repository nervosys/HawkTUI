Build a terminal UI program in Rust using the **{{FRAMEWORK}}** crate.

The program is a single binary named `app`. Its Cargo.toml must depend on
{{DEP}}.

## Layout

The screen (100 columns × 30 rows) is divided top to bottom into:

- **Header** — 3 rows tall. A bordered box titled `Dashboard`.
- **Body** — everything between the header and the footer.
- **Footer** — the bottom row, exactly one row tall.

The body is split left and right into two columns of equal width. The right
column is then split top and bottom into two boxes of equal height.

## Contents

- **Left column** — a bordered box titled `CPU` containing a horizontal gauge.
  The gauge starts at 30% and its label is the percentage followed by a percent
  sign, e.g. `30%`. The filled portion of the bar is proportional to the
  percentage.
- **Right column, top** — a bordered box titled `History` containing a
  sparkline over the data `[1, 2, 3, 4, 5, 6, 7, 8]`.
- **Right column, bottom** — a bordered box titled `Services` listing exactly
  these five items, one per line: `api`, `web`, `db`, `cache`, `queue`.
- **Footer** — the text `q quit  u up  d down`.

## Keys

- `u` increases the CPU percentage by 10, stopping at 100
- `d` decreases the CPU percentage by 10, stopping at 0
- `q` quits

{{CONTRACT}}

Work in the current directory. When you are done the command

```
cargo run --release -- --headless 100x30 --script "u,u,d,q" --dump
```

must print the frames described above.
