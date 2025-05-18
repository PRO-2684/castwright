//! Error types for the `castwright` crate.

use serde_json::Error as JsonError;
use std::io::Error as IoError;
use std::{num::ParseIntError, time::SystemTimeError};
use thiserror::Error as ThisError;
use pty_process::Error as PtyError;

/// Possible types of errors that can occur while parsing or executing a single line of a `CastWright` script. Each variant represents a specific type of error, and can be converted to an [`Error`] with the [`with_line`](`ErrorType::with_line`) method. (See the [`Error`] struct for examples)
#[derive(ThisError, Debug)]
pub enum ErrorType {
    // Foreign errors
    /// A `serde_json` error occurred while parsing.
    #[error("JSON error: \"{0}\"")]
    Json(JsonError),
    /// An io error occurred while reading the file.
    #[error("IO error: \"{0}\"")]
    Io(IoError),
    /// Subprocess does not exit successfully.
    #[error("Shell {0}")]
    Subprocess(String),
    /// System time error.
    #[error("System time error: \"{0}\"")]
    SystemTime(SystemTimeError),

    // Front matter errors
    /// Expected key-value pair, but got instruction.
    #[error("Expected key-value pair")]
    ExpectedKeyValuePair,
    /// Expected closing front matter delimiter, but got none.
    #[error("Expected closing front matter delimiter")]
    ExpectedClosingDelimiter,
    /// There is already a front matter block.
    #[error("Front matter already exists")]
    FrontMatterExists,

    // Instruction errors
    /// The first non-whitespace character of the line is not recognized.
    #[error("Unknown instruction")]
    UnknownInstruction,
    /// The instruction is not in the expected format.
    #[error("Malformed instruction")]
    MalformedInstruction,

    // Instruction type specific errors
    /// Expected a continuation line, but did not get one.
    #[error("Expected continuation")]
    ExpectedContinuation,
    /// Did not expect a continuation line, but got one.
    #[error("Unexpected continuation")]
    UnexpectedContinuation,
    /// The configuration instruction is not recognized.
    #[error("Unknown configuration")]
    UnknownConfig,
    /// The front matter instruction is not recognized.
    #[error("Unknown front matter")]
    UnknownFrontMatter,

    // Asciicast errors
    /// The header has already been written.
    #[error("Header already written")]
    HeaderAlreadyWritten,
}

impl ErrorType {
    /// Add line number information to the error, so as to form a [`Error`].
    #[must_use]
    pub const fn with_line(self, line: usize) -> Error {
        Error { error: self, line }
    }
}

impl From<JsonError> for ErrorType {
    fn from(error: JsonError) -> Self {
        Self::Json(error)
    }
}

impl From<IoError> for ErrorType {
    fn from(error: IoError) -> Self {
        Self::Io(error)
    }
}

impl From<ParseIntError> for ErrorType {
    fn from(_: ParseIntError) -> Self {
        Self::MalformedInstruction
    }
}

impl From<SystemTimeError> for ErrorType {
    fn from(error: SystemTimeError) -> Self {
        Self::SystemTime(error)
    }
}

impl From<PtyError> for ErrorType {
    fn from(error: PtyError) -> Self {
        match error {
            PtyError::Io(e) => Self::Io(e),
            PtyError::Rustix(e) => Self::Io(IoError::from(e)),
        }
    }
}

#[cfg(test)]
impl PartialEq for ErrorType {
    fn eq(&self, other: &Self) -> bool {
        // Compare the debug representations of the error types.
        // This is a rather crude way to compare instructions, but is acceptable since it is only used in tests.
        format!("{self:?}") == format!("{other:?}")
    }
}

/// The `Error` struct represents an error that occurred during parsing or execution, with the line number denoting its position.
///
/// Usually, you'll only need this struct in a function signature to propagate errors. To construct an `Error` manually, provide an [`ErrorType`] and a line number. Alternatively, you can call [`with_line`](`ErrorType::with_line`) on an [`ErrorType`] enum variant.
///
/// ## Example
///
/// ### Accessing error information
///
/// ```rust
/// use castwright::{Error, ErrorType};
///
/// let error_type = ErrorType::UnknownInstruction;
/// let error = error_type.with_line(1);
/// assert!(matches!(error.error, ErrorType::UnknownInstruction));
/// assert_eq!(error.line, 1);
/// ```
///
/// ### Propagating an error in `fn main`
///
/// ```rust should_panic
/// use castwright::{CastWright, Error};
/// use std::io::BufReader;
///
/// fn main() -> Result<(), Error> {
///     let text = r#"
///         $ unexpected
///         > continuation
///     "#;
///     let text = text.trim();
///     let mut reader = BufReader::new(text.as_bytes());
///     let mut stdout = std::io::stdout().lock();
///     let castwright = CastWright::new();
///     castwright.run(&mut reader, &mut stdout)?;
///     Ok(())
/// }
/// // Should get the following output:
/// // Error: Error { error: UnexpectedContinuation, line: 2 }
/// ```
///
/// ### Constructing an `Error` manually
///
/// ```rust should_panic
/// use castwright::{Error, ErrorType};
///
/// fn main() -> Result<(), Error> {
///     let error_type = ErrorType::UnknownInstruction;
///     let error = error_type.with_line(1);
///     // Or:
///     // let error = Error {
///     //     error: ErrorType::UnknownInstruction,
///     //     line: 1,
///     // };
///     Err(error)
/// }
/// // Should get the following output:
/// // Error: Error { error: UnknownInstruction, line: 1 }
/// ```
#[cfg_attr(test, derive(PartialEq))]
#[derive(ThisError, Debug)]
#[error("{error} at line {line}")]
pub struct Error {
    /// The type of error that occurred.
    pub error: ErrorType,
    /// The line number where the error occurred, starting at 1. If `0`, the error is not related to a specific line.
    pub line: usize,
}
