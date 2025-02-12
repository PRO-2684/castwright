//! Module for empty instructions.

use super::{AsciiCast, ErrorType, ExecutionContext, Instruction, ParseContext};

/// An empty instruction.
#[derive(Debug, PartialEq, Eq)]
pub struct EmptyInstruction;

impl Instruction for EmptyInstruction {
    /// Parse a line into an `EmptyInstruction`.
    fn parse(_s: &str, _context: &mut ParseContext) -> Result<Self, ErrorType> {
        Ok(Self)
    }
    /// Execute the instruction
    fn execute(
        &self,
        _context: &mut ExecutionContext,
        _cast: &mut AsciiCast,
    ) -> Result<(), ErrorType> {
        // Do nothing
        Ok(())
    }
}

impl EmptyInstruction {
    /// Create a new `EmptyInstruction`.
    pub const fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufRead;

    #[test]
    fn empty_instruction() {
        let mut context = ParseContext::new();
        let instruction = EmptyInstruction::parse("", &mut context).unwrap();
        assert_eq!(instruction, EmptyInstruction);

        let mut context = ExecutionContext::new();
        let mut writer = Vec::new();
        instruction
            .execute(&mut context, &mut AsciiCast::new(&mut writer))
            .unwrap();

        assert!(context.temporary.is_empty());
        let lines = writer.lines().collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(lines.len(), 1); // Only the header
    }
}
