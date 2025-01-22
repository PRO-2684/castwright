//! Module for parsing config instructions.

use super::{util, ParseErrorType, ScriptConfiguration};
use std::time::Duration;

/// A configuration instruction type.
#[derive(Debug, PartialEq)]
pub enum ConfigInstructionType {
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

/// A configuration instruction.
#[derive(Debug, PartialEq)]
pub struct ConfigInstruction {
    instruction_type: ConfigInstructionType,
    persistent: bool,
}

impl ConfigInstruction {
    /// Parse a positive integer, returning `0` if the string is `auto`.
    fn parse_auto_usize(s: &str) -> Result<usize, ParseErrorType> {
        if s == "auto" {
            Ok(0)
        } else {
            s.parse().map_err(|_| ParseErrorType::MalformedInstruction)
        }
    }
    /// Parse a line into a `ConfigInstruction`.
    pub fn parse(s: &str) -> Result<Self, ParseErrorType> {
        // The first character ('@') has been removed, thus the check is for the second character
        let s = s.trim();
        let persistent = s.starts_with("@");
        let s = if persistent { &s[1..] } else { s }; // Remove the '@' if it's present
        let mut iter = s.split_whitespace();
        let Some(first) = iter.next() else {
            return Err(ParseErrorType::MalformedInstruction);
        };
        let instruction_type = match first {
            "width" => {
                let width = iter.next().ok_or(ParseErrorType::MalformedInstruction)?;
                Ok(ConfigInstructionType::Width(Self::parse_auto_usize(width)?))
            }
            "height" => {
                let height = iter.next().ok_or(ParseErrorType::MalformedInstruction)?;
                Ok(ConfigInstructionType::Height(Self::parse_auto_usize(height)?))
            }
            "title" => {
                let title = util::parse_quoted_string(s[5..].trim());
                Ok(ConfigInstructionType::Title(title))
            }
            "shell" => {
                let shell = util::parse_quoted_string(s[5..].trim());
                Ok(ConfigInstructionType::Shell(shell))
            }
            "quit" => {
                let quit = util::parse_quoted_string(s[4..].trim());
                Ok(ConfigInstructionType::Quit(quit))
            }
            "idle" => {
                let idle = iter.next().ok_or(ParseErrorType::MalformedInstruction)?;
                Ok(ConfigInstructionType::Idle(util::parse_duration(idle)?))
            }
            "prompt" => {
                let prompt = util::parse_quoted_string(s[6..].trim());
                Ok(ConfigInstructionType::Prompt(prompt))
            }
            "secondary-prompt" => {
                let prompt = util::parse_quoted_string(s[16..].trim());
                Ok(ConfigInstructionType::SecondaryPrompt(prompt))
            }
            "line-split" => {
                let split = util::parse_quoted_string(s[10..].trim());
                Ok(ConfigInstructionType::LineSplit(split))
            }
            "hidden" => {
                let hidden = iter.next();
                if let Some(word) = hidden {
                    match word {
                        "true" => Ok(ConfigInstructionType::Hidden(true)),
                        "false" => Ok(ConfigInstructionType::Hidden(false)),
                        _ => Err(ParseErrorType::MalformedInstruction),
                    }
                } else {
                    Ok(ConfigInstructionType::Hidden(true)) // Default to true
                }
            }
            "delay" => {
                let delay = iter.next().ok_or(ParseErrorType::MalformedInstruction)?;
                Ok(ConfigInstructionType::Delay(util::parse_duration(delay)?))
            }
            _ => Err(ParseErrorType::MalformedInstruction),
        }?;
        Ok(Self {
            instruction_type,
            persistent,
        })
    }
    /// Execute the configuration instruction.
    pub fn execute(&self, _config: &mut ScriptConfiguration) {
        // TODO: Implement
    }
}

#[cfg(test)]
mod tests {
    use super::{ParseErrorType, ConfigInstruction, ConfigInstructionType::*};
    use std::time::Duration;

    #[test]
    fn config_instruction_type() {
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
            assert_eq!(ConfigInstruction::parse(input).unwrap().instruction_type, *expected);
        }
    }

    #[test]
    fn config_instruction_persistent() {
        let instructions = [
            ("width 123", false),
            ("@height 456", true),
            ("width auto", false),
            ("@height auto", true),
            ("width 0", false),
            ("@height 0", true),
            ("title castwright demo", false),
            ("@shell bash ", true),
            ("quit \"exit \"", false),
            ("@idle 1s", true),
            ("prompt \"$ \"", false),
            ("@secondary-prompt \"> \"", true),
            ("line-split \\", false),
            ("@hidden true", true),
            ("delay 2ms", false),
        ];
        for (input, expected) in instructions.iter() {
            assert_eq!(ConfigInstruction::parse(input).unwrap().persistent, *expected);
        }
    }

    #[test]
    fn malformed_config_instruction() {
        let malformed_instructions = [
            "invalid",
            "@width",
            "width -1",
            "width what",
            "hidden what",
            "@delay",
            "delay 2",
        ];
        for line in malformed_instructions.iter() {
            assert!(matches!(
                ConfigInstruction::parse(line).unwrap_err(),
                ParseErrorType::MalformedInstruction,
            ));
        }
    }
}
