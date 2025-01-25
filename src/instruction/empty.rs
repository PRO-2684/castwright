//! Module for parsing empty instructions.

use super::{AsciiCast, ErrorType, ExecutionContext, Instruction, ParseContext};

/// An empty instruction.
#[derive(Debug)]
pub struct EmptyInstruction;

impl Instruction for EmptyInstruction {
    /// Parse a line into an `EmptyInstruction`.
    fn parse(_s: &str, _context: &mut ParseContext) -> Result<Self, ErrorType> {
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
