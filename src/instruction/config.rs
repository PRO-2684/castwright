//! Module for parsing config instructions.

use super::{util, ParseError};
use std::time::Duration;

/// A configuration instruction, either persistent or temporary.
#[derive(Debug, PartialEq)]
pub enum ConfigInstruction {
    // Configuration that doesn't apply to instructions (metadata)
    /// Terminal width.
    Width(Option<usize>),
    /// Terminal height.
    Height(Option<usize>),
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
    /// Parse an optional integer, returning `None` if the string is `auto`.
    fn parse_optional_int(s: &str) -> Result<Option<usize>, ParseError> {
        if s == "auto" {
            Ok(None)
        } else {
            s.parse().map(Some).map_err(|_| ParseError::MalformedInstruction(None))
        }
    }
    /// Parse a line into a `ConfigInstruction`.
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        let s = s.trim();
        let mut iter = s.split_whitespace();
        let Some(first) = iter.next() else {
            return Err(ParseError::MalformedInstruction(None));
        };
        match first {
            "width" => {
                let width = iter.next().ok_or(ParseError::MalformedInstruction(None))?;
                Ok(Self::Width(
                    Self::parse_optional_int(width)?,
                ))
            }
            "height" => {
                let height = iter.next().ok_or(ParseError::MalformedInstruction(None))?;
                Ok(Self::Height(
                    Self::parse_optional_int(height)?,
                ))
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
                let idle = iter.next().ok_or(ParseError::MalformedInstruction(None))?;
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
                        _ => Err(ParseError::MalformedInstruction(None)),
                    }
                } else {
                    Ok(Self::Hidden(true)) // Default to true
                }
            }
            "delay" => {
                let delay = iter.next().ok_or(ParseError::MalformedInstruction(None))?;
                Ok(Self::Delay(util::parse_duration(delay)?))
            }
            _ => Err(ParseError::MalformedInstruction(None)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ConfigInstruction::{self, *},
        ParseError,
    };
    use std::time::Duration;

    #[test]
    fn config_instruction() {
        let instructions = [
            ("width 123", Width(Some(123))),
            ("height 456", Height(Some(456))),
            ("width auto", Width(None)),
            ("height auto", Height(None)),
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
        let malformed_instructions = ["invalid", "width", "hidden what", "delay", "delay 2"];
        for line in malformed_instructions.iter() {
            assert!(matches!(
                ConfigInstruction::parse(line).unwrap_err(),
                ParseError::MalformedInstruction(None)
            ));
        }
    }
}
