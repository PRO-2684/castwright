//! Module for parsing config instructions.

use super::{util, ParseError};
use std::time::Duration;

/// A configuration instruction, either persistent or temporary.
#[derive(Debug, PartialEq)]
pub enum ConfigInstruction {
    // Configuration that doesn't apply to instructions (metadata)
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

    // Configuration that applies to other instructions (directive)
    /// The shell prompt to use in the asciicast output.
    Prompt(String),
    /// The shell secondary prompt to use in the asciicast (for continuation lines).
    SecondaryPrompt(String),
    /// The string to signify a line split in a multiline command.
    LineSplit(String),
    /// Whether the command should be executed silently.
    Hidden(bool),
    /// Typing delay between characters in a command.
    Delay(Duration),
}

impl ConfigInstruction {
    /// Parse a positive integer, returning `0` if the string is `auto`.
    fn parse_auto_usize(s: &str) -> Result<usize, ParseError> {
        if s == "auto" {
            Ok(0)
        } else {
            s.parse().map_err(|_| ParseError::malformed_instruction())
        }
    }
    /// Parse a line into a `ConfigInstruction`.
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        let s = s.trim();
        let mut iter = s.split_whitespace();
        let Some(first) = iter.next() else {
            return Err(ParseError::malformed_instruction());
        };
        match first {
            "width" => {
                let width = iter.next().ok_or(ParseError::malformed_instruction())?;
                Ok(Self::Width(Self::parse_auto_usize(width)?))
            }
            "height" => {
                let height = iter.next().ok_or(ParseError::malformed_instruction())?;
                Ok(Self::Height(Self::parse_auto_usize(height)?))
            }
            "title" => {
                let title = util::parse_quoted_string(s[5..].trim());
                Ok(Self::Title(title))
            }
            "shell" => {
                let shell = util::parse_quoted_string(s[5..].trim());
                Ok(Self::Shell(shell))
            }
            "quit" => {
                let quit = util::parse_quoted_string(s[4..].trim());
                Ok(Self::Quit(quit))
            }
            "idle" => {
                let idle = iter.next().ok_or(ParseError::malformed_instruction())?;
                Ok(Self::Idle(util::parse_duration(idle)?))
            }
            "prompt" => {
                let prompt = util::parse_quoted_string(s[6..].trim());
                Ok(Self::Prompt(prompt))
            }
            "secondary-prompt" => {
                let prompt = util::parse_quoted_string(s[16..].trim());
                Ok(Self::SecondaryPrompt(prompt))
            }
            "line-split" => {
                let split = util::parse_quoted_string(s[10..].trim());
                Ok(Self::LineSplit(split))
            }
            "hidden" => {
                let hidden = iter.next();
                if let Some(word) = hidden {
                    match word {
                        "true" => Ok(Self::Hidden(true)),
                        "false" => Ok(Self::Hidden(false)),
                        _ => Err(ParseError::malformed_instruction()),
                    }
                } else {
                    Ok(Self::Hidden(true)) // Default to true
                }
            }
            "delay" => {
                let delay = iter.next().ok_or(ParseError::malformed_instruction())?;
                Ok(Self::Delay(util::parse_duration(delay)?))
            }
            _ => Err(ParseError::malformed_instruction()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ParseErrorType;

    use super::{
        ConfigInstruction::{self, *},
        ParseError,
    };
    use std::time::Duration;

    #[test]
    fn config_instruction() {
        let instructions = [
            ("width 123", Width(123)),
            ("height 456", Height(456)),
            ("width auto", Width(0)),
            ("height auto", Height(0)),
            ("width 0", Width(0)),
            ("height 0", Height(0)),
            (
                "title castwright demo",
                Title("castwright demo".to_string()),
            ),
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

    #[test]
    fn malformed_config_instruction() {
        let malformed_instructions = [
            "invalid",
            "width",
            "width -1",
            "width what",
            "hidden what",
            "delay",
            "delay 2",
        ];
        for line in malformed_instructions.iter() {
            assert!(matches!(
                ConfigInstruction::parse(line).unwrap_err(),
                ParseError {
                    error: ParseErrorType::MalformedInstruction,
                    line: 0
                }
            ));
        }
    }
}
