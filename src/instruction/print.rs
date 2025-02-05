//! Module for parsing print instructions.

use super::{util, AsciiCast, ErrorType, ExecutionContext, Instruction, ParseContext};

/// A print instruction.
#[derive(Debug, PartialEq)]
pub struct PrintInstruction(String);

impl Instruction for PrintInstruction {
    /// Parse a line into an `PrintInstruction`.
    fn parse(s: &str, context: &mut ParseContext) -> Result<Self, ErrorType> {
        context.front_matter_state.end()?;
        if context.expect_continuation {
            return Err(ErrorType::ExpectedContinuation);
        }
        let content = util::parse_loose_string(s)?;
        Ok(Self(content))
    }
    /// Execute the instruction
    fn execute(
        &self,
        context: &mut ExecutionContext,
        cast: &mut AsciiCast,
    ) -> Result<(), ErrorType> {
        let config = context.persistent.combine(context.temporary.get(true));
        context.elapsed += config.start_lag;
        let interval = config.interval;
        for character in self.0.chars() {
            context.elapsed += interval;
            cast.output(context.elapsed, character.encode_utf8(&mut [0u8; 4]))?;
        }
        context.preview(&self.0);
        context.elapsed += interval;
        context.elapsed += config.end_lag;
        cast.output(context.elapsed, "\r\n")?;
        context.preview("\r\n");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_instruction() {
        let s = "Hello, world!";
        let mut context = ParseContext::new();
        let instruction = PrintInstruction::parse(s, &mut context).unwrap();
        assert_eq!(instruction.0, s);

        let mut context = ExecutionContext::new();
        let mut writer = Vec::new();
        instruction
            .execute(&mut context, &mut AsciiCast::new(&mut writer))
            .unwrap();

        assert!(context.temporary.is_empty());
        let output = String::from_utf8_lossy(&writer);
        let lines_after = output.lines().skip(1).collect::<Vec<_>>();
        let expected = vec![
            r#"[0.100000,"o","H"]"#,
            r#"[0.200000,"o","e"]"#,
            r#"[0.300000,"o","l"]"#,
            r#"[0.400000,"o","l"]"#,
            r#"[0.500000,"o","o"]"#,
            r#"[0.600000,"o",","]"#,
            r#"[0.700000,"o"," "]"#,
            r#"[0.800000,"o","w"]"#,
            r#"[0.900000,"o","o"]"#,
            r#"[1.000000,"o","r"]"#,
            r#"[1.100000,"o","l"]"#,
            r#"[1.200000,"o","d"]"#,
            r#"[1.300000,"o","!"]"#,
            r#"[1.400000,"o","\r\n"]"#,
        ];
        assert_eq!(lines_after, expected);
    }
}
