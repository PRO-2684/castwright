//! Utility functions for parsing.

use super::ParseErrorType;
use std::time::Duration;

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
/// Parse a `"`-wrapped string. If not wrapped, return the string as it is. Note that it is a rather loose implementation, disregarding any escape sequences.
pub fn parse_quoted_string(s: &str) -> String {
    // FIXME: Check for escape sequences
    if s.starts_with('"') && s.ends_with('"') {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
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
    fn quoted_string() {
        let strings = [
            ("\"hello \"", "hello "),
            ("world", "world"),
            ("\" hello world \"", " hello world "),
        ];
        for (input, expected) in strings.iter() {
            assert_eq!(parse_quoted_string(input), *expected);
        }
    }
}
