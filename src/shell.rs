//! Module for interacting with the shell.

use std::process::{Command, Stdio};
use std::io::{self, Read, Write};
use std::time::Duration;
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};

use super::ErrorType;

const INTERVAL: Duration = Duration::from_millis(10);

/// Read from a source and send the output to a channel.
fn poll(mut source: impl Read, dest: Sender<String>) {
    let mut buffer = [0; 1024];
    loop {
        match source.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                let output = String::from_utf8_lossy(&buffer[..n]).to_string();
                if let Err(err) = dest.send(output) {
                    eprintln!("Failed to send output: {}", err);
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error reading: {}", e);
                break;
            }
        }
        thread::sleep(INTERVAL);
    }
}

/// Execute a command using given shell, returning its output as a receiver.
fn execute_command(command: &str, shell: &str) -> Result<Receiver<String>, ErrorType> {
    // Spawn the command
    let mut child = Command::new(shell)
        .arg("-c")
        .arg(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(ErrorType::Io)?;

    // Get the stdout and stderr
    let stdout = child.stdout.take().expect("Failed to get stdout");
    let stderr = child.stderr.take().expect("Failed to get stderr");

    // Create a channel to send the output
    let (tx_stdout, rx) = channel();
    let tx_stderr = tx_stdout.clone();

    // Send the output to the channel
    thread::spawn(move || poll(stdout, tx_stdout));
    thread::spawn(move || poll(stderr, tx_stderr));

    Ok(rx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn echo_stdout() {
        let command = "echo hello";
        let shell = "bash";
        let output = execute_command(command, shell).unwrap();
        let output = output.recv().unwrap();
        assert_eq!(output, "hello\n");
    }

    #[test]
    fn echo_stderr() {
        let command = "echo hello 1>&2";
        let shell = "bash";
        let output = execute_command(command, shell).unwrap();
        let output = output.recv().unwrap();
        assert_eq!(output, "hello\n");
    }

    #[test]
    fn echo_both() {
        let command = "echo hello; echo world 1>&2";
        let shell = "bash";
        let output = execute_command(command, shell).unwrap();
        let expected = vec!["hello\n", "world\n"];
        let mut actual = Vec::new();
        for message in output.iter() {
            actual.push(message);
        }
        assert_eq!(actual, expected);
    }

    #[test]
    fn echo_with_delay() {
        let command = "echo hello; sleep 1; echo world 1>&2";
        let shell = "bash";
        let output = execute_command(command, shell).unwrap();
        let expected = vec!["hello\n", "world\n"];
        let mut actual = Vec::new();

        let mut first = None;
        let mut second = None;

        for message in output.iter() {
            actual.push(message);
            if first.is_none() {
                first = Some(std::time::Instant::now());
            } else if second.is_none() {
                second = Some(std::time::Instant::now());
            }
        }
        assert_eq!(actual, expected);

        let duration = second.unwrap().duration_since(first.unwrap());
        assert!(duration >= Duration::from_secs(1), "Duration: {:?}", duration);
        assert!(duration <= Duration::from_secs(1) + INTERVAL, "Duration: {:?}", duration);
    }
}
