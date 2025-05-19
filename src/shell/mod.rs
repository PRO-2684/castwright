//! Module for interacting with the shell.

mod cd;

use super::{ErrorType, ExecutionContext};
use cd::Cd;
use std::{
    io::Read,
    process::Child,
};
use pty_process::{blocking::{open, Command, Pty}, Size};

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
    let (pty, pts) = open()?;
    pty.resize(Size::new(context.height, context.width))?;

    let child = Command::new(shell)
        .args(args)
        .current_dir(&context.directory)
        .spawn(pts)?;

    Ok(ReaderIterator::from_child(child, pty))
}

/// Iterator over a reader, replacing `\n` with `\r\n`.
pub struct ReaderIterator {
    /// Child process handle.
    child: Option<Child>,
    /// Inner pipe reader.
    reader: Option<Pty>,
    /// Buffer for reading output.
    buffer: [u8; 1024],
}

impl ReaderIterator {
    /// Create a new [`ReaderIterator`] that reads nothing.
    pub const fn new() -> Self {
        Self {
            child: None,
            reader: None,
            buffer: [0; 1024],
        }
    }
    /// Create a new [`ReaderIterator`] from a [`Child`] and reference to [`Pty`].
    pub const fn from_child(child: Child, reader: Pty) -> Self {
        Self {
            child: Some(child),
            reader: Some(reader),
            buffer: [0; 1024],
        }
    }
}

impl Iterator for ReaderIterator {
    /// - `Ok(None)` if no output is available yet.
    /// - `Ok(Some(output))` if we have some output.
    /// - `Err(ErrorType::Subprocess(...))` if the command exited with an error.
    type Item = Result<Option<String>, ErrorType>;

    fn next(&mut self) -> Option<Self::Item> {
        // End the iteration if the child process has exited (as marked by absence of `child` and `reader`).
        let Some(child) = &mut self.child else {
            // No child, or the child has been discarded
            return None;
        };
        let Some(reader) = &mut self.reader else {
            // No reader, or the reader has been discarded
            return None;
        };
        // Check the exit status of the child process
        match child.try_wait() {
            // The child process has exited
            Ok(Some(status)) => {
                // Discard the child and reader
                let _ = self.child.take().unwrap().wait();
                self.reader.take();

                let item = if status.success() {
                    Ok(None)
                } else {
                    // FIXME: The message not used?
                    Err(ErrorType::Subprocess(format!(
                        "command exited with {status}"
                    )))
                };
                return Some(item);
            }
            // Still running
            Ok(None) => {}
            // Got an error while trying to wait
            Err(e) => {
                // Discard the child and reader
                let _ = self.child.take().unwrap().wait();
                self.reader.take();

                return Some(Err(ErrorType::Io(e)));
            }
        };
        // Still running, try to read from the reader
        let item = match reader.read(&mut self.buffer) {
            // No output available yet
            Ok(0) => Ok(None),
            // Read some output
            Ok(n) => {
                let raw = String::from_utf8_lossy(&self.buffer[..n]).to_string();
                // Replace `\n` with `\r\n`
                let output = replace_newline(&raw);

                Ok(Some(output))
            }
            // Got an error while trying to read
            Err(e) => {
                self.reader.take();

                if e.raw_os_error() == Some(5) {
                    // Workaround for getting `Input/output error` if trying to read from a closed PTY
                    // See stackoverflow.com/questions/72150987
                    let status = match self.child.take().unwrap().wait() {
                        Ok(status) => status,
                        Err(e) => return Some(Err(ErrorType::Io(e))),
                    };
                    if status.success() {
                        Ok(None)
                    } else {
                        Err(ErrorType::Subprocess(format!(
                            "command exited with {status}"
                        )))
                    }
                } else {
                    // Other IO error
                    Err(ErrorType::Io(e))
                }
            }
        };
        Some(item)
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
            if let Some(chunk) = chunk.unwrap() {
                output.push_str(&chunk);
            }
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
            // output.push_str(&chunk.unwrap().unwrap());
            if let Some(chunk) = chunk.unwrap() {
                output.push_str(&chunk);
            }
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
            if let Some(chunk) = chunk.unwrap() {
                actual.push_str(&chunk);
            }
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
            let Some(chunk) = chunk.unwrap() else {
                continue;
            };
            actual.push(chunk);
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
