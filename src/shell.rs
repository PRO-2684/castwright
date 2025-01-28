//! Module for interacting with the shell.

use duct::{cmd, ReaderHandle};
use std::io::Read;

use super::ErrorType;

/// Execute a command using given shell, returning its output as a receiver.
pub fn execute_command(shell: &str, command: &str, check: bool) -> Result<ReaderIterator, ErrorType> {
    // Spawn the command
    let mut command = cmd!(shell, "-c", command);
    if !check {
        command = command.unchecked(); // Don't check for status code (TODO: Config for this)
    }
    let reader = command.stderr_to_stdout().reader().map_err(ErrorType::Io)?;
    let iter = ReaderIterator::new(reader);

    Ok(iter)
}

/// Iterator over `ReaderHandle`.
pub struct ReaderIterator{
    reader: ReaderHandle,
    buffer: [u8; 1024],
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
                let output = String::from_utf8_lossy(&self.buffer[..n]).to_string();
                Some(Ok(output))
            }
            Err(e) => {
                self.error = true;
                Some(Err(ErrorType::Io(e)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    const MAX_DELTA: Duration = Duration::from_millis(100);

    #[test]
    fn echo_stdout() {
        let command = "echo hello";
        let shell = "bash";
        let reader = execute_command(shell, command, true).unwrap();
        let mut output = String::new();
        for chunk in reader {
            output.push_str(&chunk.unwrap());
        }
        assert_eq!(output, "hello\n");
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
        assert_eq!(output, "hello\n");
    }

    #[test]
    fn echo_both() {
        let command = "echo hello; echo world 1>&2";
        let shell = "bash";
        let reader = execute_command(shell, command, true).unwrap();
        let expected = "hello\nworld\n";
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
        let expected = vec!["hello\n", "world\n"];
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
        assert!(
            duration <= Duration::from_secs(1) + MAX_DELTA,
            "Duration: {:?}",
            duration
        );
    }
}
