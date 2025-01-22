mod instruction;

pub use instruction::{Instruction, CommandInstruction, ConfigInstruction};
use thiserror::Error;
use std::io::BufRead;

/// A `.cw` script
#[derive(Debug)]
pub struct Script {
    instructions: Vec<Instruction>,
}

impl Script {
    pub fn parse(reader: impl BufRead) -> Result<Self, ParseError> {
        let mut instructions = Vec::new();
        let mut expect_continuation = false;
        for (line_number, line) in reader.lines().enumerate() {
            let line = line.map_err(|err| ParseErrorType::Io(err).with_line(line_number))?;
            let instruction =
                Instruction::parse(&line).map_err(|e| e.with_line(line_number + 1))?;
            // Check for `ExpectedContinuation` and `UnexpectedContinuation`
            if let Instruction::Command(command_inst) = &instruction {
                let is_start = command_inst.is_start();
                if is_start {
                    if expect_continuation {
                        return Err(ParseErrorType::ExpectedContinuation.with_line(line_number + 1));
                    }
                } else {
                    if !expect_continuation {
                        return Err(
                            ParseErrorType::UnexpectedContinuation.with_line(line_number + 1)
                        );
                    }
                }
                expect_continuation = command_inst.expect_continuation();
            } else if expect_continuation {
                return Err(ParseErrorType::ExpectedContinuation.with_line(line_number + 1));
            }
            instructions.push(instruction);
        }
        Ok(Self { instructions })
    }

    pub fn execute(&self) {
        execute_instructions(&self.instructions);
    }
}

fn execute_instructions(instructions: &[Instruction]) {
    // TODO: Implement this function
    println!("{:?}", instructions);
}

/// Possible types of errors that can occur while parsing a single line of a `.cw` file.
#[derive(Error, Debug)]
pub enum ParseErrorType {
    // General parsing errors
    /// An io error occurred while reading the file.
    #[error("IO error: \"{0}\"")]
    Io(std::io::Error),
    /// The first non-whitespace character of the line is not recognized.
    #[error("Unknown instruction")]
    UnknownInstruction,
    /// The instruction is not in the expected format.
    #[error("Malformed instruction")]
    MalformedInstruction,
    /// Expected a continuation line, but did not get one.
    #[error("Expected continuation")]
    ExpectedContinuation,
    /// Did not expect a continuation line, but got one.
    #[error("Unexpected continuation")]
    UnexpectedContinuation,
}

impl ParseErrorType {
    /// Add line number information to the error, so as to form a [`ParseError`].
    fn with_line(self, line: usize) -> ParseError {
        ParseError { error: self, line }
    }
}

/// An error that occurred while parsing a `.cw` file, with the line number denoting its position. To construct a `ParseError`, you should call [`ParseErrorType::with_line`].
#[derive(Error, Debug)]
#[error("{error} at line {line}")]
pub struct ParseError {
    /// The type of error that occurred.
    error: ParseErrorType,
    /// The line number where the error occurred, starting at 1. If `0`, the line number is unknown at this point.
    line: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    #[test]
    fn script() {
        let script = r#"
            @@width 123
            @height auto
            %print
            !marker
            #comment
            $single command
            $command \
            >continuation
        "#;
        let script = script.trim();
        let script = script.as_bytes();
        let script = BufReader::new(script);
        let script = Script::parse(script).unwrap();
        let expected = vec![
            Instruction::PersistentConfig(ConfigInstruction::Width(123)),
            Instruction::TemporaryConfig(ConfigInstruction::Height(0)),
            Instruction::Print("print".to_string()),
            Instruction::Marker("marker".to_string()),
            Instruction::Empty,
            Instruction::Command(CommandInstruction::parse("single command", true)),
            Instruction::Command(CommandInstruction::parse("command \\", true)),
            Instruction::Command(CommandInstruction::parse("continuation", false)),
        ];
        assert_eq!(script.instructions, expected);
    }

    #[test]
    fn script_unknown_instruction() {
        let script = r#"
            @@width 123
            @height 456
            %print
            !marker
            #comment
            $command \
            >continuation
            unknown
        "#;
        let script = script.trim();
        let script = script.as_bytes();
        let script = BufReader::new(script);
        assert!(matches!(
            Script::parse(script).unwrap_err(),
            ParseError {
                error: ParseErrorType::UnknownInstruction,
                line: 8
            }
        ));
    }

    #[test]
    fn script_malformed_instruction() {
        let malformed_scripts = [
            // Config instruction
            "#abc\n@",                   // Expected character after @
            "#abc\n@@",                  // Expected character after @@
            "#abc\n@@wid",               // Unrecognized configuration instruction
            "@@width 123\n@@height abc", // Malformed integer
            "@@width 123\n@idle 1",      // Malformed duration - no suffix
            "@@width 123\n@idle 1min",   // Malformed duration - invalid suffix
            "@@width 123\n@delay",       // Malformed duration - no value
            "@@width 123\n@hidden what", // Malformed boolean
        ];
        for script in malformed_scripts.iter() {
            let script = script.trim();
            let script = script.as_bytes();
            let script = BufReader::new(script);
            assert!(matches!(
                Script::parse(script).unwrap_err(),
                ParseError {
                    error: ParseErrorType::MalformedInstruction,
                    line: 2
                }
            ));
        }
    }

    #[test]
    fn script_expected_continuation() {
        let script = r#"
            $command \
            @width 123
        "#;
        let script = script.trim();
        let script = script.as_bytes();
        let script = BufReader::new(script);
        assert!(matches!(
            Script::parse(script).unwrap_err(),
            ParseError {
                error: ParseErrorType::ExpectedContinuation,
                line: 2
            }
        ));
    }

    #[test]
    fn script_unexpected_continuation() {
        let script = r#"
            $command
            >continuation
        "#;
        let script = script.trim();
        let script = script.as_bytes();
        let script = BufReader::new(script);
        assert!(matches!(
            Script::parse(script).unwrap_err(),
            ParseError {
                error: ParseErrorType::UnexpectedContinuation,
                line: 2
            }
        ));
    }
}
