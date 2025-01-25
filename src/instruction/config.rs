//! Module for parsing config instructions.

use super::{util, AsciiCast, ErrorType, ExecutionContext, Instruction, ParseContext};
use std::time::Duration;

/// A configuration instruction type.
#[derive(Debug, PartialEq)]
enum ConfigInstructionType {
    // Configuration that applies to other instructions (directive)
    /// The shell prompt to use in the asciicast.
    Prompt(String),
    /// The secondary prompt to use in the asciicast (for continuation lines).
    SecondaryPrompt(String),
    /// The string to signify a line split in a multiline command.
    LineSplit(String),
    /// Whether the command should be executed silently.
    Hidden(bool),
    /// Typing delay between characters in a command or print instruction.
    Delay(Duration),
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
        let instruction_type = match first {
            "prompt" => {
                let prompt = util::parse_loose_string(s[6..].trim())?;
                Ok(ConfigInstructionType::Prompt(prompt))
            }
            "secondary-prompt" => {
                let prompt = util::parse_loose_string(s[16..].trim())?;
                Ok(ConfigInstructionType::SecondaryPrompt(prompt))
            }
            "line-split" => {
                let split = util::parse_loose_string(s[10..].trim())?;
                Ok(ConfigInstructionType::LineSplit(split))
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
            "delay" => {
                let delay = iter.next().ok_or(ErrorType::MalformedInstruction)?;
                Ok(ConfigInstructionType::Delay(util::parse_duration(delay)?))
            }
            _ => Err(ErrorType::MalformedInstruction),
        }?;
        Ok(Self {
            instruction_type,
            persistent,
        })
    }
    /// Execute the configuration instruction.
    fn execute(&self, context: &mut ExecutionContext, _cast: &mut AsciiCast) {
        use ConfigInstructionType::*;
        // Modify the configuration
        if self.persistent {
            let config = &mut context.persistent;
            match &self.instruction_type {
                // Width(width) => config.width = *width,
                // Height(height) => config.height = *height,
                // Title(title) => config.title = title.clone(),
                // Shell(shell) => config.shell = shell.clone(),
                // Quit(quit) => config.quit = quit.clone(),
                // Idle(idle) => config.idle = *idle,
                Prompt(prompt) => config.prompt = prompt.clone(),
                SecondaryPrompt(secondary_prompt) => {
                    config.secondary_prompt = secondary_prompt.clone()
                }
                LineSplit(line_split) => config.line_split = line_split.clone(),
                Hidden(hidden) => config.hidden = *hidden,
                Delay(delay) => config.delay = *delay,
            }
        } else {
            let config = &mut context.temporary;
            match &self.instruction_type {
                Prompt(prompt) => config.prompt = Some(prompt.clone()),
                SecondaryPrompt(secondary_prompt) => {
                    config.secondary_prompt = Some(secondary_prompt.clone())
                }
                LineSplit(line_split) => config.line_split = Some(line_split.clone()),
                Hidden(hidden) => config.hidden = Some(*hidden),
                Delay(delay) => config.delay = Some(*delay),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ConfigInstructionType::*, *};
    use std::time::Duration;

    #[test]
    fn config_instruction_type() {
        let mut context = ParseContext::new();
        let instructions = [
            // ("@width 123", Width(123)),
            // ("@height 456", Height(456)),
            // (
            //     "@title castwright demo",
            //     Title("castwright demo".to_string()),
            // ),
            // ("@shell bash ", Shell("bash".to_string())),
            // ("@quit \"exit \"", Quit("exit ".to_string())),
            // ("@idle 1s", Idle(Duration::from_secs(1))),
            ("@prompt \"$ \"", Prompt("$ ".to_string())),
            (
                "@secondary-prompt \"> \"",
                SecondaryPrompt("> ".to_string()),
            ),
            ("@line-split \\", LineSplit("\\".to_string())),
            ("@hidden true", Hidden(true)),
            ("@hidden false", Hidden(false)),
            ("@delay 2ms", Delay(Duration::from_millis(2))),
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
            // ("@width 123", true),
            // ("@height 456", true),
            // ("@title castwright demo", true),
            // ("@shell bash ", true),
            // ("@quit \"exit \"", true),
            // ("@idle 1s", true),
            ("@prompt \"$ \"", true),
            ("secondary-prompt \"> \"", false),
            ("line-split \\", false),
            ("hidden true", false),
            ("delay 2ms", false),
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
            "invalid",
            "@width",
            "@width -1",
            "@width what",
            "hidden what",
            "delay",
            "delay 2",
        ];
        for line in malformed_instructions.iter() {
            assert!(matches!(
                ConfigInstruction::parse(line, &mut context).unwrap_err(),
                ErrorType::MalformedInstruction,
            ));
        }
    }

    #[test]
    fn meaningless_temporary_config_instruction() {
        let mut context = ParseContext::new();
        let meaningless_instructions = [
            "width 123",
            "height 456",
            "title castwright demo",
            "shell bash",
            "quit \"exit \"",
            "idle 1s",
        ];
        for line in meaningless_instructions.iter() {
            assert!(matches!(
                ConfigInstruction::parse(line, &mut context).unwrap_err(),
                ErrorType::MalformedInstruction,
            ));
        }
    }

    #[test]
    fn execute_config_instruction() {
        let mut parse_context = ParseContext::new();
        let mut context = ExecutionContext::new();
        let mut cast = AsciiCast::new();
        let instructions = [
            // "@width 123",
            // "@height 456",
            // "@title another title",
            // "@idle 2ms",
            "prompt \"~> \"",
            "secondary-prompt \"| \"",
            "line-split \\",
            "hidden",
        ];
        for line in instructions.iter() {
            ConfigInstruction::parse(line, &mut parse_context)
                .unwrap()
                .execute(&mut context, &mut cast);
        }
        let resolved = context.consume_temporary();
        // assert_eq!(resolved.width, 123);
        // assert_eq!(resolved.height, 456);
        // assert_eq!(resolved.title, "another title");
        // assert_eq!(resolved.idle, Duration::from_millis(2));
        assert_eq!(resolved.prompt, "~> ".to_string());
        assert_eq!(resolved.secondary_prompt, "| ".to_string());
        assert_eq!(resolved.line_split, "\\".to_string());
        assert!(resolved.hidden);
    }
}
