Build a terminal UI program in Rust using the **{{FRAMEWORK}}** crate.

The program is a single binary named `app`. Its Cargo.toml must depend on
{{DEP}}.

Text laid out to exact display widths. Getting the wrapping and truncation
boundaries right is the whole task.

Throughout, "width" means **terminal display columns**, not bytes and not
characters. A CJK ideograph and an emoji each occupy two columns; an accented
Latin letter occupies one.

## Layout

The screen is 40 columns × 14 rows. A bordered box titled `Text` fills
everything except the bottom row, which is a status line exactly one row tall.

## Wrapped paragraph

Inside the box, wrap this text to a maximum of **24 display columns** per line:

```
the quick 日本語 fox jumps 🙂 over café lazy dogs
```

Wrap greedily on spaces: put as many whole words on a line as fit within 24
columns, then start a new line. Never split a word, and never let a line exceed
24 columns. Do not print the separating spaces at the end of a line.

Write each wrapped line on its own row, in order, starting at the first row
inside the box.

## Truncated line

Two rows below the last wrapped line, show this text:

```
日本語テキストが長い
```

truncated so that the result is **at most 10 display columns including a
trailing `…`** (U+2026, one column). Truncate whole characters: take as many
leading characters as fit alongside the ellipsis, then append the ellipsis.

## Status line

The bottom row reads `lines: N`, where `N` is the number of wrapped lines the
paragraph produced.

## Keys

- `q` quits

{{CONTRACT}}

Work in the current directory. When you are done the command

```
cargo run --release -- --headless 40x14 --script "q" --dump
```

must print the frames described above.
