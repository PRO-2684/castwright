//! Module for front matter instructions.

use super::{
    AsciiCast, ErrorType, ExecutionContext, FrontMatterState, InstructionTrait, ParseContext, util,
};
use serde_json::de::from_str;
use std::time::Duration;

/// A front matter instruction.
#[derive(Debug, PartialEq, Eq)]
pub enum FrontMatterInstruction {
    /// Delimiter.
    Delimiter,
    /// Terminal width.
    Width(u16),
    /// Terminal height.
    Height(u16),
    /// Title of the asciicast.
    Title(String),
    /// The shell to use.
    Shell(Vec<String>),
    /// The quit command.
    Quit(String),
    /// Idle time limit.
    Idle(Duration),
    // Captured environment variables.
    Capture(Vec<String>),
}

impl InstructionTrait for FrontMatterInstruction {
    /// Parse a line into a `FrontMatterInstruction`.
    fn parse(s: &str, context: &mut ParseContext) -> Result<Self, ErrorType>
    where
        Self: Sized,
    {
        // Note that `s` is trimmed, and the first character is preserved.
        if s == "---" {
            // A delimiter line.
            context.front_matter_state.next()?;
            Ok(Self::Delimiter)
        } else if matches!(context.front_matter_state, FrontMatterState::Start) {
            // We are expecting a key-value pair.
            let mut iter = s.splitn(2, ':');
            let Some(key) = iter.next() else {
                return Err(ErrorType::ExpectedKeyValuePair);
            };
            let value = iter.next().ok_or(ErrorType::ExpectedKeyValuePair)?.trim();
            match key {
                "width" => {
                    let width = parse_positive_u16(value)?;
                    Ok(Self::Width(width))
                }
                "height" => {
                    let height = parse_positive_u16(value)?;
                    Ok(Self::Height(height))
                }
                "title" => {
                    let value = util::parse_loose_string(value)?;
                    Ok(Self::Title(value))
                }
                "shell" => {
                    let shell: Vec<String> = from_str(value)?;
                    // Ensure that the vector is not empty.
                    if shell.is_empty() {
                        return Err(ErrorType::MalformedInstruction);
                    }
                    Ok(Self::Shell(shell))
                }
                "quit" => {
                    let value = util::parse_loose_string(value)?;
                    Ok(Self::Quit(value))
                }
                "idle" => {
                    let idle = util::parse_duration(value)?;
                    Ok(Self::Idle(idle))
                }
                "capture" => {
                    let env_vars: Vec<String> = from_str(value)?;
                    Ok(Self::Capture(env_vars))
                }
                _ => Err(ErrorType::UnknownFrontMatter),
            }
        } else {
            // We are not expecting any key-value pairs, and this instruction does not match any other instruction.
            Err(ErrorType::UnknownInstruction)
        }
    }
    /// Execute the front matter instruction.
    fn execute(
        &self,
        context: &mut ExecutionContext,
        cast: &mut AsciiCast<impl std::io::Write>,
    ) -> Result<(), ErrorType> {
        match self {
            Self::Width(width) => {
                context.width = *width;
                cast.width(*width)?;
            }
            Self::Height(height) => {
                context.height = *height;
                cast.height(*height)?;
            }
            Self::Title(title) => {
                cast.title(title.clone())?;
            }
            Self::Shell(shell) => {
                context.shell.clone_from(shell);
            }
            // FrontMatterInstruction::Quit(quit) => { cast.quit(quit.clone())?; },
            Self::Idle(idle) => {
                cast.idle_time_limit(idle.as_secs_f64())?;
            }
            Self::Capture(env_vars) => {
                cast.capture(util::capture_env_vars(env_vars.clone()))?;
            }
            _ => {}
        }
        Ok(())
    }
}

/// Parse a positive integer.
fn parse_positive_u16(s: &str) -> Result<u16, ErrorType> {
    let v = s.parse()?;
    if v == 0 {
        Err(ErrorType::MalformedInstruction)
    } else {
        Ok(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_positive_u16() {
        assert_eq!(
            parse_positive_u16("0"),
            Err(ErrorType::MalformedInstruction)
        );
        assert_eq!(parse_positive_u16("1"), Ok(1));
    }

    #[test]
    fn front_matter_instruction() {
        use FrontMatterInstruction::*;
        let mut parse_context = ParseContext::new();
        let instructions = [
            ("---", Delimiter),
            ("width: 80", Width(80)),
            ("height: 24", Height(24)),
            ("title: Hello, world!", Title("Hello, world!".to_string())),
            (
                "shell: [\"/bin/bash\", \"-i\", \"-c\"]",
                Shell(vec!["/bin/bash".to_string(), "-i".to_string(), "-c".to_string()]),
            ),
            ("quit: exit", Quit("exit".to_string())),
            ("idle: 1s", Idle(Duration::from_secs(1))),
            (
                "capture: [\"SHELL\", \"TERM\"]",
                Capture(vec!["SHELL".to_string(), "TERM".to_string()]),
            ),
        ];
        for (line, expected) in &instructions {
            assert_eq!(
                &FrontMatterInstruction::parse(line, &mut parse_context).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn empty_front_matter() {
        let mut parse_context = ParseContext::new();
        let instructions = [
            ("---", FrontMatterInstruction::Delimiter),
            ("---", FrontMatterInstruction::Delimiter),
        ];
        for (line, expected) in &instructions {
            assert_eq!(
                &FrontMatterInstruction::parse(line, &mut parse_context).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn malformed_front_matter_instruction() {
        let mut parse_context = ParseContext::new();
        parse_context.front_matter_state.next().unwrap();
        let instructions = [
            "width:",
            "width: -1",
            "width: what",
            "height:",
            "height: 0",
            "idle:",
            "idle: 1",
            "idle: 1.0",
            "shell: []", // Empty shell.
        ];
        for line in &instructions {
            let parsed = FrontMatterInstruction::parse(line, &mut parse_context).unwrap_err();
            assert!(
                matches!(parsed, ErrorType::MalformedInstruction,),
                "Expected MalformedInstruction, got {parsed:?} at line `{line}`"
            );
        }
    }

    #[test]
    fn expected_key_value_pair() {
        let mut parse_context = ParseContext::new();
        parse_context.front_matter_state.next().unwrap();
        let instructions = [
            "width", "height", "title", "shell", "quit", "idle", "unknown",
        ];
        for line in &instructions {
            let parsed = FrontMatterInstruction::parse(line, &mut parse_context).unwrap_err();
            assert!(
                matches!(parsed, ErrorType::ExpectedKeyValuePair,),
                "Expected ExpectedKeyValuePair, got {parsed:?} at line `{line}`"
            );
        }
    }

    #[test]
    fn json_error() {
        let mut parse_context = ParseContext::new();
        parse_context.front_matter_state.next().unwrap();
        let instructions = [
            "capture:",
            "capture: [",
            "capture: [\"SHELL\", \"TERM",
            "capture: [\"SHELL\", \"TERM\", \"",
        ];
        for line in &instructions {
            let parsed = FrontMatterInstruction::parse(line, &mut parse_context).unwrap_err();
            assert!(
                matches!(parsed, ErrorType::Json(_)),
                "Expected Json, got {parsed:?} at line `{line}`"
            );
        }
    }

    #[test]
    fn unknown_front_matter_instruction() {
        let mut parse_context = ParseContext::new();
        parse_context.front_matter_state.next().unwrap();
        let instructions = ["unknown: 123", ": 456"];
        for line in &instructions {
            let parsed = FrontMatterInstruction::parse(line, &mut parse_context).unwrap_err();
            assert!(
                matches!(parsed, ErrorType::UnknownFrontMatter,),
                "Expected UnknownFrontMatter, got {parsed:?} at line `{line}`"
            );
        }
    }
}
