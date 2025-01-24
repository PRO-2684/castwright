//! Module for parsing empty instructions.

use super::{AsciiCast, ExecutionContext, InstructionTrait, ParseContext, ParseErrorType};

/// An empty instruction.
#[derive(Debug, PartialEq)]
pub struct MarkerInstruction(String);

impl InstructionTrait for MarkerInstruction {
    /// Parse a line into an `MarkerInstruction`.
    fn parse(s: &str, context: &mut ParseContext) -> Result<Self, ParseErrorType> {
        if context.expect_continuation {
            return Err(ParseErrorType::ExpectedContinuation);
        }
        Ok(Self(s.to_string()))
    }
    /// Execute the instruction
    fn execute(&self, context: &mut ExecutionContext, cast: &mut AsciiCast) {
        cast.marker(context.elapsed, self.0.clone());
    }
}
