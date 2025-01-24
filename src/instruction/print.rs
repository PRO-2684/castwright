//! Module for parsing empty instructions.

use super::{AsciiCast, ExecutionContext, InstructionTrait, ParseContext, ParseErrorType};

/// An empty instruction.
#[derive(Debug, PartialEq)]
pub struct PrintInstruction(String);

impl InstructionTrait for PrintInstruction {
    /// Parse a line into an `PrintInstruction`.
    fn parse(s: &str, context: &mut ParseContext) -> Result<Self, ParseErrorType> {
        if context.expect_continuation {
            return Err(ParseErrorType::ExpectedContinuation);
        }
        Ok(Self(s.to_string()))
    }
    /// Execute the instruction
    fn execute(&self, context: &mut ExecutionContext, cast: &mut AsciiCast) {
        let config = if context.has_temporary() {
            &context.consume_temporary()
        } else {
            &context.persistent
        };
        let delay = config.delay.as_millis() as u64;
        for character in self.0.chars() {
            context.elapsed += delay;
            cast.output(context.elapsed, character.to_string());
        }
    }
}
