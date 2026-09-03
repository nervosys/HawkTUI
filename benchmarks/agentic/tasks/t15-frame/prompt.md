Build a terminal UI program in Rust using the **{{FRAMEWORK}}** crate.

The program is a single binary named `app`. Its Cargo.toml must depend on
{{DEP}}.

A bordered box containing text in several scripts. The right-hand border must
line up on every row.

Throughout, "width" means **terminal display columns**. A CJK ideograph and an
emoji each occupy two columns. A combining mark occupies none: it attaches to
the character before it. An ASCII letter occupies one.

## Layout

The screen is 30 columns × 10 rows. A bordered box titled `Frame` fills
everything except the bottom row, which is a status line exactly one row tall.

## Contents

Inside the box, these five lines, in order, each written exactly as given:

```
plain text
日本語です
cafe\u{301} table
🙂 ok
mixed 語 x
```

The third line is the five characters `c`, `a`, `f`, `e`, then U+0301 COMBINING
ACUTE ACCENT, then ` table`. It must render as `café table` and occupies 10
display columns, not 11.

Each line is padded on the right with spaces so that the box's right border sits
in the **same display column on every row**, including the rows above and below
the text. Do not pad by counting characters: the lines differ in character count
and display width, and only the display width matters.

## Status line

The bottom row reads `inner: N`, where `N` is the display width of the widest
line, which determines the box's inner width.

## Keys

- `q` quits

{{CONTRACT}}

Work in the current directory. When you are done the command

```
cargo run --release -- --headless 30x10 --script "q" --dump
```

must print the frames described above.
