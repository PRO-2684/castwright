//! Module for parsing marker instructions.

use super::{AsciiCast, ErrorType, ExecutionContext, Instruction, ParseContext};

/// A marker instruction.
#[derive(Debug, PartialEq)]
pub struct MarkerInstruction(String);

impl Instruction for MarkerInstruction {
    /// Parse a line into an `MarkerInstruction`.
    fn parse(s: &str, context: &mut ParseContext) -> Result<Self, ErrorType> {
        if context.expect_continuation {
            return Err(ErrorType::ExpectedContinuation);
        }
        Ok(Self(s.to_string()))
    }
    /// Execute the instruction
    fn execute(&self, context: &mut ExecutionContext, cast: &mut AsciiCast) {
        cast.marker(context.elapsed, self.0.clone());
    }
}
