Build a terminal UI program in Rust using the **{{FRAMEWORK}}** crate.

The program is a single binary named `app`. Its Cargo.toml must depend on
{{DEP}}.

A fixed-width table of text in several scripts. Getting the column alignment
right is the whole task.

## Layout

The screen is 60 columns × 12 rows. A bordered box titled `Widths` fills it,
except the bottom row, which is a status line exactly one row tall.

## The table

Inside the box, one row per entry, in this order. Each row is the **label**,
then the **sample**, then the **count**, laid out so that all three start in the
same column on every row:

| label | sample | count |
|---|---|---|
| `ascii` | `hello` | `5` |
| `cjk` | `日本語` | `3` |
| `emoji` | `🙂🙂` | `2` |
| `accent` | `café` | `4` |
| `mixed` | `a日b` | `3` |

The **label** column starts at the first character inside the box's left border.
The **sample** column starts exactly 8 columns to the right of the label column.
The **count** column starts exactly 20 columns to the right of the label column.

"Columns" means terminal display columns, not bytes and not characters. A CJK
ideograph and an emoji each occupy two display columns; an accented Latin letter
occupies one. `日本語` is three characters and six display columns wide.

The `count` value is the number of **characters** in the sample, which is not
the same as its display width.

## Status line

The bottom row reads `total width: N`, where `N` is the sum of the display
widths of all five samples.

## Keys

- `q` quits

{{CONTRACT}}

Work in the current directory. When you are done the command

```
cargo run --release -- --headless 60x12 --script "q" --dump
```

must print the frames described above.
