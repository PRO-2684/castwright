//! Module for parsing empty instructions.

use super::{AsciiCast, ExecutionContext, InstructionTrait, ParseContext, ParseErrorType};

/// An empty instruction.
#[derive(Debug)]
pub struct MarkerInstruction(String);

impl InstructionTrait for MarkerInstruction {
    /// Parse a line into an `MarkerInstruction`.
    fn parse(s: &str, _context: &mut ParseContext) -> Result<Self, ParseErrorType> {
        Ok(Self(s.to_string()))
    }
    /// Execute the instruction
    fn execute(&self, _context: &mut ExecutionContext, _cast: &mut AsciiCast) {
        // TODO: Implement
    }
}
