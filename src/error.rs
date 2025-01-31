//! Error types for the `castwright` crate.

use thiserror::Error as ThisError;

/// Possible types of errors that can occur while parsing a single line of a `.cwrt` file. An enum variant represents a specific type of error, and can be converted to an [`Error`] with the [`with_line`](`ErrorType::with_line`) method. (See the [`Error`] struct for an example)
#[derive(ThisError, Debug)]
pub enum ErrorType {
    // Foreign errors
    /// An io error occurred while reading the file.
    #[error("IO error: \"{0}\"")]
    Io(std::io::Error),
    /// A `serde_json` error occurred while parsing.
    #[error("JSON error: \"{0}\"")]
    Json(serde_json::Error),
    /// Subprocess does not exit with expected status code.
    #[error("Shell {0}")]
    Subprocess(String),

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
    pub fn with_line(self, line: usize) -> Error {
        Error { error: self, line }
    }
}

impl From<std::io::Error> for ErrorType {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for ErrorType {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl From<std::num::ParseIntError> for ErrorType {
    fn from(_: std::num::ParseIntError) -> Self {
        Self::MalformedInstruction
    }
}

impl PartialEq for ErrorType {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::Io(_), Self::Io(_))
                | (Self::Json(_), Self::Json(_))
                | (Self::ExpectedKeyValuePair, Self::ExpectedKeyValuePair)
                | (
                    Self::ExpectedClosingDelimiter,
                    Self::ExpectedClosingDelimiter
                )
                | (Self::FrontMatterExists, Self::FrontMatterExists)
                | (Self::UnknownInstruction, Self::UnknownInstruction)
                | (Self::MalformedInstruction, Self::MalformedInstruction)
                | (Self::ExpectedContinuation, Self::ExpectedContinuation)
                | (Self::UnexpectedContinuation, Self::UnexpectedContinuation)
                | (Self::UnknownConfig, Self::UnknownConfig)
                | (Self::UnknownFrontMatter, Self::UnknownFrontMatter)
                | (Self::HeaderAlreadyWritten, Self::HeaderAlreadyWritten)
        )
    }
}

/// The `Error` struct represents an error that occurred during parsing or execution, with the line number denoting its position. To construct an `Error` manually, you should call [`with_line`](`ErrorType::with_line`) on an [`ErrorType`] enum variant. Usually, you'll only need this struct in a function signature to propagate errors.
///
/// ## Example
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
///     Err(error)
/// }
/// // Should get the following output:
/// // Error: Error { error: UnknownInstruction, line: 1 }
/// ```
#[derive(ThisError, Debug, PartialEq)]
#[error("{error} at line {line}")]
pub struct Error {
    /// The type of error that occurred.
    error: ErrorType,
    /// The line number where the error occurred, starting at 1. If `0`, the error is not related to a specific line.
    line: usize,
}
