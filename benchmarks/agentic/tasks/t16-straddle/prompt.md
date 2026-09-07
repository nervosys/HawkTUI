Build a terminal UI program in Rust using the **{{FRAMEWORK}}** crate.

The program is a single binary named `app`. Its Cargo.toml must depend on
{{DEP}}.

The program wraps text to the width of the screen. The screen is narrow and the
text contains double-width characters, so some of them do not fit in the columns
left at the end of a row.

Throughout, "width" means **terminal display columns**. A CJK ideograph and an
emoji each occupy two columns. An ASCII letter occupies one.

## Layout

The screen is 20 columns × 8 rows. The bottom row is a status line exactly one
row tall. The seven rows above it hold the wrapped text, top-aligned, and any
row not needed is left blank.

## Contents

Wrap each of these three source lines, in order, to the full 20-column width.
Each source line begins on a new row.

```
x日本語日本語日本語日
🙂🙂🙂🙂🙂🙂🙂🙂🙂🙂🙂
abc def
```

Wrap by display column, not by character. A character is placed on the current
row only if the whole of it fits: **a double-width character is never split
across a row boundary, and it never overhangs the last column.** When only one
column is left and the next character needs two, that column stays empty and the
character starts the next row.

Do not insert spaces between the characters of the text. A double-width
character is already two columns wide; nothing needs to be added to make it so.
(Padding each row out to 20 columns with spaces is required by the dump format
below, and is not what this forbids.)

## Status line

The bottom row reads `rows: N`, where `N` is the number of rows the wrapped text
occupies.

## Keys

- `q` quits

{{CONTRACT}}

Work in the current directory. When you are done the command

```
cargo run --release -- --headless 20x8 --script "q" --dump
```

must print the frames described above.
