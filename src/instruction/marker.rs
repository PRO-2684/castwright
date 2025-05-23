//! Module for marker instructions.

use super::{AsciiCast, ErrorType, ExecutionContext, InstructionTrait, ParseContext};

/// A marker instruction.
#[derive(Debug, PartialEq, Eq)]
pub struct MarkerInstruction(String);

impl InstructionTrait for MarkerInstruction {
    /// Parse a trimmed line into an `MarkerInstruction`.
    fn parse(s: &str, context: &mut ParseContext) -> Result<Self, ErrorType> {
        context.front_matter_state.end()?;
        if context.expect_continuation {
            return Err(ErrorType::ExpectedContinuation);
        }

        Ok(Self(s.to_string()))
    }
    /// Execute the instruction
    fn execute(
        &self,
        context: &mut ExecutionContext,
        cast: &mut AsciiCast<impl std::io::Write>,
    ) -> Result<(), ErrorType> {
        cast.marker(context.elapsed, &self.0)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker_instruction() {
        let s = "I'm a marker";
        let mut context = ParseContext::new();
        let instruction = MarkerInstruction::parse(s, &mut context).unwrap();
        assert_eq!(instruction.0, s);

        let mut context = ExecutionContext::new();
        let mut writer = Vec::new();
        instruction
            .execute(&mut context, &mut AsciiCast::new(&mut writer))
            .unwrap();

        assert_eq!(context.elapsed, 0);
        let output = String::from_utf8_lossy(&writer);
        let second_line = output.lines().nth(1).unwrap();
        assert_eq!(second_line, r#"[0.000000,"m","I'm a marker"]"#);
    }
}
