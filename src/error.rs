//! Error types for the `castwright` crate.

use thiserror::Error;

/// Possible types of errors that can occur while parsing a single line of a `.cw` file.
#[derive(Error, Debug)]
pub enum ParseErrorType {
    // General parsing errors
    /// An io error occurred while reading the file.
    #[error("IO error: \"{0}\"")]
    Io(std::io::Error),
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
}

impl ParseErrorType {
    /// Add line number information to the error, so as to form a [`ParseError`].
    pub fn with_line(self, line: usize) -> ParseError {
        ParseError { error: self, line }
    }
}

impl PartialEq for ParseErrorType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Io(_), Self::Io(_)) => true,
            (Self::UnknownInstruction, Self::UnknownInstruction) => true,
            (Self::MalformedInstruction, Self::MalformedInstruction) => true,
            (Self::ExpectedContinuation, Self::ExpectedContinuation) => true,
            (Self::UnexpectedContinuation, Self::UnexpectedContinuation) => true,
            _ => false,
        }
    }
}

/// An error that occurred while parsing a `.cw` file, with the line number denoting its position. To construct a `ParseError`, you should call [`ParseErrorType::with_line`].
#[derive(Error, Debug, PartialEq)]
#[error("{error} at line {line}")]
pub struct ParseError {
    /// The type of error that occurred.
    error: ParseErrorType,
    /// The line number where the error occurred, starting at 1. If `0`, the line number is unknown at this point.
    line: usize,
}
