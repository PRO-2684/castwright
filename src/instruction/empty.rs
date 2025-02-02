//! Module for parsing empty instructions.

use super::{AsciiCast, ErrorType, ExecutionContext, Instruction, ParseContext};

/// An empty instruction.
#[derive(Debug, PartialEq)]
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
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_instruction() {
        let mut context = ParseContext::new();
        let instruction = EmptyInstruction::parse("", &mut context).unwrap();
        assert_eq!(instruction, EmptyInstruction);

        let mut context = ExecutionContext::new();
        let mut writer = Vec::new();
        let mut cast = AsciiCast::new(&mut writer);
        instruction.execute(&mut context, &mut cast).unwrap();
        assert_eq!(context.temporary.is_empty(), true);
        assert_eq!(writer.len(), 0);
    }
}
