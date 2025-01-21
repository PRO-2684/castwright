mod instruction;

pub use instruction::{ConfigInstruction, Instruction, Script};
use std::{error::Error, fmt::Display};

/// Possible types of errors that can occur while parsing a single line of a `.cw` file.
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

impl ParseErrorType {
    /// Add line number information to the error, so as to form a [`ParseError`].
    fn with_line(self, line: usize) -> ParseError {
        ParseError { error: self, line }
    }
}

/// An error that occurred while parsing a `.cw` file, with the line number denoting its position. To construct a `ParseError`, you should call [`ParseErrorType::with_line`].
#[derive(Debug)]
pub struct ParseError {
    /// The type of error that occurred.
    error: ParseErrorType,
    /// The line number where the error occurred, starting at 1. If `0`, the line number is unknown at this point.
    line: usize,
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

impl Error for ParseError {}
