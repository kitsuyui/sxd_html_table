# Contributing to sxd_html_table

Thank you for your interest in contributing!

## Development Setup

Prerequisites: Rust toolchain (stable). Install via [rustup](https://rustup.rs/).

```sh
git clone https://github.com/kitsuyui/sxd_html_table.git
cd sxd_html_table
cargo build
```

This repository uses [lefthook](https://lefthook.dev/) to run the same checks as CI locally:

```sh
lefthook install
```

## Running Tests

```sh
cargo test
```

## Code Style

Format and lint before committing:

```sh
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
```

The pre-commit hook enforces both checks automatically once you install lefthook.

## Submitting Changes

1. Fork the repository and create a topic branch from `main`.
2. Make your changes and add tests for new behaviour.
3. Run `cargo fmt`, `cargo clippy`, and `cargo test` to verify everything passes.
4. Open a pull request against `main` and fill in the PR template.

## Reporting Bugs

Please use the [bug report template](.github/ISSUE_TEMPLATE/bug_report.md) when opening an issue.
Include a minimal reproducible example whenever possible.

## License

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed (MIT OR Apache-2.0) without any additional terms or conditions.
