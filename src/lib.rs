//! # CastWright
//!
//! 🎥 Scripted terminal recording.
//!
//! ## Note
//!
//! - This project is still in the early stages of development, with some core features missing or incomplete.
//! - If you see this message, it means that you're viewing the documentation of the `castwright` library. For the CLI, please refer to the [README](https://github.com/PRO-2684/castwright#castwright); for the CastWright script format, please refer to the [REFERENCE](https://github.com/PRO-2684/castwright/blob/main/REFERENCE.md).
//!
//! ## Usage
//!
//! Mostly, you'll deal with the [`CastWright`] struct and [`Error`] struct. When you want to you manually create errors, you need to deal with the [`ErrorType`] enum. If you're writing your own tool for generating asciicasts, you can use the [`AsciiCast`] struct.
//!
//! ## Example
//!
//! ```rust
//! use castwright::{CastWright, Error};
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
//!     let mut reader = BufReader::new(text.as_bytes());
//!     let mut stdout = std::io::stdout().lock();
//!     let castwright = CastWright::new();
//!     castwright.run(&mut reader, &mut stdout)?;
//!     Ok(())
//! }
//! ```
//!
//! See `src/main.rs` for a complete example.

#![deny(missing_docs)]

mod asciicast;
mod error;
mod instruction;
mod shell;
mod util;

pub use asciicast::AsciiCast;
pub use error::{Error, ErrorType};
use instruction::parse_instruction;
use shell::execute_command;
use std::io::{BufRead, Write};

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
    /// Typing delay between characters in a command or print instruction, in microseconds (µs).
    delay: u64,
}

impl Configuration {
    /// Create a new `Configuration` with default values.
    fn new() -> Self {
        Self {
            prompt: "$ ".to_string(),
            secondary_prompt: "> ".to_string(),
            line_split: " \\".to_string(),
            hidden: false,
            delay: 100_000,
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
    delay: Option<u64>,
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
    #[cfg(test)]
    fn with_start(&self, start: char) -> Self {
        Self { start, ..*self }
    }
}

/// An execution context for the script.
struct ExecutionContext {
    // General
    /// Persistent configuration.
    persistent: Configuration,
    /// Temporary configuration.
    temporary: TemporaryConfiguration,
    /// The shell to use.
    shell: String,
    /// Elapsed time in microseconds (µs).
    elapsed: u64,

    // Configuration
    /// Whether to actually execute the commands.
    execute: bool,
    /// Whether to preview the asciicast.
    preview: bool,

    // Instruction-specific
    /// Previous commands to be concatenated.
    command: String,
}

impl ExecutionContext {
    /// Create a new `ExecutionContext` with default values.
    fn new() -> Self {
        Self {
            persistent: Configuration::new(),
            temporary: TemporaryConfiguration::new(),
            shell: "bash".to_string(),
            elapsed: 0,
            execute: false,
            preview: false,
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

    /// Print given string if preview is enabled.
    fn preview(&self, s: &str) {
        if self.preview {
            print!("{}", s);
        }
    }
}

/// The `CastWright` struct represents the main entry point for the CastWright library. An instance of `CastWright` can be configured, parses and executes CastWright scripts, and writes the resulting asciicast to a writer.
///
/// ## Instantiation
///
/// To instantiate a `CastWright` instance, use the [`CastWright::new`] method.
///
/// ## Configuration
///
/// You can then configure the instance using the following methods:
///
/// - [`execute`](`CastWright::execute`): Set whether to execute and capture the output of shell commands.
///
/// ## Running
///
/// To parse and execute a CastWright script and write the resulting asciicast, use the [`run`](`CastWright::run`) method, which takes mutable references to a reader and a writer.
///
/// ## Example
///
/// ```rust
/// use castwright::CastWright;
/// use std::io::BufReader;
///
/// // Input & output
/// let text = r#"
///     $ echo "Hello, World!"
/// "#;
/// let text = text.trim();
/// let mut reader = BufReader::new(text.as_bytes());
/// let mut writer = Vec::new();
/// // CastWright
/// let mut castwright = CastWright::new(); // Instantiation
/// castwright
///     .execute(true) // Configuration
///     .run(&mut reader, &mut writer) // Running
///     .unwrap();
/// ```
///
/// If you prefer one-liners:
///
/// ```rust
/// # use castwright::CastWright;
/// # use std::io::BufReader;
/// # let text = r#"
/// #     $ echo "Hello, World!"
/// # "#;
/// # let text = text.trim();
/// # let mut reader = BufReader::new(text.as_bytes());
/// # let mut writer = Vec::new();
/// CastWright::new().execute(true).run(&mut reader, &mut writer).unwrap();
/// ```
#[derive(Debug)]
pub struct CastWright {
    /// Whether to execute and capture the output of shell commands.
    execute: bool,
    /// Whether to preview the asciicast.
    preview: bool,
}

impl CastWright {
    /// Create a new `CastWright` instance.
    pub fn new() -> Self {
        Self { execute: false, preview: false }
    }
    /// Set whether to execute and capture the output of shell commands.
    pub fn execute(&mut self, execute: bool) -> &mut Self {
        self.execute = execute;
        self
    }
    /// Set whether to preview the asciicast.
    pub fn preview(&mut self, preview: bool) -> &mut Self {
        self.preview = preview;
        self
    }
    /// Interpret and run a CastWright script from a reader, writing the asciicast to a writer.
    pub fn run(&self, reader: &mut impl BufRead, writer: &mut impl Write) -> Result<(), Error> {
        let mut parse_context = ParseContext::new();
        let mut execution_context = ExecutionContext::new();
        let mut cast = AsciiCast::new(writer);
        let mut line_cnt = 0;
        execution_context.execute = self.execute;
        execution_context.preview = self.preview;
        for (line_number, line) in reader.lines().enumerate() {
            self.run_line(line, &mut parse_context, &mut execution_context, &mut cast)
                .map_err(|e| e.with_line(line_number + 1))?;
            line_cnt += 1;
        }
        if parse_context.front_matter_state == FrontMatterState::Start {
            return Err(ErrorType::ExpectedClosingDelimiter.with_line(line_cnt + 1));
        }
        Ok(())
    }
    /// Interpret and run a line of a CastWright script.
    fn run_line(
        &self,
        line: Result<String, std::io::Error>,
        parse_context: &mut ParseContext,
        execution_context: &mut ExecutionContext,
        cast: &mut AsciiCast,
    ) -> Result<(), ErrorType> {
        let instruction = parse_instruction(&line?, parse_context)?;
        instruction.execute(execution_context, cast)?;
        Ok(())
    }
}

impl Default for CastWright {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    #[test]
    fn expected_key_value_pair() {
        let text = r#"
            ---
            $echo "Hello, World!"
            ---
        "#;
        let text = text.trim();
        let mut reader = BufReader::new(text.as_bytes());
        assert_eq!(
            CastWright::new()
                .run(&mut reader, &mut std::io::sink())
                .unwrap_err(),
            ErrorType::ExpectedKeyValuePair.with_line(2)
        );
    }

    #[test]
    fn expected_closing_delimiter() {
        let text = r#"
            ---
            width: 123
        "#;
        let text = text.trim();
        let mut reader = BufReader::new(text.as_bytes());
        assert_eq!(
            CastWright::new()
                .run(&mut reader, &mut std::io::sink())
                .unwrap_err(),
            ErrorType::ExpectedClosingDelimiter.with_line(3)
        );
    }

    #[test]
    fn front_matter_exists() {
        let text = r#"
            ---
            width: 123
            ---
            ---
        "#;
        let text = text.trim();
        let mut reader = BufReader::new(text.as_bytes());
        assert_eq!(
            CastWright::new()
                .run(&mut reader, &mut std::io::sink())
                .unwrap_err(),
            ErrorType::FrontMatterExists.with_line(4)
        );
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
            $echo "Hello, World!" \
            >continuation
            unknown
        "#;
        let text = text.trim();
        let mut reader = BufReader::new(text.as_bytes());
        assert_eq!(
            CastWright::new()
                .run(&mut reader, &mut std::io::sink())
                .unwrap_err(),
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
            let mut reader = BufReader::new(text.as_bytes());
            assert_eq!(
                CastWright::new()
                    .run(&mut reader, &mut std::io::sink())
                    .unwrap_err(),
                ErrorType::MalformedInstruction.with_line(2)
            );
        }
    }

    #[test]
    fn script_expected_continuation() {
        let text = r#"
            $echo "Hello, World!" \
            @hidden true
        "#;
        let text = text.trim();
        let mut reader = BufReader::new(text.as_bytes());
        assert_eq!(
            CastWright::new()
                .run(&mut reader, &mut std::io::sink())
                .unwrap_err(),
            ErrorType::ExpectedContinuation.with_line(2)
        );
    }

    #[test]
    fn script_unexpected_continuation() {
        let text = r#"
            $echo "Hello, World!"
            >continuation
        "#;
        let text = text.trim();
        let mut reader = BufReader::new(text.as_bytes());
        assert_eq!(
            CastWright::new()
                .run(&mut reader, &mut std::io::sink())
                .unwrap_err(),
            ErrorType::UnexpectedContinuation.with_line(2)
        );
    }

    #[test]
    fn execution_context_consume_temporary() {
        let mut context = ExecutionContext::new();
        context.temporary.prompt = Some("$$ ".to_string());
        context.temporary.secondary_prompt = Some(">> ".to_string());
        let expected_config = Configuration {
            prompt: "$$ ".to_string(),
            secondary_prompt: ">> ".to_string(),
            line_split: " \\".to_string(),
            hidden: false,
            delay: 100_000,
        };
        let calculated_config = context.consume_temporary();
        assert_eq!(calculated_config, expected_config);
        assert!(context.temporary.is_empty());
    }
}
