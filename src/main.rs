#![deny(clippy::pedantic)]

use atty::Stream;
use clap::{Parser, Subcommand};
use edit::edit;
use log::{debug, info, warn};
use regex::Regex;
use std::fs;
use std::io::{self, Read};
use which::which;

#[derive(Parser)]
#[command(name = "macfmt")]
#[command(about = "A tool to format MAC addresses in various formats")]
#[command(
    long_about = "A tool to format MAC addresses in various formats. Uses standard format (xx:xx:xx:xx:xx:xx) by default when no subcommand is specified."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(help = "Input file (if not provided, reads from stdin)")]
    file: Option<String>,

    #[arg(long, help = "Convert output to lowercase")]
    lower: bool,

    #[arg(long, help = "Convert output to uppercase")]
    upper: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Format MAC addresses in Cisco format (xxxx.xxxx.xxxx)")]
    Cisco,
    #[command(about = "Format MAC addresses in standard format (xx:xx:xx:xx:xx:xx) [default]")]
    Standard,
    #[command(about = "Format MAC addresses in Windows format (xx-xx-xx-xx-xx-xx)")]
    Windows,
    #[command(about = "Format MAC addresses in bare format (xxxxxxxxxxxx)")]
    Bare,
}

/// Represents a MAC address with its 6-byte value and original case information.
///
/// This structure stores both the parsed MAC address bytes and information about
/// the original case (upper/lower) of each hexadecimal character to preserve
/// formatting when no case conversion is requested.
struct MacAddress {
    /// The 6 bytes of the MAC address
    bytes: [u8; 6],
    /// Case information for each hex character (true = uppercase, false = lowercase)
    original_case: Vec<bool>,
}

impl MacAddress {
    /// Creates a new `MacAddress` from a string input.
    ///
    /// Accepts various MAC address formats:
    /// - Colon-separated: "aa:bb:cc:dd:ee:ff"
    /// - Dash-separated: "aa-bb-cc-dd-ee-ff"
    /// - Dot-separated: "aabb.ccdd.eeff"
    /// - Bare format: "aabbccddeeff"
    ///
    /// # Arguments
    /// * `input` - The MAC address string to parse
    ///
    /// # Returns
    /// * `Ok(MacAddress)` if parsing succeeds
    /// * `Err(String)` with error description if parsing fails
    ///
    /// # Examples
    /// ```
    /// let mac = MacAddress::new("aa:bb:cc:dd:ee:ff").unwrap();
    /// let mac = MacAddress::new("aabb.ccdd.eeff").unwrap();
    /// ```
    fn new(input: &str) -> Result<Self, String> {
        debug!("Parsing MAC address: {input}");
        let cleaned = input.replace(['-', ':', '.', ' '], "");

        if cleaned.len() != 12 {
            return Err(format!("Invalid MAC address length: {input}"));
        }

        let mut bytes = [0u8; 6];
        let mut original_case = Vec::new();

        for (i, chunk) in cleaned.as_bytes().chunks(2).enumerate() {
            if i >= 6 {
                return Err(format!("Invalid MAC address: {input}"));
            }
            let hex_str = std::str::from_utf8(chunk)
                .map_err(|_| format!("Invalid characters in MAC address: {input}"))?;

            for &byte in chunk {
                original_case.push(byte.is_ascii_uppercase());
            }

            bytes[i] = u8::from_str_radix(hex_str, 16)
                .map_err(|_| format!("Invalid hex in MAC address: {input}"))?;
        }

        Ok(MacAddress {
            bytes,
            original_case,
        })
    }

    /// Formats the MAC address bytes as hexadecimal characters with case handling.
    ///
    /// # Arguments
    /// * `force_case` - Optional case conversion:
    ///   - `Some(true)` forces uppercase
    ///   - `Some(false)` forces lowercase  
    ///   - `None` preserves original case
    ///
    /// # Returns
    /// A vector of 12 hexadecimal characters representing the MAC address
    fn format_hex_chars(&self, force_case: Option<bool>) -> Vec<char> {
        let mut chars = Vec::new();
        for (i, &byte) in self.bytes.iter().enumerate() {
            let hex = format!("{byte:02x}");
            for (j, ch) in hex.chars().enumerate() {
                let char_index = i * 2 + j;
                let should_be_upper = match force_case {
                    Some(true) => true,
                    Some(false) => false,
                    None => char_index < self.original_case.len() && self.original_case[char_index],
                };
                chars.push(if should_be_upper {
                    ch.to_ascii_uppercase()
                } else {
                    ch
                });
            }
        }
        chars
    }

    /// Formats the MAC address in Cisco format (xxxx.xxxx.xxxx).
    ///
    /// # Arguments
    /// * `force_case` - Optional case conversion
    ///
    /// # Returns
    /// MAC address formatted as "xxxx.xxxx.xxxx"
    fn to_cisco(&self, force_case: Option<bool>) -> String {
        let chars = self.format_hex_chars(force_case);
        format!(
            "{}{}{}{}.{}{}{}{}.{}{}{}{}",
            chars[0],
            chars[1],
            chars[2],
            chars[3],
            chars[4],
            chars[5],
            chars[6],
            chars[7],
            chars[8],
            chars[9],
            chars[10],
            chars[11]
        )
    }

    /// Formats the MAC address in standard format (xx:xx:xx:xx:xx:xx).
    ///
    /// # Arguments
    /// * `force_case` - Optional case conversion
    ///
    /// # Returns
    /// MAC address formatted as "xx:xx:xx:xx:xx:xx"
    fn to_standard(&self, force_case: Option<bool>) -> String {
        let chars = self.format_hex_chars(force_case);
        format!(
            "{}{}:{}{}:{}{}:{}{}:{}{}:{}{}",
            chars[0],
            chars[1],
            chars[2],
            chars[3],
            chars[4],
            chars[5],
            chars[6],
            chars[7],
            chars[8],
            chars[9],
            chars[10],
            chars[11]
        )
    }

    /// Formats the MAC address in Windows format (xx-xx-xx-xx-xx-xx).
    ///
    /// # Arguments
    /// * `force_case` - Optional case conversion
    ///
    /// # Returns
    /// MAC address formatted as "xx-xx-xx-xx-xx-xx"
    fn to_windows(&self, force_case: Option<bool>) -> String {
        let chars = self.format_hex_chars(force_case);
        format!(
            "{}{}-{}{}-{}{}-{}{}-{}{}-{}{}",
            chars[0],
            chars[1],
            chars[2],
            chars[3],
            chars[4],
            chars[5],
            chars[6],
            chars[7],
            chars[8],
            chars[9],
            chars[10],
            chars[11]
        )
    }

    /// Formats the MAC address in bare format (xxxxxxxxxxxx).
    ///
    /// # Arguments
    /// * `force_case` - Optional case conversion
    ///
    /// # Returns
    /// MAC address formatted as "xxxxxxxxxxxx"
    fn to_bare(&self, force_case: Option<bool>) -> String {
        let chars = self.format_hex_chars(force_case);
        chars.into_iter().collect()
    }
}

/// Finds all MAC addresses in the given text using various regex patterns.
///
/// Supports the following formats:
/// - Colon-separated: xx:xx:xx:xx:xx:xx
/// - Dash-separated: xx-xx-xx-xx-xx-xx  
/// - Dot-separated (Cisco): xxxx.xxxx.xxxx
/// - Bare format: xxxxxxxxxxxx
///
/// # Arguments
/// * `text` - The input text to search for MAC addresses
///
/// # Returns
/// A vector of MAC address strings found in the input
fn find_mac_addresses(text: &str) -> Vec<String> {
    debug!(
        "Searching for MAC addresses in {} bytes of text",
        text.len()
    );
    let patterns = [
        r"([0-9a-fA-F]{2}[:-]){5}[0-9a-fA-F]{2}", // xx:xx:xx:xx:xx:xx or xx-xx-xx-xx-xx-xx
        r"([0-9a-fA-F]{4}\.){2}[0-9a-fA-F]{4}",   // xxxx.xxxx.xxxx
        r"[0-9a-fA-F]{12}",                       // xxxxxxxxxxxx
    ];

    let mut addresses = Vec::new();

    for (i, pattern) in patterns.iter().enumerate() {
        let re = Regex::new(pattern).unwrap();
        let matches: Vec<_> = re.find_iter(text).collect();
        debug!("Pattern {} found {} matches", i, matches.len());
        for mat in matches {
            addresses.push(mat.as_str().to_string());
        }
    }

    info!("Found {} MAC addresses total", addresses.len());
    addresses
}

/// Processes the input text, finds MAC addresses, and formats them according to the specified format.
///
/// # Arguments
/// * `input` - The input text to process
/// * `format_fn` - The formatting function to apply to each MAC address
/// * `force_case` - Optional case conversion: Some(true) for upper, Some(false) for lower, None to preserve
///
/// # Returns
/// Ok(()) if processing succeeds, Err with error message if no MAC addresses found or parsing fails
fn process_input(
    input: &str,
    format_fn: fn(&MacAddress, Option<bool>) -> String,
    force_case: Option<bool>,
) -> Result<(), String> {
    debug!("Processing input with {} characters", input.len());
    let mac_strings = find_mac_addresses(input);

    if mac_strings.is_empty() {
        return Err("No MAC addresses found in input".to_string());
    }

    for mac_str in mac_strings {
        debug!("Processing MAC address: {mac_str}");
        match MacAddress::new(&mac_str) {
            Ok(mac) => {
                let formatted = format_fn(&mac, force_case);
                debug!("Formatted '{mac_str}' as '{formatted}'");
                println!("{formatted}");
            }
            Err(e) => {
                eprintln!("Error parsing '{mac_str}': {e}");
            }
        }
    }

    Ok(())
}

/// Gathers input interactively using an editor when running in a TTY environment.
///
/// This function checks for an available editor (from EDITOR environment variable)
/// and launches it to allow the user to input text. If no editor is available or
/// configured, it falls back to reading from stdin.
///
/// # Returns
/// * `Ok(String)` with the input text from the editor or stdin
/// * `Err` if there's an error reading input
fn gather_interactive_input() -> io::Result<String> {
    let mut input = String::new();
    let editor_env = std::env::var("EDITOR").unwrap_or_default();

    if editor_env.is_empty() || which(&editor_env).is_err() {
        if !editor_env.is_empty() {
            warn!("Editor not found. EDITOR={editor_env:?}");
        }
        eprintln!("Input text. End input with Ctrl-d or EOF on a new line.");
        io::stdin().read_to_string(&mut input)?;
    } else {
        debug!("Opening $EDITOR ({editor_env:?}) for input. Save and quit to continue.");
        input = edit("").map_err(io::Error::other)?;
    }

    Ok(input)
}

fn main() {
    env_logger::init();

    let cli = Cli::parse();
    info!(
        "Starting macfmt with arguments: {:?}",
        std::env::args().collect::<Vec<_>>()
    );

    let input = if let Some(file_path) = &cli.file {
        info!("Reading input from file: {file_path}");
        match fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(e) => {
                match e.kind() {
                    std::io::ErrorKind::NotFound => {
                        eprintln!("Error: File not found: {file_path}");
                    }
                    std::io::ErrorKind::PermissionDenied => {
                        eprintln!("Error: Permission denied reading file: {file_path}");
                    }
                    _ => {
                        eprintln!("Error: Failed to read file '{file_path}': {e}");
                    }
                }
                std::process::exit(1);
            }
        }
    } else if atty::is(Stream::Stdin) {
        debug!("Detected interactive terminal, launching editor for input");
        match gather_interactive_input() {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error: Failed to read input: {e}");
                std::process::exit(1);
            }
        }
    } else {
        debug!("Reading input from stdin");
        let mut buffer = String::new();
        match io::stdin().read_to_string(&mut buffer) {
            Ok(_) => buffer,
            Err(e) => {
                eprintln!("Error: Failed to read from stdin: {e}");
                std::process::exit(1);
            }
        }
    };

    if cli.lower && cli.upper {
        eprintln!("Error: Cannot specify both --lower and --upper flags");
        std::process::exit(1);
    }

    let force_case = if cli.upper {
        Some(true)
    } else if cli.lower {
        Some(false)
    } else {
        None
    };

    let format_fn: fn(&MacAddress, Option<bool>) -> String = match cli.command {
        Some(Commands::Cisco) => MacAddress::to_cisco,
        Some(Commands::Standard) | None => MacAddress::to_standard,
        Some(Commands::Windows) => MacAddress::to_windows,
        Some(Commands::Bare) => MacAddress::to_bare,
    };

    if let Err(e) = process_input(&input, format_fn, force_case) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }

    info!("Processing completed successfully");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mac_address_parsing_colon_format() {
        let mac = MacAddress::new("aa:bb:cc:dd:ee:ff").unwrap();
        assert_eq!(mac.bytes, [0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]);
    }

    #[test]
    fn test_mac_address_parsing_dash_format() {
        let mac = MacAddress::new("aa-bb-cc-dd-ee-ff").unwrap();
        assert_eq!(mac.bytes, [0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]);
    }

    #[test]
    fn test_mac_address_parsing_dot_format() {
        let mac = MacAddress::new("aabb.ccdd.eeff").unwrap();
        assert_eq!(mac.bytes, [0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]);
    }

    #[test]
    fn test_mac_address_parsing_bare_format() {
        let mac = MacAddress::new("aabbccddeeff").unwrap();
        assert_eq!(mac.bytes, [0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]);
    }

    #[test]
    fn test_mac_address_parsing_case_preservation() {
        let mac = MacAddress::new("AA:bb:CC:dd:EE:ff").unwrap();
        assert_eq!(mac.bytes, [0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]);
        assert_eq!(
            mac.original_case,
            vec![
                true, true, false, false, true, true, false, false, true, true, false, false
            ]
        );
    }

    #[test]
    fn test_mac_address_parsing_invalid_length() {
        assert!(MacAddress::new("aa:bb:cc:dd:ee").is_err());
        assert!(MacAddress::new("aa:bb:cc:dd:ee:ff:gg").is_err());
    }

    #[test]
    fn test_mac_address_parsing_invalid_hex() {
        assert!(MacAddress::new("zz:bb:cc:dd:ee:ff").is_err());
        assert!(MacAddress::new("aa:bb:cc:dd:ee:gg").is_err());
    }

    #[test]
    fn test_to_cisco_format() {
        let mac = MacAddress::new("aabbccddeeff").unwrap();
        assert_eq!(mac.to_cisco(None), "aabb.ccdd.eeff");
    }

    #[test]
    fn test_to_standard_format() {
        let mac = MacAddress::new("aabbccddeeff").unwrap();
        assert_eq!(mac.to_standard(None), "aa:bb:cc:dd:ee:ff");
    }

    #[test]
    fn test_to_windows_format() {
        let mac = MacAddress::new("aabbccddeeff").unwrap();
        assert_eq!(mac.to_windows(None), "aa-bb-cc-dd-ee-ff");
    }

    #[test]
    fn test_to_bare_format() {
        let mac = MacAddress::new("aa:bb:cc:dd:ee:ff").unwrap();
        assert_eq!(mac.to_bare(None), "aabbccddeeff");
    }

    #[test]
    fn test_case_preservation() {
        let mac = MacAddress::new("AA:bb:CC:dd:EE:ff").unwrap();
        assert_eq!(mac.to_standard(None), "AA:bb:CC:dd:EE:ff");
        assert_eq!(mac.to_cisco(None), "AAbb.CCdd.EEff");
    }

    #[test]
    fn test_force_lowercase() {
        let mac = MacAddress::new("AA:BB:CC:DD:EE:FF").unwrap();
        assert_eq!(mac.to_standard(Some(false)), "aa:bb:cc:dd:ee:ff");
        assert_eq!(mac.to_cisco(Some(false)), "aabb.ccdd.eeff");
    }

    #[test]
    fn test_force_uppercase() {
        let mac = MacAddress::new("aa:bb:cc:dd:ee:ff").unwrap();
        assert_eq!(mac.to_standard(Some(true)), "AA:BB:CC:DD:EE:FF");
        assert_eq!(mac.to_cisco(Some(true)), "AABB.CCDD.EEFF");
    }

    #[test]
    fn test_find_mac_addresses_multiple_formats() {
        let text = "Device 1: aa:bb:cc:dd:ee:ff\nDevice 2: 1122.3344.5566\nDevice 3: aabbccddeeff";
        let addresses = find_mac_addresses(text);
        assert_eq!(addresses.len(), 3);
        assert!(addresses.contains(&"aa:bb:cc:dd:ee:ff".to_string()));
        assert!(addresses.contains(&"1122.3344.5566".to_string()));
        assert!(addresses.contains(&"aabbccddeeff".to_string()));
    }

    #[test]
    fn test_find_mac_addresses_no_matches() {
        let text = "No MAC addresses here!";
        let addresses = find_mac_addresses(text);
        assert_eq!(addresses.len(), 0);
    }

    #[test]
    fn test_find_mac_addresses_mixed_case() {
        let text = "MAC: AA:BB:CC:DD:EE:FF and also aa:bb:cc:dd:ee:ff";
        let addresses = find_mac_addresses(text);
        assert_eq!(addresses.len(), 2);
    }

    #[test]
    fn test_process_input_no_mac_addresses() {
        let result = process_input("No MACs here", MacAddress::to_standard, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No MAC addresses found"));
    }

    #[test]
    fn test_edge_case_all_zeros() {
        let mac = MacAddress::new("00:00:00:00:00:00").unwrap();
        assert_eq!(mac.to_standard(None), "00:00:00:00:00:00");
        assert_eq!(mac.to_cisco(None), "0000.0000.0000");
    }

    #[test]
    fn test_edge_case_all_fs() {
        let mac = MacAddress::new("ff:ff:ff:ff:ff:ff").unwrap();
        assert_eq!(mac.to_standard(None), "ff:ff:ff:ff:ff:ff");
        assert_eq!(mac.to_cisco(None), "ffff.ffff.ffff");
    }

    #[test]
    fn test_mixed_separators_in_single_mac() {
        // The current implementation actually handles mixed separators by stripping all
        // This test verifies that mixed separators are parsed correctly
        let mac = MacAddress::new("aa:bb-cc.dd ee:ff").unwrap();
        assert_eq!(mac.bytes, [0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]);
    }

    #[test]
    fn test_regex_patterns() {
        let text1 = "aa:bb:cc:dd:ee:ff";
        let text2 = "aa-bb-cc-dd-ee-ff";
        let text3 = "aabb.ccdd.eeff";
        let text4 = "aabbccddeeff";

        assert_eq!(find_mac_addresses(text1).len(), 1);
        assert_eq!(find_mac_addresses(text2).len(), 1);
        assert_eq!(find_mac_addresses(text3).len(), 1);
        assert_eq!(find_mac_addresses(text4).len(), 1);
    }
}
