//! Utility functions for parsing.

use super::ParseErrorType;
use std::time::Duration;
use terminal_size::{terminal_size, Width, Height};

/// Parse a string into a `Duration`. Supported suffixes: s, ms, us.
pub fn parse_duration(s: &str) -> Result<Duration, ParseErrorType> {
    // Split the number and the suffix
    let split_at = s
        .chars()
        .position(|c| !c.is_digit(10))
        .ok_or(ParseErrorType::MalformedInstruction)?;
    let (num, suffix) = s.split_at(split_at);
    // Parse the number
    let num = num
        .parse()
        .map_err(|_| ParseErrorType::MalformedInstruction)?;
    // Parse the suffix
    match suffix {
        "s" => Ok(Duration::from_secs(num)),
        "ms" => Ok(Duration::from_millis(num)),
        "us" => Ok(Duration::from_micros(num)),
        _ => Err(ParseErrorType::MalformedInstruction),
    }
}
/// Parse a loose string. If starting with `"`, will deserialize it. Else, return the string as it is.
pub fn parse_loose_string(s: &str) -> Result<String, ParseErrorType> {
    if s.starts_with('"') && s.ends_with('"') {
        serde_json::from_str(s).map_err(ParseErrorType::Json)
    } else {
        Ok(s.to_string())
    }
}
/// Detect terminal size, defaulting to 80x24 if it fails.
pub fn get_terminal_size() -> (u16, u16) {
    terminal_size()
        .map(|(Width(w), Height(h))| (w, h))
        .unwrap_or((80, 24))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration() {
        let durations = [
            ("1s", Duration::from_secs(1)),
            ("2ms", Duration::from_millis(2)),
            ("3us", Duration::from_micros(3)),
        ];
        for (input, expected) in durations.iter() {
            assert_eq!(parse_duration(input).unwrap(), *expected);
        }
    }

    #[test]
    fn loose_string() {
        let strings = [
            ("\"hello \"", "hello "),
            ("world", "world"),
            ("\" hello \\\"world \"", " hello \"world "),
        ];
        for (input, expected) in strings.iter() {
            assert_eq!(parse_loose_string(input).unwrap(), *expected);
        }
    }

    #[test]
    fn loose_string_error() {
        let strings = ["\"hello\" world\"", "\"hello\" world\" again\""];
        for input in strings.iter() {
            assert!(matches!(
                parse_loose_string(input).unwrap_err(),
                ParseErrorType::Json(_)
            ));
        }
    }
}
