//! Module for interacting with the shell.

mod cd;

use super::{ErrorType, ExecutionContext};
use cd::Cd;
use duct::{cmd, ReaderHandle};
use std::io::{ErrorKind, Read};

/// Execute a command using given shell, returning its output as an iterator, with `\n` replaced by `\r\n`.
pub fn execute_command(
    context: &mut ExecutionContext,
    command: &str,
) -> Result<ReaderIterator, ErrorType> {
    // Check if the command is a built-in command
    if execute_built_in_command(context, command)? {
        return Ok(ReaderIterator::new());
    }
    // Spawn the command
    let (shell, args) = context.shell.split_at(1);
    let shell = shell[0].as_str();
    let command = [command];
    let args = args.iter().map(String::as_str).chain(command);

    let expr = cmd(shell, args).dir(&context.directory);
    let reader = expr.stderr_to_stdout().reader()?;
    Ok(reader.into())
}

/// Iterator over `ReaderHandle`, replacing `\n` with `\r\n`.
pub struct ReaderIterator {
    /// Buffer for reading output.
    buffer: [u8; 1024],
    /// Inner reader handle.
    reader: Option<ReaderHandle>,
}

impl ReaderIterator {
    /// Create a new `ReaderIterator` that does nothing.
    pub const fn new() -> Self {
        Self {
            reader: None,
            buffer: [0; 1024],
        }
    }
    /// Create a new `ReaderIterator` from a `ReaderHandle`.
    ///
    /// Alternatively, `ReaderIterator` implements `From<ReaderHandle>`, so you can call `.into()` on a `ReaderHandle`.
    pub const fn from_handle(reader: ReaderHandle) -> Self {
        Self {
            reader: Some(reader),
            buffer: [0; 1024],
        }
    }
}

impl Iterator for ReaderIterator {
    type Item = Result<String, ErrorType>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(reader) = &mut self.reader else {
            // No reader, or the reader has been discarded
            return None;
        };
        match reader.read(&mut self.buffer) {
            Ok(0) => None,
            Ok(n) => {
                let raw = String::from_utf8_lossy(&self.buffer[..n]).to_string();
                // Replace `\n` with `\r\n`
                let output = replace_newline(&raw);

                Some(Ok(output))
            }
            Err(e) => {
                // Discard the reader
                self.reader.take();

                if matches!(e.kind(), ErrorKind::Other) {
                    Some(Err(ErrorType::Subprocess(e.to_string())))
                } else {
                    Some(Err(ErrorType::Io(e)))
                }
            }
        }
    }
}

impl From<ReaderHandle> for ReaderIterator {
    fn from(reader: ReaderHandle) -> Self {
        Self::from_handle(reader)
    }
}

trait BuiltInCommand {
    /// Create a new instance of the command.
    fn new(arg: &str) -> Self
    where
        Self: Sized;
    /// Execute the command.
    fn execute(&self, context: &mut ExecutionContext) -> Result<(), ErrorType>;
}

/// Replace `\n` with `\r\n`, except `\n` that are part of `\r\n`.
fn replace_newline(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let chars = s.chars();
    let mut prev = ' ';
    for c in chars {
        if c == '\n' && prev != '\r' {
            result.push('\r');
        }
        result.push(c);
        prev = c;
    }
    result
}

/// Try to execute a built-in command. Return `Ok(false)` if the command is not a built-in command, `Ok(true)` if the command is a built-in command and executed successfully, and `Err` if an error occurred.
fn execute_built_in_command(
    context: &mut ExecutionContext,
    command: &str,
) -> Result<bool, ErrorType> {
    // Split the command in two parts: the command itself and its argument.
    let Some((cmd, arg)) = command.split_once(' ') else {
        return Ok(false);
    };
    let builtin: &dyn BuiltInCommand = &match cmd {
        "cd" => Cd::new(arg.trim()),
        _ => return Ok(false),
    };
    builtin.execute(context)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn echo_stdout() {
        let command = "echo hello".to_string();
        let mut context = ExecutionContext::new();
        let reader = execute_command(&mut context, &command).unwrap();
        let mut output = String::new();
        for chunk in reader {
            output.push_str(&chunk.unwrap());
        }
        assert_eq!(output, "hello\r\n");
    }

    #[test]
    fn echo_stderr() {
        let command = "echo hello 1>&2".to_string();
        let mut context = ExecutionContext::new();
        let reader = execute_command(&mut context, &command).unwrap();
        let mut output = String::new();
        for chunk in reader {
            output.push_str(&chunk.unwrap());
        }
        assert_eq!(output, "hello\r\n");
    }

    #[test]
    fn echo_both() {
        let command = "echo hello; echo world 1>&2".to_string();
        let mut context = ExecutionContext::new();
        let reader = execute_command(&mut context, &command).unwrap();
        let expected = "hello\r\nworld\r\n";
        let mut actual = String::new();
        for chunk in reader {
            actual.push_str(&chunk.unwrap());
        }

        assert_eq!(actual, expected);
    }

    #[test]
    fn echo_with_delay() {
        let command = "echo hello; sleep 1; echo world 1>&2".to_string();
        let mut context = ExecutionContext::new();
        let reader = execute_command(&mut context, &command).unwrap();
        let expected = vec!["hello\r\n", "world\r\n"];
        let mut actual = Vec::new();

        let mut first = None;
        let mut second = None;

        for chunk in reader {
            let output = chunk.unwrap();
            actual.push(output);
            if first.is_none() {
                first = Some(std::time::Instant::now());
            } else {
                second = Some(std::time::Instant::now());
            }
        }

        assert_eq!(actual, expected);

        let duration = second.unwrap().duration_since(first.unwrap());
        assert!(duration >= Duration::from_secs(1), "Duration: {duration:?}",);
    }

    #[test]
    fn replaced_newline() {
        let cases = [
            ("hello\nworld\r\n", "hello\r\nworld\r\n"),
            ("hello\nworld\n", "hello\r\nworld\r\n"),
            ("hello\nworld", "hello\r\nworld"),
            ("hello\r\nworld", "hello\r\nworld"),
            ("hello\r\nworld\n", "hello\r\nworld\r\n"),
            ("hello\rworld", "hello\rworld"),
        ];
        for (input, expected) in &cases {
            let actual = replace_newline(input);
            assert_eq!(actual, *expected);
        }
    }
}
