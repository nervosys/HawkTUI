Build a terminal UI program in Rust using the **{{FRAMEWORK}}** crate.

The program is a single binary named `app`. Its Cargo.toml must depend on
{{DEP}}.

## Layout

The screen (100 columns × 30 rows) is split left and right into two columns of
equal width, above a bottom row exactly one row tall.

## Left column

A bordered box titled `Plot` containing a drawing surface that addresses points
at **finer resolution than one character cell** — a single character must be
able to show several distinct lit points. Draw a circle on it, centred in the
surface, with a radius of roughly a third of the surface's width.

## Right column

A bordered box titled `Calendar` showing one month as a grid: a row of
weekday abbreviations, then the days of that month laid out in weeks under the
correct weekday columns. The title line inside the box names the month and year
in full, for example `March 2026`.

It starts on **March 2026**, and the 15th is highlighted by wrapping the day
number in square brackets: `[15]`.

## Bottom row

The text `n next  p prev  q quit`.

## Keys

- `n` moves to the next month, rolling the year over after December
- `p` moves to the previous month, rolling the year back before January
- the highlight stays on the 15th of whichever month is shown
- `q` quits

{{CONTRACT}}

Work in the current directory. When you are done the command

```
cargo run --release -- --headless 100x30 --script "n,p,p,q" --dump
```

must print the frames described above.
