//! # Castwright
//!
//! 🎥 Scripted terminal recording.
//!
//! ## Note
//!
//! - This project is still in the early stages of development, with some core features missing or incomplete.
//! - If you see this message, it means that you're viewing the documentation of the `castwright` library. For the CLI, please refer to the [README](https://github.com/PRO-2684/castwright#castwright); for the castwright script format, please refer to the [REFERENCE](https://github.com/PRO-2684/castwright/blob/main/REFERENCE.md).
//!
//! ## Usage
//!
//! Mostly, you'll deal with the [`Script`] struct and [`Error`] struct. Under rare circumstances, you might need to use the [`AsciiCast`] struct and the [`ErrorType`] enum.
//!
//! ## Example
//!
//! ```rust
//! use castwright::{Script, Error};
//! use std::io::BufReader;
//!
//! fn main() -> Result<(), Error> {
//!     let text = r#"
//!         $ echo "Hello, World!"
//!         $ echo "Multi-" \
//!         > "line" \
//!         > "command"
//!     "#;
//!     let text = text.trim();
//!     let reader = BufReader::new(text.as_bytes());
//!     let script = Script::parse(reader)?;
//!     let cast = script.execute();
//!     let mut stdout = std::io::stdout().lock();
//!     cast.write(&mut stdout)?;
//!     Ok(())
//! }
//! ```
//!
//! See `src/main.rs` for a complete example.

#![deny(missing_docs)]

mod asciicast;
mod error;
mod instruction;
mod util;

pub use asciicast::AsciiCast;
pub use error::{Error, ErrorType};
use instruction::{parse_instruction, Instruction};
use std::{io::BufRead, time::Duration};

/// Front matter parsing state.
#[derive(Debug, PartialEq, Clone, Copy)]
enum FrontMatterState {
    /// Nothing has been parsed yet.
    None,
    /// We're expecting key-value pairs. (First occurrence of `---`)
    Start,
    /// We've parsed the key-value pairs. (Second occurrence of `---`)
    End,
}

impl FrontMatterState {
    /// Take in an occurrence of `---`.
    fn next(&mut self) -> Result<(), ErrorType> {
        match self {
            Self::None => *self = Self::Start,
            Self::Start => *self = Self::End,
            Self::End => return Err(ErrorType::FrontMatterExists),
        }
        Ok(())
    }
    /// End the front matter parsing, since an instruction has been encountered.
    fn end(&mut self) -> Result<(), ErrorType> {
        match self {
            Self::None => *self = Self::End,
            Self::Start => return Err(ErrorType::ExpectedKeyValuePair),
            Self::End => {} // Do nothing
        }
        Ok(())
    }
}

/// Configuration for the script.
#[derive(Clone, Debug, PartialEq)]
struct Configuration {
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
        Self {
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
    /// The shell prompt to use in the asciicast.
    prompt: Option<String>,
    /// The secondary prompt to use in the asciicast (for continuation lines).
    secondary_prompt: Option<String>,
    /// The string to signify a line split in a multiline command.
    line_split: Option<String>,
    /// Whether the command should be executed silently.
    hidden: Option<bool>,
    /// Typing delay between characters in a command or print instruction.
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

/// A parsing context for the script.
struct ParseContext {
    /// Front matter parsing state.
    front_matter_state: FrontMatterState,
    /// The starting character of the command.
    start: char,
    /// Whether we're expecting a continuation.
    expect_continuation: bool,
}

impl ParseContext {
    /// Create a new `ParseContext`.
    fn new() -> Self {
        Self {
            front_matter_state: FrontMatterState::None,
            start: ' ',
            expect_continuation: false,
        }
    }
    /// Create a context with a different starting character.
    #[allow(dead_code, reason = "Only used in tests")]
    fn with_start(&self, start: char) -> Self {
        Self { start, ..*self }
    }
    /// Create a context with a different expectation for continuation.
    #[allow(dead_code, reason = "Only used in tests")]
    fn expect_continuation(&self, expect_continuation: bool) -> Self {
        Self {
            expect_continuation,
            ..*self
        }
    }
}

/// An execution context for the script.
struct ExecutionContext {
    // /// Front matter.
    // front_matter: FrontMatter,
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
            // front_matter: FrontMatter::new(),
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

/// The `Script` struct represents a castwright script, conventionally with the `.cwrt` extension. You can **parse** a script from a reader (`impl BufRead`) using the [`Script::parse`] method, which returns a `Script` instance if the parsing is successful, or an [`Error`] instance if it fails.
///
/// You can then **execute** the `Script` instance using the [`execute`](`Script::execute`) method to get an [`AsciiCast`] struct, which can be **written** to a writer (`impl Write`) using the [`write`](`AsciiCast::write`) method.
///
/// ## Example
///
/// ```rust
/// use castwright::Script;
/// use std::io::BufReader;
///
/// let text = r#"
///     $ echo "Hello, World!"
/// "#;
/// let text = text.trim();
/// let reader = BufReader::new(text.as_bytes());
/// let script = Script::parse(reader).unwrap();
/// ```
#[derive(Debug)]
pub struct Script {
    /// The instructions in the script.
    instructions: Vec<Box<dyn Instruction>>,
}

impl Script {
    /// Parse a castwright script from a reader.
    pub fn parse(reader: impl BufRead) -> Result<Self, Error> {
        let mut instructions = Vec::new();
        let mut context = ParseContext::new();
        for (line_number, line) in reader.lines().enumerate() {
            let line = line.map_err(|err| ErrorType::Io(err).with_line(line_number))?;
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
        cast
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use instruction::{
        CommandInstruction, ConfigInstruction, EmptyInstruction, FrontMatterInstruction,
        MarkerInstruction, PrintInstruction,
    };
    use std::io::BufReader;

    #[test]
    fn script() {
        let text = r#"
            ---
            width: 123
            ---
            @hidden true
            %print
            !marker
            #comment
            $single command
            $command \
            >continuation
        "#;
        let text = text.trim();
        let reader = BufReader::new(text.as_bytes());
        let script = Script::parse(reader).unwrap();
        let mut context = ParseContext::new();
        let expected: Vec<Box<dyn Instruction>> = vec![
            Box::new(FrontMatterInstruction::Delimiter),
            Box::new(FrontMatterInstruction::Width(123)),
            Box::new(FrontMatterInstruction::Delimiter),
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
        let text = r#"
            ---
            width: 123
            ---
            @hidden
            %print
            !marker
            #comment
            $command \
            >continuation
            unknown
        "#;
        let text = text.trim();
        let reader = BufReader::new(text.as_bytes());
        assert_eq!(
            Script::parse(reader).unwrap_err(),
            ErrorType::UnknownInstruction.with_line(10)
        );
    }

    #[test]
    fn script_malformed_instruction() {
        let malformed_scripts = [
            // Config instruction
            "#abc\n@",                  // Expected character after @
            "#abc\n@@",                 // Expected character after @@
            "#abc\n@@wid",              // Unrecognized configuration instruction
            "@@prompt $\n@@height abc", // Malformed integer
            "@@prompt $\n@idle 1",      // Malformed duration - no suffix
            "@@prompt $\n@idle 1min",   // Malformed duration - invalid suffix
            "@@prompt $\n@delay",       // Malformed duration - no value
            "@@prompt $\n@hidden what", // Malformed boolean
        ];
        for text in malformed_scripts.iter() {
            let text = text.trim();
            let reader = BufReader::new(text.as_bytes());
            assert_eq!(
                Script::parse(reader).unwrap_err(),
                ErrorType::MalformedInstruction.with_line(2)
            );
        }
    }

    #[test]
    fn script_expected_continuation() {
        let text = r#"
            $command \
            @hidden true
        "#;
        let text = text.trim();
        let reader = BufReader::new(text.as_bytes());
        assert_eq!(
            Script::parse(reader).unwrap_err(),
            ErrorType::ExpectedContinuation.with_line(2)
        );
    }

    #[test]
    fn script_unexpected_continuation() {
        let text = r#"
            $command
            >continuation
        "#;
        let text = text.trim();
        let reader = BufReader::new(text.as_bytes());
        assert_eq!(
            Script::parse(reader).unwrap_err(),
            ErrorType::UnexpectedContinuation.with_line(2)
        );
    }

    #[test]
    fn execution_context_consume_temporary() {
        let mut context = ExecutionContext::new();
        context.temporary.prompt = Some("$$ ".to_string());
        context.temporary.secondary_prompt = Some(">> ".to_string());
        // let (width, height) = util::get_terminal_size();
        let expected_config = Configuration {
            // width,
            // height,
            // title: "Castwright Script".to_string(),
            // shell: "bash".to_string(),
            // quit: "exit".to_string(),
            // idle: Duration::from_secs(5),
            prompt: "$$ ".to_string(),
            secondary_prompt: ">> ".to_string(),
            line_split: " \\".to_string(),
            hidden: false,
            delay: Duration::from_millis(100),
        };
        let calculated_config = context.consume_temporary();
        assert_eq!(calculated_config, expected_config);
        assert!(context.temporary.is_empty());
    }
}
