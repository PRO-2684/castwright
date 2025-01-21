mod instruction;

pub use instruction::{ConfigInstruction, Instruction, Script};
use std::fmt::Display;

/// Possible types of errors that can occur while parsing a `.cw` file.
#[derive(Debug)]
pub enum ParseErrorType {
    // General parsing errors
    /// An io error occurred while reading the file.
    Io(std::io::Error),
    /// Unknown instruction: The first character of the line is not recognized.
    UnknownInstruction,
    /// Malformed instruction: The instruction is not in the expected format.
    MalformedInstruction,
    /// Did not expect a continuation line, but got one.
    UnexpectedContinuation,
}

impl Display for ParseErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseErrorType::Io(e) => write!(f, "IO error: \"{}\"", e),
            ParseErrorType::UnknownInstruction => write!(f, "Unknown instruction"),
            ParseErrorType::MalformedInstruction => write!(f, "Malformed instruction"),
            ParseErrorType::UnexpectedContinuation => write!(f, "Unexpected continuation"),
        }
    }
}

/// An error that occurred while parsing a `.cw` file.
#[derive(Debug)]
pub struct ParseError {
    /// The type of error that occurred.
    error: ParseErrorType,
    /// The line number where the error occurred, starting at 1. If `0`, the line number is unknown at this point.
    line: usize,
}

impl ParseError {
    /// Create a new `ParseError` with the given error type.
    pub fn new(error: ParseErrorType) -> Self {
        Self { error, line: 0 }
    }
    /// Create a new `ParseError` with error type `Io` and the given io error.
    pub fn io(error: std::io::Error) -> Self {
        Self::new(ParseErrorType::Io(error))
    }
    /// Create a new `ParseError` with error type `UnknownInstruction`.
    pub fn unknown_instruction() -> Self {
        Self::new(ParseErrorType::UnknownInstruction)
    }
    /// Create a new `ParseError` with error type `MalformedInstruction`.
    pub fn malformed_instruction() -> Self {
        Self::new(ParseErrorType::MalformedInstruction)
    }
    /// Create a new `ParseError` with error type `UnexpectedContinuation`.
    pub fn unexpected_continuation() -> Self {
        Self::new(ParseErrorType::UnexpectedContinuation)
    }
    /// Set the line number where the error occurred.
    pub fn with_line(mut self, line: usize) -> Self {
        self.line = line;
        self
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.line == 0 {
            write!(f, "{}", self.error)
        } else {
            write!(f, "{} at line {}", self.error, self.line)
        }
    }
}
