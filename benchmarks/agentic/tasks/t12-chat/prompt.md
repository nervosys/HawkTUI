Build a terminal UI program in Rust using the **{{FRAMEWORK}}** crate.

The program is a single binary named `app`. Its Cargo.toml must depend on
{{DEP}}.

A chat transcript viewer over hard-coded, in-memory data. It talks to no
network and no model.

## Layout

The screen (100 columns × 30 rows) is divided top to bottom into:

- **Session bar** — the top row, exactly one row tall.
- **Transcript** — everything between the session bar and the composer.
- **Composer** — 3 rows tall, directly above the footer.
- **Footer** — the bottom row, exactly one row tall.

## Session bar

Two sessions, `Alpha` and `Beta`, shown on one row. The selected one is wrapped
in square brackets, so at startup the row reads `[Alpha] Beta`. `Alpha` is
selected at startup.

## Transcript

A bordered box titled `Transcript`, showing the selected session's messages in
order, one message per line, each prefixed with its speaker.

Session `Alpha` starts with:

```
you: how do I center text
bot: use the **center** alignment
```

Session `Beta` starts with:

```
you: what is a gauge
bot: a bar showing a `ratio` from 0 to 1
```

Text between double asterisks is rendered **bold** with the asterisks removed,
and text between backticks is rendered as code with the backticks removed. So
the `Alpha` reply appears as `bot: use the center alignment`, with `center`
styled bold — the asterisks must not appear on screen.

## Composer

A bordered box titled `Message` containing the text being typed, which starts
empty.

## Footer

The text `Enter send  Tab session  Esc quit`.

## Keys

- any printable character is appended to the composer
- `Backspace` removes the last character
- `Enter` appends `you: <composer text>` to the selected session's transcript,
  then appends `bot: ok`, then clears the composer. Pressing `Enter` with an
  empty composer does nothing
- `Tab` selects the other session; each session keeps its own transcript and its
  own composer text
- `Esc` quits. Every other printable key is text, including `q`

{{CONTRACT}}

Work in the current directory. When you are done the command

```
cargo run --release -- --headless 100x30 --script "h,i,Enter,Tab,Esc" --dump
```

must print the frames described above.
