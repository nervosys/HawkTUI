Build a terminal UI program in Rust using the **{{FRAMEWORK}}** crate.

The program is a single binary named `app`. Its Cargo.toml must depend on
{{DEP}}.

The program wraps text to the width of the screen. The text is built from emoji
that are *several codepoints each*: a family emoji is three people joined by
zero-width joiners, and a thumbs-up carries a separate skin-tone modifier. Each
of them is one glyph occupying two terminal columns.

Throughout, "width" means **terminal display columns**, and the unit that is
placed, wrapped and counted is the **grapheme cluster**, not the codepoint. The
family emoji below is one cluster of two columns, not three glyphs of two.

## Layout

The screen is 20 columns x 8 rows. The bottom row is a status line exactly one
row tall. The seven rows above it hold the wrapped text, top-aligned, and any
row not needed is left blank.

## Contents

Wrap each of these three source lines, in order, to the full 20-column width.
Each source line begins on a new row.

```
👨‍👩‍👧👨‍👩‍👧👨‍👩‍👧👨‍👩‍👧👨‍👩‍👧👨‍👩‍👧👨‍👩‍👧👨‍👩‍👧👨‍👩‍👧👨‍👩‍👧👨‍👩‍👧
👍🏽👍🏽👍🏽👍🏽👍🏽👍🏽👍🏽👍🏽👍🏽👍🏽👍🏽
abc def
```

The first line is eleven family emoji; the second is eleven thumbs-up, each
with a skin-tone modifier. Ten of a two-column glyph fill a 20-column row
exactly, so each of the first two lines occupies two rows.

Never split a cluster. A family emoji is not three emoji, and a thumbs-up and
its skin tone are not two glyphs: splitting either at a row boundary, or
counting its codepoints as separate columns, is wrong.

Do not insert spaces between the characters of the text. A two-column glyph is
already two columns wide; nothing needs to be added to make it so. (Padding each
row out to 20 columns with spaces is required by the dump format below, and is
not what this forbids.)

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
