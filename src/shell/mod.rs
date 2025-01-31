//! Module for interacting with the shell.

mod cd;

use super::{ErrorType, ExecutionContext};
use cd::Cd;
use duct::{cmd, ReaderHandle};
use std::io::Read;

/// Execute a command using given shell, returning its output as an iterator, with `\n` replaced by `\r\n`.
pub fn execute_command(
    context: &mut ExecutionContext,
    command: &str,
    check: bool,
) -> Result<ReaderIterator, ErrorType> {
    // Check if the command is a built-in command
    if execute_built_in_command(context, command)? {
        return Ok(ReaderIterator::new());
    }
    // Spawn the command
    let mut command = cmd!(&context.shell, "-c", command).dir(&context.directory);
    if !check {
        command = command.unchecked(); // Don't check for status code (TODO: Config for this)
    }
    let reader = command.stderr_to_stdout().reader()?;
    let iter = ReaderIterator::from_handle(reader);

    Ok(iter)
}

/// Iterator over `ReaderHandle`. Replace `\n` with `\r\n`.
pub struct ReaderIterator {
    /// Buffer for reading output.
    buffer: [u8; 1024],
    /// Inner reader handle.
    reader: Option<ReaderHandle>,
    /// Error flag.
    error: bool,
}

impl ReaderIterator {
    /// Create a new `ReaderIterator` that does nothing.
    pub fn new() -> Self {
        Self {
            reader: None,
            buffer: [0; 1024],
            error: false,
        }
    }
    /// Create a new `ReaderIterator` from a `ReaderHandle`.
    pub fn from_handle(reader: ReaderHandle) -> Self {
        Self {
            reader: Some(reader),
            buffer: [0; 1024],
            error: false,
        }
    }
    /// Read until EOF, discarding the output.
    pub fn consume(&mut self) -> Result<(), ErrorType> {
        for output in self {
            output?;
        }
        Ok(())
    }
}

impl Iterator for ReaderIterator {
    type Item = Result<String, ErrorType>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.error {
            // An error occurred
            return None;
        }
        let Some(reader) = &mut self.reader else {
            // No reader
            return None;
        };
        match reader.read(&mut self.buffer) {
            Ok(0) => None,
            Ok(n) => {
                let raw = String::from_utf8_lossy(&self.buffer[..n]).to_string();
                // Replace `\n` with `\r\n`
                let output = replace_newline(&raw);
                // FIXME: Edge case: if the previous chunk ends with `\r`, and the next chunk starts with `\n`, the `\n` will be replaced by `\r\n`.
                Some(Ok(output))
            }
            Err(e) => {
                self.error = true;
                Some(Err(ErrorType::Io(e)))
            }
        }
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

/// Try to execute a built-in command.
fn execute_built_in_command(context: &mut ExecutionContext, command: &str) -> Result<bool, ErrorType> {
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
        let command = "echo hello";
        let mut context = ExecutionContext::new();
        let reader = execute_command(&mut context, command, true).unwrap();
        let mut output = String::new();
        for chunk in reader {
            output.push_str(&chunk.unwrap());
        }
        assert_eq!(output, "hello\r\n");
    }

    #[test]
    fn echo_stderr() {
        let command = "echo hello 1>&2";
        let mut context = ExecutionContext::new();
        let reader = execute_command(&mut context, command, true).unwrap();
        let mut output = String::new();
        for chunk in reader {
            output.push_str(&chunk.unwrap());
        }
        assert_eq!(output, "hello\r\n");
    }

    #[test]
    fn echo_both() {
        let command = "echo hello; echo world 1>&2";
        let mut context = ExecutionContext::new();
        let reader = execute_command(&mut context, command, true).unwrap();
        let expected = "hello\r\nworld\r\n";
        let mut actual = String::new();
        for chunk in reader {
            actual.push_str(&chunk.unwrap());
        }

        assert_eq!(actual, expected);
    }

    #[test]
    fn echo_with_delay() {
        let command = "echo hello; sleep 1; echo world 1>&2";
        let mut context = ExecutionContext::new();
        let reader = execute_command(&mut context, command, true).unwrap();
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
        assert!(
            duration >= Duration::from_secs(1),
            "Duration: {:?}",
            duration
        );
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
        for (input, expected) in cases.iter() {
            let actual = replace_newline(input);
            assert_eq!(actual, *expected);
        }
    }
}
