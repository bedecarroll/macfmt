# macfmt

A command-line tool for formatting MAC addresses in various formats. The tool
can read from stdin or files, detect MAC addresses in different formats, and
output them in your preferred format while preserving or converting case.

## Features

- **Multiple Input Formats**: Supports colon-separated (`aa:bb:cc:dd:ee:ff`),
dash-separated (`aa-bb-cc-dd-ee-ff`), dot-separated (`aabb.ccdd.eeff`), and
bare (`aabbccddeeff`) formats
- **Multiple Output Formats**: Convert to standard, Cisco, Windows, or bare
formats
- **Case Preservation**: Preserves original case by default
- **Case Conversion**: Force uppercase or lowercase output with `--upper` and
`--lower` flags
- **Flexible Input**: Read from stdin or specify a file
- **Default Format**: Uses standard format (colon-separated) when no subcommand
is specified

## Installation

### Download Pre-built Binary

Download the latest release for your platform:

**Linux (x86_64):**

```bash
curl -L https://github.com/bedecarroll/macfmt/releases/latest/download/macfmt-linux-x86_64 -o macfmt
chmod +x macfmt
sudo mv macfmt /usr/local/bin/
```

**Linux (musl):**

```bash
curl -L https://github.com/bedecarroll/macfmt/releases/latest/download/macfmt-linux-x86_64-musl -o macfmt
chmod +x macfmt
sudo mv macfmt /usr/local/bin/
```

**macOS (Intel):**

```bash
curl -L https://github.com/bedecarroll/macfmt/releases/latest/download/macfmt-macos-x86_64 -o macfmt
chmod +x macfmt
sudo mv macfmt /usr/local/bin/
```

**macOS (Apple Silicon):**

```bash
curl -L https://github.com/bedecarroll/macfmt/releases/latest/download/macfmt-macos-aarch64 -o macfmt
chmod +x macfmt
sudo mv macfmt /usr/local/bin/
```

**Windows:**
Download `macfmt-windows-x86_64.exe` from the [latest
release](https://github.com/bedecarroll/macfmt/releases/latest) and run
directly.

### Build from Source

```bash
cargo build --release
```

## Usage

### Basic Usage

```bash
# Default format (standard) from stdin
echo "aabbccddeeff" | macfmt
# Output: aa:bb:cc:dd:ee:ff

# From file
macfmt input.txt

# Specific format
echo "aa:bb:cc:dd:ee:ff" | macfmt cisco
# Output: aabb.ccdd.eeff
```

### Output Formats

#### Standard Format (default)

```bash
echo "aabbccddeeff" | macfmt standard
# Output: aa:bb:cc:dd:ee:ff
```

#### Cisco Format

```bash
echo "aa:bb:cc:dd:ee:ff" | macfmt cisco
# Output: aabb.ccdd.eeff
```

#### Windows Format

```bash
echo "aabbccddeeff" | macfmt windows
# Output: aa-bb-cc-dd-ee-ff
```

#### Bare Format

```bash
echo "aa:bb:cc:dd:ee:ff" | macfmt bare
# Output: aabbccddeeff
```

### Case Conversion

#### Preserve Original Case (default)

```bash
echo "AA:bb:CC:dd:EE:ff" | macfmt
# Output: AA:bb:CC:dd:EE:ff
```

#### Force Lowercase

```bash
echo "AA:BB:CC:DD:EE:FF" | macfmt --lower cisco
# Output: aabb.ccdd.eeff
```

#### Force Uppercase

```bash
echo "aa:bb:cc:dd:ee:ff" | macfmt --upper standard
# Output: AA:BB:CC:DD:EE:FF
```

### Processing Multiple MAC Addresses

The tool can process multiple MAC addresses from the same input:

```bash
echo "Device 1: aa:bb:cc:dd:ee:ff
Device 2: 1122.3344.5566
Device 3: aabbccddeeff" | macfmt cisco
# Output:
# aabb.ccdd.eeff
# 1122.3344.5566
# aabb.ccdd.eeff
```

### Input from File

```bash
# Create a file with MAC addresses
echo -e "Router: aa:bb:cc:dd:ee:ff\nSwitch: 1122.3344.5566" > network.txt

# Process the file
macfmt cisco network.txt
# Output:
# aabb.ccdd.eeff
# 1122.3344.5566
```

## Supported MAC Address Formats

The tool automatically detects these input formats:

- **Colon-separated**: `aa:bb:cc:dd:ee:ff`
- **Dash-separated**: `aa-bb-cc-dd-ee-ff`
- **Dot-separated (Cisco)**: `aabb.ccdd.eeff`
- **Bare format**: `aabbccddeeff`

All formats support mixed case (uppercase and lowercase hexadecimal characters).

## Command Reference

```bash
macfmt [OPTIONS] [SUBCOMMAND] [FILE]

SUBCOMMANDS:
    cisco       Format MAC addresses in Cisco format (xxxx.xxxx.xxxx)
    standard    Format MAC addresses in standard format (xx:xx:xx:xx:xx:xx)
    windows     Format MAC addresses in Windows format (xx-xx-xx-xx-xx-xx)
    bare        Format MAC addresses in bare format (xxxxxxxxxxxx)

OPTIONS:
    --lower     Convert output to lowercase
    --upper     Convert output to uppercase
    -h, --help  Print help information

ARGUMENTS:
    [FILE]      Input file (if not provided, reads from stdin)
```

## Logging

The tool supports logging via the `RUST_LOG` environment variable:

```bash
# Enable debug logging
RUST_LOG=debug echo "aa:bb:cc:dd:ee:ff" | macfmt

# Enable info logging
RUST_LOG=info echo "aa:bb:cc:dd:ee:ff" | macfmt cisco
```

Log levels: `error`, `warn`, `info`, `debug`, `trace`

## Error Handling

The tool provides helpful error messages for common issues:

- Invalid MAC address formats
- Invalid hexadecimal characters
- Incorrect MAC address length
- No MAC addresses found in input
- File not found or read errors

Example:

```bash
echo "invalid" | macfmt
# Error: No MAC addresses found in input
```

## Examples

### Network Device Configuration

```bash
# macfmt and format MAC addresses from device output
show interfaces | macfmt cisco

# Convert existing config to different format
cat cisco-config.txt | macfmt windows
```

### Log Processing

```bash
# Process network logs
tail -f /var/log/network.log | macfmt --upper standard
```

### Batch Processing

```bash
# Process multiple files
for file in *.log; do
    echo "=== $file ==="
    macfmt cisco "$file"
done
```

## Development

### Running Tests

```bash
cargo test
```

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release
```

The tool includes comprehensive unit tests covering:

- MAC address parsing for all input formats
- Output formatting for all target formats
- Case preservation and conversion
- Error handling and edge cases
- Regex pattern matching

## License

This project is licensed under the MIT License.
