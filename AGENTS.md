# Repository Guidance

This repository contains **macfmt**, a single-binary Rust CLI tool for formatting
MAC addresses in various formats. The main source code is located in `src/main.rs`
with 22 comprehensive unit tests covering all functionality and edge cases.

## Project Overview

macfmt reads MAC addresses from stdin, files, or interactive editor input and
outputs them in different formats:

- Standard format: `xx:xx:xx:xx:xx:xx` (default)
- Cisco format: `xxxx.xxxx.xxxx`
- Windows format: `xx-xx-xx-xx-xx-xx`
- Bare format: `xxxxxxxxxxxx`

The tool preserves original case by default and supports `--upper`/`--lower` flags
for case conversion.

## Development Workflow

When modifying Rust source code in this project:

1. **Format** the code using rustfmt: `cargo fmt`
2. **Lint** with clippy at the pedantic level: `cargo clippy -- -D warnings`
3. **Test** thoroughly using the comprehensive test suite: `cargo test`
4. **Build** release version to verify: `cargo build --release`
5. **Lint** Markdown files using markdownlint: `markdownlint-cli2 "**/*.md"`
6. **Commit** using [Conventional Commits](https://www.conventionalcommits.org/)
   specification

## Code Quality Standards

- Clippy pedantic lints are enforced (`#![deny(clippy::pedantic)]`)
- All 22 unit tests must pass
- Code must follow rustfmt formatting
- Functions should be documented with rustdoc comments
- Error messages should be user-friendly, not technical

Ensure all steps succeed before committing changes.
