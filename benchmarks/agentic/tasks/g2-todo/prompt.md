Build a GUI program in Rust using the **{{FRAMEWORK}}** crate.

The program is a single binary named `app`. Its Cargo.toml must depend on
{{DEP}}.

## Layout

A 320×240 logical-pixel window holding, stacked vertically:

1. a label with agent id `title`, reading `Todo`
2. a label with agent id `count`, reading `N done of M`, where `M` is the
   number of items and `N` how many are complete
3. one row per item, in order, each holding
   - a checkbox or toggle with agent id `check-<index>`, starting unchecked
   - a label with agent id `item-<index>` reading the item's text
4. a button with agent id `clear`, reading `Clear done`

The three items are `Write tests`, `Fix wrapping`, `Ship it`, indexed from `0`.

Every agent id above must be addressable by id: the harness clicks widgets by
id and never by position.

## Behaviour

- Clicking `check-<index>` toggles that item between done and not done, and
  the `count` label updates.
- Clicking `clear` removes every item that is done. Remaining items keep their
  order and are re-indexed from `0`, so `item-0` is always the first item
  still on the list.
- The `q` key quits.

{{CONTRACT}}

Work in the current directory. When you are done the command

```
cargo run --release -- --headless 320x240 --script "click:check-0,click:check-2,click:clear,key:q" --dump
```

must print the frames described above.
