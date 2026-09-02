Build a terminal UI program in Rust using the **{{FRAMEWORK}}** crate.

The program is a single binary named `app`. Its Cargo.toml must depend on
{{DEP}}.

## What it must display

A single bordered box that fills the whole screen. The box has the title
`Greeting`. Inside the box, on its own line, is the text `Hello, world!`.

Pressing `q` quits.

{{CONTRACT}}

Work in the current directory. When you are done the command

```
cargo run --release -- --headless 80x24 --script "q" --dump
```

must print the frames described above.
