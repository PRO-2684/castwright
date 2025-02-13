//! 🎥 Scripted terminal recording.
//!
//! ## Note
//!
//! If you see this message, it means that you're viewing the documentation of the `castwright` library. For the CLI, please refer to the [README](https://github.com/PRO-2684/castwright#-castwright); for the `CastWright` script format, please refer to the [REFERENCE](https://github.com/PRO-2684/castwright/blob/main/REFERENCE.md).
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
//!     let mut stdout = std::io::stdout().lock(); // Consider wrapping in a BufWriter for better performance
//!     let castwright = CastWright::new();
//!     castwright.run(&mut reader, &mut stdout)?;
//!     Ok(())
//! }
//! ```
//!
//! See `src/main.rs` for a complete example.

#![deny(missing_docs)]
#![warn(clippy::all, clippy::nursery, clippy::pedantic, clippy::cargo)]

mod asciicast;
mod error;
mod instruction;
mod shell;
mod util;

pub use asciicast::AsciiCast;
pub use error::{Error, ErrorType};
use instruction::parse_instruction;
use optfield::optfield;
use shell::execute_command;
use std::{
    borrow::Cow,
    io::{BufRead, Write},
    path::PathBuf,
};

/// The version of the `CastWright` library.
pub const VERSION: &str = env!("CARGO_PKG_VERSION", "can't determine version");

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
#[optfield(TemporaryConfiguration,
    rewrap,
    field_doc,
    doc = "Temporary configuration for the script.",
    attrs = add(derive(Default)),
    merge_fn = merge, // Merge function, only used in `Configuration::combine`
)]
#[derive(Clone, Debug, PartialEq)]
struct Configuration {
    // NOTE: Remember to update [`TemporaryConfiguration::is_empty`] when adding new fields
    /// The shell prompt to use in the asciicast.
    prompt: String,
    /// The secondary prompt to use in the asciicast (for continuation lines).
    secondary_prompt: String,
    /// The string to signify a line continuation in a multiline command.
    line_continuation: String,
    /// Whether the command should be executed silently.
    hidden: bool,
    /// Expected exit status of the command. `true` for success, `false` for failure, `None` for any.
    expect: Option<bool>,
    /// Typing interval between characters in a command or print instruction, in microseconds (µs).
    interval: u128,
    /// The start lag in microseconds (µs). i.e. Additional delay after displaying the prompt, before printing the command for command instructions, or before printing the content for print instructions.
    start_lag: u128,
    /// The end lag in microseconds (µs). i.e. Additional delay after printing the command for command instructions, or after printing the content for print instructions.
    end_lag: u128,
}

impl Configuration {
    /// Create a new `Configuration` with default values.
    fn new() -> Self {
        Self::default()
    }
    /// Combine with a temporary configuration.
    fn combine(&self, temporary: TemporaryConfiguration) -> Cow<Self> {
        if temporary.is_empty() {
            // No temporary configuration - use a borrowed reference to the persistent configuration
            Cow::Borrowed(self)
        } else {
            // Temporary configuration exists - clone and merge persistent configuration
            let mut config = self.clone();
            config.merge(temporary);
            Cow::Owned(config)
        }
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            prompt: "$ ".to_string(),
            secondary_prompt: "> ".to_string(),
            line_continuation: " \\".to_string(),
            hidden: false,
            expect: Some(true),
            interval: 100_000,
            start_lag: 0,
            end_lag: 0,
        }
    }
}

impl TemporaryConfiguration {
    /// Create a new `TemporaryConfiguration` with default values.
    fn new() -> Self {
        Self::default()
    }
    /// Check if the temporary configuration is empty.
    const fn is_empty(&self) -> bool {
        self.prompt.is_none()
            && self.secondary_prompt.is_none()
            && self.line_continuation.is_none()
            && self.hidden.is_none()
            && self.expect.is_none()
            && self.interval.is_none()
            && self.start_lag.is_none()
            && self.end_lag.is_none()
    }
    /// Take or clone self, depending on the `consume` parameter.
    fn get(&mut self, consume: bool) -> Self {
        if consume {
            std::mem::take(self)
        } else {
            self.clone()
        }
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
    const fn new() -> Self {
        Self {
            front_matter_state: FrontMatterState::None,
            start: ' ',
            expect_continuation: false,
        }
    }
    /// Create a context with a different starting character.
    #[cfg(test)]
    const fn with_start(&self, start: char) -> Self {
        Self { start, ..*self }
    }
}

/// An execution context for the script.
struct ExecutionContext {
    // State
    /// Persistent configuration.
    persistent: Configuration,
    /// Temporary configuration.
    temporary: TemporaryConfiguration,
    /// The shell to use.
    shell: Vec<String>,
    /// Working directory. Must be an absolute path.
    directory: PathBuf,
    /// Elapsed time in microseconds (µs).
    elapsed: u128,

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
            shell: vec!["bash".to_string(), "-c".to_string()],
            directory: PathBuf::from(".")
                .canonicalize()
                .expect("Failed to canonicalize current directory"),
            elapsed: 0,
            execute: false,
            preview: false,
            command: String::new(),
        }
    }

    /// Print given string if preview is enabled.
    fn preview(&self, s: &str) {
        if self.preview {
            print!("{s}");
        }
    }
}

/// The `CastWright` struct represents the main entry point for the `CastWright` library.
///
/// An instance of `CastWright` can be configured. It parses and executes `CastWright` scripts, and writes the resulting asciicast to a writer.
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
/// - [`timestamp`](`CastWright::timestamp`): Set whether to include timestamp information in the output.
/// - [`preview`](`CastWright::preview`): Set whether to preview the asciicast.
///
/// ## Running
///
/// To parse and execute a `CastWright` script and write the resulting asciicast, use the [`run`](`CastWright::run`) method, which takes mutable references to a reader and a writer. For better performance, a buffered writer is recommended.
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
/// let castwright = CastWright::new() // Instantiation & configuration
///     .execute(true)
///     .preview(true);
/// castwright.run(&mut reader, &mut writer).unwrap(); // Running
/// let asciicast = String::from_utf8_lossy(&writer); // Output
/// ```
///
/// If you prefer to use the default configuration:
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
/// CastWright::new().run(&mut reader, &mut writer).unwrap();
/// let asciicast = String::from_utf8_lossy(&writer);
/// ```
#[derive(Debug, Default)]
pub struct CastWright {
    /// Whether to execute and capture the output of shell commands.
    execute: bool,
    /// Whether to include timestamp information in the output.
    timestamp: bool,
    /// Whether to preview the asciicast.
    preview: bool,
}

impl CastWright {
    /// Create a new `CastWright` instance.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Set whether to execute and capture the output of shell commands.
    #[must_use]
    pub const fn execute(self, execute: bool) -> Self {
        Self { execute, ..self }
    }
    /// Set whether to include timestamp information in the output.
    #[must_use]
    pub const fn timestamp(self, timestamp: bool) -> Self {
        Self { timestamp, ..self }
    }
    /// Set whether to preview the asciicast.
    #[must_use]
    pub const fn preview(self, preview: bool) -> Self {
        Self { preview, ..self }
    }
    /// Interpret and run a `CastWright` script from a reader, writing the asciicast to a writer.
    ///
    /// ## Errors
    ///
    /// This method returns an error if the script contains any syntax errors, or any errors occur during execution.
    pub fn run(&self, reader: &mut impl BufRead, writer: &mut impl Write) -> Result<(), Error> {
        let mut parse_context = ParseContext::new();
        let mut execution_context = ExecutionContext::new();
        let mut cast = AsciiCast::new(writer);
        let mut line_cnt = 0;
        execution_context.execute = self.execute;
        execution_context.preview = self.preview;

        if self.timestamp {
            let timestamp = util::timestamp().map_err(|e| e.with_line(0))?;
            cast.timestamp(timestamp).map_err(|e| e.with_line(0))?;
        }

        for (line_number, line) in reader.lines().enumerate() {
            Self::run_line(line, &mut parse_context, &mut execution_context, &mut cast)
                .map_err(|e| e.with_line(line_number + 1))?;
            line_cnt += 1;
        }

        cast.finish().map_err(|e| e.with_line(line_cnt))?; // Finish writing the asciicast

        if parse_context.front_matter_state == FrontMatterState::Start {
            Err(ErrorType::ExpectedClosingDelimiter.with_line(line_cnt + 1))
        } else if parse_context.expect_continuation {
            Err(ErrorType::ExpectedContinuation.with_line(line_cnt + 1))
        } else {
            Ok(())
        }
    }
    /// Interpret and run a line of a `CastWright` script.
    fn run_line(
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    #[test]
    fn version_correct() {
        // Assert that `VERSION` exists in `Cargo.toml`
        let content = std::fs::read_to_string("Cargo.toml").unwrap();
        assert!(content.contains(&format!("version = \"{VERSION}\"")));
    }

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
        let text = r"
            ---
            width: 123
        ";
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
        let text = r"
            ---
            width: 123
            ---
            ---
        ";
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
    fn script_expected_continuation_1() {
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
    fn script_expected_continuation_2() {
        let text = r#"
            $echo "Hello, World!" \
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
        context.temporary.expect = Some(None);
        let expected_config = Configuration {
            prompt: "$$ ".to_string(),
            secondary_prompt: ">> ".to_string(),
            line_continuation: " \\".to_string(),
            hidden: false,
            expect: None,
            interval: 100_000,
            start_lag: 0,
            end_lag: 0,
        };
        let calculated_config = context
            .persistent
            .combine(context.temporary.get(true))
            .into_owned();
        assert_eq!(calculated_config, expected_config);
        assert!(context.temporary.is_empty());
    }
}
