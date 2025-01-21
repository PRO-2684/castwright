//! Module for parsing instructions.

mod command;
mod config;

use super::{ParseError, ParseErrorType};
use command::CommandInstruction;
use config::ConfigInstruction;
use std::io::BufRead;

/// A single instruction
#[derive(Debug, PartialEq)]
pub enum Instruction {
    /// Persistent configuration instruction or metadata. (`@@`)
    PersistentConfig(ConfigInstruction),
    /// Temporary configuration instruction. (`@`)
    TemporaryConfig(ConfigInstruction),
    /// Print a string as it is. (`%`)
    Print(String),
    /// Marker. (`!`)
    Marker(String),
    /// A shell command. (`$` or `>`)
    Command(CommandInstruction),
    /// Comment (`#`) or empty line.
    Empty,
}

impl Instruction {
    /// Parse a line into an `Instruction`.
    pub fn parse(s: &str) -> Result<Self, ParseErrorType> {
        let s = s.trim();
        let Some(first) = s.chars().next() else {
            return Ok(Self::Empty);
        };
        let trimmed = s[1..].trim().to_string();
        match first {
            '@' => {
                let Some(second) = s.chars().nth(1) else {
                    return Err(ParseErrorType::MalformedInstruction);
                };
                match second {
                    '@' => Ok(Self::PersistentConfig(ConfigInstruction::parse(&s[2..])?)),
                    _ => Ok(Self::TemporaryConfig(ConfigInstruction::parse(&trimmed)?)),
                }
            }
            '%' => Ok(Self::Print(trimmed)),
            '!' => Ok(Self::Marker(trimmed)),
            '#' => Ok(Self::Empty),
            '$' | '>' => Ok(Self::Command(CommandInstruction::parse(
                &trimmed,
                first == '$',
            ))),
            _ => Err(ParseErrorType::UnknownInstruction),
        }
    }
}

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

mod util {
    use super::ParseErrorType;
    use std::time::Duration;
    /// Parse a string into a `Duration`. Supported suffixes: s, ms, us.
    pub fn parse_duration(s: &str) -> Result<Duration, ParseErrorType> {
        // Split the number and the suffix
        let split_at = s
            .chars()
            .position(|c| !c.is_digit(10))
            .ok_or(ParseErrorType::MalformedInstruction)?;
        let (num, suffix) = s.split_at(split_at);
        // Parse the number
        let num = num
            .parse()
            .map_err(|_| ParseErrorType::MalformedInstruction)?;
        // Parse the suffix
        match suffix {
            "s" => Ok(Duration::from_secs(num)),
            "ms" => Ok(Duration::from_millis(num)),
            "us" => Ok(Duration::from_micros(num)),
            _ => Err(ParseErrorType::MalformedInstruction),
        }
    }
    /// Parse a `"`-wrapped string. If not wrapped, return the string as it is. Note that it is a rather loose implementation, disregarding any escape sequences.
    pub fn parse_quoted_string(s: &str) -> String {
        // FIXME: Check for escape sequences
        if s.starts_with('"') && s.ends_with('"') {
            s[1..s.len() - 1].to_string()
        } else {
            s.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ParseErrorType;
    use std::{io::BufReader, time::Duration};

    #[test]
    fn instruction_with_space() {
        use Instruction::*;
        let instructions = [
            (
                " @@width auto",
                PersistentConfig(ConfigInstruction::Width(0)),
            ),
            (
                " @height 456",
                TemporaryConfig(ConfigInstruction::Height(456)),
            ),
            (" %print", Print("print".to_string())),
            (" !marker", Marker("marker".to_string())),
            (" #comment", Empty),
            (
                " $command",
                Command(CommandInstruction::parse("command", true)),
            ),
            (
                " >continuation",
                Command(CommandInstruction::parse("continuation", false)),
            ),
            (
                "@@ width 123",
                PersistentConfig(ConfigInstruction::Width(123)),
            ),
            (
                "@ height 456",
                TemporaryConfig(ConfigInstruction::Height(456)),
            ),
            ("% print", Print("print".to_string())),
            ("! marker", Marker("marker".to_string())),
            ("# comment", Empty),
            (
                "$ command",
                Command(CommandInstruction::parse("command", true)),
            ),
            (
                "> continuation",
                Command(CommandInstruction::parse("continuation", false)),
            ),
        ];
        for (input, expected) in instructions.iter() {
            assert_eq!(&Instruction::parse(input).unwrap(), expected);
        }
    }

    #[test]
    fn instruction_without_space() {
        use Instruction::*;
        let instructions = [
            (
                "@@width auto",
                PersistentConfig(ConfigInstruction::Width(0)),
            ),
            (
                "@height 456",
                TemporaryConfig(ConfigInstruction::Height(456)),
            ),
            ("%print", Print("print".to_string())),
            ("!marker", Marker("marker".to_string())),
            ("#comment", Empty),
            (
                "$command",
                Command(CommandInstruction::parse("command", true)),
            ),
            (
                ">continuation",
                Command(CommandInstruction::parse("continuation", false)),
            ),
        ];
        for (input, expected) in instructions.iter() {
            assert_eq!(&Instruction::parse(input).unwrap(), expected);
        }
    }

    #[test]
    fn empty_instruction() {
        let empty_lines = ["", " ", "\t", "\t ", " \t", "\n", "\r\n", "# some comment"];
        for line in empty_lines.iter() {
            assert_eq!(&Instruction::parse(line).unwrap(), &Instruction::Empty);
        }
    }

    #[test]
    fn invalid_instruction() {
        let unknown_instructions = ["invalid", "&", "~"];
        for line in unknown_instructions.iter() {
            assert!(matches!(
                Instruction::parse(line).unwrap_err(),
                ParseErrorType::UnknownInstruction,
            ));
        }
        let malformed_instructions = ["@", "@@"];
        for line in malformed_instructions.iter() {
            assert!(matches!(
                Instruction::parse(line).unwrap_err(),
                ParseErrorType::MalformedInstruction,
            ));
        }
    }

    #[test]
    fn duration() {
        let durations = [
            ("1s", Duration::from_secs(1)),
            ("2ms", Duration::from_millis(2)),
            ("3us", Duration::from_micros(3)),
        ];
        for (input, expected) in durations.iter() {
            assert_eq!(util::parse_duration(input).unwrap(), *expected);
        }
    }

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
