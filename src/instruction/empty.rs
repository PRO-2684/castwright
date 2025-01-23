//! Module for parsing empty instructions.

use super::{AsciiCast, ExecutionContext, InstructionTrait, ParseContext, ParseErrorType};

/// An empty instruction.
#[derive(Debug)]
pub struct EmptyInstruction;

impl InstructionTrait for EmptyInstruction {
    /// Parse a line into an `EmptyInstruction`.
    fn parse(_s: &str, _context: &mut ParseContext) -> Result<Self, ParseErrorType> {
        Ok(Self)
    }
    /// Execute the instruction
    fn execute(&self, _context: &mut ExecutionContext, _cast: &mut AsciiCast) {
        // Do nothing
    }
}

impl EmptyInstruction {
    /// Create a new `EmptyInstruction`.
    pub fn new() -> Self {
        Self
    }
}
