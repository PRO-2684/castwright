//! Module for parsing command instructions.

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

impl CommandInstruction {
    /// Parse a line into a `CommandInstruction`.
    pub fn parse(s: &str, start: bool) -> Self {
        let s = s.trim();
        let continuation = s.ends_with('\\');
        let command = if continuation {
            s[..s.len() - 1].trim_end()
        } else {
            s
        };
        Self {
            command: command.to_string(),
            start,
            continuation,
        }
    }
    /// Whether the command is a starting command. `true` if starting with `$`, `false` if starting with `>`.
    pub fn is_start(&self) -> bool {
        self.start
    }
    /// Whether the command expects a continuation. `true` if ending with `\`, `false` otherwise.
    pub fn expect_continuation(&self) -> bool {
        self.continuation
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
            let instruction = CommandInstruction::parse(input, *start_input);
            assert_eq!(instruction.command, *command);
            assert_eq!(instruction.start, *start_output);
            assert_eq!(instruction.continuation, *continuation);
        }
    }
}
