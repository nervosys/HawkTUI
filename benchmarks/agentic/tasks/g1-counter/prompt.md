Build a GUI program in Rust using the **{{FRAMEWORK}}** crate.

The program is a single binary named `app`. Its Cargo.toml must depend on
{{DEP}}.

## Layout

A 240×120 logical-pixel window holding three widgets stacked vertically:

1. a label with agent id `title`, reading `Counter`
2. a label with agent id `count`, reading `Count: N`, where `N` starts at `0`
3. a button with agent id `inc`, reading `+`

Every one of those agent ids must be addressable: the harness clicks `inc` by
id, and reads `count` by id.

## Behaviour

- Clicking `inc` increases the count by one.
- The `q` key quits.

{{CONTRACT}}

Work in the current directory. When you are done the command

```
cargo run --release -- --headless 240x120 --script "click:inc,click:inc,key:q" --dump
```

must print the frames described above.
