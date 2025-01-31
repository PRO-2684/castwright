//! Module for parsing print instructions.

use super::{util, AsciiCast, ErrorType, ExecutionContext, Instruction, ParseContext};

/// A print instruction.
#[derive(Debug, PartialEq)]
pub struct PrintInstruction(String);

impl Instruction for PrintInstruction {
    /// Parse a line into an `PrintInstruction`.
    fn parse(s: &str, context: &mut ParseContext) -> Result<Self, ErrorType> {
        context.front_matter_state.end()?;
        if context.expect_continuation {
            return Err(ErrorType::ExpectedContinuation);
        }
        let content = util::parse_loose_string(s)?;
        Ok(Self(content))
    }
    /// Execute the instruction
    fn execute(
        &self,
        context: &mut ExecutionContext,
        cast: &mut AsciiCast,
    ) -> Result<(), ErrorType> {
        let config = if context.has_temporary() {
            &context.consume_temporary()
        } else {
            &context.persistent
        };
        let interval = config.interval;
        for character in self.0.chars() {
            context.elapsed += interval;
            cast.output(context.elapsed, character.encode_utf8(&mut [0u8; 4]))?;
        }
        context.preview(&self.0);
        context.elapsed += interval;
        cast.output(context.elapsed, "\r\n")?;
        context.preview("\r\n");
        Ok(())
    }
}
