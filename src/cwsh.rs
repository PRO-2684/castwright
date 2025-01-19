//! Module for parsing `.cwsh` files.

use std::time::Duration;

/// Represents a single line of instruction in a `.cwsh` file.
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

/// A configuration instruction, either persistent or temporary.
#[derive(Debug, PartialEq)]
enum ConfigInstruction {
    // Configuration that doesn't apply to commands (metadata)
    /// Terminal width.
    Width(usize),
    /// Terminal height.
    Height(usize),
    /// Title of the asciicast.
    Title(String),
    /// The shell to use.
    Shell(String),
    /// The quit command.
    Quit(String),
    /// Idle time limit.
    Idle(Duration),

    // Configuration that applies to commands
    /// The shell prompt to use in the asciicast output.
    Prompt(String),
    /// The shell secondary prompt to use in the asciicast (for continuation lines).
    SecondaryPrompt(String),
    /// The string to signify a line split in a multiline command.
    LineSplit(String),
    /// Whether the command should be executed silently.
    Hidden(bool),
    /// Delay between characters in a command.
    Delay(Duration),
}

impl ConfigInstruction {
    /// Parse a line into a `ConfigInstruction`.
    fn parse(s: &str) -> Result<Self, ParseError> {
        let s = s.trim();
        let mut iter = s.split_whitespace();
        let Some(first) = iter.next() else {
            return Err(ParseError::InvalidInstruction);
        };
        match first {
            "width" => {
                let width = iter.next().ok_or(ParseError::InvalidInstruction)?;
                Ok(Self::Width(width.parse().map_err(|_| ParseError::InvalidInstruction)?))
            },
            "height" => {
                let height = iter.next().ok_or(ParseError::InvalidInstruction)?;
                Ok(Self::Height(height.parse().map_err(|_| ParseError::InvalidInstruction)?))
            },
            "title" => {
                let title = util::parse_quoted_string(s[5..].trim());
                Ok(Self::Title(title))
            },
            "shell" => {
                let shell = util::parse_quoted_string(s[5..].trim());
                Ok(Self::Shell(shell))
            },
            "quit" => {
                let quit = util::parse_quoted_string(s[4..].trim());
                Ok(Self::Quit(quit))
            },
            "idle" => {
                let idle = iter.next().ok_or(ParseError::InvalidInstruction)?;
                Ok(Self::Idle(util::parse_duration(idle)?))
            },
            "prompt" => {
                let prompt = util::parse_quoted_string(s[6..].trim());
                Ok(Self::Prompt(prompt))
            },
            "secondary-prompt" => {
                let prompt = util::parse_quoted_string(s[16..].trim());
                Ok(Self::SecondaryPrompt(prompt))
            },
            "line-split" => {
                let split = util::parse_quoted_string(s[10..].trim());
                Ok(Self::LineSplit(split))
            },
            "hidden" => {
                let hidden = iter.next();
                if let Some(word) = hidden {
                    match word {
                        "true" => Ok(Self::Hidden(true)),
                        "false" => Ok(Self::Hidden(false)),
                        _ => Err(ParseError::InvalidInstruction),
                    }
                } else {
                    Ok(Self::Hidden(true)) // Default to true
                }
            },
            "delay" => {
                let delay = iter.next().ok_or(ParseError::InvalidInstruction)?;
                Ok(Self::Delay(util::parse_duration(delay)?))
            },
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

    #[test]
    fn test_parse_config_instruction() {
        use ConfigInstruction::*;
        let instructions = [
            ("width 123", Width(123)),
            ("height 456", Height(456)),
            ("title castwright demo", Title("castwright demo".to_string())),
            ("shell bash ", Shell("bash".to_string())),
            ("quit \"exit \"", Quit("exit ".to_string())),
            ("idle 1s", Idle(Duration::from_secs(1))),
            ("prompt \"$ \"", Prompt("$ ".to_string())),
            ("secondary-prompt \"> \"", SecondaryPrompt("> ".to_string())),
            ("line-split \\", LineSplit("\\".to_string())),
            ("hidden true", Hidden(true)),
            ("hidden false", Hidden(false)),
            ("delay 2ms", Delay(Duration::from_millis(2))),
        ];
        for (input, expected) in instructions.iter() {
            assert_eq!(ConfigInstruction::parse(input).unwrap(), *expected);
        }
    }
}
