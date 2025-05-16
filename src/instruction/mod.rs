//! Module for parsing and executing instructions.

mod command;
mod config;
mod empty;
mod frontmatter;
mod marker;
mod print;
mod wait;

use super::{
    AsciiCast, ErrorType, ExecutionContext, FrontMatterState, ParseContext, execute_command, util,
};
pub use command::CommandInstruction;
pub use config::ConfigInstruction;
pub use empty::EmptyInstruction;
pub use frontmatter::FrontMatterInstruction;
pub use marker::MarkerInstruction;
pub use print::PrintInstruction;
pub use wait::WaitInstruction;

/// Trait for instructions.
pub trait InstructionTrait: std::fmt::Debug {
    /// Parse a line into `Self`. Remember to:
    ///
    /// Check `expect_continuation` for non-empty instructions, like:
    ///
    /// ```rust ignore
    /// if context.expect_continuation {
    ///    return Err(ErrorType::ExpectedContinuation);
    /// }
    /// ```
    ///
    /// End front matter state for non-empty non-frontmatter instructions, like:
    ///
    /// ```rust ignore
    /// context.front_matter_state.end()?;
    /// ```
    fn parse(s: &str, context: &mut ParseContext) -> Result<Self, ErrorType>
    where
        Self: Sized;
    /// Execute the instruction.
    fn execute(
        &self,
        context: &mut ExecutionContext,
        cast: &mut AsciiCast<impl std::io::Write>,
    ) -> Result<(), ErrorType>;
}

/// An instruction.
#[derive(Debug)]
pub enum Instruction {
    Config(ConfigInstruction),
    Print(PrintInstruction),
    Marker(MarkerInstruction),
    Empty(EmptyInstruction),
    Command(CommandInstruction),
    Wait(WaitInstruction),
    FrontMatter(FrontMatterInstruction),
}

impl InstructionTrait for Instruction {
    /// Parse an instruction from a string.
    fn parse(s: &str, context: &mut ParseContext) -> Result<Self, ErrorType> {
        let s = s.trim();
        let Some(first) = s.chars().next() else {
            return Ok(Self::Empty(EmptyInstruction::new()));
        };
        let trimmed = s[1..].trim().to_string();
        context.start = first;

        match first {
            '@' => Ok(Self::Config(ConfigInstruction::parse(&trimmed, context)?)),
            '%' => Ok(Self::Print(PrintInstruction::parse(&trimmed, context)?)),
            '!' => Ok(Self::Marker(MarkerInstruction::parse(&trimmed, context)?)),
            '#' => Ok(Self::Empty(EmptyInstruction::new())),
            '$' | '>' => Ok(Self::Command(CommandInstruction::parse(&trimmed, context)?)),
            '~' => Ok(Self::Wait(WaitInstruction::parse(&trimmed, context)?)),
            _ => Ok(Self::FrontMatter(FrontMatterInstruction::parse(
                s, context,
            )?)),
        }
    }

    /// Execute the instruction
    fn execute(
        &self,
        context: &mut ExecutionContext,
        cast: &mut AsciiCast<impl std::io::Write>,
    ) -> Result<(), ErrorType> {
        match self {
            Self::Config(instruction) => instruction.execute(context, cast),
            Self::Print(instruction) => instruction.execute(context, cast),
            Self::Marker(instruction) => instruction.execute(context, cast),
            Self::Empty(instruction) => instruction.execute(context, cast),
            Self::Command(instruction) => instruction.execute(context, cast),
            Self::Wait(instruction) => instruction.execute(context, cast),
            Self::FrontMatter(instruction) => instruction.execute(context, cast),
        }
    }
}

#[cfg(test)]
impl PartialEq for Instruction {
    fn eq(&self, other: &Self) -> bool {
        // Compare the debug representations of the instructions.
        // This is a rather crude way to compare instructions, but is acceptable since it is only used in tests.
        format!("{self:?}") == format!("{other:?}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instruction_with_space() {
        let mut context = ParseContext::new();
        let instructions: [(&str, Instruction); 10] = [
            (
                " @interval 2ms",
                Instruction::Config(
                    ConfigInstruction::parse("interval 2ms", &mut context).unwrap(),
                ),
            ),
            (
                " %print",
                Instruction::Print(PrintInstruction::parse("print", &mut context).unwrap()),
            ),
            (
                " !marker",
                Instruction::Marker(MarkerInstruction::parse("marker", &mut context).unwrap()),
            ),
            (" #comment", Instruction::Empty(EmptyInstruction::new())),
            (
                " $echo \"Hello, World!\"",
                Instruction::Command(
                    CommandInstruction::parse(
                        "echo \"Hello, World!\"",
                        &mut context.with_start('$'),
                    )
                    .unwrap(),
                ),
            ),
            (
                " @ interval 2ms",
                Instruction::Config(
                    ConfigInstruction::parse("interval 2ms", &mut context).unwrap(),
                ),
            ),
            (
                "% print",
                Instruction::Print(PrintInstruction::parse("print", &mut context).unwrap()),
            ),
            (
                "! marker",
                Instruction::Marker(MarkerInstruction::parse("marker", &mut context).unwrap()),
            ),
            ("# comment", Instruction::Empty(EmptyInstruction::new())),
            (
                "$ echo \"Hello, World!\"",
                Instruction::Command(
                    CommandInstruction::parse(
                        "echo \"Hello, World!\"",
                        &mut context.with_start('$'),
                    )
                    .unwrap(),
                ),
            ),
        ];
        for (input, expected) in &instructions {
            assert_eq!(&Instruction::parse(input, &mut context).unwrap(), expected);
        }
    }

    #[test]
    fn instruction_without_space() {
        let mut context = ParseContext::new();
        let instructions: [(&str, Instruction); 5] = [
            (
                "@interval 2ms",
                Instruction::Config(
                    ConfigInstruction::parse("interval 2ms", &mut context).unwrap(),
                ),
            ),
            (
                "%print",
                Instruction::Print(PrintInstruction::parse("print", &mut context).unwrap()),
            ),
            (
                "!marker",
                Instruction::Marker(MarkerInstruction::parse("marker", &mut context).unwrap()),
            ),
            ("#comment", Instruction::Empty(EmptyInstruction::new())),
            (
                "$echo \"Hello, World!\"",
                Instruction::Command(
                    CommandInstruction::parse(
                        "echo \"Hello, World!\"",
                        &mut context.with_start('$'),
                    )
                    .unwrap(),
                ),
            ),
        ];
        for (input, expected) in &instructions {
            assert_eq!(&Instruction::parse(input, &mut context).unwrap(), expected);
        }
    }

    #[test]
    fn empty_instruction() {
        let empty_lines = ["", " ", "\t", "\t ", " \t", "\n", "\r\n", "# some comment"];
        let mut context = ParseContext::new();
        let expected = Instruction::Empty(EmptyInstruction::new());
        for line in &empty_lines {
            assert_eq!(&Instruction::parse(line, &mut context).unwrap(), &expected);
        }
    }

    #[test]
    fn invalid_instruction() {
        let unknown_instructions = ["invalid", "&", "^"];
        let mut context = ParseContext::new();
        for line in &unknown_instructions {
            let err = Instruction::parse(line, &mut context).unwrap_err();
            assert!(
                matches!(err, ErrorType::UnknownInstruction,),
                "Expected UnknownInstruction, got {err:?}"
            );
        }
        let malformed_instructions = ["@", "@@"];
        for line in &malformed_instructions {
            let err = Instruction::parse(line, &mut context).unwrap_err();
            assert!(
                matches!(err, ErrorType::MalformedInstruction,),
                "Expected MalformedInstruction, got {err:?}"
            );
        }
    }
}
