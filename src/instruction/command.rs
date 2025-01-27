//! Module for parsing command instructions.

use super::{AsciiCast, ErrorType, ExecutionContext, Instruction, ParseContext};

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

impl Instruction for CommandInstruction {
    /// Parse a line into a `CommandInstruction`.
    fn parse(s: &str, context: &mut ParseContext) -> Result<Self, ErrorType> {
        context.front_matter_state.end()?;
        let s = s.trim();
        let start = match context.start {
            '$' => true,
            '>' => false,
            _ => return Err(ErrorType::UnknownInstruction),
        };
        let continuation = s.ends_with('\\');
        if start && context.expect_continuation {
            return Err(ErrorType::ExpectedContinuation);
        } else if !start && !context.expect_continuation {
            return Err(ErrorType::UnexpectedContinuation);
        }
        context.expect_continuation = continuation;
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
    fn execute(
        &self,
        context: &mut ExecutionContext,
        cast: &mut AsciiCast,
    ) -> Result<(), ErrorType> {
        let config = if context.has_temporary() {
            if self.continuation {
                // The temporary context is needed for the continuation commands
                &context.merge_temporary()
            } else {
                // The temporary context is consumed for the ending command
                &context.consume_temporary()
            }
        } else {
            // No temporary context
            &context.persistent
        };
        if config.hidden {
            // TODO: Execute command silently
            return Ok(());
        }
        let prompt = if self.start {
            &config.prompt
        } else {
            &config.secondary_prompt
        };
        let delay = config.delay;
        cast.output(context.elapsed, prompt)?;
        for character in self.command.chars() {
            context.elapsed += delay;
            // https://stackoverflow.com/a/67898224/16468609
            cast.output(context.elapsed, character.encode_utf8(&mut [0u8; 4]))?;
        }
        if self.continuation {
            context.elapsed += delay;
            cast.output(context.elapsed, &config.line_split)?;
            context.elapsed += delay;
            cast.output(context.elapsed, "\r\n")?;
            context.command.push_str(&self.command);
            context.command.push(' ');
        } else {
            context.elapsed += delay;
            cast.output(context.elapsed, "\r\n")?;
            // Take `context.command` out, replacing with an empty string
            let mut command = std::mem::take(&mut context.command);
            command.push_str(&self.command);
            // Dummy output to simulate the command being executed
            // TODO: Implement actual command execution
            context.elapsed += delay;
            cast.output(context.elapsed, &format!("Executed command: {command}"))?;
            context.elapsed += delay;
            cast.output(context.elapsed, "\r\n")?;
        }
        Ok(())
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
            context.expect_continuation = !start_input;
            let instruction = CommandInstruction::parse(input, &mut context).unwrap();
            assert_eq!(instruction.command, *command);
            assert_eq!(instruction.start, *start_output);
            assert_eq!(instruction.continuation, *continuation);
        }
    }
}
