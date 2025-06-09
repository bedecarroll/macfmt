# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with
code in this repository.

## Project Overview

macfmt is a command-line tool for formatting MAC addresses in various formats.
It's a single-binary Rust application that reads MAC addresses from stdin or
files and outputs them in different formats (standard, Cisco, Windows, bare)
with optional case conversion.

## Core Architecture

The application is structured around a central `MacAddress` struct that:

- Stores the 6-byte MAC address value
- Preserves original case information for each hexadecimal character
- Provides formatting methods for different output formats

Key components:

- **CLI parsing**: Uses clap with derive macros for command-line interface
- **MAC detection**: Regex-based pattern matching for multiple input formats
- **Case handling**: Preserves original case by default, with --upper/--lower
override flags
- **Input processing**: Handles both stdin and file input with proper error
handling

## Development Commands

### Building and Testing

```bash
# Build debug version
cargo build

# Build release version  
cargo build --release

# Run all tests (22 comprehensive unit tests)
cargo test

# Run a specific test
cargo test test_mac_address_parsing_colon_format

# Run tests with output
cargo test -- --nocapture
```

### Code Quality

```bash
# Run clippy with pedantic lints (enforced in code)
cargo clippy -- -D warnings

# Check formatting
cargo fmt --check

# Apply formatting
cargo fmt
```

### Running the Application

```bash
# Basic usage (defaults to standard format)
echo "aa:bb:cc:dd:ee:ff" | cargo run

# With specific format
echo "aa:bb:cc:dd:ee:ff" | cargo run -- cisco

# With flags
echo "aa:bb:cc:dd:ee:ff" | cargo run -- --upper cisco

# From file
cargo run -- input.txt cisco

# Enable logging
RUST_LOG=debug cargo run -- cisco
```

## Key Implementation Details

### MAC Address Parsing

The `MacAddress::new()` method:

1. Strips all separators (-, :, ., space) from input
2. Validates 12-character hex length
3. Records original case for each character
4. Parses hex bytes with detailed error messages

### Format Detection

Uses three regex patterns to detect MAC addresses in text:

- Colon/dash separated: `([0-9a-fA-F]{2}[:-]){5}[0-9a-fA-F]{2}`
- Cisco dot format: `([0-9a-fA-F]{4}\.){2}[0-9a-fA-F]{4}`
- Bare format: `[0-9a-fA-F]{12}`

### Case Preservation Logic

- `force_case: Option<bool>` parameter controls case conversion
- `Some(true)` = uppercase, `Some(false)` = lowercase, `None` = preserve original
- Original case stored as `Vec<bool>` where `true` = uppercase character

### Error Handling Philosophy

- Clean user-facing error messages via `eprintln!`
- Detailed logging only when `RUST_LOG` is enabled
- Specific error types for file not found, permission denied, etc.
- Exit with code 1 on any error

## Testing Strategy

The codebase has 22 unit tests covering:

- All input format parsing (colon, dash, dot, bare)
- All output format generation
- Case preservation and conversion
- Edge cases (all zeros, all Fs)
- Error conditions (invalid length, invalid hex)
- Multiple MAC address processing
- Regex pattern validation

## Dependencies

- `clap` (v4.0+): Command-line argument parsing with derive features
- `regex` (v1.5+): MAC address pattern matching
- `log` (v0.4+): Structured logging
- `env_logger` (v0.11+): Environment-based log configuration

## Code Style Notes

- Clippy pedantic lints are enforced (`#![deny(clippy::pedantic)]`)
- Uses modern Rust patterns (if-let, match expressions)
- Format strings use inline arguments (`format!("{variable}")` not
`format!("{}", variable)`)
- Functions are well-documented with rustdoc comments
- Error messages are user-friendly, not technical
