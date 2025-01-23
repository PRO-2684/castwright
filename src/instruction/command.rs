//! Module for parsing command instructions.

use super::{AsciiCast, ExecutionContext, InstructionTrait, ParseContext, ParseErrorType};

/// A command instruction.
#[derive(Debug, PartialEq)]
pub struct CommandInstruction {
    /// The command to execute.
    command: String,
    /// Whether the command is a starting command. `true` if starting with `$`, `false` if starting with `>`.
    start: bool,
    /// Whether the command expects a continuation. `true` if ending with `\`, `false` otherwise.
    continuation: bool,
}

impl InstructionTrait for CommandInstruction {
    /// Parse a line into a `CommandInstruction`.
    fn parse(s: &str, context: &mut ParseContext) -> Result<Self, ParseErrorType> {
        let s = s.trim();
        let start = context.start == '$';
        let continuation = s.ends_with('\\');
        let command = if continuation {
            s[..s.len() - 1].trim_end()
        } else {
            s
        };
        Ok(Self {
            command: command.to_string(),
            start,
            continuation,
        })
    }
    /// Execute the instruction
    fn execute(&self, _context: &mut ExecutionContext, cast: &mut AsciiCast) {
        // TODO: Implement
        cast.push(format!("command: {}", self.command));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_instruction() {
        let instructions = [
            (("hello", true), ("hello", true, false)),
            (("world", false), ("world", false, false)),
            ((" hello \\", true), ("hello", true, true)),
            (("world\\", false), ("world", false, true)),
        ];
        for ((input, start_input), (command, start_output, continuation)) in instructions.iter() {
            assert_eq!(start_input, start_output);
            let mut context = ParseContext::new();
            context.start = if *start_input { '$' } else { '>' };
            let instruction = CommandInstruction::parse(input, &mut context).unwrap();
            assert_eq!(instruction.command, *command);
            assert_eq!(instruction.start, *start_output);
            assert_eq!(instruction.continuation, *continuation);
        }
    }
}
