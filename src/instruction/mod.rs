//! Module for parsing instructions.

mod command;
mod config;
mod empty;
mod frontmatter;
mod marker;
mod print;
mod wait;

use super::{
    execute_command, util, AsciiCast, ErrorType, ExecutionContext, FrontMatterState, ParseContext,
};
pub(super) use command::CommandInstruction;
pub(super) use config::ConfigInstruction;
pub(super) use empty::EmptyInstruction;
pub(super) use frontmatter::FrontMatterInstruction;
pub(super) use marker::MarkerInstruction;
pub(super) use print::PrintInstruction;
pub(super) use wait::WaitInstruction;

/// Trait for instructions.
pub(super) trait Instruction: std::fmt::Debug {
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
        cast: &mut AsciiCast,
    ) -> Result<(), ErrorType>;
}

/// Parse an instruction from a string.
pub(super) fn parse_instruction(
    s: &str,
    context: &mut ParseContext,
) -> Result<Box<dyn Instruction>, ErrorType> {
    let s = s.trim();
    let Some(first) = s.chars().next() else {
        return Ok(Box::new(EmptyInstruction::new()));
    };
    let trimmed = s[1..].trim().to_string();
    context.start = first;
    match first {
        '@' => Ok(Box::new(ConfigInstruction::parse(&trimmed, context)?)),
        '%' => Ok(Box::new(PrintInstruction::parse(&trimmed, context)?)),
        '!' => Ok(Box::new(MarkerInstruction::parse(&trimmed, context)?)),
        '#' => Ok(Box::new(EmptyInstruction::new())),
        '$' | '>' => Ok(Box::new(CommandInstruction::parse(&trimmed, context)?)),
        '~' => Ok(Box::new(WaitInstruction::parse(&trimmed, context)?)),
        // _ => Err(ErrorType::UnknownInstruction),
        _ => Ok(Box::new(FrontMatterInstruction::parse(s, context)?)),
    }
}

#[cfg(test)]
impl PartialEq for dyn Instruction {
    fn eq(&self, other: &Self) -> bool {
        // Compare the debug representations of the instructions.
        // This is a rather crude way to compare instructions, but is acceptable since it is only used in tests.
        format!("{:?}", self) == format!("{:?}", other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ErrorType;

    #[test]
    fn instruction_with_space() {
        let mut context = ParseContext::new();
        let instructions: [(&str, Box<dyn Instruction>); 10] = [
            (
                " @interval 2ms",
                Box::new(ConfigInstruction::parse("interval 2ms", &mut context).unwrap()),
            ),
            (
                " %print",
                Box::new(PrintInstruction::parse("print", &mut context).unwrap()),
            ),
            (
                " !marker",
                Box::new(MarkerInstruction::parse("marker", &mut context).unwrap()),
            ),
            (" #comment", Box::new(EmptyInstruction::new())),
            (
                " $echo \"Hello, World!\"",
                Box::new(
                    CommandInstruction::parse(
                        "echo \"Hello, World!\"",
                        &mut context.with_start('$'),
                    )
                    .unwrap(),
                ),
            ),
            (
                " @ interval 2ms",
                Box::new(ConfigInstruction::parse("interval 2ms", &mut context).unwrap()),
            ),
            (
                "% print",
                Box::new(PrintInstruction::parse("print", &mut context).unwrap()),
            ),
            (
                "! marker",
                Box::new(MarkerInstruction::parse("marker", &mut context).unwrap()),
            ),
            ("# comment", Box::new(EmptyInstruction::new())),
            (
                "$ echo \"Hello, World!\"",
                Box::new(
                    CommandInstruction::parse(
                        "echo \"Hello, World!\"",
                        &mut context.with_start('$'),
                    )
                    .unwrap(),
                ),
            ),
        ];
        for (input, expected) in instructions.iter() {
            assert_eq!(&parse_instruction(input, &mut context).unwrap(), expected);
        }
    }

    #[test]
    fn instruction_without_space() {
        let mut context = ParseContext::new();
        let instructions: [(&str, Box<dyn Instruction>); 5] = [
            (
                "@interval 2ms",
                Box::new(ConfigInstruction::parse("interval 2ms", &mut context).unwrap()),
            ),
            (
                "%print",
                Box::new(PrintInstruction::parse("print", &mut context).unwrap()),
            ),
            (
                "!marker",
                Box::new(MarkerInstruction::parse("marker", &mut context).unwrap()),
            ),
            ("#comment", Box::new(EmptyInstruction::new())),
            (
                "$echo \"Hello, World!\"",
                Box::new(
                    CommandInstruction::parse(
                        "echo \"Hello, World!\"",
                        &mut context.with_start('$'),
                    )
                    .unwrap(),
                ),
            ),
        ];
        for (input, expected) in instructions.iter() {
            assert_eq!(&parse_instruction(input, &mut context).unwrap(), expected);
        }
    }

    #[test]
    fn empty_instruction() {
        let empty_lines = ["", " ", "\t", "\t ", " \t", "\n", "\r\n", "# some comment"];
        let mut context = ParseContext::new();
        let expected: Box<dyn Instruction> = Box::new(EmptyInstruction::new());
        for line in empty_lines.iter() {
            assert_eq!(&parse_instruction(line, &mut context).unwrap(), &expected);
        }
    }

    #[test]
    fn invalid_instruction() {
        let unknown_instructions = ["invalid", "&", "^"];
        let mut context = ParseContext::new();
        for line in unknown_instructions.iter() {
            assert!(matches!(
                parse_instruction(line, &mut context).unwrap_err(),
                ErrorType::UnknownInstruction,
            ));
        }
        let malformed_instructions = ["@", "@@"];
        for line in malformed_instructions.iter() {
            assert!(matches!(
                parse_instruction(line, &mut context).unwrap_err(),
                ErrorType::MalformedInstruction,
            ));
        }
    }
}
