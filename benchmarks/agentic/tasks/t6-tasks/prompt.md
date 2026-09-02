Build a terminal UI program in Rust using the **{{FRAMEWORK}}** crate.

The program is a single binary named `app`. Its Cargo.toml must depend on
{{DEP}}.

A task manager over an in-memory list. It reads and writes no files.

## Layout

The screen (100 columns × 30 rows) is divided top to bottom into:

- **Tab bar** — the top row, exactly one row tall.
- **Table** — everything between the tab bar and the footer.
- **Footer** — the bottom row, exactly one row tall.

## Tab bar

Three tabs in this order: `All`, `Active`, `Done`. The selected tab is wrapped
in square brackets and the others are not, so at startup the row reads
`[All] Active Done`. `All` is selected at startup.

## Table

A bordered box titled `Tasks` containing a table with a header row of exactly
three column labels: `ID`, `Title`, `Status`. The seed rows, in this order:

| ID | Title | Status |
|----|-------|--------|
| 1 | Deploy staging | Done |
| 2 | Review PR | Active |
| 3 | Write tests | Active |
| 4 | Ship release | Active |
| 5 | Audit deps | Done |

The `All` tab shows every row; `Active` shows only rows whose status is
`Active`; `Done` shows only rows whose status is `Done`.

## Modal dialog

Pressing `a` opens a modal dialog centred on the screen, drawn on top of the
table so that it hides the part of the table it covers. It is a bordered box
titled `New Task`, at least 30 columns wide and at least 5 rows tall,
containing a text input that starts empty.

While the dialog is open:

- any printable character is appended to the input
- `Backspace` removes the last character
- `Enter` appends a new task whose title is the input text, whose ID is one
  greater than the highest existing ID, and whose status is `Active`, then
  closes the dialog
- `Esc` closes the dialog without adding anything

While the dialog is open, no other key does anything.

## Keys

- `Tab` selects the next tab, wrapping from `Done` back to `All`
- `s` sorts the table rows by title, ascending on the first press and toggling
  to descending on the next
- `a` opens the dialog
- `q` quits

## Footer

The text `a add  s sort  Tab switch  q quit`.

{{CONTRACT}}

Work in the current directory. When you are done the command

```
cargo run --release -- --headless 100x30 --script "a,x,y,Enter,s,Tab,q" --dump
```

must print the frames described above.
