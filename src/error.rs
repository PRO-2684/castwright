//! Error types for the `castwright` crate.

use thiserror::Error as ThisError;

/// Possible types of errors that can occur while parsing a single line of a `.cwrt` file.
#[derive(ThisError, Debug)]
pub enum ErrorType {
    // General parsing errors
    /// An io error occurred while reading the file.
    #[error("IO error: \"{0}\"")]
    Io(std::io::Error),
    /// A `serde_json` error occurred while parsing.
    #[error("JSON error: \"{0}\"")]
    Json(serde_json::Error),
    /// The first non-whitespace character of the line is not recognized.
    #[error("Unknown instruction")]
    UnknownInstruction,
    /// The instruction is not in the expected format.
    #[error("Malformed instruction")]
    MalformedInstruction,
    /// Expected a continuation line, but did not get one.
    #[error("Expected continuation")]
    ExpectedContinuation,
    /// Did not expect a continuation line, but got one.
    #[error("Unexpected continuation")]
    UnexpectedContinuation,
    /// The feature is not implemented.
    #[error("Not implemented {0}")]
    NotImplemented(&'static str),
}

impl ErrorType {
    /// Add line number information to the error, so as to form a [`Error`].
    pub fn with_line(self, line: usize) -> Error {
        Error { error: self, line }
    }
}

impl PartialEq for ErrorType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Io(_), Self::Io(_)) => true,
            (Self::Json(_), Self::Json(_)) => true,
            (Self::UnknownInstruction, Self::UnknownInstruction) => true,
            (Self::MalformedInstruction, Self::MalformedInstruction) => true,
            (Self::ExpectedContinuation, Self::ExpectedContinuation) => true,
            (Self::UnexpectedContinuation, Self::UnexpectedContinuation) => true,
            (Self::NotImplemented(a), Self::NotImplemented(b)) => a == b,
            _ => false,
        }
    }
}

/// An error that occurred, with the line number denoting its position. To construct an `Error`, you should call [`ErrorType::with_line`].
#[derive(ThisError, Debug, PartialEq)]
#[error("{error} at line {line}")]
pub struct Error {
    /// The type of error that occurred.
    error: ErrorType,
    /// The line number where the error occurred, starting at 1. If `0`, the line number is unknown at this point.
    line: usize,
}
