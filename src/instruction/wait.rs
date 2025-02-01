//! Module for parsing wait instructions.

use super::{util, AsciiCast, ErrorType, ExecutionContext, Instruction, ParseContext};

/// A wait instruction.
#[derive(Debug, PartialEq)]
pub struct WaitInstruction(u64);

impl Instruction for WaitInstruction {
    /// Parse a line into an `WaitInstruction`.
    fn parse(s: &str, context: &mut ParseContext) -> Result<Self, ErrorType> {
        context.front_matter_state.end()?;
        if context.expect_continuation {
            return Err(ErrorType::ExpectedContinuation);
        }
        let time = util::parse_duration(s)?;
        Ok(Self(time.as_micros() as u64))
    }
    /// Execute the instruction
    fn execute(
        &self,
        context: &mut ExecutionContext,
        _cast: &mut AsciiCast,
    ) -> Result<(), ErrorType> {
        context.elapsed += self.0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wait_instruction() {
        let s = "1s";
        let mut context = ParseContext::new();
        let instruction = WaitInstruction::parse(s, &mut context).unwrap();
        assert_eq!(instruction.0, 1_000_000);

        let mut context = ExecutionContext::new();
        let mut writer = Vec::new();
        let mut cast = AsciiCast::new(&mut writer);
        instruction.execute(&mut context, &mut cast).unwrap();
        assert_eq!(context.elapsed, 1_000_000);
    }
}
