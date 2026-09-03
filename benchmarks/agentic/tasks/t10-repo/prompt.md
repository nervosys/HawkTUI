Build a terminal UI program in Rust using the **{{FRAMEWORK}}** crate.

The program is a single binary named `app`. Its Cargo.toml must depend on
{{DEP}}.

A git repository browser over hard-coded, in-memory data. It runs no git
commands and reads no files.

## Layout

The screen (100 columns × 30 rows) has a bottom row exactly one row tall. Above
it, the space is split left and right: the left column is exactly 40 columns
wide, the right column takes the rest.

The left column is split top to bottom into four boxes of equal height, in this
order: `Status`, `Files`, `Branches`, `Commits`.

## Contents

**Status** — two lines: `On branch main` and `2 staged, 3 modified`.

**Files** — these five entries, in order, each written exactly as shown:

```
M  src/main.rs
A  src/lib.rs
M  README.md
D  old.txt
?? notes.md
```

**Branches** — `* main`, `  feature/ui`, `  fix/parser`.

**Commits** — `a1b2c3d Initial commit`, `b2c3d4e Add parser`,
`c3d4e5f Fix wrapping`, `d4e5f6a Update docs`.

**Right column** — a box titled `Diff` showing the diff of the currently
selected file, as four lines:

```
--- a/<name>
+++ b/<name>
-old line
+new line
```

where `<name>` is the selected file's path without its status prefix — so
`--- a/src/main.rs` when the first file is selected.

**Bottom row** — the text `Tab pane  q quit`.

## Focus and selection

Exactly one of the four left boxes is focused. Its title is prefixed with an
asterisk — `*Files` — and the other three titles have no asterisk. `Status` is
focused when the program starts.

`Files`, `Branches` and `Commits` each keep their own selection, which starts on
their first entry and does not change when focus moves elsewhere. The selected
line in the focused box begins with the two characters `> `; every other line in
that box begins with two spaces. A box that is not focused shows no `> ` marker
at all.

`Status` has no selection.

The `Diff` box always shows the file selected in `Files`, whichever box is
focused.

## Keys

- `Tab` moves focus to the next box, wrapping from `Commits` back to `Status`
- `Down` / `Up` move the selection within the focused box, stopping at either
  end; they do nothing when `Status` is focused
- `q` quits

{{CONTRACT}}

Work in the current directory. When you are done the command

```
cargo run --release -- --headless 100x30 --script "Tab,Down,Down,Tab,Down,q" --dump
```

must print the frames described above.
