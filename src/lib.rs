mod instruction;
pub use instruction::{ConfigInstruction, Instruction, Script};

/// Possible errors that can occur while parsing a `.cw` file.
#[derive(Debug)]
pub enum ParseError {
    // General parsing errors
    /// An io error occurred while reading the file.
    Io(std::io::Error),
    /// Unknown instruction: The first character of the line is not recognized. The contained `Option<usize>` is the line number, starting at 1.
    UnknownInstruction(Option<usize>),
    /// Malformed instruction: The instruction is not in the expected format. The contained `Option<usize>` is the line number, starting at 1.
    MalformedInstruction(Option<usize>),
    /// Did not expect a continuation line, but got one. The contained `Option<usize>` is the line number, starting at 1.
    UnexpectedContinuation(Option<usize>),
}

impl ParseError {
    pub fn with_line(self, line: usize) -> Self {
        match self {
            ParseError::UnknownInstruction(_) => ParseError::UnknownInstruction(Some(line)),
            ParseError::MalformedInstruction(_) => ParseError::MalformedInstruction(Some(line)),
            ParseError::UnexpectedContinuation(_) => ParseError::UnexpectedContinuation(Some(line)),
            other => other,
        }
    }
}
