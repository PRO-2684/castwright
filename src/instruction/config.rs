//! Module for parsing config instructions.

use super::{util, AsciiCast, ErrorType, ExecutionContext, Instruction, ParseContext};

/// A configuration instruction type.
#[derive(Debug, PartialEq)]
enum ConfigInstructionType {
    // Configuration that applies to other instructions (directive)
    /// The shell prompt to use in the asciicast.
    Prompt(String),
    /// The secondary prompt to use in the asciicast (for continuation lines).
    SecondaryPrompt(String),
    /// The string to signify a line continuation in a multiline command.
    LineContinuation(String),
    /// Whether the command should be executed silently.
    Hidden(bool),
    /// Typing interval between characters in a command or print instruction, in microseconds (µs).
    Interval(u64),
    /// The start lag in microseconds (µs). i.e. Additional delay after displaying the prompt, before printing the command for command instructions, or before printing the content for print instructions.
    StartLag(u64),
    /// The end lag in microseconds (µs). i.e. Additional delay after printing the command for command instructions, or after printing the content for print instructions.
    EndLag(u64),
}

/// A configuration instruction.
#[derive(Debug, PartialEq)]
pub struct ConfigInstruction {
    instruction_type: ConfigInstructionType,
    persistent: bool,
}

impl Instruction for ConfigInstruction {
    /// Parse a line into a `ConfigInstruction`.
    fn parse(s: &str, context: &mut ParseContext) -> Result<Self, ErrorType> {
        context.front_matter_state.end()?;
        if context.expect_continuation {
            return Err(ErrorType::ExpectedContinuation);
        }
        let s = s.trim();
        // The first character ('@') has been removed, thus the check is for the second character
        let persistent = s.starts_with("@");
        let s = if persistent { &s[1..] } else { s }; // Remove the '@' if it's present
        let mut iter = s.split_whitespace();
        let Some(first) = iter.next() else {
            return Err(ErrorType::MalformedInstruction);
        };
        let len = first.len();
        let instruction_type = match first {
            "prompt" => {
                let prompt = util::parse_loose_string(s[len..].trim())?;
                Ok(ConfigInstructionType::Prompt(prompt))
            }
            "secondary" | "secondary-prompt" => {
                let prompt = util::parse_loose_string(s[len..].trim())?;
                Ok(ConfigInstructionType::SecondaryPrompt(prompt))
            }
            "continuation" | "line-continuation" => {
                let split = util::parse_loose_string(s[len..].trim())?;
                Ok(ConfigInstructionType::LineContinuation(split))
            }
            "hidden" => {
                let hidden = iter.next();
                if let Some(word) = hidden {
                    match word {
                        "true" => Ok(ConfigInstructionType::Hidden(true)),
                        "false" => Ok(ConfigInstructionType::Hidden(false)),
                        _ => Err(ErrorType::MalformedInstruction),
                    }
                } else {
                    Ok(ConfigInstructionType::Hidden(true)) // Default to true
                }
            }
            "interval" => {
                let interval = iter.next().ok_or(ErrorType::MalformedInstruction)?;
                Ok(ConfigInstructionType::Interval(
                    util::parse_duration(interval)?.as_micros() as u64,
                ))
            }
            "start-lag" => {
                let delay = iter.next().ok_or(ErrorType::MalformedInstruction)?;
                Ok(ConfigInstructionType::StartLag(
                    util::parse_duration(delay)?.as_micros() as u64,
                ))
            }
            "end-lag" => {
                let delay = iter.next().ok_or(ErrorType::MalformedInstruction)?;
                Ok(ConfigInstructionType::EndLag(
                    util::parse_duration(delay)?.as_micros() as u64,
                ))
            }
            _ => Err(ErrorType::UnknownConfig),
        }?;
        Ok(Self {
            instruction_type,
            persistent,
        })
    }
    /// Execute the configuration instruction.
    fn execute(
        &self,
        context: &mut ExecutionContext,
        _cast: &mut AsciiCast,
    ) -> Result<(), ErrorType> {
        use ConfigInstructionType::*;
        // Modify the configuration
        if self.persistent {
            let config = &mut context.persistent;
            match &self.instruction_type {
                Prompt(prompt) => config.prompt = prompt.clone(),
                SecondaryPrompt(secondary_prompt) => {
                    config.secondary_prompt = secondary_prompt.clone()
                }
                LineContinuation(line_continuation) => {
                    config.line_continuation = line_continuation.clone()
                }
                Hidden(hidden) => config.hidden = *hidden,
                Interval(interval) => config.interval = *interval,
                StartLag(delay) => config.start_lag = *delay,
                EndLag(delay) => config.end_lag = *delay,
            }
        } else {
            let config = &mut context.temporary;
            match &self.instruction_type {
                Prompt(prompt) => config.prompt = Some(prompt.clone()),
                SecondaryPrompt(secondary_prompt) => {
                    config.secondary_prompt = Some(secondary_prompt.clone())
                }
                LineContinuation(line_continuation) => {
                    config.line_continuation = Some(line_continuation.clone())
                }
                Hidden(hidden) => config.hidden = Some(*hidden),
                Interval(interval) => config.interval = Some(*interval),
                StartLag(delay) => config.start_lag = Some(*delay),
                EndLag(delay) => config.end_lag = Some(*delay),
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{ConfigInstructionType::*, *};

    #[test]
    fn config_instruction_type() {
        let mut context = ParseContext::new();
        let instructions = [
            ("@prompt \"$ \"", Prompt("$ ".to_string())),
            ("@secondary \"> \"", SecondaryPrompt("> ".to_string())),
            (
                "@secondary-prompt \"> \"",
                SecondaryPrompt("> ".to_string()),
            ),
            ("@continuation \\", LineContinuation("\\".to_string())),
            ("@line-continuation \\", LineContinuation("\\".to_string())),
            ("@hidden true", Hidden(true)),
            ("@hidden false", Hidden(false)),
            ("@interval 2ms", Interval(2_000)),
            ("@start-lag 1s", StartLag(1_000_000)),
            ("@end-lag 1s", EndLag(1_000_000)),
        ];
        for (line, expected) in instructions.iter() {
            assert_eq!(
                ConfigInstruction::parse(line, &mut context)
                    .unwrap()
                    .instruction_type,
                *expected
            );
        }
    }

    #[test]
    fn config_instruction_persistent() {
        let mut context = ParseContext::new();
        let instructions = [
            ("@prompt \"$ \"", true),
            ("secondary \"> \"", false),
            ("continuation \\", false),
            ("hidden true", false),
            ("interval 2ms", false),
            ("@start-lag 1s", true),
        ];
        for (line, expected) in instructions.iter() {
            assert_eq!(
                ConfigInstruction::parse(line, &mut context)
                    .unwrap()
                    .persistent,
                *expected
            );
        }
    }

    #[test]
    fn malformed_config_instruction() {
        let mut context = ParseContext::new();
        let malformed_instructions = [
            "hidden what",
            "interval",
            "interval 2",
            "start-lag",
            "start-lag 1",
        ];
        for line in malformed_instructions.iter() {
            let parsed = ConfigInstruction::parse(line, &mut context).unwrap_err();
            assert!(
                matches!(parsed, ErrorType::MalformedInstruction,),
                "Expected MalformedInstruction, got {parsed:?} at line `{line}`"
            );
        }
    }

    #[test]
    fn unknown_config_instruction() {
        let mut context = ParseContext::new();
        let unknown_instructions = [
            "invalid",
            "width 123",
            "@height 456",
            "title CastWright demo",
            "shell bash",
            "quit \"exit \"",
            "idle 1s",
        ];
        for line in unknown_instructions.iter() {
            let parsed = ConfigInstruction::parse(line, &mut context).unwrap_err();
            assert!(
                matches!(parsed, ErrorType::UnknownConfig,),
                "Expected UnknownConfig, got {parsed:?} at line `{line}`"
            );
        }
    }

    #[test]
    fn execute_config_instruction() {
        let mut parse_context = ParseContext::new();
        let mut context = ExecutionContext::new();
        let mut sink = std::io::sink(); // Drop all output
        let mut cast = AsciiCast::new(&mut sink);
        let instructions = [
            "prompt \"~> \"",
            "secondary \"| \"",
            "continuation \\",
            "hidden",
            "interval 2ms",
        ];
        for line in instructions.iter() {
            ConfigInstruction::parse(line, &mut parse_context)
                .unwrap()
                .execute(&mut context, &mut cast)
                .unwrap();
        }
        let resolved = context.consume_temporary();
        assert_eq!(resolved.prompt, "~> ".to_string());
        assert_eq!(resolved.secondary_prompt, "| ".to_string());
        assert_eq!(resolved.line_continuation, "\\".to_string());
        assert!(resolved.hidden);
        assert_eq!(resolved.interval, 2_000);
    }
}
