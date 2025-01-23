//! Module for parsing empty instructions.

use super::{AsciiCast, ExecutionContext, InstructionTrait, ParseContext, ParseErrorType};

/// An empty instruction.
#[derive(Debug)]
pub struct PrintInstruction(String);

impl InstructionTrait for PrintInstruction {
    /// Parse a line into an `PrintInstruction`.
    fn parse(s: &str, _context: &mut ParseContext) -> Result<Self, ParseErrorType> {
        Ok(Self(s.to_string()))
    }
    /// Execute the instruction
    fn execute(&self, _context: &mut ExecutionContext, _cast: &mut AsciiCast) {
        // TODO: Implement
    }
}
