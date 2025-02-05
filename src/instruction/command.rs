//! Module for parsing command instructions.

use super::{execute_command, AsciiCast, ErrorType, ExecutionContext, Instruction, ParseContext};
use std::io::Write;

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
            // _ => unreachable!("Should be handled by frontmatter.rs"),
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
        let temp = context.temporary.get(!self.continuation);
        let config = context.persistent.combine(temp);
        if config.hidden {
            if context.execute {
                // Execute command silently
                let expect = config.expect;
                let reader = execute_command(context, &self.command)?;
                let result = || -> Result<(), ErrorType> {
                    for chunk in reader {
                        let _ = chunk?;
                    }
                    Ok(())
                }();
                handle_error(result, expect)?;
            }
            return Ok(());
        }
        let prompt = if self.start {
            &config.prompt
        } else {
            &config.secondary_prompt
        };
        let interval = config.interval;
        cast.output(context.elapsed, prompt)?;
        context.preview(prompt);
        context.elapsed += config.start_lag;
        for character in self.command.chars() {
            context.elapsed += interval;
            // https://stackoverflow.com/a/67898224/16468609
            cast.output(context.elapsed, character.encode_utf8(&mut [0u8; 4]))?;
        }
        context.preview(&self.command);
        if self.continuation {
            context.elapsed += interval;
            cast.output(context.elapsed, &config.line_continuation)?;
            context.preview(&config.line_continuation);
            context.elapsed += interval;
            context.elapsed += config.end_lag;
            cast.output(context.elapsed, "\r\n")?;
            context.preview("\r\n");
            context.command.push_str(&self.command);
            context.command.push(' ');
        } else {
            context.elapsed += interval;
            context.elapsed += config.end_lag;
            cast.output(context.elapsed, "\r\n")?;
            context.preview("\r\n");
            // Take `context.command` out, replacing with an empty string
            let mut command = std::mem::take(&mut context.command);
            command.push_str(&self.command);
            if context.execute {
                let expect = config.expect;
                let mut prev = std::time::Instant::now();
                let reader = execute_command(context, &command)?;
                let mut lock = std::io::stdout().lock();
                let result = || -> Result<(), ErrorType> {
                    for chunk in reader {
                        let chunk = chunk?;
                        let now = std::time::Instant::now();
                        context.elapsed += now.duration_since(prev).as_micros() as u64;
                        prev = now;
                        cast.output(context.elapsed, &chunk)?;
                        // context.preview(&chunk);
                        // 1. Ensure that the output is flushed in real-time
                        // 2. Use lock to improve performance in case there are many chunks
                        if context.preview {
                            print!("{}", chunk);
                            lock.flush()?;
                        }
                    }
                    Ok(())
                }();
                handle_error(result, expect)?;
            }
        }
        Ok(())
    }
}

/// Handle the result of executing a command (see if it fulfills the expectation).
fn handle_error(result: Result<(), ErrorType>, expect: Option<bool>) -> Result<(), ErrorType> {
    // If the `result` is not an `ErrorType::Subprocess`, always return it directly.
    if let Err(e) = &result {
        if !matches!(e, ErrorType::Subprocess(_)) {
            return result;
        }
    }
    // If the `expect` is `None`, always return `Ok`.
    let Some(expect) = expect else {
        return Ok(());
    };
    match result {
        Ok(()) => {
            // If the `result` is `Ok`:
            if expect {
                // If the `expect` is `true`, return `Ok`.
                Ok(())
            } else {
                // If the `expect` is `false`, return `Err`.
                Err(ErrorType::Subprocess(
                    "command expected failure, but succeeded".to_string(),
                ))
            }
        }
        Err(e) => {
            // If the `result` is `Err`:
            if expect {
                // If the `expect` is `true`, return `Err`.
                Err(e)
            } else {
                // If the `expect` is `false`, return `Ok`.
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    /// Create an `io::Error` for testing.
    fn io_error() -> Result<(), ErrorType> {
        Err(ErrorType::Io(io::Error::new(io::ErrorKind::Other, "error")))
    }

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

    #[test]
    fn error_handling() {
        let should_succeed: [(Result<(), ErrorType>, Option<_>); 4] = [
            (Ok(()), None),
            (Err(ErrorType::Subprocess("error".to_string())), None),
            (Ok(()), Some(true)),
            (Err(ErrorType::Subprocess("error".to_string())), Some(false)),
        ];
        for (result, expect) in should_succeed.into_iter() {
            let desc = format!("handle_error({result:?}, {expect:?})");
            assert!(handle_error(result, expect).is_ok(), "{desc}");
        }
        let should_fail: [(Result<(), ErrorType>, Option<_>); 5] = [
            (Ok(()), Some(false)),
            (Err(ErrorType::Subprocess("error".to_string())), Some(true)),
            (io_error(), None),
            (io_error(), Some(true)),
            (io_error(), Some(false)),
        ];
        for (result, expect) in should_fail.into_iter() {
            let desc = format!("handle_error({result:?}, {expect:?})");
            assert!(handle_error(result, expect).is_err(), "{desc}");
        }
    }
}
