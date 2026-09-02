Build a terminal UI program in Rust using the **{{FRAMEWORK}}** crate.

The program is a single binary named `app`. Its Cargo.toml must depend on
{{DEP}}.

A master/detail file browser over an in-memory, hard-coded file set. It touches
no real files.

## Layout

The screen (100 columns × 30 rows) is divided top to bottom into:

- **Body** — everything except the bottom row.
- **Filter bar** — the bottom row, exactly one row tall.

The body is split left and right: the left pane is exactly 30 columns wide, the
right pane takes the rest.

## Contents

- **Left pane** — a bordered box titled `Files`, listing the file names that
  match the current filter, in this order: `alpha.txt`, `beta.txt`,
  `gamma.txt`, `delta.log`, `epsilon.txt`, `zeta.log`, `eta.md`, `theta.md`.
  Exactly one visible file is selected; its line begins with the two characters
  `> ` and unselected lines begin with two spaces. The first visible file is
  selected at startup, and whenever the filter changes.
- **Right pane** — a bordered box titled `Preview` showing the content of the
  selected file: 40 lines, where line `n` (1-based) is `<name> line NN`,
  zero-padded to two digits — for example `alpha.txt line 01`. Only as many
  lines as fit are shown, starting from the current scroll offset, which is 0
  whenever the selection changes.
- **Filter bar** — the text `Filter: ` followed by the current filter string,
  which starts empty.

## Filtering

The left pane shows only files whose name contains the filter string as a
substring. An empty filter shows all eight.

## Focus

Exactly one of the two is focused: the file list or the filter bar. The file
list is focused at startup. The focused one's label is prefixed with an
asterisk — so the box title is `*Files` or the bar reads `*Filter: ` — and the
unfocused one's label has no asterisk.

## Keys

- `Tab` moves focus between the file list and the filter bar
- When the **file list** is focused:
  - `Down` / `Up` move the selection within the visible files, stopping at
    either end
  - `j` / `k` scroll the preview down / up by one line, stopping at either end
- When the **filter bar** is focused:
  - any printable character is appended to the filter string
  - `Backspace` removes the last character
- `q` quits, from either focus

{{CONTRACT}}

Work in the current directory. When you are done the command

```
cargo run --release -- --headless 100x30 --script "Down,Tab,a,Tab,j,q" --dump
```

must print the frames described above.
