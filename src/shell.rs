//! Module for interacting with the shell.

use duct::{cmd, ReaderHandle};
use std::{io::Read, time::Duration};

use super::ErrorType;

const INTERVAL: Duration = Duration::from_millis(10);

/// Execute a command using given shell, returning its output as a receiver.
fn execute_command(command: &str, shell: &str) -> Result<ReaderHandle, ErrorType> {
    // Spawn the command
    let command = cmd!(shell, "-c", command);
    let reader = command.stderr_to_stdout().reader().map_err(ErrorType::Io)?;

    Ok(reader)
}

/// Poll the reader for output and error.
fn poll_reader(
    reader: &mut impl Read,
    mut on_output: impl FnMut(String),
    on_error: impl FnOnce(std::io::Error),
) {
    let mut buffer = [0; 1024];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                let output = String::from_utf8_lossy(&buffer[..n]).to_string();
                on_output(output);
            }
            Err(e) => {
                on_error(e);
                break;
            }
        }
        std::thread::sleep(INTERVAL);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn echo_stdout() {
        let command = "echo hello";
        let shell = "bash";
        let mut reader = execute_command(command, shell).unwrap();
        let mut output = String::new();
        reader.read_to_string(&mut output).unwrap();
        assert_eq!(output, "hello\n");
    }

    #[test]
    fn echo_stderr() {
        let command = "echo hello 1>&2";
        let shell = "bash";
        let mut reader = execute_command(command, shell).unwrap();
        let mut output = String::new();
        reader.read_to_string(&mut output).unwrap();
        assert_eq!(output, "hello\n");
    }

    #[test]
    fn echo_both() {
        let command = "echo hello; echo world 1>&2";
        let shell = "bash";
        let mut reader = execute_command(command, shell).unwrap();
        let expected = "hello\nworld\n";
        let mut actual = String::new();
        reader.read_to_string(&mut actual).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn echo_with_delay() {
        let command = "echo hello; sleep 1; echo world 1>&2";
        let shell = "bash";
        let mut reader = execute_command(command, shell).unwrap();
        let expected = vec!["hello\n", "world\n"];
        let mut actual = Vec::new();

        let mut first = None;
        let mut second = None;

        poll_reader(
            &mut reader,
            |output| {
                actual.push(output);
                if first.is_none() {
                    first = Some(std::time::Instant::now());
                } else {
                    second = Some(std::time::Instant::now());
                }
            },
            |e| eprintln!("Error reading: {}", e),
        );

        assert_eq!(actual, expected);

        let duration = second.unwrap().duration_since(first.unwrap());
        assert!(
            duration >= Duration::from_secs(1),
            "Duration: {:?}",
            duration
        );
        assert!(
            duration <= Duration::from_secs(1) + 2 * INTERVAL,
            "Duration: {:?}",
            duration
        );
    }
}
