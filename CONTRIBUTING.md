# Contributing to Hawk TUI

Thank you for your interest in contributing! This document covers the basics.

## Getting Started

```sh
git clone https://github.com/nervosys/HawkTUI.git
cd Hawk TUI
cargo test --lib --tests
```

Minimum supported Rust version (MSRV): **1.80**

## Development Workflow

1. Fork the repo and create a feature branch from `master`.
2. Make your changes — keep PRs small and focused.
3. Run all checks locally before pushing:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --lib --tests
cargo doc --no-deps
```

4. Open a pull request against `master`.

## Code Style

- Run `cargo fmt` before committing.
- No `clippy::allow` without a comment explaining why.
- Public items should have doc comments (`/// ...`). The crate uses `#![warn(missing_docs)]`.

## Adding a Widget

1. Create `src/widget/my_widget.rs` implementing `Widget` (or `StatefulWidget`).
2. Implement `Discoverable` with a schema, capabilities, semantic role, and actions.
3. Re-export from `src/widget/mod.rs`.
4. Add render tests in `tests/widget_render.rs`.

## Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/):

```
feat: add sparkline gradient support
fix: prevent buffer panic on OOB index
docs: add missing doc comments for event module
```

## Reporting Issues

Search existing issues first. When filing a new issue, include:

- Rust / OS version
- Steps to reproduce
- Expected vs actual behaviour

## License

By contributing you agree that your contributions will be licensed under
[AGPL-3.0-or-later](./LICENSE).
