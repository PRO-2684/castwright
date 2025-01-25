//! Module for parsing front matter instructions.

use super::{
    util, AsciiCast, ErrorType, ExecutionContext, FrontMatterState, Instruction, ParseContext,
};
use std::time::Duration;

/// A front matter instruction.
#[derive(Debug, PartialEq)]
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
    Shell(String),
    /// The quit command.
    Quit(String),
    /// Idle time limit.
    Idle(Duration),
}

impl Instruction for FrontMatterInstruction {
    /// Parse a line into a `FrontMatterInstruction`.
    fn parse(s: &str, context: &mut ParseContext) -> Result<Self, ErrorType>
    where
        Self: Sized,
    {
        // Note that `s` is trimmed, and the first character is preserved.
        if s == "---" {
            // A delimiter line.
            context.front_matter_state.next()?;
            return Ok(FrontMatterInstruction::Delimiter);
        } else if matches!(context.front_matter_state, FrontMatterState::Start) {
            // We are expecting a key-value pair.
            let mut iter = s.splitn(2, ':');
            let Some(key) = iter.next() else {
                return Err(ErrorType::ExpectedKeyValuePair);
            };
            let value = iter.next().ok_or(ErrorType::ExpectedKeyValuePair)?.trim();
            match key {
                "width" => {
                    let width = parse_positive_u16(&value)?;
                    Ok(FrontMatterInstruction::Width(width))
                }
                "height" => {
                    let height = parse_positive_u16(&value)?;
                    Ok(FrontMatterInstruction::Height(height))
                }
                "title" => {
                    let value = util::parse_loose_string(value)?;
                    Ok(FrontMatterInstruction::Title(value))
                }
                "shell" => {
                    let value = util::parse_loose_string(value)?;
                    Ok(FrontMatterInstruction::Shell(value))
                }
                "quit" => {
                    let value = util::parse_loose_string(value)?;
                    Ok(FrontMatterInstruction::Quit(value))
                }
                "idle" => {
                    let idle = util::parse_duration(&value)?;
                    Ok(FrontMatterInstruction::Idle(idle))
                }
                _ => Err(ErrorType::MalformedInstruction),
            }
        } else {
            // We are not expecting any key-value pairs, and this instruction does not match any other instruction.
            Err(ErrorType::UnknownInstruction)
        }
    }
    /// Execute the front matter instruction.
    fn execute(&self, _context: &mut ExecutionContext, cast: &mut AsciiCast) {
        match self {
            FrontMatterInstruction::Width(width) => {
                cast.width(*width);
            }
            FrontMatterInstruction::Height(height) => {
                cast.height(*height);
            }
            FrontMatterInstruction::Title(title) => {
                cast.title(title.clone());
            }
            // FrontMatterInstruction::Shell(shell) => { cast.shell(shell.clone()); },
            // FrontMatterInstruction::Quit(quit) => { cast.quit(quit.clone()); },
            FrontMatterInstruction::Idle(idle) => {
                cast.idle_time_limit(idle.as_secs_f64());
            }
            _ => {}
        }
    }
}

/// Parse a positive integer.
fn parse_positive_u16(s: &str) -> Result<u16, ErrorType> {
    let v = s.parse().map_err(|_| ErrorType::MalformedInstruction)?;
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
        assert_eq!(parse_positive_u16("0"), Err(ErrorType::MalformedInstruction));
        assert_eq!(parse_positive_u16("1"), Ok(1));
    }

    #[test]
    fn front_matter_instruction() {
        let mut parse_context = ParseContext::new();
        assert_eq!(
            FrontMatterInstruction::parse("---", &mut parse_context),
            Ok(FrontMatterInstruction::Delimiter)
        );
        assert_eq!(
            FrontMatterInstruction::parse("width: 80", &mut parse_context),
            Ok(FrontMatterInstruction::Width(80))
        );
        assert_eq!(
            FrontMatterInstruction::parse("height: 24", &mut parse_context),
            Ok(FrontMatterInstruction::Height(24))
        );
        assert_eq!(
            FrontMatterInstruction::parse("title: Hello, world!", &mut parse_context),
            Ok(FrontMatterInstruction::Title("Hello, world!".to_string()))
        );
        assert_eq!(
            FrontMatterInstruction::parse("shell: /bin/bash", &mut parse_context),
            Ok(FrontMatterInstruction::Shell("/bin/bash".to_string()))
        );
        assert_eq!(
            FrontMatterInstruction::parse("quit: exit", &mut parse_context),
            Ok(FrontMatterInstruction::Quit("exit".to_string()))
        );
        assert_eq!(
            FrontMatterInstruction::parse("idle: 1s", &mut parse_context),
            Ok(FrontMatterInstruction::Idle(Duration::from_secs(1)))
        );
        assert_eq!(
            FrontMatterInstruction::parse("---", &mut parse_context),
            Ok(FrontMatterInstruction::Delimiter)
        );
    }
}
