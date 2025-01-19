//! Module for parsing config instructions.

use super::{util, ParseError};
use std::time::Duration;

/// A configuration instruction, either persistent or temporary.
#[derive(Debug, PartialEq)]
pub enum ConfigInstruction {
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
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        let s = s.trim();
        let mut iter = s.split_whitespace();
        let Some(first) = iter.next() else {
            return Err(ParseError::InvalidInstruction);
        };
        match first {
            "width" => {
                let width = iter.next().ok_or(ParseError::InvalidInstruction)?;
                Ok(Self::Width(
                    width.parse().map_err(|_| ParseError::InvalidInstruction)?,
                ))
            }
            "height" => {
                let height = iter.next().ok_or(ParseError::InvalidInstruction)?;
                Ok(Self::Height(
                    height.parse().map_err(|_| ParseError::InvalidInstruction)?,
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
                let idle = iter.next().ok_or(ParseError::InvalidInstruction)?;
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
                        _ => Err(ParseError::InvalidInstruction),
                    }
                } else {
                    Ok(Self::Hidden(true)) // Default to true
                }
            }
            "delay" => {
                let delay = iter.next().ok_or(ParseError::InvalidInstruction)?;
                Ok(Self::Delay(util::parse_duration(delay)?))
            }
            _ => Err(ParseError::InvalidInstruction),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ConfigInstruction::{self, *};
    use std::time::Duration;

    #[test]
    fn test_parse_config_instruction() {
        let instructions = [
            ("width 123", Width(123)),
            ("height 456", Height(456)),
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
}
