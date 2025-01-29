//! Module for interacting with the shell.

use duct::{cmd, ReaderHandle};
use std::io::Read;

use super::ErrorType;

/// Execute a command using given shell, returning its output as an iterator, with `\n` replaced by `\r\n`.
pub fn execute_command(
    shell: &str,
    command: &str,
    check: bool,
) -> Result<ReaderIterator, ErrorType> {
    // Spawn the command
    // TODO: Shell session
    let mut command = cmd!(shell, "-c", command);
    if !check {
        command = command.unchecked(); // Don't check for status code (TODO: Config for this)
    }
    let reader = command.stderr_to_stdout().reader()?;
    let iter = ReaderIterator::new(reader);

    Ok(iter)
}

/// Iterator over `ReaderHandle`. Replace `\n` with `\r\n`.
pub struct ReaderIterator {
    /// Buffer for reading output.
    buffer: [u8; 1024],
    /// Inner reader handle.
    reader: ReaderHandle,
    /// Error flag.
    error: bool,
}

impl ReaderIterator {
    /// Create a new `ReaderIterator`.
    pub fn new(reader: ReaderHandle) -> Self {
        Self {
            reader,
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
            return None;
        }
        match self.reader.read(&mut self.buffer) {
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

/// Replace `\n` with `\r\n`, except `\n` that are part of `\r\n`.
fn replace_newline(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    let mut prev = ' ';
    while let Some(c) = chars.next() {
        if c == '\n' && prev != '\r' {
            result.push('\r');
        }
        result.push(c);
        prev = c;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn echo_stdout() {
        let command = "echo hello";
        let shell = "bash";
        let reader = execute_command(shell, command, true).unwrap();
        let mut output = String::new();
        for chunk in reader {
            output.push_str(&chunk.unwrap());
        }
        assert_eq!(output, "hello\r\n");
    }

    #[test]
    fn echo_stderr() {
        let command = "echo hello 1>&2";
        let shell = "bash";
        let reader = execute_command(shell, command, true).unwrap();
        let mut output = String::new();
        for chunk in reader {
            output.push_str(&chunk.unwrap());
        }
        assert_eq!(output, "hello\r\n");
    }

    #[test]
    fn echo_both() {
        let command = "echo hello; echo world 1>&2";
        let shell = "bash";
        let reader = execute_command(shell, command, true).unwrap();
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
        let shell = "bash";
        let reader = execute_command(shell, command, true).unwrap();
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
