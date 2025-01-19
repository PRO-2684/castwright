//! Module for parsing instructions.

mod config;
use config::ConfigInstruction;

/// A single instruction
#[derive(Debug, PartialEq)]
enum Instruction {
    /// Persistent configuration instruction or metadata. (`@@`)
    PersistentConfig(ConfigInstruction),
    /// Temporary configuration instruction. (`@`)
    TemporaryConfig(ConfigInstruction),
    /// Print a string as it is. (`%`)
    Print(String),
    /// Marker. (`!`)
    Marker(String),
    /// Comment (`#`) or empty line.
    Empty,
    /// One-line shell command. (`$`)
    Command(String),
    /// Continuation of a multi-line shell command. (`>`)
    Continuation(String),
}

#[allow(dead_code)]
impl Instruction {
    /// Parse a line into an `Instruction`.
    fn parse(s: &str) -> Result<Self, ParseError> {
        let s = s.trim();
        let Some(first) = s.chars().next() else {
            return Ok(Self::Empty);
        };
        let trimmed = s[1..].trim().to_string();
        match first {
            '@' => {
                let Some(second) = s.chars().nth(1) else {
                    return Err(ParseError::InvalidInstruction);
                };
                match second {
                    '@' => Ok(Self::PersistentConfig(ConfigInstruction::parse(&s[2..])?)),
                    _ => Ok(Self::TemporaryConfig(ConfigInstruction::parse(&trimmed)?)),
                }
            },
            '%' => Ok(Self::Print(trimmed)),
            '!' => Ok(Self::Marker(trimmed)),
            '#' => Ok(Self::Empty),
            '$' => Ok(Self::Command(trimmed)),
            '>' => Ok(Self::Continuation(trimmed)),
            _ => Err(ParseError::InvalidInstruction),
        }
    }
}

/// Possible errors that can occur while parsing a `.cwsh` file.
#[derive(Debug, PartialEq)]
enum ParseError {
    // General parsing errors
    /// This line does not form a valid instruction.
    InvalidInstruction,
    /// Did not expect a continuation line, but got one.
    UnexpectedContinuation,
}

mod util {
    use std::time::Duration;
    use super::ParseError;
    /// Parse a string into a `Duration`. Supported suffixes: s, ms, us.
    pub fn parse_duration(s: &str) -> Result<Duration, ParseError> {
        // Split the number and the suffix
        let split_at = s.chars().position(|c| !c.is_digit(10)).ok_or(ParseError::InvalidInstruction)?;
        let (num, suffix) = s.split_at(split_at);
        // Parse the number
        let num = num.parse().map_err(|_| ParseError::InvalidInstruction)?;
        // Parse the suffix
        match suffix {
            "s" => Ok(Duration::from_secs(num)),
            "ms" => Ok(Duration::from_millis(num)),
            "us" => Ok(Duration::from_micros(num)),
            _ => Err(ParseError::InvalidInstruction),
        }
    }
    /// Parse a `"`-wrapped string. If not wrapped, return the string as it is. Note that it is a rather loose implementation, disregarding any escape sequences.
    pub fn parse_quoted_string(s: &str) -> String {
        // FIXME: Check for escape sequences
        if s.starts_with('"') && s.ends_with('"') {
            s[1..s.len()-1].to_string()
        } else {
            s.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_instruction_parse_with_space() {
        use Instruction::*;
        let instructions = [
            (" @@width 123", PersistentConfig(ConfigInstruction::Width(123))),
            (" @height 456", TemporaryConfig(ConfigInstruction::Height(456))),
            (" %print", Print("print".to_string())),
            (" !marker", Marker("marker".to_string())),
            (" #comment", Empty),
            (" $command", Command("command".to_string())),
            (" >continuation", Continuation("continuation".to_string())),
            ("@@ width 123", PersistentConfig(ConfigInstruction::Width(123))),
            ("@ height 456", TemporaryConfig(ConfigInstruction::Height(456))),
            ("% print", Print("print".to_string())),
            ("! marker", Marker("marker".to_string())),
            ("# comment", Empty),
            ("$ command", Command("command".to_string())),
            ("> continuation", Continuation("continuation".to_string())),
        ];
        for (input, expected) in instructions.iter() {
            assert_eq!(&Instruction::parse(input).unwrap(), expected);
        }
    }

    #[test]
    fn test_instruction_parse_without_space() {
        use Instruction::*;
        let instructions = [
            ("@@width 123", PersistentConfig(ConfigInstruction::Width(123))),
            ("@height 456", TemporaryConfig(ConfigInstruction::Height(456))),
            ("%print", Print("print".to_string())),
            ("!marker", Marker("marker".to_string())),
            ("#comment", Empty),
            ("$command", Command("command".to_string())),
            (">continuation", Continuation("continuation".to_string())),
        ];
        for (input, expected) in instructions.iter() {
            assert_eq!(&Instruction::parse(input).unwrap(), expected);
        }
    }

    #[test]
    fn test_instruction_parse_empty() {
        let empty_lines = [
            "",
            " ",
            "  ",
            "\t",
            "\t ",
            " \t",
            "\n",
        ];
        for line in empty_lines.iter() {
            assert_eq!(&Instruction::parse(line).unwrap(), &Instruction::Empty);
        }
    }

    #[test]
    fn test_instruction_parse_invalid() {
        let invalid_lines = [
            "invalid",
            "&",
            "~",
        ];
        for line in invalid_lines.iter() {
            assert_eq!(Instruction::parse(line).unwrap_err(), ParseError::InvalidInstruction);
        }
    }

    #[test]
    fn test_parse_duration() {
        let durations = [
            ("1s", Duration::from_secs(1)),
            ("2ms", Duration::from_millis(2)),
            ("3us", Duration::from_micros(3)),
        ];
        for (input, expected) in durations.iter() {
            assert_eq!(util::parse_duration(input).unwrap(), *expected);
        }
    }
}
