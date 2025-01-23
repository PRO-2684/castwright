//! Module for parsing instructions.

mod command;
mod config;

use super::{util, AsciiCast, ExecutionContext, ParseContext, ParseErrorType};
pub use command::CommandInstruction;
pub use config::ConfigInstruction;

/// A single instruction
#[derive(Debug, PartialEq)]
pub enum Instruction {
    /// A configuration instruction. (`@` or `@@`)
    Config(ConfigInstruction),
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
            '@' => Ok(Self::Config(ConfigInstruction::parse(&trimmed)?)),
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
    /// Execute the instruction
    pub fn execute(&self, context: &mut ExecutionContext, cast: &mut AsciiCast) {
        match self {
            Self::Config(instruction) => instruction.execute(context),
            Self::Print(s) => {
                // TODO: Implement
                cast.push(format!("print: {}", s));
            }
            Self::Marker(s) => {
                // TODO: Implement
                cast.push(format!("marker: {}", s));
            }
            Self::Command(instruction) => instruction.execute(context, cast),
            Self::Empty => {}
        }
    }
}

pub trait InstructionTrait {
    /// Parse a line into `Self`.
    fn parse(s: &str, context: &mut ParseContext) -> Result<Self, ParseErrorType>
    where
        Self: Sized;
    /// Execute the instruction.
    fn execute(&self, context: &mut ExecutionContext, cast: &mut AsciiCast);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ParseErrorType;
    use std::time::Duration;

    #[test]
    fn instruction_with_space() {
        use Instruction::*;
        let instructions = [
            (
                " @@width auto",
                Config(ConfigInstruction::parse("@width auto").unwrap()),
            ),
            (
                " @delay 2ms",
                Config(ConfigInstruction::parse("delay 2ms").unwrap()),
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
                Config(ConfigInstruction::parse("@width 123").unwrap()),
            ),
            (
                "@ delay 2ms",
                Config(ConfigInstruction::parse("delay 2ms").unwrap()),
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
                Config(ConfigInstruction::parse("@width auto").unwrap()),
            ),
            (
                "@delay 2ms",
                Config(ConfigInstruction::parse("delay 2ms").unwrap()),
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
}
