Build a terminal UI program in Rust using the **{{FRAMEWORK}}** crate.

The program is a single binary named `app`. Its Cargo.toml must depend on
{{DEP}}.

A system monitor over hard-coded, in-memory data. It reads nothing from the
real system.

## Layout

The screen (100 columns × 30 rows) is divided top to bottom into:

- **Meters** — 8 rows tall.
- **History** — 7 rows tall.
- **Processes** — everything between History and the footer.
- **Footer** — the bottom row, exactly one row tall.

The Meters region is split left and right into two halves of equal width.

## Contents

**Meters, left** — a box titled `CPU` containing four horizontal bars, one per
core, each labelled with its core name and its percentage: `CPU0 10%`,
`CPU1 20%`, `CPU2 30%`, `CPU3 40%`. Each bar's filled portion is proportional
to its percentage.

**Meters, right** — a box titled `Memory` containing one horizontal bar
labelled `MEM 55%`, and below it the line `6.4 GiB / 16.0 GiB`.

**History** — a box titled `Network` containing a sparkline over the data
`[1, 3, 2, 5, 4, 6, 3, 7]`.

**Processes** — a box titled `Processes` containing a table with a header row
of exactly four column labels: `PID`, `NAME`, `CPU%`, `MEM%`. The rows, in this
order:

| PID | NAME | CPU% | MEM% |
|-----|------|------|------|
| 1201 | rustc | 42 | 18 |
| 880 | firefox | 17 | 35 |
| 2310 | cargo | 63 | 9 |
| 145 | systemd | 3 | 2 |
| 1990 | zsh | 11 | 4 |

**Footer** — the text `s sort  r refresh  q quit`.

## Keys

- `r` adds 5 to every core percentage and to the memory percentage, wrapping
  back to 0 after 100
- `s` sorts the process rows by `CPU%`, descending on the first press and
  toggling to ascending on the next
- `q` quits

{{CONTRACT}}

Work in the current directory. When you are done the command

```
cargo run --release -- --headless 100x30 --script "r,s,s,q" --dump
```

must print the frames described above.
