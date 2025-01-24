mod asciicast;
mod error;
mod instruction;
mod util;

pub use error::{ParseError, ParseErrorType};
use instruction::{parse_instruction, InstructionTrait};
use std::{io::BufRead, time::Duration};
use asciicast::AsciiCast;

/// A parsing context for the script.
struct ParseContext {
    /// The starting character of the command.
    start: char,
    /// Whether we're expecting a continuation.
    expect_continuation: bool,
}

impl ParseContext {
    /// Create a new `ParseContext`.
    fn new() -> Self {
        Self {
            start: ' ',
            expect_continuation: false,
        }
    }
    /// Create a context with given start character.
    #[allow(dead_code, reason = "Only used in tests")]
    fn with_start(&self, start: char) -> Self {
        Self { start, ..*self }
    }
    /// Create a context with given continuation expectation.
    #[allow(dead_code, reason = "Only used in tests")]
    fn expect_continuation(&self, expect_continuation: bool) -> Self {
        Self {
            expect_continuation,
            ..*self
        }
    }
}

/// Persistent configuration for the script.
#[derive(Clone, Debug, PartialEq)]
struct Configuration {
    /// Terminal width.
    width: u16,
    /// Terminal height.
    height: u16,
    /// Title of the asciicast.
    title: String,
    /// The shell to use.
    shell: String,
    /// The quit command.
    quit: String,
    /// Idle time limit.
    idle: Duration,
    /// The shell prompt to use in the asciicast.
    prompt: String,
    /// The secondary prompt to use in the asciicast (for continuation lines).
    secondary_prompt: String,
    /// The string to signify a line split in a multiline command.
    line_split: String,
    /// Whether the command should be executed silently.
    hidden: bool,
    /// Typing delay between characters in a command or print instruction.
    delay: Duration,
}

impl Configuration {
    /// Create a new `Configuration` with default values.
    fn new() -> Self {
        let (width, height) = util::get_terminal_size();
        Self {
            width,
            height,
            title: "Castwright Script".to_string(),
            shell: "bash".to_string(),
            quit: "exit".to_string(),
            idle: Duration::from_secs(5),
            prompt: "$ ".to_string(),
            secondary_prompt: "> ".to_string(),
            line_split: " \\".to_string(),
            hidden: false,
            delay: Duration::from_millis(100),
        }
    }
}

/// Temporary configuration for the script.
struct TemporaryConfiguration {
    prompt: Option<String>,
    secondary_prompt: Option<String>,
    line_split: Option<String>,
    hidden: Option<bool>,
    delay: Option<Duration>,
}

impl TemporaryConfiguration {
    /// Create a new `TemporaryConfiguration` with default values.
    fn new() -> Self {
        Self {
            prompt: None,
            secondary_prompt: None,
            line_split: None,
            hidden: None,
            delay: None,
        }
    }
    /// Check if the temporary configuration is empty.
    fn is_empty(&self) -> bool {
        self.prompt.is_none()
            && self.secondary_prompt.is_none()
            && self.line_split.is_none()
            && self.hidden.is_none()
            && self.delay.is_none()
    }
}

/// An execution context for the script.
struct ExecutionContext {
    /// Persistent configuration.
    persistent: Configuration,
    /// Temporary configuration.
    temporary: TemporaryConfiguration,
    /// Elapsed time in microseconds (µs).
    elapsed: u64,
    /// Previous commands to be concatenated.
    command: String,
}

impl ExecutionContext {
    /// Create a new `ExecutionContext` with default values.
    fn new() -> Self {
        Self {
            persistent: Configuration::new(),
            temporary: TemporaryConfiguration::new(),
            elapsed: 0,
            command: String::new(),
        }
    }
    /// Check if the temporary configuration has any values.
    fn has_temporary(&self) -> bool {
        !self.temporary.is_empty()
    }
    /// Merge the temporary configuration and return a new `Configuration`.
    fn merge_temporary(&self) -> Configuration {
        let mut config = self.persistent.clone();
        if let Some(prompt) = &self.temporary.prompt {
            config.prompt = prompt.clone();
        }
        if let Some(secondary_prompt) = &self.temporary.secondary_prompt {
            config.secondary_prompt = secondary_prompt.clone();
        }
        if let Some(line_split) = &self.temporary.line_split {
            config.line_split = line_split.clone();
        }
        if let Some(hidden) = self.temporary.hidden {
            config.hidden = hidden;
        }
        if let Some(delay) = self.temporary.delay {
            config.delay = delay;
        }
        config
    }
    /// Consume the temporary configuration and return a new `Configuration`.
    fn consume_temporary(&mut self) -> Configuration {
        let mut config = self.persistent.clone();
        if let Some(prompt) = self.temporary.prompt.take() {
            config.prompt = prompt;
        }
        if let Some(secondary_prompt) = self.temporary.secondary_prompt.take() {
            config.secondary_prompt = secondary_prompt;
        }
        if let Some(line_split) = self.temporary.line_split.take() {
            config.line_split = line_split;
        }
        if let Some(hidden) = self.temporary.hidden.take() {
            config.hidden = hidden;
        }
        if let Some(delay) = self.temporary.delay.take() {
            config.delay = delay;
        }
        config
    }
}

/// A `.cw` script
#[derive(Debug)]
pub struct Script {
    instructions: Vec<Box<dyn InstructionTrait>>,
}

impl Script {
    /// Parse a castwright script from a reader.
    pub fn parse(reader: impl BufRead) -> Result<Self, ParseError> {
        let mut instructions = Vec::new();
        let mut context = ParseContext::new();
        for (line_number, line) in reader.lines().enumerate() {
            let line = line.map_err(|err| ParseErrorType::Io(err).with_line(line_number))?;
            let instruction =
                parse_instruction(&line, &mut context).map_err(|e| e.with_line(line_number + 1))?;
            instructions.push(instruction);
        }
        Ok(Self { instructions })
    }

    /// Execute the script and return the generated asciicast.
    pub fn execute(&self) -> AsciiCast {
        let mut context = ExecutionContext::new();
        let mut cast = AsciiCast::new();
        for instruction in &self.instructions {
            instruction.execute(&mut context, &mut cast);
        }
        // Update the header with the final configuration
        cast.width(context.persistent.width)
            .height(context.persistent.height)
            .title(context.persistent.title.clone())
            .idle_time_limit(context.persistent.idle.as_secs_f64());
        cast
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use instruction::{
        CommandInstruction, ConfigInstruction, EmptyInstruction, MarkerInstruction,
        PrintInstruction,
    };
    use std::io::BufReader;

    #[test]
    fn script() {
        let script = r#"
            @@width 123
            @hidden true
            %print
            !marker
            #comment
            $single command
            $command \
            >continuation
        "#;
        let script = script.trim();
        let script = script.as_bytes();
        let script = BufReader::new(script);
        let script = Script::parse(script).unwrap();
        let mut context = ParseContext::new();
        let expected: Vec<Box<dyn InstructionTrait>> = vec![
            Box::new(ConfigInstruction::parse("@width 123", &mut context).unwrap()),
            Box::new(ConfigInstruction::parse("hidden true", &mut context).unwrap()),
            Box::new(PrintInstruction::parse("print", &mut context).unwrap()),
            Box::new(MarkerInstruction::parse("marker", &mut context).unwrap()),
            Box::new(EmptyInstruction::new()),
            Box::new(
                CommandInstruction::parse("single command", &mut context.with_start('$')).unwrap(),
            ),
            Box::new(
                CommandInstruction::parse("command \\", &mut context.with_start('$')).unwrap(),
            ),
            Box::new(
                CommandInstruction::parse(
                    "continuation",
                    &mut context.with_start('>').expect_continuation(true),
                )
                .unwrap(),
            ),
        ];
        assert_eq!(script.instructions, expected);
    }

    #[test]
    fn script_unknown_instruction() {
        let script = r#"
            @@width 123
            @hidden
            %print
            !marker
            #comment
            $command \
            >continuation
            unknown
        "#;
        let script = script.trim();
        let script = script.as_bytes();
        let script = BufReader::new(script);
        assert_eq!(
            Script::parse(script).unwrap_err(),
            ParseErrorType::UnknownInstruction.with_line(8)
        );
    }

    #[test]
    fn script_malformed_instruction() {
        let malformed_scripts = [
            // Config instruction
            "#abc\n@",                   // Expected character after @
            "#abc\n@@",                  // Expected character after @@
            "#abc\n@@wid",               // Unrecognized configuration instruction
            "@@width 123\n@@height abc", // Malformed integer
            "@@width 123\n@idle 1",      // Malformed duration - no suffix
            "@@width 123\n@idle 1min",   // Malformed duration - invalid suffix
            "@@width 123\n@delay",       // Malformed duration - no value
            "@@width 123\n@hidden what", // Malformed boolean
        ];
        for script in malformed_scripts.iter() {
            let script = script.trim();
            let script = script.as_bytes();
            let script = BufReader::new(script);
            assert_eq!(
                Script::parse(script).unwrap_err(),
                ParseErrorType::MalformedInstruction.with_line(2)
            );
        }
    }

    #[test]
    fn script_expected_continuation() {
        let script = r#"
            $command \
            @@width 123
        "#;
        let script = script.trim();
        let script = script.as_bytes();
        let script = BufReader::new(script);
        assert_eq!(
            Script::parse(script).unwrap_err(),
            ParseErrorType::ExpectedContinuation.with_line(2)
        );
    }

    #[test]
    fn script_unexpected_continuation() {
        let script = r#"
            $command
            >continuation
        "#;
        let script = script.trim();
        let script = script.as_bytes();
        let script = BufReader::new(script);
        assert_eq!(
            Script::parse(script).unwrap_err(),
            ParseErrorType::UnexpectedContinuation.with_line(2)
        );
    }

    #[test]
    fn execution_context() {
        let mut context = ExecutionContext::new();
        context.temporary.prompt = Some("$$ ".to_string());
        context.temporary.secondary_prompt = Some(">> ".to_string());
        let (width, height) = util::get_terminal_size();
        let expected = Configuration {
            width,
            height,
            title: "Castwright Script".to_string(),
            shell: "bash".to_string(),
            quit: "exit".to_string(),
            idle: Duration::from_secs(5),
            prompt: "$$ ".to_string(),
            secondary_prompt: ">> ".to_string(),
            line_split: " \\".to_string(),
            hidden: false,
            delay: Duration::from_millis(100),
        };
        let calculated = context.consume_temporary();
        assert_eq!(calculated, expected);
        assert!(context.temporary.is_empty());
    }
}
